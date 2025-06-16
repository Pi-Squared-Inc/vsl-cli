#![allow(unused)]

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use vsl_cli::commands::Commands;
use vsl_cli::configs::Config;
use vsl_cli::configs::Configs;
use vsl_cli::configs::RpcServer;
use vsl_cli::configs::VSL_TMP_CONFIG;
use vsl_cli::configs::vsl_directory;
use vsl_cli::execute::execute_command;
use vsl_cli::execute::launch_server;
use vsl_cli::networks::Network;
use vsl_cli::rpc_client::RpcClient;
use vsl_cli::rpc_client::check_network_is_up;
use vsl_cli::rpc_server::dump_server;
use vsl_cli::rpc_server::stop_server;

#[derive(Debug, Clone, Copy)]
pub struct StressTestConfig {
    pub num_requests: usize,
    pub max_concurrent: usize,
    pub timeout_seconds: u64,
    pub verbosity: u32,
}

#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_duration: Duration,
    pub avg_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub requests_per_second: f64,
    pub errors: Vec<String>,
}

impl StressTestResult {
    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
struct RequestResult {
    value: Value,
    success: bool,
    duration: Duration,
    error: Option<String>,
}

fn create_accounts(
    stress_config: StressTestConfig,
    n: usize,
    indent: String,
    config: &mut Config,
) -> Result<Vec<Config>> {
    let mut client = RpcClient::new();
    let mut success = true;
    let mut configs = Vec::new();
    for n in 0..stress_config.max_concurrent {
        let acc_name = format!("acc_{}", n);
        let create_account_comm = Commands::AccountCreate {
            name: acc_name.clone(),
            overwrite: true,
        };
        let create_acc_response = execute_single_request(config, &create_account_comm, &mut client);
        let address = config
            .get_account(None)
            .expect("account is absent!!!!")
            .credentials
            .address;
        if create_acc_response.success {
            match create_acc_response.value {
                Value::String(ref account_id) => {
                    let supply_account_comm = Commands::Pay {
                        network: None,
                        to: address.clone(),
                        amount: "0x1000".to_string(),
                    };
                    config.use_account("master");
                    let supply_account_response =
                        execute_single_request(config, &supply_account_comm, &mut client);
                    if supply_account_response.success {
                        config.use_account(&acc_name);
                        let balance_comm = Commands::AccountBalance {
                            network: None,
                            account: None,
                        };
                        let balance_response =
                            execute_single_request(config, &balance_comm, &mut client);
                        let balance = match balance_response.value {
                            Value::String(ref balance) => balance.clone(),
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "{}Failed to get balance",
                                    indent.clone()
                                ));
                            }
                        };
                        let master_balance_comm = Commands::AccountBalance {
                            network: None,
                            account: Some("master".to_string()),
                        };
                        let master_balance_response =
                            execute_single_request(config, &master_balance_comm, &mut client);
                        let master_balance = match master_balance_response.value {
                            Value::String(ref balance) => balance.clone(),
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "{}Failed to get master balance",
                                    indent.clone()
                                ));
                            }
                        };
                        if stress_config.verbosity > 1 {
                            println!(
                                "{}  Account: acc_{}(address: {}) balance: {}, master balance: {}",
                                indent.clone(),
                                n,
                                address,
                                balance,
                                master_balance
                            );
                        }
                        configs.push(config.clone());
                    }
                }
                _ => {
                    success = false;
                    println!(
                        "{}  create_acc_response - UNEXPECTED: {}",
                        indent.clone(),
                        create_acc_response.value
                    );
                }
            }
        } else {
            success = false;
            println!(
                "{}  Account: acc_{} == FAILED: {}",
                indent.clone(),
                n,
                create_acc_response.clone().error.unwrap()
            );
        }
    }
    if success {
        Ok(configs)
    } else {
        Err(anyhow::anyhow!(
            "{}Failed to create account",
            indent.clone()
        ))
    }
}

fn run_one_thread(
    stress_config: StressTestConfig,
    n: usize,
    indent: String,
    config: &mut Config,
) -> thread::JoinHandle<Vec<RequestResult>> {
    let indent = indent.clone();
    let mut config = config.clone();
    let mut client = RpcClient::new();

    if stress_config.verbosity > 1 {
        println!(
            "{}  Running synchroniously, thread: {}...",
            indent.clone(),
            n
        );
    }
    thread::spawn(move || {
        let mut results = Vec::new();
        let timeout = Duration::from_secs(stress_config.timeout_seconds);

        for i in 0..stress_config.num_requests {
            let submit_command = Commands::ClaimSubmit {
                network: None,
                claim: format!("test_claim_{}", i),
                claim_type: "identity".to_string(),
                proof: format!("proof_{}", i),
                expires: None,
                lifetime: Some(3600),
                fee: "0x1".to_string(),
            };
            let submit_response = execute_single_request(&mut config, &submit_command, &mut client);
            results.push(submit_response.clone());
            if submit_response.success {
                match submit_response.value {
                    Value::String(ref claim_id) => {
                        let submitted_command = Commands::ClaimSubmitted {
                            network: None,
                            address: None,
                            since: None,
                            within: Some(3600),
                        };
                        let submitted_response =
                            execute_single_request(&mut config, &submitted_command, &mut client);
                        results.push(submitted_response.clone());
                        if submitted_response.success {
                            let settle_command = Commands::ClaimSettle {
                                network: None,
                                claim: format!("test_claim_{}", i), //format!("{}", claim_id.clone()),
                                address: None,
                            };
                            let settle_response =
                                execute_single_request(&mut config, &settle_command, &mut client);
                            results.push(settle_response.clone());
                            if settle_response.success {
                                let settled_command = Commands::ClaimSettled {
                                    network: None,
                                    address: None,
                                    since: None,
                                    within: Some(3600),
                                };
                                let settled_response = execute_single_request(
                                    &mut config,
                                    &settled_command,
                                    &mut client,
                                );
                                results.push(settle_response);
                            } else if !settle_response.success {
                                println!(
                                    "FAILED TO SUBMIT: {}, response: {}, submitted: {}",
                                    claim_id.clone(),
                                    settle_response.error.unwrap_or("???".to_string()),
                                    submit_response.value
                                )
                            }
                        }
                    }
                    _ => {}
                }
            }
            // Progress reporting
            if stress_config.verbosity > 2 {
                if (i + 1) % 100 == 0 {
                    println!(
                        "{}Dispatched {}/{} requests",
                        indent,
                        i + 1,
                        stress_config.num_requests,
                    );
                }
            }
        }
        if stress_config.verbosity > 1 {
            println!("{}  Job {} completed", indent, n);
        }
        results
    })
}

pub fn run_stress_test(
    indent: &str,
    stress_config: StressTestConfig,
    config: &mut Config,
) -> Result<StressTestResult> {
    if stress_config.verbosity > 0 {
        println!(
            "{}Starting stress test with {} requests, {} max concurrent",
            indent, stress_config.num_requests, stress_config.max_concurrent
        );
    }

    let start_time = Instant::now();
    let indent = indent.to_string();

    let configs = create_accounts(
        stress_config,
        stress_config.max_concurrent,
        indent.to_string(),
        config,
    )?;

    let n_results: Vec<_> = (0..stress_config.max_concurrent)
        .map(|n| {
            let mut config = configs.get(n).expect("config must be there").clone();
            run_one_thread(stress_config, n, indent.to_string(), &mut config)
        })
        .collect();

    // Collect results
    let mut results = Vec::new();
    for handle in n_results {
        for result in handle.join().unwrap() {
            results.push(result);
        }
    }

    let total_duration = start_time.elapsed();
    let analysis = analyze_results(results, total_duration, stress_config.num_requests);

    Ok(analysis)
}

fn execute_single_request(
    config: &mut Config,
    command: &Commands,
    rpc_client: &mut RpcClient,
) -> RequestResult {
    let start = Instant::now();
    match execute_command(config, command, rpc_client) {
        Ok(value) => RequestResult {
            value: value,
            success: true,
            duration: start.elapsed(),
            error: None,
        },
        Err(err) => RequestResult {
            value: Value::Null,
            success: false,
            duration: start.elapsed(),
            error: Some(format!("RPC error: {:?}", err)),
        },
    }
}

fn check_server(verbosity: u32) -> Result<()> {
    let mut config = Configs::new(VSL_TMP_CONFIG.to_string(), String::new(), false)?;
    let mut client = RpcClient::new();
    let network = config.get_network(None).unwrap_or(Network::default());
    if verbosity > 1 {
        println!("  Checking server on: {}...", network);
    }
    let status = if check_network_is_up(&mut client, network) {
        "up"
    } else {
        "down"
    };
    if verbosity > 1 {
        println!("  Server is: {}...", status);
    }
    if status == "up" {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Server is down"))
    }
}

fn analyze_results(
    results: Vec<RequestResult>,
    total_duration: Duration,
    expected_requests: usize,
) -> StressTestResult {
    let successful_requests = results.iter().filter(|r| r.success).count();
    let failed_requests = results.len() - successful_requests;

    let successful_durations: Vec<Duration> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.duration)
        .collect();

    let (avg_response_time, min_response_time, max_response_time) =
        if successful_durations.is_empty() {
            (
                Duration::from_secs(0),
                Duration::from_secs(0),
                Duration::from_secs(0),
            )
        } else {
            let total_response_time: Duration = successful_durations.iter().sum();
            let avg = total_response_time / successful_durations.len() as u32;
            let min = *successful_durations.iter().min().unwrap();
            let max = *successful_durations.iter().max().unwrap();
            (avg, min, max)
        };

    let requests_per_second = if total_duration.as_secs_f64() > 0.0 {
        successful_requests as f64 / total_duration.as_secs_f64()
    } else {
        0.0
    };

    let errors: Vec<String> = results
        .iter()
        .filter_map(|r| r.error.as_ref())
        .cloned()
        .collect();

    StressTestResult {
        total_requests: results.len(),
        successful_requests,
        failed_requests,
        total_duration,
        avg_response_time,
        min_response_time,
        max_response_time,
        requests_per_second,
        errors,
    }
}

fn print_results(indent: &str, result: &StressTestResult) {
    println!("");
    println!("{}=== STRESS TEST RESULTS ===", indent);
    println!("{}  Total Requests: {}", indent, result.total_requests);
    println!("{}  Successful: {}", indent, result.successful_requests);
    println!("{}  Failed: {}", indent, result.failed_requests);
    println!(
        "{}  Success Rate: {:.2}%",
        indent,
        (result.successful_requests as f64 / result.total_requests as f64) * 100.0
    );
    println!(
        "{}  Total Duration: {:.2}s",
        indent,
        result.total_duration.as_secs_f64()
    );
    println!(
        "{}  Requests/Second: {:.2}",
        indent, result.requests_per_second
    );
    println!(
        "{}  Avg Response Time: {:.2}ms",
        indent,
        result.avg_response_time.as_millis()
    );
    println!(
        "{}  Min Response Time: {:.2}ms",
        indent,
        result.min_response_time.as_millis()
    );
    println!(
        "{}  Max Response Time: {:.2}ms",
        indent,
        result.max_response_time.as_millis()
    );

    if !result.errors.is_empty() {
        println!("");
        println!("{}=== ERROR SUMMARY ===", indent);
        let mut error_counts = HashMap::new();
        let mut i = 0;
        let show_n_errs = 32;
        for error in &result.errors {
            *error_counts.entry(error).or_insert(0) += 1;
            i += 1;
            if i > show_n_errs {
                break;
            }
        }
        for (error, count) in error_counts {
            println!("{}  {}: {} occurrences", indent, error, count);
        }
        if (result.errors.len() > i) {
            println!("{}  ... other {} errors", indent, result.errors.len() - i);
        }
    }
}

pub fn display_stress_test_results(results: &HashMap<usize, StressTestResult>) {
    if results.is_empty() {
        println!("No stress test results to display.");
        return;
    }

    // Sort by concurrency level for consistent display
    let mut sorted_results: Vec<(&usize, &StressTestResult)> = results.iter().collect();
    sorted_results.sort_by_key(|(concurrency, _)| *concurrency);

    println!("\n{}", "=".repeat(120));
    println!("{:^120}", "STRESS TEST RESULTS");
    println!("{}", "=".repeat(120));

    // Table header
    println!(
        "{:<12} {:<8} {:<8} {:<8} {:<12} {:<12} {:<12} {:<12} {:<12} {:<8}",
        "Concurrency",
        "Total",
        "Success",
        "Failed",
        "Success%",
        "Avg Time",
        "Min Time",
        "Max Time",
        "RPS",
        "Errors"
    );
    println!("{}", "-".repeat(120));

    // Table rows
    for (concurrency, result) in &sorted_results {
        println!(
            "{:<12} {:<8} {:<8} {:<8} {:<12.1} {:<12} {:<12} {:<12} {:<12.1} {:<8}",
            concurrency,
            result.total_requests,
            result.successful_requests,
            result.failed_requests,
            result.success_rate(),
            format_duration(result.avg_response_time),
            format_duration(result.min_response_time),
            format_duration(result.max_response_time),
            result.requests_per_second,
            result.errors.len()
        );
    }

    println!("{}", "=".repeat(120));

    // Summary section
    if let Some(best_rps) = sorted_results.iter().max_by(|a, b| {
        a.1.requests_per_second
            .partial_cmp(&b.1.requests_per_second)
            .unwrap()
    }) {
        println!("\n📊 SUMMARY:");
        println!(
            "Best RPS: {:.1} at concurrency {}",
            best_rps.1.requests_per_second, best_rps.0
        );
    }

    // Show errors if any exist
    let total_errors: usize = sorted_results
        .iter()
        .map(|(_, result)| result.errors.len())
        .sum();
    if total_errors > 0 {
        println!("\n❌ ERRORS DETECTED ({} total):", total_errors);
        println!("{}", "-".repeat(60));

        for (concurrency, result) in sorted_results {
            if !result.errors.is_empty() {
                println!(
                    "Concurrency {}: {} errors",
                    concurrency,
                    result.errors.len()
                );
                for (i, error) in result.errors.iter().enumerate().take(3) {
                    // Show first 3 errors
                    println!("  • {}", error);
                }
                if result.errors.len() > 3 {
                    println!("  ... and {} more errors", result.errors.len() - 3);
                }
                //for (i, error) in result.errors.iter().enumerate() {
                //    println!("  • {}", error);
                //}
                println!();
            }
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        format!("{:.1}ms", micros as f64 / 1000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut result_table: HashMap<usize, StressTestResult> = HashMap::new();
    let mut n = 2;
    let degree = 5;
    let verbosity = 0;
    for i in 1..=degree {
        n = n * 4;
        if verbosity > 0 {
            println!("CONCURRENCY: {}", n);
        }
        let indent_0 = "  ";
        let stress_config = StressTestConfig {
            num_requests: 30,
            max_concurrent: n,
            timeout_seconds: 30,
            verbosity: verbosity,
        };
        let mut config = Configs::new(VSL_TMP_CONFIG.to_string(), String::new(), false)?;
        let mut server = launch_server(
            &mut config,
            "tmp".to_string(),
            "info".to_string(),
            "master".to_string(),
            "100000000".to_string(),
        )?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        check_server(stress_config.verbosity)?;
        match run_stress_test(indent_0, stress_config, &mut config) {
            Ok(results) => {
                result_table.insert(n, results.clone());
                if stress_config.verbosity > 0 {
                    print_results(indent_0, &results);
                }
                if stress_config.verbosity > 3 {
                    println!(
                        "{}Server stdout:\n{}",
                        indent_0,
                        dump_server(&server.local.clone().unwrap())?
                    )
                }
                stop_server(&server)?;
                std::thread::sleep(std::time::Duration::from_millis(50));
                if stress_config.verbosity > 1 {
                    println!("{}Server successfully stopped", indent_0)
                }
                if stress_config.verbosity > 0 {
                    if results.failed_requests > 0 {
                        println!("{}FAILED\n\n\n", indent_0);
                        break;
                    } else {
                        println!("{}PASSED\n\n\n", indent_0);
                    }
                }
            }
            Err(err) => {
                eprintln!("{}error: {}", indent_0, err);
                if stress_config.verbosity > 3 {
                    println!(
                        "{}Server stdout:\n{}",
                        indent_0,
                        dump_server(&server.local.clone().unwrap())?
                    )
                }
                stop_server(&server)?;
                std::thread::sleep(std::time::Duration::from_millis(50));
                break;
            }
        }
    }
    display_stress_test_results(&result_table);
    Ok(())
}
