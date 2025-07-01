#![allow(unused)]

use crate::networks::Network;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::client::error::Error as ConnectionError;
use jsonrpsee::core::params::ObjectParams;
use jsonrpsee::http_client::HttpClient;
use log::debug;
use log::error;
use log::info;
use log::warn;
use serde_json::Value;
use std::collections::HashMap;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct RpcClient {
    /// The list of active connections
    connections: HashMap<String, HttpClient>,
    /// The network which is used currently
    active: String,
    /// The runtime to perform async calls
    runtime: Runtime,
}

impl RpcClient {
    pub fn new() -> Self {
        RpcClient {
            connections: HashMap::new(),
            active: String::new(),
            runtime: Runtime::new().unwrap(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    #[error("Network: {0} is unknown. Please specify it with `network:add` command.")]
    NetworkIsAbsent(String),
    #[error("Request is incorrect: {0}")]
    IncorrectRequest(String),
    #[error("Response is incorrect: {0}")]
    IncorrectResponse(String),
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("{0}")]
    GeneralError(String),
}

impl From<serde_json::Error> for RpcClientError {
    fn from(err: serde_json::Error) -> Self {
        RpcClientError::GeneralError(err.to_string())
    }
}

impl From<anyhow::Error> for RpcClientError {
    fn from(err: anyhow::Error) -> Self {
        RpcClientError::GeneralError(err.to_string())
    }
}

impl From<alloy::signers::Error> for RpcClientError {
    fn from(err: alloy::signers::Error) -> Self {
        RpcClientError::GeneralError(err.to_string())
    }
}

// The RPC client trait (interface)
pub trait RpcClientInterface {
    fn close_connection(&mut self, network: &str);
    fn active_connection(&self) -> String;
    fn get_nonce(&mut self, network: Network, address: &str) -> Result<u64, RpcClientError>;
    #[allow(async_fn_in_trait)]
    fn make_request(
        &mut self,
        network: Network,
        meth: &str,
        params: ObjectParams,
    ) -> Result<Value, RpcClientError>;
}

impl RpcClientInterface for RpcClient {
    fn close_connection(&mut self, name: &str) {
        self.connections.remove(name);
    }
    fn active_connection(&self) -> String {
        self.active.clone()
    }
    fn get_nonce(&mut self, network: Network, address: &str) -> Result<u64, RpcClientError> {
        let mut params = ObjectParams::new();
        params.insert("account_id", address)?;
        let response = self.make_request(network, "vsl_getAccountNonce", params)?;
        match &response {
            Value::Number(num) => num
                .as_u64()
                .ok_or(RpcClientError::IncorrectResponse(format!(
                    "must return an nonce integer value, got: {}",
                    response
                ))),
            _ => Err(RpcClientError::IncorrectResponse(format!(
                "must return an nonce integer value, got: {}",
                response
            ))),
        }
    }
    fn make_request(
        &mut self,
        network: Network,
        meth: &str,
        params: ObjectParams,
    ) -> Result<Value, RpcClientError> {
        self.runtime.handle().clone().block_on(async {
            match self.get_connection(network) {
                Ok(conn) => match conn.request::<Value, ObjectParams>(meth, params).await {
                    Ok(response) => Ok(response),
                    Err(err) => Err(RpcClientError::ConnectionError(err)),
                },
                Err(err) => Err(err),
            }
        })
    }
}

impl RpcClient {
    pub fn get_connection(
        &mut self,
        network: Network,
    ) -> anyhow::Result<&HttpClient, RpcClientError> {
        debug!("connection_name: {} ...", network.name);
        if self.connections.contains_key(&network.name) {
            info!("Using connection: {} ...", &network.name);
            self.active = network.name.clone();
            return Ok(self.connections.get(&network.name).unwrap());
        }
        let url = if network.port != 0 {
            format!("{}:{}", network.url, network.port)
        } else {
            network.url.clone()
        };
        debug!("Starting connection to: {} ...", &url);
        match HttpClient::builder().build(url.clone()) {
            Ok(client) => {
                info!("Connection to {} is established", &url);
                self.connections.insert(network.name.clone(), client);
                self.active = network.name.clone();
                Ok(self.connections.get(&network.name).unwrap())
            }
            Err(err) => {
                error!("{}", err);
                Err(RpcClientError::ConnectionError(err))
            }
        }
    }
}

pub fn check_network_is_up<T: RpcClientInterface>(rpc_client: &mut T, network: Network) -> bool {
    let response = rpc_client.make_request(network, "vsl_getHealth", ObjectParams::new());
    match response {
        Ok(value) => match value {
            Value::String(str) => str == "ok",
            _ => false,
        },
        Err(_) => false,
    }
}
