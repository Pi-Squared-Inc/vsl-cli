#![allow(unused)]

use crate::commands::Cli;
use crate::rpc_client::RpcClientError;
use anyhow::Result;
use clap::Parser;
use serde_json::Value;
use vsl_cli::repl::exec_command;

mod accounts;
mod commands;
mod configs;
mod execute;
mod networks;
mod repl;
mod rpc_client;
mod rpc_server;

fn output_result(result: anyhow::Result<Value, RpcClientError>) {
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

fn main() -> Result<()> {
    env_logger::init();
    let command_str = if std::env::var("VSL_CLI_PRINT_COMMANDS").unwrap_or(String::new()) == "1" {
        Some(std::env::args().collect::<Vec<String>>().join(" "))
    } else {
        None
    };
    repl::exec_command(Cli::parse().command, Box::new(output_result), command_str)
}
