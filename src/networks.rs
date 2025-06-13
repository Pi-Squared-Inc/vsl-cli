use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use vsl_utils::PORT;

/// The simple representation of a URL
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    pub name: String,
    pub url: String,
    pub port: u32,
}

pub const VSL_CLI_DEFAULT_NETWORK_URL: &str = "http://localhost";
pub const VSL_CLI_DEFAULT_NETWORK_PORT: u32 = PORT;

impl Default for Network {
    fn default() -> Self {
        Network {
            name: "default".to_string(),
            url: VSL_CLI_DEFAULT_NETWORK_URL.to_string(),
            port: VSL_CLI_DEFAULT_NETWORK_PORT,
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -- {}:{}", self.name, self.url, self.port)
    }
}

/// The little DB of networks.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Networks {
    /// List of known networks.
    known: HashMap<String, Network>,
    /// The default network name.
    current: String,
}

impl Default for Networks {
    fn default() -> Self {
        let default = Network::default();
        let mut known = HashMap::new();
        known.insert(default.name.clone(), default.clone());
        Networks {
            known: known,
            current: default.name,
        }
    }
}

impl Networks {
    /// Returns the known network `name`.
    pub fn get(&mut self, name: Option<String>) -> Option<Network> {
        match name {
            Some(name) => {
                // In case the name is explicity given - lookup the network
                self.known.get(&name).map(|network| network.clone())
            }
            None => {
                // Otherwise check for a default network. If it's present - return it.
                if self.known.contains_key(&self.current) {
                    self.known.get(&self.current).map(|network| network.clone())
                } else {
                    // No default network is found - we'll create it.
                    if self.current == "" {
                        // In case the default name if not set - do it.
                        self.current = "default".to_string();
                    }
                    // Try to find the `default` network among existing
                    if self.known.contains_key(&self.current) {
                        self.known.get(&self.current).map(|network| network.clone())
                    } else {
                        // Finally add the `default` network.
                        self.add(self.current.clone(), None, None).ok()
                    }
                }
            }
        }
    }

    /// Add the network.
    pub fn add(&mut self, name: String, url: Option<String>, port: Option<u32>) -> Result<Network> {
        if self.known.contains_key(&name) {
            return Err(anyhow::anyhow!("'{}' is already present", name));
        }
        let network = Network {
            name: name.clone(),
            url: url.unwrap_or(VSL_CLI_DEFAULT_NETWORK_URL.to_string()),
            port: port.unwrap_or(VSL_CLI_DEFAULT_NETWORK_PORT),
        };
        self.known.insert(name.clone(), network.clone());
        Ok(network)
    }

    /// Lists all known networks.
    pub fn list(&self) -> Vec<(&String, &Network)> {
        self.known.iter().collect()
    }

    /// Change the current default network name.
    pub fn set_using(&mut self, name: String) -> Result<()> {
        if !self.known.contains_key(&name) {
            return Err(anyhow::anyhow!("'{}' is not present", name));
        }
        self.current = name;
        Ok(())
    }

    /// Returns the current default network name.
    pub fn get_using(&self) -> String {
        self.current.clone()
    }

    /// Updates the network from the database.
    pub fn update(&mut self, name: String, url: Option<String>, port: Option<u32>) -> Result<()> {
        if !self.known.contains_key(&name) {
            return Err(anyhow::anyhow!("'{}' is absent", name));
        }
        let network = self.known.get(&name).context("network is not present")?;
        let updated = Network {
            name: name.clone(),
            url: url.unwrap_or(network.url.clone()),
            port: port.unwrap_or(network.port),
        };
        self.known.insert(name, updated);
        Ok(())
    }

    /// Remove a network.
    pub fn remove(&mut self, name: &String) -> Result<()> {
        if !self.known.contains_key(name) {
            return Err(anyhow::anyhow!("'{}' is absent", name));
        }
        self.known.remove(name);
        Ok(())
    }
}
