#![allow(unused)]

use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time;
use vsl_cli::utils::split_with_quotes;

#[test]
fn test_cli_end_to_end_separate() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let batch_file = std::fs::read_to_string(dir.join("tests").join("batch_commands"))
        .expect("Failed to read the batch command file");
    let error_prefix = "CLI Error";
    let mut errors = Vec::new();
    for line in batch_file.split("\n") {
        let line = line.to_string().trim().to_string();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        println!("vsl-cli {}", line);
        let mut args = vec!["run".to_string(), "-p".to_string(), "vsl-cli".to_string()];
        args.extend(split_with_quotes(&line));
        if let Ok(local_docker) = std::env::var("VSL_CLI_TEST_LOCAL_DOCKER") {
            if line.starts_with("server:init") && local_docker == "1" {
                args.push("--local-docker".to_string());
            }
        }
        let output = Command::new("cargo")
            .env("RUST_LOG", "info")
            .env("VSL_CLI_ERROR_PREFIX", error_prefix)
            .args(args)
            .output()
            .expect("failed to execute CLI batch file");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        print!("{}", stdout);

        if !output.status.success() {
            eprintln!(
                "command: '{}', cli stdout:\n{}, stderr:\n{}",
                line, stdout, stderr
            );
            panic!("`vsl-cli` process failed with status: {}", output.status);
        }

        let err_predicate = |line: &str| {
            line.contains(error_prefix) && !line.contains("Endpoint not yet implemented")
        };

        for line in stdout.split("\n") {
            if err_predicate(line) {
                println!("Error: {}", line);
                errors.push(line.to_string());
            }
        }
    }

    if errors.len() > 0 {
        panic!("FAILED:\n{}", errors.join("\n"))
    } else {
        print!("PASSED\n")
    }
}
