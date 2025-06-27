use crate::accounts::Account;
use crate::accounts::Accounts;
use crate::accounts::Credentials;
use crate::networks::Network;
use crate::networks::Networks;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use dirs::config_dir;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::path::PathBuf;
use std::time::SystemTime;
use vsl_sdk::rpc_messages::SubmittedClaim;

pub const VSL_TMP_CONFIG: &str = "tmp";

/// Data about an instance of `vsl-core`, launched within `vsl-cli`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpcServerLocal {
    /// Timestamp, when a server was started
    pub started: SystemTime,
    /// The exact command, by which a server was launched
    pub command: Vec<String>,
    /// The `vsl-core` DB directory
    pub db_dir: PathBuf,
}

/// The way RPC server is initialized
pub enum RpcServerInit {
    /// When an initial genesis config is passed as a file
    GenesisFile(String),
    /// When an initial genesis config is passed as a JSON value
    GenesisJson(String),
}

/// The mode of `vsl-cli` running.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CliMode {
    /// After executing a single command the `vsl-cli` application exits
    SingleCommand = 0,
    /// Multy commands may be executed during the `vsl-cli` run. Typically this means REPL mode.
    MultiCommand = 1,
}

/// The database of `vsl-cli` application. Stored persistently.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The user-defined name for a vsl-cli configuration
    pub name: String,
    /// The path to the file, where the config is stored. For the "tmp" config the file is `None`
    pub file: Option<PathBuf>,
    /// DB of known networks.
    pub networks: Networks,
    /// DB of a user accounts.
    accounts: Accounts,
    /// Collection of known etherium addresses
    addresses: HexMap,
    /// Collection of known VSL identifiers
    identifiers: HexMap,
    /// Collection of submitted, but not yet settled claims
    pub submitted: HashMap<String, SubmittedClaim>,
    /// If a local server was started via `vsl-cli`, the info about it is stored here.
    pub server: Option<RpcServerLocal>,
    /// The flag of being in REPL mode
    #[serde(skip, default = "default_mode")]
    pub mode: CliMode,
}

fn default_mode() -> CliMode {
    CliMode::SingleCommand
}

/// The mapping of config names to their locations. Stored persistently.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configs {
    /// Collection of all registred configs: maps the name of a config to its path
    pub configs: HashMap<String, PathBuf>,
    /// The name of the currently used config
    pub current: Option<String>,
}

impl Configs {
    /// Read the `.config/vsl/configs.json`
    pub fn read() -> Result<Configs> {
        let config_dir = vsl_config_dir()?;
        let configs_file = config_dir.join("configs.json");
        if configs_file.exists() {
            serde_json::from_str(&std::fs::read_to_string(configs_file)?)
                .or(Err(anyhow::anyhow!("Configs are corrupted")))
        } else {
            let configs = Configs {
                configs: HashMap::new(),
                current: None,
            };
            Configs::save(&configs)?;
            Ok(configs)
        }
    }
    /// Save the current state of `Configs` to the `.config/vsl/configs.json`
    fn save(configs: &Configs) -> Result<()> {
        let config_dir = vsl_config_dir()?;
        let path = config_dir.join("configs.json");
        // Make sure parent directories exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        // Serialize the config to JSON
        let json = serde_json::to_string_pretty(configs).context("Failed to serialize configs")?;

        // Write the JSON to file, creating it if it doesn't exist
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .context("Failed to open configs file for writing")?;
        file.write_all(json.as_bytes())
            .context("Failed to write configs file")
    }
    /// Create a new configuration with a user provided name and optional path.
    pub fn new(name: String, file: String, overwrite: bool, mode: CliMode) -> Result<Config> {
        if name == VSL_TMP_CONFIG {
            if file != "" {
                return Err(anyhow::anyhow!(
                    "The path to the temporary config makes no sense"
                ));
            }
            Ok(Config::new(name, None, mode))
        } else {
            let mut configs: Configs = Configs::read()?;
            if !overwrite && configs.configs.contains_key(&name) {
                return Err(anyhow::anyhow!("Config '{}' is already created", name));
            }
            let file = if file != "" {
                if !overwrite && std::fs::exists(&file).unwrap_or(false) {
                    return Err(anyhow::anyhow!("Config file '{}' is already created", file));
                }
                PathBuf::from(file)
            } else {
                let file = vsl_config_dir()?.join(format!("{}.json", name));
                if !overwrite && std::fs::exists(&file).unwrap_or(false) {
                    return Err(anyhow::anyhow!(
                        "Config file '{}' is already created",
                        file.to_str().unwrap_or("?")
                    ));
                }
                file
            };
            let new_config = Config::new(name.clone(), Some(file.clone()), mode);
            configs.configs.insert(name.clone(), file);
            configs.current = Some(name);
            Configs::save(&configs)?;
            new_config.save()?;
            Ok(new_config)
        }
    }
    /// Load a previously used configuration by name. If name is omitted, the current is used.
    pub fn load(name: Option<String>, mode: CliMode) -> Result<Config> {
        let configs = Configs::read()?;
        let name = match name {
            Some(name) => Some(name),
            None => configs.current,
        };
        match name {
            None => Ok(Config::new(VSL_TMP_CONFIG.to_string(), None, mode)),
            Some(name) => {
                if name == VSL_TMP_CONFIG {
                    Ok(Config::new(VSL_TMP_CONFIG.to_string(), None, mode))
                } else {
                    match configs.configs.get(&name) {
                        None => Err(anyhow::anyhow!("Configuration {} is not found", name)),
                        Some(path) => {
                            let mut config: Config =
                                serde_json::from_str(&std::fs::read_to_string(path)?)
                                    .or(Err(anyhow::anyhow!("Configs are corrupted")))?;
                            config.mode = mode;
                            Ok(config)
                        }
                    }
                }
            }
        }
    }
    /// Remove the configuration.
    pub fn remove(name: String) -> Result<()> {
        if name == VSL_TMP_CONFIG {
            return Err(anyhow::anyhow!(
                "There's no sense to remove temporary configuration"
            ));
        }
        let mut configs: Configs = Configs::read()?;
        match configs.configs.get(&name) {
            None => Err(anyhow::anyhow!("Configuration {} is not found", name)),
            Some(path) => {
                if !path.exists() {
                    Err(anyhow::anyhow!(
                        "Configuration file {} doesn't exist",
                        path.to_str().unwrap_or("?")
                    ))
                } else {
                    std::fs::remove_file(path);
                    match &configs.current {
                        Some(current) => {
                            if *current == name {
                                configs.current = None;
                            }
                        }
                        _ => {}
                    }
                    configs.configs.remove(&name);
                    Configs::save(&configs);
                    Ok(())
                }
            }
        }
    }
    /// Use the given configuration.
    pub fn use_(name: String) -> Result<()> {
        let mut configs: Configs = Configs::read()?;
        configs.current = Some(name);
        Configs::save(&configs);
        Ok(())
    }
}

impl Config {
    /// Create a new clear configuration
    pub fn new(name: String, file: Option<PathBuf>, mode: CliMode) -> Self {
        Config {
            name: name.clone(),
            file: file,
            networks: Networks::default(),
            accounts: Accounts::default(),
            addresses: HexMap::new("0x", 40),
            identifiers: HexMap::new("(0x)?", 64),
            submitted: HashMap::default(),
            server: None,
            mode: mode,
        }
    }

    /// Save current config to the persistent storage
    pub fn save(&self) -> Result<()> {
        // In case the config is temporary, it is not saved.
        match &self.file {
            Some(path) => {
                // Make sure parent directories exist
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).context("Failed to create config directory")?;
                }

                // Serialize the config to JSON
                let json =
                    serde_json::to_string_pretty(self).context("Failed to serialize config")?;

                // Write the JSON to file, creating it if it doesn't exist
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .context("Failed to open config file for writing")?;
                file.write_all(json.as_bytes())
                    .context("Failed to write config file")
            }
            None => match self.mode {
                CliMode::SingleCommand => Err(anyhow!(
                    "The changed config state cannot be saved: no path is provided.\nPlease make use of some config, `vsl-cli config:create --help`"
                )),
                CliMode::MultiCommand => Ok(()),
            },
        }
    }

    /// Returns the known network `name`.
    pub fn get_network(&mut self, name: Option<String>) -> Result<Network> {
        self.networks.get(name.clone()).ok_or(anyhow::anyhow!(
            "network '{}' is not present",
            name.clone().unwrap_or("default".to_string())
        ))
    }

    /// Add the network.
    pub fn add_network(
        &mut self,
        name: &String,
        url: &Option<String>,
        port: &Option<u32>,
    ) -> Result<Network> {
        self.networks
            .add(name.clone(), url.clone(), port.clone())
            .and_then(|network| {
                self.save()?;
                Ok(network)
            })
    }

    /// Lists all known networks.
    pub fn list_networks(&self) -> Vec<(&String, &Network)> {
        self.networks.list()
    }
    /// Change the current default network name.
    pub fn use_network(&mut self, name: String) -> Result<()> {
        self.networks.set_using(name).and_then(|_| self.save())
    }
    /// Returns the current default network name.
    pub fn using_network(&self) -> String {
        self.networks.get_using()
    }
    /// Updates the network from the database.
    pub fn update_network(
        &mut self,
        name: &String,
        url: &Option<String>,
        port: &Option<u32>,
    ) -> Result<()> {
        self.networks
            .update(name.clone(), url.clone(), port.clone())
            .and_then(|_| self.save())
    }

    /// Remove certain network.
    pub fn remove_network(&mut self, name: &String) -> Result<()> {
        self.networks.remove(name).and_then(|_| self.save())
    }

    /// Generates the credentials from a private key and checks if the private key is not used twice.
    pub fn generate_credentials(&self, private_key_opt: Option<String>) -> Result<Credentials> {
        self.accounts.generate_credentials(private_key_opt)
    }
    /// Creates a new account with label `name`.
    pub fn create_account(
        &mut self,
        name: String,
        credentials: Credentials,
        overwrite: bool,
    ) -> Result<Account> {
        self.accounts
            .create(name.clone(), credentials, overwrite)
            .and_then(|account| {
                self.addresses
                    .add_id(&name, account.credentials.address.clone())?;
                self.save()?;
                Ok(account)
            })
    }
    /// Returns the known account with label `name`.
    pub fn get_account(&mut self, name: Option<&str>) -> Result<Account> {
        self.accounts.get(name)
    }
    /// Sets the default account name.
    pub fn use_account(&mut self, nm: &str) -> Result<()> {
        self.accounts.set_using(nm).and_then(|_| self.save())
    }
    /// Deletes the known account with label `name`.
    pub fn remove_account(&mut self, name: &str) -> Result<()> {
        self.addresses.remove_id(name)?;
        self.accounts.remove(name).and_then(|_| self.save())
    }
    /// Lists all accounts.
    pub fn list_accounts(&self) -> Vec<(&String, &Account)> {
        self.accounts.list()
    }

    pub fn add_address(&mut self, name: &str, address: String) -> Result<()> {
        self.addresses
            .add_id(name, address)
            .and_then(|_| self.save())
    }
    pub fn add_identifier(&mut self, name: &str, address: String) -> Result<()> {
        self.identifiers
            .add_id(name, address)
            .and_then(|_| self.save())
    }
    pub fn lookup_address(&mut self, name: &str) -> Result<String> {
        self.addresses.lookup_id(name)
    }
    pub fn lookup_identifier(&mut self, name: &str) -> Result<String> {
        self.identifiers.lookup_id(name)
    }

    /// Add the submitted claim
    pub fn add_claim(&mut self, submitted: SubmittedClaim, claim_id: String) -> Result<()> {
        let claim = submitted.claim.clone();
        if self.submitted.contains_key(&claim) || self.submitted.contains_key(&claim_id) {
            return Err(anyhow::anyhow!("claim '{}' is already submtted", claim));
        }
        self.submitted.insert(claim.clone(), submitted.clone());
        self.submitted.insert(claim_id.clone(), submitted);
        self.save()
    }
    /// Try to find the submitted claim
    pub fn get_claim(&mut self, claim: &str) -> Result<&SubmittedClaim> {
        self.submitted
            .get(claim)
            .ok_or(anyhow::anyhow!("Claim '{}' was not submitted.", claim))
    }
    /// Check the claim was submitted
    pub fn has_claim(&mut self, claim: &str) -> bool {
        self.submitted.contains_key(claim)
    }
    /// Try to find the submitted claim
    pub fn remove_claim(&mut self, claim: &str) -> Result<()> {
        match self.submitted.remove(claim) {
            Some(_) => {
                self.save()?;
                Ok(())
            }
            None => Err(anyhow::anyhow!("'{}' is not present", claim)),
        }
    }

    /// Checks if the `address` is valid and converts it to the normal form
    pub fn make_valid_address(&self, address: &str) -> Result<String> {
        self.addresses.check_hex_format(address)
    }
    /// Checks if the `identifier` is valid and converts it to the normal form
    pub fn make_valid_identifier(&self, identifier: &str) -> Result<String> {
        self.identifiers.check_hex_format(identifier)
    }

    // Server getter - just a wrapper
    pub fn get_server(&mut self) -> Option<RpcServerLocal> {
        self.server.clone()
    }
    // Server setter - saves the config, if called.
    pub fn set_server(&mut self, server: Option<RpcServerLocal>) -> Result<()> {
        self.server = server;
        self.save()
    }
}

/// The mapping of strings to hex-string IDs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HexMap {
    /// Prefix for a hex string. May be the empty string, or `0x`.
    prefix: String,
    /// The common values for len are: 40 for etherium addresses, 64 for general purpose addresses
    len: u32,
    /// Mapping of arbitrary string to its ID
    str_to_id: HashMap<String, Vec<String>>,
    /// Mapping of an ID to the corresponding arbitrary string
    id_to_str: HashMap<String, String>,
}

impl HexMap {
    pub fn new(prefix: &str, len: u32) -> Self {
        HexMap {
            prefix: prefix.to_string(),
            len: len,
            str_to_id: HashMap::new(),
            id_to_str: HashMap::new(),
        }
    }

    // The common values for len are: 40 for etherium addresses, 64 for general purpose addresses
    pub fn check_hex_format(&self, address: &str) -> Result<String> {
        let address_re =
            Regex::new(&format!(r"^{}[0-9a-fA-F]{{{}}}$", self.prefix, self.len)).unwrap();
        if address_re.is_match(&address) {
            Ok(address.to_ascii_lowercase())
        } else {
            Err(anyhow::anyhow!(
                "hex string '{}' has incorrect format. Must be a hex string of length {} with prefix '{}'",
                address,
                self.len,
                self.prefix
            ))
        }
    }
    /// Add the pair: name - id to the list of known ids
    pub fn add_id(&mut self, name: &str, address: String) -> Result<()> {
        // Check and normalize the hex representation
        let address = self.check_hex_format(&address)?;
        match self.str_to_id.get_mut(name) {
            Some(addrs) => {
                if !addrs.contains(&address) {
                    addrs.push(address)
                }
            }
            None => {
                self.str_to_id
                    .insert(name.to_string(), vec![address.clone()]);
            }
        }
        Ok(())
    }
    /// Try to find the id corresponding to the name
    pub fn lookup_id(&mut self, name: &str) -> Result<String> {
        match self.str_to_id.get(name) {
            Some(addrs) => {
                if addrs.len() == 1 {
                    Ok(addrs
                        .get(0)
                        .expect("vector of addresses must contain exactly one element")
                        .clone())
                } else {
                    self.check_hex_format(name).or(Err(anyhow::anyhow!(
                        "name '{}' has several hexademical IDs associated with it. Please use one of these IDs instead of a name. The list of IDs:\n{}",
                        name,
                        addrs.iter().map(|address| format!("\t{}", address)).collect::<Vec<String>>().join("\n")
                    )))
                }
            }
            None => self.check_hex_format(name),
        }
    }
    /// Try to remove the name from the `str_to_ids` mapping
    pub fn remove_id(&mut self, name: &str) -> Result<()> {
        match self.str_to_id.remove(name) {
            None => Err(anyhow::anyhow!(
                "failed to remove id for the string '{}' - it's not present",
                name
            )),
            Some(_) => Ok(()),
        }
    }
}

/// The directory of persistent storage of `vsl-cli` application.
pub fn vsl_config_dir() -> Result<PathBuf> {
    match config_dir() {
        Some(dir) => {
            let path = dir.join("vsl");
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
            Ok(path)
        }
        None => Err(anyhow::anyhow!("vsl config directory is not found")),
    }
}
