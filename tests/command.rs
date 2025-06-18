#![allow(unused)]

use vsl_cli::commands::Commands;
use vsl_cli::configs::CliMode;
use vsl_cli::configs::Configs;
use vsl_cli::execute::execute_command;
use vsl_cli::rpc_client::RpcClient;
use vsl_sdk::rpc_messages::IdentifiableClaim as _;
use vsl_sdk::rpc_messages::VerifiedClaim;

const CLIENT: &str = "0xDB4a76394D34E39802ee169Ec9527b9223A16f0F";

#[test]
fn test_endpoints() -> anyhow::Result<()> {
    let network = String::from("default_localhost");
    exec_command(Commands::ClaimSubmit {
        network: None,
        claim: "***All men are liars".to_string(),
        claim_type: "logical".to_string(),
        proof: "Obvious".to_string(),
        expires: None,
        lifetime: Some(3600),
        fee: "1".to_string(),
    });
    exec_command(Commands::ClaimSettle {
        network: None,
        claim: "***All men are liars".to_string(),
        address: Some(String::new()),
    });
    exec_command(Commands::ClaimSettled {
        network: None,
        address: Some(CLIENT.to_string()),
        since: None,
        within: Some(3600),
    });
    exec_command(Commands::ClaimGet {
        network: None,
        id: VerifiedClaim::claim_id_hash(CLIENT, "1234", "***All men are liars").to_string(),
    });
    Ok(())
}

fn exec_command(comm: Commands) {
    let mut config = Configs::new(
        "tmp".to_string(),
        String::new(),
        false,
        CliMode::MultiCommand,
    )
    .unwrap();
    let mut client = RpcClient::new();
    // TODO: enable checks here. Currently the correct responses are not returned, so
    // checking has no sense.
    execute_command(&mut config, &comm, &mut client);
}
