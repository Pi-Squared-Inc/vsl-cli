use crate::configs::RpcServer;
use crate::configs::RpcServerLocal;

use crate::accounts::InitAccount;
use crate::networks::VSL_CLI_DEFAULT_NETWORK_PORT;
use anyhow::Context;
use anyhow::Result;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::SystemTime;

/// Starts the server in a separate child process.
/// - vls_dir: the directory of the VSL repository/distribution
/// - db_path: path to the VSL storage directory
/// - log_level: one of RUST_LOG values - info, warn, error, etc....
pub fn start_server(
    vsl_dir: PathBuf,
    db: &String,
    log_level: String,
    maybe_master_account: Option<InitAccount>,
) -> Result<RpcServer> {
    let mut claim_db_path = String::new();
    let mut tokens_db_path = String::new();
    let mut use_genesis = false;
    let mut tempdir: Option<tempfile::TempDir> = None;
    if db == "tmp" {
        if claim_db_path != "" || tokens_db_path != "" {
            return Err(anyhow::anyhow!(
                "Cannot use `--tmp` and `--claim-db-path` or `--tokens-db-path` at the same time"
                    .to_string(),
            ));
        }
        // Create temporary directories for DB and tokens
        let temp_dir = tempfile::TempDir::new().map_err(|err| {
            anyhow::anyhow!(format!("Failed to create temporary directory: {}", err))
        })?;
        log::info!(
            "Temporary directory for VSL server: {}",
            temp_dir.path().to_str().unwrap()
        );
        claim_db_path = String::from(
            temp_dir
                .path()
                .to_path_buf()
                .join("vsl-db")
                .to_str()
                .ok_or(anyhow::anyhow!(
                    "Failed to create temporary directory".to_string(),
                ))?,
        );
        tokens_db_path = String::from(
            temp_dir
                .path()
                .to_path_buf()
                .join("tokens.db")
                .to_str()
                .ok_or(anyhow::anyhow!(
                    "Failed to create temporary directory".to_string(),
                ))?,
        );
        use_genesis = true;
        tempdir = Some(temp_dir);
    } else {
        let claim_db_dir = PathBuf::from(db).join("vsl-db");
        claim_db_path = String::from(claim_db_dir.to_str().ok_or(anyhow::anyhow!(
            "Failed to get the vsl-db directory".to_string(),
        ))?);
        let token_db_dir = PathBuf::from(db).join("tokens.db");
        tokens_db_path = String::from(token_db_dir.to_str().ok_or(anyhow::anyhow!(
            "Failed to get the tokens.db directory".to_string(),
        ))?);
        use_genesis = !token_db_dir.exists() && !claim_db_dir.exists();
    }
    std::fs::create_dir_all(&claim_db_path).context("Failed to create claim DB directory")?;
    std::fs::create_dir_all(&tokens_db_path).context("Failed to create tokens DB directory")?;
    let log_path = vsl_dir.join("vsl-cli").join("logs");
    std::fs::create_dir_all(log_path.clone()).context("Failed to create tokens logs directory")?;

    let mut args = Vec::new();
    if claim_db_path != "" {
        args.push("--claim-db-path".to_string());
        args.push(claim_db_path)
    }
    if tokens_db_path != "" {
        args.push("--tokens-db-path".to_string());
        args.push(tokens_db_path)
    }
    if use_genesis {
        args.push("--use-genesis".to_string());
    }
    if let Some(master_account) = maybe_master_account {
        args.push("--master-account".to_string());
        args.push(
            serde_json::to_string(&master_account).context("Failed to serialize master account")?,
        );
    }
    // Create timestamped log files
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let stdout_path = log_path
        .clone()
        .join(format!("server-{}.stdout.log", timestamp));
    let stderr_path = log_path
        .clone()
        .join(format!("server-{}.stderr.log", timestamp));

    // Create/open log files
    let stdout_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&stdout_path)?;

    let stderr_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&stderr_path)?;

    let vsl_binary = "target/release/vsl-core";
    //let vsl_binary = "target/debug/vsl-core";
    let mut child = Command::new(vsl_binary)
        .current_dir(vsl_dir.clone())
        .env("RUST_LOG", log_level)
        .args(args.clone())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .or(Err(anyhow::anyhow!(
            "Failed to launch server {} in vsl directory: {}",
            vsl_binary,
            vsl_dir.to_str().unwrap_or("<unknown>")
        )))?;

    let config = RpcServer {
        pid: child.id(),
        local: Some(RpcServerLocal {
            command: vec!["target/release/vsl-core".to_string()]
                .iter()
                .chain(args.iter())
                .cloned()
                .collect(),
            started: SystemTime::now(),
            db_dir: PathBuf::from(db),
            stdout: stdout_path,
            stderr: stderr_path,
        }),
    };

    // Wait for the server to start
    std::thread::sleep(std::time::Duration::from_millis(50));

    if let Some(status) = child.try_wait()? {
        Err(anyhow::anyhow!(
            "Process exited immediately with status: {:?}",
            status
        ))
    } else {
        // Don't wait for the child - let it run in background
        std::mem::forget(child);
        Ok(config)
    }
}

/// Stop server using system commands by a server PID.
pub fn stop_server(server: &RpcServer) -> Result<()> {
    let output = if cfg!(target_os = "windows") {
        Command::new("taskkill")
            .args(["/F", "/PID", &server.pid.to_string()])
            .output()?
    } else {
        Command::new("kill").arg(server.pid.to_string()).output()?
    };
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to kill process {}: {}",
            server.pid,
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

/// Dump both stdout and stderr with timestamps (if available)
pub fn dump_server(server: &RpcServerLocal) -> Result<String> {
    let mut output = String::new();

    output.push_str("=== STDOUT ===\n");
    match std::fs::read_to_string(&server.stdout) {
        Ok(stdout) => output.push_str(&stdout),
        Err(e) => output.push_str(&format!("Error reading stdout: {}\n", e)),
    }

    output.push_str("\n=== STDERR ===\n");
    match std::fs::read_to_string(&server.stderr) {
        Ok(stderr) => output.push_str(&stderr),
        Err(e) => output.push_str(&format!("Error reading stderr: {}\n", e)),
    }

    Ok(output)
}

/// Check if a server is still running
pub fn check_server_is_alive(server: &RpcServer) -> bool {
    if cfg!(target_os = "windows") {
        // Windows: Use tasklist
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", server.pid), "/FO", "CSV"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().count() > 1 // Header + process line if exists
            })
            .unwrap_or(false)
    } else {
        // Unix-like: Use kill -0
        Command::new("kill")
            .args(["-0", &server.pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

pub fn find_server_by_netstat() -> Result<Option<u32>> {
    let output = if cfg!(target_os = "windows") {
        Command::new("netstat").args(["-ano"]).output()?
    } else {
        Command::new("netstat").args(["-tlnp"]).output()?
    };
    let stdout = String::from_utf8(output.stdout)?;
    for line in stdout.lines() {
        if line.contains(&format!(":{}", VSL_CLI_DEFAULT_NETWORK_PORT)) {
            if cfg!(target_os = "windows") {
                // Windows netstat format: TCP    0.0.0.0:8080    0.0.0.0:0    LISTENING    1234
                if let Some(pid_str) = line.split_whitespace().last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return Ok(Some(pid));
                    }
                }
            } else {
                // Linux netstat format: tcp 0 0 0.0.0.0:8080 0.0.0.0:* LISTEN 1234/program
                if let Some(last_part) = line.split_whitespace().last() {
                    if let Some(pid_str) = last_part.split('/').next() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            return Ok(Some(pid));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}
