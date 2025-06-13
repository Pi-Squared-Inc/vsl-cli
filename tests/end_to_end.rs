#![allow(unused)]

use std::fs::File;
use std::process::Command;
use std::thread;
use std::time;

#[test]
fn test_cli_end_to_end() {
    // Make sure that `vsl-core` - the server - is built.
    Command::new("cargo")
        .current_dir("../vsl-core")
        .args(["build", "--release"])
        .output()
        .expect("failed to build vsl-core (RPC server)");

    // VSL_CLI_PRINT_COMMANDS=1 VSL_CLI_PERSISTENT_CONFIG=0 ./cli.sh repl < ../vsl-cli/tests/batch_commands
    let batch_file = File::open("../vsl-cli/tests/batch_commands")
        .expect("Failed to open the batch command file");
    let error_prefix = "CLI Error";
    let output = Command::new("../target/debug/vsl-cli")
        .env("RUST_LOG", "info")
        .env("VSL_CLI_ERROR_PREFIX", error_prefix)
        .args(["repl", "--print-commands", "--tmp-config"])
        .stdin(batch_file)
        .output()
        .expect("failed to execute CLI batch file");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("cli stdout:\n{}", stdout);
    println!("");

    let err_predicate =
        |line: &str| line.contains(error_prefix) && !line.contains("Endpoint not yet implemented");

    let mut errors = Vec::new();
    for line in stdout.split("\n") {
        if err_predicate(line) {
            println!("Error: {}", line);
            errors.push(line);
        }
    }
    if errors.len() > 0 {
        panic!("FAILED:\n{}\n{}", errors.join("\n"), stderr)
    } else {
        print!("PASSED\n")
    }
}
