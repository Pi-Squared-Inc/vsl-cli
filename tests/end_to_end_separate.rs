#![allow(unused)]

use std::fs::File;
use std::process::Command;
use std::thread;
use std::time;

#[test]
fn test_cli_end_to_end_separate() {
    let batch_file = std::fs::read_to_string("../vsl-cli/tests/batch_commands")
        .expect("Failed to read the batch command file");
    let error_prefix = "CLI Error";
    let mut errors = Vec::new();
    for line in batch_file.split("\n") {
        let line = line.to_string().trim().to_string();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        println!("vsl-cli {}", line);
        let output = Command::new("../target/debug/vsl-cli")
            .env("RUST_LOG", "info")
            .env("VSL_CLI_ERROR_PREFIX", error_prefix)
            .args(split_with_quotes(&line))
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

fn split_with_quotes(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes1 = false; // In quotes of first order, i.e. `'--opt <val>'``
    let mut in_quotes2 = false; // In quotes of second order, i.e. `--opt='<val1> <val2>'`
    let mut chars = input.chars().peekable();
    let mut prev_ch = '\0';

    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                if in_quotes1 {
                    // Ending a quote - add the current token (don't trim quoted content)
                    result.push(current.clone());
                    current.clear();
                    in_quotes1 = false;
                } else if in_quotes2 {
                    current.push(ch);
                    in_quotes2 = false;
                } else {
                    if prev_ch == '=' {
                        // Special case: the argument value is enclosed into quotations: `--opt='<val1> <val2>'`
                        current.push(ch);
                        in_quotes2 = true;
                    } else {
                        // Starting a quote: general case - first add any pending unquoted content
                        if !current.is_empty() {
                            result.push(current.trim().to_string());
                            current.clear();
                        }
                        in_quotes1 = true;
                    }
                }
            }
            ' ' if !(in_quotes1 || in_quotes2) => {
                // Space outside quotes - end current token if it's not empty
                if !current.is_empty() {
                    result.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                // Regular character or space inside quotes
                if in_quotes1 || in_quotes2 {
                    // Inside quotes: preserve all characters including whitespace
                    current.push(ch);
                } else if !ch.is_whitespace() {
                    // Outside quotes: only add non-whitespace characters
                    current.push(ch);
                }
            }
        }
        prev_ch = ch;
    }

    // Add the last token if it exists
    if !current.is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_splitting() {
        let result = split_with_quotes("hello world");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_quoted_strings() {
        let result = split_with_quotes("hello 'world test' end");
        assert_eq!(result, vec!["hello", "world test", "end"]);
    }

    #[test]
    fn test_multiple_quotes() {
        let result = split_with_quotes("'first quote' 'second quote'");
        assert_eq!(result, vec!["first quote", "second quote"]);
    }

    #[test]
    fn test_whitespace_handling() {
        let result = split_with_quotes("'  spaced  content  '");
        assert_eq!(result, vec!["  spaced  content  "]);
    }

    #[test]
    fn test_empty_quotes() {
        let result = split_with_quotes("before '' after");
        assert_eq!(result, vec!["before", "", "after"]);
    }

    #[test]
    fn test_arg_eq_quotes() {
        let result = split_with_quotes("before --key='val1 val2' after");
        assert_eq!(result, vec!["before", "--key='val1 val2'", "after"]);
    }

    #[test]
    fn test_empty_arg_eq_quotes() {
        let result = split_with_quotes("before --key='' after");
        assert_eq!(result, vec!["before", "--key=''", "after"]);
    }

    #[test]
    fn test_spaced_arg_eq_quotes() {
        let result = split_with_quotes("before --key='   val1   val2   ' after");
        assert_eq!(result, vec!["before", "--key='   val1   val2   '", "after"]);
    }
}
