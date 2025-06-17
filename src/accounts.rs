#![allow(unused)]

use alloy::signers::k256::SecretKey;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use log::debug;
use rand::thread_rng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use vsl_utils::private_key_to_public;
use vsl_utils::private_key_to_signer;

/// The primary data of an account: a private key and
/// corresponding address.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credentials {
    /// The VSL address of a client, corresponding to the private key.
    pub address: String,
    /// The private part of an account
    /// WARNING !!! PRIVATE KEYS ARE STORED AS IS !!! ONLY FOR THE DEVNET PURPOSES !!!
    pub private_key: String,
}

/// The simple representation of a VSL account, with private data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    // The user-defined label for the key
    pub name: String,
    /// The set of verifiers for this account
    pub signatures: Vec<String>,
    /// the minimum quorum of signatures
    pub quorum: u16,
    /// The private key and address
    pub credentials: Credentials,
}

fn generate_private_key() -> String {
    let mut rng = thread_rng();
    // Generate a random secret key
    let secret_key = SecretKey::random(&mut rng);
    hex::encode(secret_key.to_bytes()).to_ascii_lowercase()
}

impl Account {
    fn new(name: String, credentials: Credentials, mut verifiers: Vec<String>) -> Self {
        // Add the deafult verifier - the one with the address from the signer
        verifiers.push(credentials.address.clone());
        Account {
            name: name,
            signatures: verifiers,
            quorum: 1,
            credentials: credentials,
        }
    }
}

/// The database of user accounts.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Accounts {
    /// Accounts storage
    accounts: HashMap<String, Account>,
    /// The default account name
    using: String,
}

impl Accounts {
    /// Generates the credentials from a private key and checks if the private key is not used twice.
    pub fn generate_credentials(&self, private_key: Option<String>) -> Result<Credentials> {
        let private_key = match private_key {
            Some(key) => {
                let trimmed = key.strip_prefix("0x").unwrap_or(&key).to_string();
                if !trimmed.is_empty()
                    && trimmed.len() >= 64
                    && trimmed.chars().all(|c| c.is_ascii_hexdigit())
                {
                    trimmed.to_string()
                } else {
                    use std::io::Read;
                    let mut key_str = String::new();
                    std::fs::File::open(key)
                        .or(Err(anyhow::anyhow!(
                            "The private key file: {} doesn't exist",
                            trimmed
                        )))?
                        .read_to_string(&mut key_str)
                        .or(Err(anyhow::anyhow!(
                            "Failed to read private key file: {}",
                            trimmed
                        )))?;
                    key_str
                }
            }
            None => generate_private_key(),
        };
        if self
            .accounts
            .iter()
            .find(|(_, account)| account.credentials.private_key == private_key)
            .is_some()
        {
            return Err(anyhow::anyhow!("This private key is already registred"));
        }
        Ok(Credentials {
            private_key: private_key.clone(),
            address: private_key_to_signer(&private_key)
                .address()
                .to_string()
                .to_ascii_lowercase(),
        })
    }
    /// Creates a new account with label `name` with a given private key.
    pub fn create(
        &mut self,
        name: String,
        credentials: Credentials,
        owerrwrite: bool,
    ) -> Result<Account> {
        if !owerrwrite && self.accounts.contains_key(&name) {
            return Err(anyhow::anyhow!("'{}' is already present", name));
        }
        // TODO: pass the set verifiers as well??
        let account = Account::new(name.clone(), credentials, Vec::new());
        self.accounts.insert(name.clone(), account);
        self.using = name.clone();
        self.get(Some(&name))
    }
    /// Returns the known account with label `name`.
    pub fn get(&mut self, nm: Option<&str>) -> Result<Account> {
        let name = nm.unwrap_or(&self.using);
        if name == "" {
            return Err(anyhow::anyhow!(
                "Currently not using any account. Please create an account first."
            ));
        }
        self.accounts
            .get(name)
            .ok_or(anyhow::anyhow!("account '{}' in not found", name))
            .cloned()
    }
    /// Sets the default account name.
    pub fn set_using(&mut self, name: &str) -> Result<()> {
        if !self.accounts.contains_key(name) {
            return Err(anyhow::anyhow!("'{}' is not present", name));
        }
        self.using = name.to_string();
        Ok(())
    }
    /// Deletes the known account with label `name`.
    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.accounts
            .remove(name)
            .ok_or(anyhow::anyhow!("failed to remove account '{}'", name))
            .map(|_| {})
    }
    /// Lists all accounts.
    pub fn list(&self) -> Vec<(&String, &Account)> {
        self.accounts.iter().collect()
    }
}
