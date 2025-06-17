#![allow(unused)]

mod accounts;
mod commands;
mod configs;
mod execute;
mod networks;
mod rpc_client;
mod rpc_server;

use anyhow::Context;
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use commands::Cli;
use commands::Commands;
use configs::Config;
use configs::Configs;
use configs::RpcServer;
use execute::execute_command;
use rpc_client::RpcClient;
use rpc_client::RpcClientError;
use rpc_server::check_server_is_alive;
use rpc_server::try_to_find_server;
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

fn run_repl_loop(
    config: &mut Config,
    rpc_client: &mut RpcClient,
    print_commands: bool,
) -> anyhow::Result<Value, RpcClientError> {
    println!("Welcome to vsl-cli REPL.");
    println!("Type 'help' for available commands, 'exit' or 'quit' or 'bye' to leave.");
    println!("Use ↑/↓ arrows to browse command history, tab for completion.");

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
    let history_file = configs::vsl_config_dir()?.join("cli_history");
    if history_file.exists() {
        if let Err(e) = rl.load_history(&history_file) {
            println!("Warning: Could not load history: {}", e);
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
                        println!("Goodbye!");
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
                                output_result(&execute_command(config, &command, rpc_client));
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
                    println!("Error reading input: {}", err);
                    break;
                }
            },
        }
    }

    // Save history
    if let Err(e) = rl.save_history(&history_file) {
        println!("Warning: Could not save history: {}", e);
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

fn output_result(result: &anyhow::Result<Value, RpcClientError>) {
    match result {
        Ok(value) => match value {
            Value::String(str) => println!("{}", str),
            _ => match serde_json::to_string_pretty(&value) {
                Ok(pretty_json) => println!("{}", pretty_json),
                Err(e) => println!("Invalid JSON: {}, the response: {}", e, value),
            },
        },
        Err(err) => match std::env::var("VSL_CLI_ERROR_PREFIX") {
            Ok(error_prefix) => println!("{}: {}", error_prefix, err),
            Err(_) => println!("{}", err),
        },
    }
}

fn make_config() -> Result<Config> {
    let mut config = Configs::load(None).context("Failed to load a current config")?;
    match config.get_server() {
        Some(server) => {
            // In case server is not running - remove it
            if !check_server_is_alive(&server) {
                config.set_server(None);
                match try_to_find_server() {
                    Some(pid) => config
                        .set_server(Some(RpcServer { pid, local: None }))
                        .context("Failed to set server")?,
                    None => {}
                }
            }
        }
        None => match try_to_find_server() {
            Some(pid) => config
                .set_server(Some(RpcServer { pid, local: None }))
                .context("Failed to set server")?,
            None => {}
        },
    }
    Ok(config)
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    // Create the client connection
    let mut rpc_client = RpcClient::new();

    // Load the existing config
    let mut config = make_config()?;

    match cli.command {
        Commands::Repl {
            print_commands,
            tmp_config,
        } => {
            run_repl_loop(&mut config, &mut rpc_client, print_commands);
        }
        _ => {
            if std::env::var("VSL_CLI_PRINT_COMMANDS").unwrap_or(String::new()) == "1" {
                println!("{}", std::env::args().collect::<Vec<String>>().join(" "));
            }
            output_result(&execute_command(&mut config, &cli.command, &mut rpc_client))
        }
    }
    Ok(())
}
