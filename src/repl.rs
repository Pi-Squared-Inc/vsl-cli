#![allow(unused)]

use crate::commands::Cli;
use crate::commands::Commands;
use crate::configs::CliMode;
use crate::configs::Config;
use crate::configs::Configs;
use crate::execute::execute_command;
use crate::rpc_client::RpcClient;
use crate::rpc_client::RpcClientError;
use crate::rpc_server::local_server_is_running;
use anyhow::Context;
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use rustyline::ColorMode;
use rustyline::Result as RustyResult;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::history::History;
use serde_json::Value;

use rustyline::CompletionType;
use rustyline::EditMode;
use rustyline::Editor;
use rustyline::completion::Completer;
use rustyline::completion::FilenameCompleter;
use rustyline::completion::Pair;
use rustyline::hint::HistoryHinter;
use rustyline_derive::Helper;
use rustyline_derive::Highlighter;
use rustyline_derive::Hinter;
use rustyline_derive::Validator;
use std::thread;
use std::time::Duration;

#[derive(Helper, Highlighter, Hinter, Validator)]
struct Helper {
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Helper {
    fn new() -> Self {
        Self {
            hinter: HistoryHinter::new(),
            colored_prompt: "".to_owned(),
        }
    }
}

impl Completer for Helper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> RustyResult<(usize, Vec<Pair>)> {
        let mut commands: Vec<String> = Cli::command()
            .get_subcommands()
            .map(|cmd| cmd.get_name().to_string())
            .collect();
        commands.push("screen:clear".to_string());
        commands.push("history:clear".to_string());
        commands.push("history:list".to_string());
        commands.push("exit".to_string());
        commands.push("bye".to_string());
        commands.push("quit".to_string());
        commands.push("help".to_string());

        let mut candidates = Vec::new();
        let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let prefix = &line[start..pos];

        for cmd in commands {
            if cmd.starts_with(prefix) {
                candidates.push(Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                });
            }
        }

        Ok((start, candidates))
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    use std::io::Write;
    use std::io::{self};
    io::stdout().flush().unwrap();
}

fn print_history(rl: &Editor<Helper, FileHistory>) {
    println!("Command history:");
    let history = rl.history();
    for (i, entry) in history.iter().enumerate() {
        println!("{:3}: {}", i + 1, entry);
    }
    if history.is_empty() {
        println!("  (no commands in history)");
    }
}

fn print_repl_help() {
    println!("REPL commands:");
    println!("  help             Show the help message");
    println!("  history:list     Show command history");
    println!("  clear:screen     Clear the screen");
    println!("  clear:history    Clear command history");
    println!("  exit, quit, bye  Exit the REPL");
    println!();
    println!("Navigation:");
    println!("  ↑/↓ arrows       Browse command history");
    println!("  Ctrl+C           Interrupt current input");
    println!("  Ctrl+D           Exit REPL");
    println!("  Tab              Auto-completion of commands");
    println!();
}

type OutputResultFn = Box<dyn FnMut(anyhow::Result<Value, RpcClientError>)>;

fn run_repl_loop(
    config: &mut Config,
    rpc_client: &mut RpcClient,
    print_commands: bool,
    mut output_fn: OutputResultFn,
) -> anyhow::Result<Value, RpcClientError> {
    let mut println = |output_fn: &mut OutputResultFn, s: String| output_fn(Ok(Value::String(s)));
    println(&mut output_fn, "Welcome to vsl-cli REPL.".to_string());
    println(
        &mut output_fn,
        "Type 'help' for available commands, 'exit' or 'quit' or 'bye' to leave.".to_string(),
    );
    println(
        &mut output_fn,
        "Use ↑/↓ arrows to browse command history, tab for completion.".to_string(),
    );

    // init rustyline
    let rl_config = rustyline::Config::builder()
        .history_ignore_space(true)
        .max_history_size(64)
        .or_else(|err| {
            Err(RpcClientError::GeneralError(format!(
                "Failed to set `max_history_size: {}",
                err
            )))
        })?
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .color_mode(ColorMode::Enabled)
        .build();

    let mut rl = Editor::with_config(rl_config)
        .map_err(|e| RpcClientError::IncorrectRequest(format!("Failed to create editor: {}", e)))?;
    let helper = Helper::new();
    rl.set_helper(Some(helper));

    // Load history
    let history_file = crate::configs::vsl_config_dir()?.join("cli_history");
    if history_file.exists() {
        if let Err(e) = rl.load_history(&history_file) {
            println(
                &mut output_fn,
                format!("Warning: Could not load history: {}", e),
            );
        }
    }

    loop {
        match rl.readline("vsl> ") {
            Ok(input) => {
                let input = input.trim();
                if print_commands
                    || std::env::var("VSL_CLI_PRINT_COMMANDS").unwrap_or(String::new()) == "1"
                {
                    println!("vsl> {}", input);
                }
                if input.is_empty() || input.to_string().starts_with("#") {
                    continue;
                }

                // Handle special REPL commands and meaningful VSL commands
                match input {
                    ("exit" | "quit" | "bye") => {
                        println(&mut output_fn, "Goodbye!".to_string());
                        break;
                    }
                    "history:list" => print_history(&rl),
                    "screen:clear" => clear_screen(),
                    "history:clear" => rl.clear_history().or_else(|err| {
                        Err(RpcClientError::GeneralError(format!(
                            "failed to clear command line history: {}",
                            err
                        )))
                    })?,
                    _ => {
                        // Parse the input as command line arguments
                        match parse_repl_command(input) {
                            Ok(command) => {
                                // Add to history (but don't add duplicates or special commands)
                                if !input.is_empty() && input != "help" {
                                    rl.add_history_entry(input).map_err(|e| {
                                        RpcClientError::IncorrectRequest(format!(
                                            "History error: {}",
                                            e
                                        ))
                                    })?;
                                }
                                // Execute the parsed command
                                output_fn(execute_command(config, &command, rpc_client));
                            }
                            Err(err) => {
                                println!("{}", err);
                                print_repl_help();
                            }
                        }
                    }
                }
            }
            Err(err) => match err {
                ReadlineError::Eof => break,
                _ => {
                    println(&mut output_fn, format!("Error reading input: {}", err));
                    break;
                }
            },
        }
    }

    // Save history
    if let Err(e) = rl.save_history(&history_file) {
        println(
            &mut output_fn,
            format!("Warning: Could not save history: {}", e),
        );
    }

    Ok(Value::String("REPL session ended".to_string()))
}

fn parse_repl_command(input: &str) -> Result<Commands, String> {
    // Split the input into arguments, handling quotes properly
    let args = match shlex::split(input) {
        Some(args) => args,
        None => return Err("Invalid command syntax".to_string()),
    };

    // Prepend the program name (required by clap)
    let mut full_args = vec!["vsl-cli".to_string()];
    full_args.extend(args);

    // Parse using clap
    match Cli::try_parse_from(full_args) {
        Ok(cli) => Ok(cli.command),
        Err(err) => Err(err.to_string()),
    }
}

pub fn exec_command(
    command: Commands,
    mut output_fn: OutputResultFn,
    command_str: Option<String>,
) -> Result<()> {
    // Create the client connection
    let mut rpc_client = RpcClient::new();

    match command {
        Commands::Repl {
            print_commands,
            tmp_config,
        } => {
            // Load the existing config in REPL mode
            let mut config = Configs::load(None, CliMode::MultiCommand)
                .context("Failed to load a current config")?;
            run_repl_loop(&mut config, &mut rpc_client, print_commands, output_fn);
        }
        _ => {
            // Load the existing config in a single command mode
            let mut config = Configs::load(None, CliMode::SingleCommand)
                .context("Failed to load a current config")?;
            match command_str {
                Some(str) => output_fn(Ok(Value::String(str))),
                None => {}
            }
            output_fn(execute_command(&mut config, &command, &mut rpc_client))
        }
    }
    Ok(())
}
