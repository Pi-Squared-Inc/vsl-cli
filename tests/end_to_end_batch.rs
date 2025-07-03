#![allow(unused)]

use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time;

use alloy::serde::quantity::vec;

#[test]
fn test_cli_end_to_end_batch() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // VSL_CLI_PRINT_COMMANDS=1 VSL_CLI_PERSISTENT_CONFIG=0 ./cli.sh repl < ../vsl-cli/tests/batch_commands
    let batch_file = File::open(dir.join("tests").join("batch_commands"))
        .expect("Failed to open the batch command file");
    let error_prefix = "CLI Error";
    //let output = Command::new(dir.join("target").join("debug").join("vsl-cli"))
    let mut args = vec![
        "run".to_string(),
        "-p".to_string(),
        "vsl-cli".to_string(),
        "repl".to_string(),
        "--print-commands".to_string(),
        "--tmp-config".to_string(),
    ];
    if let Ok(local_docker) = std::env::var("VSL_CLI_TEST_LOCAL_DOCKER") {
        if local_docker == "1" {
            args.push("--local-docker".to_string());
        }
    }
    let output = Command::new("cargo")
        .env("RUST_LOG", "info")
        .env("VSL_CLI_ERROR_PREFIX", error_prefix)
        .args(args)
        .stdin(batch_file)
        .output()
        .expect("failed to execute CLI batch file");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("cli stdout:\n{}", stdout);
    println!("");

    if !output.status.success() {
        eprintln!("cli stdout:\n{}", stdout);
        panic!("`vsl-cli` process failed with status: {}", output.status);
    }

    let err_predicate =
        |line: &str| line.contains(error_prefix) && !line.contains("Endpoint not yet implemented");

    let mut errors = Vec::new();

    // Once one of these linse is not present - that's a sign of an error.
    for must_have in vec![
        "Welcome to vsl-cli REPL.",
        "The configuration local_test is created",
        "Local RPC server is initialized and spawned",
        "vsl> health:check\nok",
        "Available networks:\n  default - http://localhost:44444 -- up",
        "Account acc1 is created, address:",
        "Account acc2 is created, address:",
        "Local RPC server is stopped",
        "Configuration local_test was removed",
    ] {
        if !stdout.contains(must_have) {
            let err = format!("Error: must have line: '{}' is not present", must_have);
            println!("{}", err);
            errors.push(err);
        }
    }
    for line in stdout.split("\n") {
        if err_predicate(line) {
            println!("Error: {}", line);
            errors.push(line.to_string());
        }
    }
    if errors.len() > 0 {
        panic!("FAILED:\n{}\n{}", errors.join("\n"), stderr)
    } else {
        print!("PASSED\n")
    }
}
