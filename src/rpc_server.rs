use crate::configs::RpcServerInit;
use crate::configs::RpcServerLocal;

use crate::networks::VSL_CLI_DEFAULT_NETWORK_PORT;
use anyhow::Context;
use anyhow::Result;
use std::clone;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::SystemTime;
use tempfile::TempDir;

/// Starts the server in a separate child process.
/// - db_path: path to the VSL storage directory
/// - init: the initial genesis JSON either as a string or as a file
/// - log_level: one of RUST_LOG values - info, warn, error, etc....
#[must_use]
pub fn start_local_server(
    db: &String,
    init: RpcServerInit,
    force: bool,
) -> Result<(RpcServerLocal, Option<TempDir>)> {
    let vsl_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !is_docker_installed() {
        return Err(anyhow::anyhow!(
            "'docker compose' is necessary to run RPC VSL server"
        ));
    }
    let mut tempdir: Option<tempfile::TempDir> = None;
    let mut db_dir = db.clone();
    if db == "tmp" {
        // Create temporary directories for DB
        let temp_dir = tempfile::TempDir::with_prefix("vsl-").map_err(|err| {
            anyhow::anyhow!(format!("Failed to create temporary directory: {}", err))
        })?;
        log::info!(
            "Temporary directory for VSL server: {}",
            temp_dir.path().to_str().unwrap()
        );
        db_dir = temp_dir.path().to_str().unwrap_or("?").to_string();
        tempdir = Some(temp_dir);
    }

    make_dockerfile(db_dir, init, force)?;

    // If the docker image was not yet downloaded - do it.
    let pull_docker_image = Command::new("docker")
        .current_dir(vsl_dir.clone())
        .args(["compose", "-f", DOCKERFILE_NAME, "pull"])
        .output()
        .or(Err(anyhow::anyhow!(
            "Failed to launch server in vsl directory: {}",
            vsl_dir.to_str().unwrap_or("<unknown>")
        )))?;
    if !pull_docker_image.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to download docker image of vsl-core: stderr:\n{}\nstdout:{}",
            String::from_utf8(pull_docker_image.stderr).unwrap_or("?".to_string()),
            String::from_utf8(pull_docker_image.stdout).unwrap_or("?".to_string()),
        ));
    }

    // Create timestamped log files
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    // Start the server in daemon mode.
    let mut child = Command::new("docker")
        .current_dir(vsl_dir.clone())
        .args(["compose", "-f", DOCKERFILE_NAME, "up", "-d"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .or(Err(anyhow::anyhow!(
            "Failed to launch server in vsl directory: {}",
            vsl_dir.to_str().unwrap_or("<unknown>")
        )))?;

    // Check the started server for the immediate failure.
    let output = child.wait_with_output()?;
    if !output.status.success() {
        // Check on startup failure
        return Err(anyhow::anyhow!(
            "docker compose failed with exit code: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Initial wait for the server to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Wait for the server to be ready with proper health check
    let max_attempts = 10;
    let mut attempt = 0;

    while attempt < max_attempts {
        // Wait a little between attempts
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Test if server is responding
        match std::process::Command::new("curl")
            .args([
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "-d",
                "{\"jsonrpc\":\"2.0\",\"id\":\"id\",\"method\":\"vsl_getHealth\"}",
                "http://localhost:44444",
            ])
            .output()
        {
            Ok(output) if output.status.success() => {
                return Ok((
                    RpcServerLocal {
                        command: Vec::new(),
                        started: SystemTime::now(),
                        db_dir: PathBuf::from(db),
                    },
                    tempdir,
                ));
            }
            _ => attempt += 1,
        }
    }
    Err(anyhow::anyhow!(
        "Server failed to start within {} attempts to connect",
        max_attempts
    ))
}

fn make_dockerfile(db_dir: String, init: RpcServerInit, force: bool) -> Result<()> {
    let vsl_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let contents = DOCKERFILE_TEMPLATE
        .replace("$DB_DIR", &db_dir)
        .replace("$FORCE", if force { "- --force" } else { "" })
        .replace(
            "$GENESIS_COMMAND",
            match &init {
                RpcServerInit::GenesisFile(_) => "--genesis-file",
                RpcServerInit::GenesisJson(_) => "--genesis-json",
            },
        )
        .replace(
            "$GENESIS_FILE",
            match &init {
                RpcServerInit::GenesisFile(file) => &file,
                RpcServerInit::GenesisJson(_) => "./tests/genesis.json",
            },
        )
        .replace(
            "$GENESIS_VALUE",
            match &init {
                RpcServerInit::GenesisFile(_) => "/genesis.json",
                RpcServerInit::GenesisJson(json) => json.trim_matches('\''),
            },
        );

    let path = vsl_dir.join(DOCKERFILE_NAME);
    if path.exists() {
        let old_contents = String::from_utf8(std::fs::read(&path)?)?;
        if contents == old_contents {
            Ok(())
        } else {
            if force {
                std::fs::write(path, contents);
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Docker-compose file {} already exists. Use it or delete to re-create",
                    DOCKERFILE_NAME
                ))
            }
        }
    } else {
        std::fs::write(path, contents);
        Ok(())
    }
}

/// Stop server using system commands by a server PID.
pub fn stop_local_server() -> Result<String> {
    if !is_docker_installed() {
        return Err(anyhow::anyhow!(
            "No local RPC server is runnig - you need the `docker compose`  plugin installed for that"
        ));
    }
    let vsl_cli_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !vsl_cli_dir.join(DOCKERFILE_NAME).exists() {
        return Ok("No local RPC server is runnig".to_string());
    }
    match Command::new("docker")
        .current_dir(vsl_cli_dir)
        .args(["compose", "-f", DOCKERFILE_NAME, "down"])
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                Ok("Local RPC server is stopped".to_string())
            } else {
                Err(anyhow::anyhow!(
                    "stopping server error:\n{:?}\n{:?}",
                    String::from_utf8(out.stdout),
                    String::from_utf8(out.stderr)
                ))
            }
        }
        Err(err) => Err(anyhow::anyhow!("{}", err)),
    }
}

/// Dump both stdout and stderr with timestamps (if available)
pub fn dump_local_server(lines: u32, all: bool) -> Result<String> {
    if !is_docker_installed() {
        return Err(anyhow::anyhow!(
            "No local RPC server is running - you need the `docker compose` plugin installed for that"
        ));
    }
    let vsl_cli_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !vsl_cli_dir.join(DOCKERFILE_NAME).exists() {
        return Err(anyhow::anyhow!("No local RPC server is runnig"));
    }
    let make_output = |str: String| {
        if all {
            str
        } else {
            str.lines()
                .rev()
                .take(lines as usize)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n")
        }
    };
    match Command::new("docker")
        .current_dir(vsl_cli_dir)
        .args(["compose", "-f", DOCKERFILE_NAME, "logs", "vsl-core"])
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                Ok(format!(
                    "=== STDOUT ===\n{}\n=== STDERR ===\n{}",
                    make_output(String::from_utf8(out.stdout)?),
                    make_output(String::from_utf8(out.stderr)?),
                ))
            } else {
                Err(anyhow::anyhow!(
                    "dumping server error:\n{:?}\n{:?}",
                    String::from_utf8(out.stdout),
                    String::from_utf8(out.stderr)
                ))
            }
        }
        Err(err) => Err(anyhow::anyhow!("{}", err)),
    }
}

/// Check if a server is still running
pub fn local_server_is_running() -> bool {
    let vsl_cli_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !is_docker_installed() || !vsl_cli_dir.join(DOCKERFILE_NAME).exists() {
        return false;
    }
    // Command: `docker ps`
    match Command::new("docker")
        .current_dir(vsl_cli_dir)
        .args(["compose", "-f", DOCKERFILE_NAME, "ps"])
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                match String::from_utf8(out.stdout) {
                    Ok(stdout) => {
                        for line in stdout.lines() {
                            if line.contains("vsl-core") && line.contains("(healthy)") {
                                return true;
                            }
                        }
                        false
                    }
                    Err(_) => false,
                }
            } else {
                false
            }
        }
        Err(err) => false,
    }
}

fn is_docker_installed() -> bool {
    Command::new("docker")
        .args(["compose", "--help"])
        .output()
        .map_or(false, |output| output.status.success())
}

const DOCKERFILE_NAME: &str = "docker-compose.public.local.yml";

const DOCKERFILE_TEMPLATE: &str = r#"
services:
  vsl-core:
    image: ghcr.io/pi-squared-inc/vsl/vsl-core:main
    ports:
      - "44444:44444"
    command:
        - $GENESIS_COMMAND
        - '$GENESIS_VALUE'
        - "--claim-db-path"
        - "/var/lib/vsl/vsl-db"
        - "--tokens-db-path"
        - "/var/lib/vsl/tokens.db"
        $FORCE
    healthcheck:
      test:
        - "CMD"
        - "curl"
        - "-X"
        - "POST"
        - "-H"
        - "Content-Type: application/json"
        - "-d"
        - '{"jsonrpc":"2.0","id":"id","method":"vsl_getHealth"}'
        - "http://localhost:44444"
      interval: 1s
      timeout: 5s
      retries: 30
    volumes:
      - $DB_DIR:/var/lib/vsl
      - $GENESIS_FILE:/genesis.json:ro

  explorer-backend:
    image: ghcr.io/pi-squared-inc/vsl/explorer-backend:main
    network_mode: host
    depends_on:
      vsl-core:
        condition: service_healthy
    volumes:
      - explorer-data:/var/lib/vsl/explorer

  explorer-frontend:
    image: ghcr.io/pi-squared-inc/vsl/explorer-frontend:main
    ports:
      - "4000:4000"
    depends_on:
      vsl-core:
        condition: service_healthy

volumes:
  explorer-data:
"#;
