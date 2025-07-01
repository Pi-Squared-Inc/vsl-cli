#![allow(unused)]

use crate::accounts::private_key_to_signer;
use crate::commands::Commands;
use crate::configs::Config;
use crate::configs::Configs;
use crate::configs::RpcServerInit;
use crate::networks::Network;
use crate::rpc_client::RpcClientError;
use crate::rpc_client::RpcClientInterface;
use crate::rpc_client::check_network_is_up;
use crate::rpc_server::dump_local_server;
use crate::rpc_server::init_local_server;
use crate::rpc_server::start_local_server;
use crate::rpc_server::stop_local_server;

use jsonrpsee::core::params::ObjectParams;
use log::info;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use vsl_sdk::IntoSigned as _;
use vsl_sdk::Timestamp;
use vsl_sdk::rpc_messages::CreateAssetMessage;
use vsl_sdk::rpc_messages::IdentifiableClaim;
use vsl_sdk::rpc_messages::PayMessage;
use vsl_sdk::rpc_messages::SetStateMessage;
use vsl_sdk::rpc_messages::SettleClaimMessage;
use vsl_sdk::rpc_messages::SettledVerifiedClaim;
use vsl_sdk::rpc_messages::SubmittedClaim;
use vsl_sdk::rpc_messages::Timestamped;
use vsl_sdk::rpc_messages::TransferAssetMessage;
use vsl_sdk::rpc_messages::ValidatorVerifiedClaim;
use vsl_sdk::rpc_messages::VerifiedClaim;

pub fn execute_command<T: RpcClientInterface>(
    config: &mut Config,
    command: &Commands,
    rpc_client: &mut T,
) -> anyhow::Result<Value, RpcClientError> {
    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match command {
        Commands::ClaimSubmit {
            network,
            claim,
            claim_type,
            proof,
            expires,
            lifetime,
            fee,
        } => {
            let now = Timestamp::now().seconds();
            // Sanity checks
            if config.has_claim(claim) {
                return Err(RpcClientError::GeneralError(format!(
                    "Claim '{}' was already submitted",
                    claim
                )));
            }
            if expires.is_none() && lifetime.is_none() {
                return Err(RpcClientError::GeneralError(format!(
                    "Claim '{}' expiration value is not set: both `--expires` and `--lifetime` are not set",
                    claim
                )));
            } else {
                if expires.is_some() && lifetime.is_some() {
                    return Err(RpcClientError::GeneralError(format!(
                        "Claim '{}' expiration value is set ambiguously: both `--expires` and `--lifetime` are set",
                        claim
                    )));
                } else {
                    match expires {
                        Some(timestamp) => {
                            if *timestamp <= now {
                                return Err(RpcClientError::GeneralError(format!(
                                    "Claim '{}' expiration time {} has already passed",
                                    claim, now
                                )));
                            }
                        }
                        None => {}
                    }
                }
            }
            info!("Submitting claim json: '{}', fee: {}", claim, fee);
            let account = config.get_account(None)?;
            let network = config.get_network(network.clone())?;
            let nonce = rpc_client.get_nonce(network.clone(), &account.credentials.address)?;
            let expires = expires.unwrap_or(now + lifetime.unwrap());
            let to_submit: SubmittedClaim = SubmittedClaim {
                claim: claim.clone(),
                claim_type: claim_type.clone(),
                proof: proof.clone(),
                nonce: nonce.to_string(),
                to: account.signatures.clone(),
                quorum: account.quorum,
                from: account.credentials.address,
                expires: Timestamp::from_seconds(expires),
                fee: to_hex(fee)?,
            };
            let mut params = ObjectParams::new();
            let message_signed = to_submit
                .clone()
                .into_signed(&private_key_to_signer(&account.credentials.private_key))?;
            params.insert("claim", message_signed);
            let response = rpc_client.make_request(network, "vsl_submitClaim", params)?;
            match response {
                Value::String(ref claim_id) => {
                    config.add_claim(to_submit, claim_id.clone())?;
                    config.add_identifier(claim, claim_id.clone())?;
                    Ok(response)
                }
                _ => Err(RpcClientError::GeneralError(format!(
                    "Response to `claim:submit` must be a claim_id (string), got: '{}'",
                    response
                ))),
            }
        }
        Commands::ClaimSettle {
            network,
            claim,
            address,
        } => {
            info!("Settling claim: '{}'", claim);
            let account = config.get_account(None)?;
            let address = match address {
                Some(address) => config.lookup_address(address)?,
                None => account.credentials.address.clone(),
            };
            let network = config.get_network(network.clone())?;
            let nonce = rpc_client.get_nonce(network.clone(), &address)?;
            let submitted = config.get_claim(claim)?;
            let target_claim_id =
                VerifiedClaim::claim_id_hash(&submitted.from, &submitted.nonce, &submitted.claim);
            let message = SettleClaimMessage {
                from: address,
                nonce: nonce.to_string(),
                target_claim_id: target_claim_id.to_string(),
            };
            let message_signed =
                message.into_signed(&private_key_to_signer(&account.credentials.private_key))?;
            let mut params = ObjectParams::new();
            params.insert("settled_claim", message_signed);
            let response = rpc_client.make_request(network, "vsl_settleClaim", params)?;
            match response {
                Value::String(ref claim_id) => {
                    config.remove_claim(claim)?;
                    config.add_identifier(claim, claim_id.clone())?;
                    Ok(response)
                }
                _ => Err(RpcClientError::GeneralError(format!(
                    "Response to `claim:settle` must be a claim_id (string), got: '{}'",
                    response
                ))),
            }
        }
        Commands::ClaimSettled {
            network,
            address,
            since,
            within,
        } => {
            let now = Timestamp::now().seconds();
            if since.is_none() && within.is_none() {
                return Err(RpcClientError::GeneralError(
                    "When quering claim the since value is not set: both `--since` and `--within` are not set".to_string())
                );
            } else {
                if since.is_some() && within.is_some() {
                    return Err(RpcClientError::GeneralError(
                        "since value is set ambiguously: both `--since` and `--within` are set"
                            .to_string(),
                    ));
                } else {
                    match since {
                        Some(timestamp) => {
                            if *timestamp > now {
                                return Err(RpcClientError::GeneralError(format!(
                                    "Since time {} is in the future, query makes no sense",
                                    now
                                )));
                            }
                        }
                        None => {}
                    }
                }
            }
            let account = config.get_account(None)?;
            let address = match address {
                Some(address) => config.lookup_address(address)?,
                None => account.credentials.address,
            };
            let since = since.unwrap_or(now - within.unwrap());
            info!(
                "Fetch verified claims targeted for a client address since a timestamp, address: '{}', since: {}",
                address, since
            );
            let mut params = ObjectParams::new();
            params.insert("address", address)?;
            params.insert("since", Timestamp::from_seconds(since))?;
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_listSettledClaimsForReceiver",
                params,
            )
        }
        Commands::ClaimSubmitted {
            network,
            address,
            since,
            within,
        } => {
            let now = Timestamp::now().seconds();
            if since.is_none() && within.is_none() {
                return Err(RpcClientError::GeneralError(
                    "When quering claim the since value is not set: both `--since` and `--within` are not set".to_string())
                );
            } else {
                if since.is_some() && within.is_some() {
                    return Err(RpcClientError::GeneralError(
                        "since value is set ambiguously: both `--since` and `--within` are set"
                            .to_string(),
                    ));
                } else {
                    match since {
                        Some(timestamp) => {
                            if *timestamp > now {
                                return Err(RpcClientError::GeneralError(format!(
                                    "Since time {} is in the future, query makes no sense",
                                    now
                                )));
                            }
                        }
                        None => {}
                    }
                }
            }
            let account = config.get_account(None)?;
            let address = match address {
                Some(address) => config.lookup_address(address)?,
                None => account.credentials.address,
            };
            let since = since.unwrap_or(now - within.unwrap());
            info!(
                "Fetch verified claims targeted for a client address since a timestamp, address: '{}', since: {}",
                address, since
            );
            let mut params = ObjectParams::new();
            params.insert("address", address)?;
            params.insert("since", Timestamp::from_seconds(since))?;
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_listSubmittedClaimsForReceiver",
                params,
            )
        }
        Commands::ClaimGet { network, id } => {
            info!("Getting settled claim with (claim_id): '{}'", id);
            let mut params = ObjectParams::new();
            let address = config.lookup_identifier(&id)?;
            params.insert("claim_id", address);
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getSettledClaimById",
                params,
            )
        }
        Commands::Pay {
            network,
            to,
            amount,
        } => {
            info!("Making payment: to {} amount {}", to, amount);
            let account = config.get_account(None)?;
            let network = config.get_network(network.clone())?;
            let nonce = rpc_client.get_nonce(network.clone(), &account.credentials.address)?;
            let pay_message = PayMessage {
                from: account.credentials.address,
                to: config.lookup_address(&to)?,
                amount: to_hex(amount)?,
                nonce: nonce.to_string(),
            };
            let message_signed = pay_message
                .into_signed(&private_key_to_signer(&account.credentials.private_key))?;
            let mut params = ObjectParams::new();
            params.insert("payment", message_signed);
            let response = rpc_client.make_request(network, "vsl_pay", params)?;
            match response {
                Value::String(ref claim_id) => Ok(response),
                _ => Err(RpcClientError::GeneralError(format!(
                    "Response to `pay` must be a claim_id (string), got: '{}'",
                    response
                ))),
            }
        }
        Commands::AccountCreate { name, overwrite } => {
            let credentials = config.generate_credentials(None)?;
            let new_account = config.create_account(name.clone(), credentials, *overwrite)?;
            config.use_account(&new_account.name);
            Ok(Value::String(format!(
                "Account {} is created, address: {}",
                name, new_account.credentials.address
            )))
        }
        Commands::AccountLoad {
            name,
            private_key,
            overwrite,
        } => {
            let credentials = config.generate_credentials(private_key.clone())?;
            let new_account = config.create_account(name.clone(), credentials, *overwrite)?;
            config.use_account(&new_account.name);
            Ok(Value::String(format!("Account {} is loaded", name)))
        }
        Commands::AccountExport { name, file } => {
            let account = config.get_account(if name != "" { Some(&name) } else { None })?;
            if file == "" {
                Ok(Value::String(account.credentials.private_key.clone()))
            } else {
                std::fs::write(file, account.credentials.private_key.clone()).map_err(|err| {
                    RpcClientError::GeneralError(format!(
                        "Failed to save the private key to the file: '{}'",
                        file
                    ))
                })?;
                Ok(Value::String(format!(
                    "Account private key is exported to file {}",
                    file
                )))
            }
        }
        Commands::AccountGet { network, account } => {
            let account_id = match account {
                Some(acc) => config.lookup_address(&acc)?,
                None => config.get_account(None)?.credentials.address,
            };
            let mut params = ObjectParams::new();
            params.insert("account_id", account_id);
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getAccount",
                params,
            )
        }
        Commands::AccountBalance { network, account } => {
            let account_id = match account {
                Some(acc) => config.lookup_address(&acc)?,
                None => config.get_account(None)?.credentials.address,
            };
            info!("Getting balance of account: '{}'", account_id);
            let mut params = ObjectParams::new();
            params.insert("account_id", account_id)?;
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getBalance",
                params,
            )
        }
        Commands::AccountStateGet { network, account } => {
            let account_id = match account {
                Some(acc) => config.lookup_address(&acc)?,
                None => config.get_account(None)?.credentials.address,
            };
            info!("Getting balance of account: '{}'", account_id);
            let mut params = ObjectParams::new();
            params.insert("account_id", account_id)?;
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getAccountState",
                params,
            )
        }
        Commands::AccountStateSet {
            network,
            account,
            state,
        } => {
            let account_name = account.as_deref();
            let account = config.get_account(account_name)?;
            let network = config.get_network(network.clone())?;
            let nonce = rpc_client.get_nonce(network.clone(), &account.credentials.address)?;
            info!(
                "Setting state of account: '{}'",
                account.credentials.address
            );
            let message = SetStateMessage {
                from: account.credentials.address.clone(),
                state: state.clone(),
                nonce: nonce.to_string(),
            };
            let message_signed =
                message.into_signed(&private_key_to_signer(&account.credentials.private_key))?;
            let mut params = ObjectParams::new();
            params.insert("state", message_signed)?;
            let response = rpc_client.make_request(network, "vsl_setAccountState", params)?;
            match response {
                Value::String(_) => Ok(response),
                _ => Err(RpcClientError::GeneralError(format!(
                    "Response to `vsl_setAccountState` must be settled claim id, got: '{}'",
                    response
                ))),
            }
        }
        Commands::AccountUse { network, name } => match config.use_account(name) {
            Ok(_) => {
                let account = config.get_account(None)?;
                Ok(Value::String(format!(
                    "Using account {}: {}",
                    name, account.credentials.address
                )))
            }
            Err(err) => Err(RpcClientError::GeneralError(format!(
                "failed to use account '{}': {}",
                name, err
            ))),
        },
        Commands::AccountCurrent { json, table } => {
            if *json && *table {
                Err(RpcClientError::GeneralError(
                    "--table= cannot also be provided when using --json=".to_string(),
                ))
            } else {
                let account = config.get_account(None)?;
                if *json {
                    let value: Value =
                        json!({ "name": account.name, "address": account.credentials.address });
                    Ok(value)
                } else {
                    Ok(Value::String(format!(
                        "  {}: {}",
                        account.name, account.credentials.address
                    )))
                }
            }
        }
        Commands::AccountList { json, table } => {
            if *json && *table {
                Ok(Value::String(
                    "--table= cannot also be provided when using --json=".to_string(),
                ))
            } else {
                let networks = config.list_accounts();
                if *json {
                    let mut json_map = serde_json::Map::new();
                    for (name, account) in networks {
                        json_map.insert(
                            name.clone(),
                            json!({
                                "name": account.name,
                                "address": account.credentials.address
                            }),
                        );
                    }
                    Ok(Value::Object(json_map))
                } else {
                    let mut lines = Vec::new();
                    lines.push(String::from("Available accounts:"));
                    if networks.is_empty() {
                        lines.push(String::from("   No accounts are present."));
                    } else {
                        for (name, account) in networks {
                            lines.push(format!("  {}: {}", name, account.credentials.address));
                        }
                    }
                    Ok(Value::String(lines.join("\n")))
                }
            }
        }
        Commands::AccountRemove { name } => {
            info!("Removing account '{}'", name);
            match config.remove_account(name) {
                Ok(()) => Ok(Value::String(format!("Account '{}' is removed", name))),
                Err(err) => Err(RpcClientError::GeneralError(format!(
                    "Failed to remove the account '{}': {}",
                    name, err
                ))),
            }
        }
        Commands::AssetBalance {
            network,
            asset,
            account,
        } => {
            let account_id = match account {
                Some(acc) => config.lookup_address(&acc)?,
                None => config.get_account(None)?.credentials.address,
            };
            let asset_id = config.lookup_identifier(asset)?;
            info!("Getting balance of asset: '{}'", asset_id);
            let mut params = ObjectParams::new();
            params.insert("account_id", account_id)?;
            params.insert("assert_id", asset_id)?;
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getAssetBalance",
                params,
            )
        }
        Commands::AssetBalances { network, account } => {
            let account_id = match account {
                Some(acc) => config.lookup_identifier(&acc)?,
                None => config.get_account(None)?.credentials.address,
            };
            info!("Getting balances of all assets of: '{}'", account_id);
            let mut params = ObjectParams::new();
            params.insert("account_id", account_id)?;
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getAssetBalances",
                params,
            )
        }
        Commands::AssetCreate {
            network,
            symbol,
            decimals,
            supply,
        } => {
            let account = config.get_account(None)?;
            let network = config.get_network(network.clone())?;
            let nonce = rpc_client.get_nonce(network.clone(), &account.credentials.address)?;
            let message = CreateAssetMessage {
                account_id: account.credentials.address,
                nonce: nonce.to_string(),
                ticker_symbol: symbol.clone(),
                decimals: u8::from_str_radix(&decimals, 10).unwrap(),
                total_supply: to_hex(supply)?,
            };
            let message_signed =
                message.into_signed(&private_key_to_signer(&account.credentials.private_key))?;
            let mut params = ObjectParams::new();
            params.insert("asset_data", message_signed);
            let response = rpc_client.make_request(network, "vsl_createAsset", params)?;
            match response {
                Value::Object(ref map) => match map.get("asset_id") {
                    Some(value) => match value {
                        Value::String(asset_id) => {
                            config.add_identifier(symbol, asset_id.clone())?;
                            Ok(response)
                        }
                        _ => Err(RpcClientError::GeneralError(format!(
                            "Field `asset_id` in response to `asset:create` must be a string, got: '{}'",
                            response
                        ))),
                    },
                    None => Err(RpcClientError::GeneralError(format!(
                        "Response to `asset:create` must contain field asset_id, got: '{}'",
                        response
                    ))),
                },
                _ => Err(RpcClientError::GeneralError(format!(
                    "Response to `asset:create` must be an object, got: '{}'",
                    response
                ))),
            }
        }
        Commands::AssetTransfer {
            network,
            asset,
            to,
            amount,
        } => {
            let account = config.get_account(None)?;
            let network = config.get_network(network.clone())?;
            let nonce = rpc_client.get_nonce(network.clone(), &account.credentials.address)?;
            let message = TransferAssetMessage {
                asset_id: config.lookup_identifier(asset)?,
                from: account.credentials.address,
                to: config.lookup_address(&to)?,
                amount: to_hex(amount)?,
                nonce: nonce.to_string(),
            };
            let message_signed =
                message.into_signed(&private_key_to_signer(&account.credentials.private_key))?;
            let mut params = ObjectParams::new();
            params.insert("transfer_asset", message_signed);
            let response = rpc_client.make_request(network, "vsl_transferAsset", params)?;
            match response {
                Value::String(ref claim_id) => Ok(response),
                _ => Err(RpcClientError::GeneralError(format!(
                    "Response to `asset:transfer` must be a asset_id (string), got: '{}'",
                    response
                ))),
            }
        }
        Commands::AssetGet { network, asset } => {
            let mut params = ObjectParams::new();
            let asset_id = config.lookup_identifier(&asset)?;
            params.insert("asset_id", asset_id);
            rpc_client.make_request(
                config.get_network(network.clone())?,
                "vsl_getAssetById",
                params,
            )
        }
        Commands::HealthCheck { network } => rpc_client.make_request(
            config.get_network(network.clone())?,
            "vsl_getHealth",
            ObjectParams::new(),
        ),
        Commands::NetworkAdd { name, url, port } => match config.add_network(name, url, port) {
            Ok(network) => {
                if check_network_is_up(rpc_client, network.clone()) {
                    config.use_network(name.clone())?;
                }
                Ok(Value::String(format!(
                    "Network {}:{} as '{}' was added",
                    network.url, network.port, network.name
                )))
            }
            Err(err) => Err(RpcClientError::GeneralError(format!(
                "While adding a network: {}",
                err
            ))),
        },
        Commands::NetworkList { json, table } => {
            if *json && *table {
                Err(RpcClientError::GeneralError(
                    "--table= cannot also be provided when using --json=".to_string(),
                ))
            } else {
                let networks = config.list_networks();
                if *json {
                    let mut json_map = serde_json::Map::new();
                    for (name, network) in networks {
                        json_map.insert(
                            name.clone(),
                            json!({
                                "url": network.url,
                                "port": network.port
                            }),
                        );
                    }
                    Ok(Value::Object(json_map))
                } else {
                    let mut lines = Vec::new();
                    lines.push(String::from("Available networks:"));
                    if networks.is_empty() {
                        lines.push(String::from("   No networks are present."));
                    } else {
                        for (name, network) in networks {
                            let status = if check_network_is_up(rpc_client, network.clone()) {
                                "up"
                            } else {
                                "down"
                            };
                            if network.port > 0 {
                                lines.push(format!(
                                    "  {} - {}:{} -- {}",
                                    name, network.url, network.port, status
                                ));
                            } else {
                                lines.push(format!("  {} - {} -- {}", name, network.url, status));
                            }
                        }
                    }
                    Ok(Value::String(lines.join("\n")))
                }
            }
        }
        Commands::NetworkUse { name } => {
            info!("Using network {}", name);
            match config.get_network(Some(name.clone())) {
                Ok(network) => {
                    if check_network_is_up(rpc_client, network) {
                        config.use_network(name.clone())?;
                        Ok(Value::String(format!(
                            "Using the network '{}' as default",
                            name
                        )))
                    } else {
                        Err(RpcClientError::GeneralError(format!(
                            "Network {} is down",
                            name
                        )))
                    }
                }
                Err(err) => Err(RpcClientError::GeneralError(format!(
                    "Network {} is unknown: {}",
                    name, err
                ))),
            }
        }
        Commands::NetworkCurrent { json, table } => {
            if *json && *table {
                Err(RpcClientError::GeneralError(
                    "Cannot use --table and --json at the same time".to_string(),
                ))
            } else {
                match config.get_network(None) {
                    Ok(network) => {
                        let status = if check_network_is_up(rpc_client, network.clone()) {
                            "up"
                        } else {
                            "down"
                        };
                        if *json {
                            let value: Value = json!({ "name": network.name, "url": network.url, "port": network.port, "status": status });
                            Ok(value)
                        } else {
                            Ok(Value::String(format!(
                                "  {}: {}:{} -- {}",
                                network.name, network.url, network.port, status
                            )))
                        }
                    }
                    Err(err) => Ok(Value::String(format!(
                        "  no current network is set. Please add a known network with `network:add` command: {}",
                        err
                    ))),
                }
            }
        }
        Commands::NetworkUpdate { name, url, port } => {
            info!("Updating network {}", name);
            match config.update_network(name, url, port) {
                Ok(()) => {
                    let network = config.get_network(Some(name.clone()))?;
                    let status = if check_network_is_up(rpc_client, network.clone()) {
                        "up"
                    } else {
                        "down"
                    };
                    Ok(Value::String(format!(
                        "Updated: {}:{} as {} -- {}",
                        network.url, network.port, name, status
                    )))
                }
                Err(err) => Err(RpcClientError::GeneralError(format!(
                    "Failed to update network {}: {}",
                    name, err
                ))),
            }
        }
        Commands::NetworkRemove { name } => {
            info!("Removing network '{}'", name);
            match config.remove_network(name) {
                Ok(()) => Ok(Value::String(format!("Network '{}' was removed", name))),
                Err(err) => Err(RpcClientError::GeneralError(format!(
                    "Failed to remove network '{}': {}",
                    name, err
                ))),
            }
        }
        Commands::ServerInit { db, init, force } => {
            let init = match init {
                Some(genesis) => {
                    if std::fs::exists(&genesis).expect("failed to test on existance of a file") {
                        // The `genesis` points to an existent file - decide that it's a file
                        RpcServerInit::GenesisFile(genesis.clone())
                    } else {
                        let genesis = genesis.trim_matches('\'').trim_matches('"').trim();
                        if genesis.starts_with("{") && genesis.ends_with("}") {
                            // The `genesis` looks like a json object - decide that it's a json
                            RpcServerInit::GenesisJson(genesis.to_string())
                        } else {
                            return Err(RpcClientError::GeneralError(format!(
                                "The `--genesis` argument must be either a file or a json value, got: {}",
                                genesis
                            )));
                        }
                    }
                }
                None => RpcServerInit::None,
            };
            let local_network = Network::default();
            if config.get_server().is_some() {
                Ok(Value::String("Local RPC server is already up".to_string()))
            } else if check_network_is_up(rpc_client, local_network.clone()) {
                Ok(Value::String("Local RPC server is already up".to_string()))
            } else {
                info!("starting vsl-core (server)...");
                let (new_server, opt_tempdir) = init_local_server(&db, init, *force)?;
                config.set_server(Some(new_server));
                if let Some(tempdir) = opt_tempdir {
                    Ok(Value::String(format!(
                        "Local RPC server is initialized and spawned, created temp db directory: {}",
                        tempdir.keep().display()
                    )))
                } else {
                    Ok(Value::String(format!(
                        "Local RPC server is initialized and spawned"
                    )))
                }
            }
        }
        Commands::ServerStart {} => {
            let local_network = Network::default();
            if check_network_is_up(rpc_client, local_network.clone()) {
                Ok(Value::String("Local RPC server is already up".to_string()))
            } else {
                info!("starting vsl-core (server)...");
                start_local_server()?;
                Ok(Value::String(format!("Local RPC server is started")))
            }
        }
        Commands::ServerDump { lines, all } => Ok(Value::String(dump_local_server(*lines, *all)?)),
        Commands::ServerStop {} => match stop_local_server() {
            Ok(val) => Ok(Value::String(val)),
            Err(err) => Err(RpcClientError::GeneralError(format!(
                "Failed to stop process: {}",
                err
            ))),
        },
        Commands::Repl {
            print_commands,
            tmp_config,
        } => Err(RpcClientError::GeneralError(
            "Cannot start REPL from within REPL".to_string(),
        )),
        Commands::ConfigCreate {
            name,
            copy,
            file,
            overwrite,
        } => {
            let mut new_config = Configs::new(name.clone(), file.clone(), *overwrite, config.mode)?;
            if copy != "" {
                let old_config = Configs::load(Some(copy.clone()), config.mode)?;
                new_config = old_config;
            }
            Ok(Value::String(format!(
                "The configuration {} is created",
                name
            )))
        }
        Commands::ConfigUse { name } => {
            *config = Configs::load(Some(name.clone()), config.mode)?;
            Configs::use_(name.clone());
            Ok(Value::String(format!("Using configuration {}", name)))
        }
        Commands::ConfigCurrent {} => {
            let configs = Configs::read()?;
            match configs.current {
                Some(current) => {
                    let path =
                        configs
                            .configs
                            .get(&current)
                            .ok_or(RpcClientError::GeneralError(format!(
                                "Config '{}' has no corresponding path",
                                current
                            )))?;
                    Ok(Value::String(format!(
                        "Current configuration: {} at {}",
                        current,
                        path.to_str().unwrap_or("?")
                    )))
                }
                None => Ok(Value::String(format!("No active current configuration"))),
            }
        }
        Commands::ConfigList { json, table } => {
            if *json && *table {
                Err(RpcClientError::GeneralError(
                    "--table= cannot also be provided when using --json=".to_string(),
                ))
            } else {
                let configs = Configs::read()?;
                if *json {
                    let mut json_map = serde_json::Map::new();
                    for (name, path) in configs.configs {
                        json_map.insert(
                            name.clone(),
                            json!({
                                "name": name,
                                "path": path.to_str().unwrap_or("?")
                            }),
                        );
                    }
                    Ok(Value::Object(json_map))
                } else {
                    let mut lines = Vec::new();
                    lines.push(String::from("Available configurations:"));
                    if configs.configs.is_empty() {
                        lines.push(String::from("   No configurations are present."));
                    } else {
                        for (name, path) in configs.configs {
                            lines.push(format!("  {} at {}", name, path.to_str().unwrap_or("?")));
                        }
                    }
                    Ok(Value::String(lines.join("\n")))
                }
            }
        }
        Commands::ConfigRempove { name } => {
            Configs::remove(name.clone())?;
            Ok(Value::String(format!("Configuration {} was removed", name)))
        }
    }
}

/// Converts the argument, which may be hexadecimal or decimal to a hexadecimal representation
fn to_hex(s: &str) -> Result<String, RpcClientError> {
    if s.starts_with("0x") {
        // Detect the hex input
        if u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).is_ok() {
            // Return as is
            Ok(s.to_string())
        } else {
            Err(RpcClientError::GeneralError(format!(
                "Invalid number format: {}, must be a hexadecimal or decimal integer",
                s
            )))
        }
    } else {
        // Decimal input, parsing as decimal
        if let Ok(num) = s.parse::<u64>() {
            // Convert to hexadecimal
            Ok(format!("0x{:x}", num))
        } else {
            Err(RpcClientError::GeneralError(format!(
                "Invalid number format: {}, must be a hexadecimal or decimal integer",
                s
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::to_hex;
    #[test]
    fn test_to_hex() {
        assert!(to_hex("0x100").expect("must be correct input to 'to_hex'") == "0x100");
        assert!(to_hex("100").expect("must be correct input to 'to_hex'") == "0x64");
        assert!(
            to_hex("1234567890123456").expect("must be correct input to 'to_hex'")
                == "0x462d53c8abac0"
        );
        assert!(
            to_hex("12345678901234567").expect("must be correct input to 'to_hex'")
                == "0x2bdc545d6b4b87"
        );
        assert!(
            to_hex("0x462d53c8abac0").expect("must be correct input to 'to_hex'")
                == "0x462d53c8abac0"
        );
        assert!(
            to_hex("0x2bdc545d6b4b87").expect("must be correct input to 'to_hex'")
                == "0x2bdc545d6b4b87"
        );
    }
}
