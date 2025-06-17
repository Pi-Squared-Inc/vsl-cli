#![allow(unused)]

use jsonrpsee::core::client::error::Error as ConnectionError;
use jsonrpsee::core::params::ObjectParams;
use serde_json::Value;
use std::collections::HashMap;
use vsl_cli::networks;
use vsl_cli::networks::Network;
use vsl_cli::networks::Networks;
use vsl_cli::rpc_client::RpcClientError;
use vsl_cli::rpc_client::RpcClientInterface;
use vsl_utils::PORT;

// Mock implementation for testing
#[derive(Debug, Default)]
pub struct MockRpcClient {
    pub connections: HashMap<String, String>,
    pub should_fail_connection: bool,
    pub should_fail_network: bool,
}

impl MockRpcClient {
    pub fn new(should_fail_connection: bool, should_fail_network: bool) -> Self {
        MockRpcClient {
            connections: HashMap::new(),
            should_fail_connection,
            should_fail_network,
        }
    }
}

impl RpcClientInterface for MockRpcClient {
    fn get_nonce(&mut self, network: Network, address: &str) -> Result<u64, RpcClientError> {
        Ok(0)
    }
    fn make_request(
        &mut self,
        network: Network,
        meth: &str,
        params: ObjectParams,
    ) -> Result<Value, RpcClientError> {
        let connection_name = network.name;
        println!("[MOCK] Requesting connection: {} ...", &connection_name);

        // Test case: simulate network absence
        if self.should_fail_network {
            println!("[MOCK] Simulating network absence");
            return Err(RpcClientError::NetworkIsAbsent("test".to_string()));
        }

        // Reuse existing connection if available
        if self.connections.contains_key(&connection_name) {
            println!("[MOCK] Reusing mock connection: {}", &connection_name);
            return Ok(Value::from(
                self.connections.get(&connection_name).unwrap().clone(),
            ));
        }

        // Test case: simulate connection error
        if self.should_fail_connection {
            println!("[MOCK] Simulating connection error");
            return Err(RpcClientError::ConnectionError(ConnectionError::Custom(
                "test".to_string(),
            )));
        }

        // Create a new mock connection

        let url = format!("{}:{}", network.url, network.port);
        println!("[MOCK] Creating mock connection to: {}", &url);

        self.connections
            .insert(connection_name.clone(), String::from("???"));
        Ok(Value::from(
            self.connections.get(&connection_name).unwrap().clone(),
        ))
    }

    fn close_connection(&mut self, name: &str) {
        println!("[MOCK] Removing connection: {}", name);
        self.connections.remove(name);
    }
    fn active_connection(&self) -> String {
        "active".to_string()
    }
}

// Usage example for testing
#[test]
fn test_rpc_client_with_mock() {
    // Create a test configuration
    let mut networks = Networks::default();
    networks.add("default_network".to_string(), None, None);

    // Test successful case
    let mut mock_client = MockRpcClient::new(false, false);
    let mut params = ObjectParams::new();
    params.insert("address", "vsl0x00");
    let result = mock_client.make_request(
        networks.get(None).expect("default network is not present"),
        "vsl_getBalance",
        params,
    );
    assert!(result.is_ok());
}
