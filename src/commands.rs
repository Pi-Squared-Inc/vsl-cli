use crate::networks::VSL_CLI_DEFAULT_NETWORK_PORT;
use crate::networks::VSL_CLI_DEFAULT_NETWORK_URL;

use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(subcommand_help_heading = "Claim management commands")]
    /// Request verification of a claim
    #[command(name = "claim:submit")]
    ClaimSubmit {
        /// the claim to be submitted
        #[arg(help = "Request verification of a claim")]
        claim: String,
        /// the claim type
        #[arg(short = 't', long = "type", default_value = "")]
        claim_type: String,
        /// the proof of the claim
        #[arg(short, long, default_value = "")]
        proof: String,
        /// The expiration timestamp, when the submitted claim will be erased.
        #[arg(short, long, default_value = None)]
        expires: Option<u64>,
        /// how much the claim is considered alive after creation, seconds. Default is 1 hour.
        #[arg(short, long, default_value = "3600")]
        lifetime: Option<u64>,
        /// the total fee for verification and claim validation. Must be non-negative integer.
        #[arg(short, long, default_value = "0x1")]
        fee: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Submit a verified claim for validation only
    #[command(name = "claim:settle")]
    ClaimSettle {
        /// The claim to be settled
        #[arg(help = "Submit a verified claim for validation only")]
        claim: String,
        /// Client address, to whom the claim is settled. By default the current account address is used.
        #[arg(short, long, default_value = None)]
        address: Option<String>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Fetch verification request claims targeted for a verifier address since a timestamp
    #[command(name = "claim:submitted")]
    ClaimSubmitted {
        /// Client address. By default the current account address is used.
        #[arg(short, long, default_value = None)]
        address: Option<String>,
        /// Since the timestamp
        #[arg(short, long, default_value = None)]
        since: Option<u64>,
        /// Within a certain number seconds before now. Default value is 1 hour.
        #[arg(short, long, default_value = "3600")]
        within: Option<u64>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Fetch verified claims targeted for a client address since a timestamp
    #[command(name = "claim:settled")]
    ClaimSettled {
        /// Client address. By default the current account addess is used.
        #[arg(short, long, default_value = None)]
        address: Option<String>,
        /// Since the timestamp
        #[arg(short, long, default_value = None)]
        since: Option<u64>,
        /// Within a certain number seconds before now. Default value is 1 hour.
        #[arg(short, long, default_value = "3600")]
        within: Option<u64>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Fetch a claim and its metadata by its ID
    #[command(name = "claim:get")]
    ClaimGet {
        /// JSON of the claim to query
        #[arg(help = "Fetch a claim and its metadata by its ID")]
        id: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },

    #[command(subcommand_help_heading = "Payment commands")]
    /// Transfer funds to another account.
    Pay {
        /// Recipient of the transfer
        #[arg(short, long)]
        to: String,
        /// Amount to transfer
        #[arg(short, long)]
        amount: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },

    #[command(subcommand_help_heading = "Account management commands")]
    /// Generates a new account in VSL.
    #[command(name = "account:create")]
    AccountCreate {
        /// Account name
        name: String,
        /// Overwrite the existing account
        #[arg(short, long, default_value_t = false)]
        overwrite: bool,
    },
    /// Makes use of an existing account with a provided private key.
    #[command(name = "account:load")]
    AccountLoad {
        /// Account name
        name: String,
        /// Account private key. May be a private key itself, or a path to a file with private key.
        #[arg(short, long, default_value = None)]
        private_key: Option<String>,
        /// Overwrite the existing account
        #[arg(short, long, default_value_t = false)]
        overwrite: bool,
    },
    /// Exports the accounts private key.
    #[command(name = "account:export")]
    AccountExport {
        /// Account name, optional. If ommited, the current is exported.
        #[arg(short, long, default_value = "")]
        name: String,
        /// The target file, where the private key would be written, optional. Otherwise, the private key will be shown in console.
        #[arg(short, long, default_value = "")]
        file: String,
    },
    /// Fetches the information about account.
    #[command(name = "account:get")]
    AccountGet {
        // Account in the form of hex string
        #[arg(default_value = None)]
        account: Option<String>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Ask for the balance of an account
    #[command(name = "account:balance")]
    AccountBalance {
        // Account in the form of hex string
        #[arg(default_value = None)]
        account: Option<String>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Ask for the state of an account
    #[command(name = "account:state-get")]
    AccountStateGet {
        // Account in the form of hex string
        #[arg(default_value = None)]
        account: Option<String>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Update for the state of an account
    #[command(name = "account:state-set")]
    AccountStateSet {
        /// Account in the form of hex string
        #[arg(short, long, default_value = None)]
        account: Option<String>,
        /// The new account state, hex string
        state: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Switches to another account.
    #[command(name = "account:use")]
    AccountUse {
        /// Account name
        name: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Prints data of current account.
    #[command(name = "account:current")]
    AccountCurrent {
        #[arg(long, help = "Display data in a json structure.", default_value_t = false, action = ArgAction::SetTrue)]
        json: bool,
        #[arg(long, help = "Display data in a table structure.", default_value_t = false, action = ArgAction::SetTrue)]
        table: bool,
    },
    /// Lists all available accounts.
    #[command(name = "account:list")]
    AccountList {
        #[arg(long, help = "Display data in a json structure.", default_value_t = false, action = ArgAction::SetTrue)]
        json: bool,
        #[arg(long, help = "Display data in a table structure.", default_value_t = false, action = ArgAction::SetTrue)]
        table: bool,
    },
    /// Delete the account.
    #[command(name = "account:remove")]
    AccountRemove {
        /// Account name
        name: String,
    },

    #[command(subcommand_help_heading = "Asset management commands")]
    /// Ask for the balance of an asset for the account
    #[command(name = "asset:balance")]
    AssetBalance {
        // Asset in the form of hex string
        asset: String,
        // Account in the form of hex string
        #[arg(short, long, default_value = None)]
        account: Option<String>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Ask for the balance of all assets for the account
    #[command(name = "asset:balances")]
    AssetBalances {
        // Account in the form of hex string
        #[arg(short, long, default_value = None)]
        account: Option<String>,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Creates a new native asset.
    #[command(name = "asset:create")]
    AssetCreate {
        /// Name of the asset
        #[arg(long)]
        symbol: String,
        /// Number of decimals used for this asset
        #[arg(long, default_value = "18")]
        decimals: String,
        /// Total number of tokens that exist
        #[arg(long)]
        supply: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// The transfer of an asset.
    #[command(name = "asset:transfer")]
    AssetTransfer {
        /// Name of the asset
        #[arg(long)]
        asset: String,
        /// Account name
        #[arg(long)]
        to: String,
        #[arg(long)]
        /// Total number of tokens that exist
        amount: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },
    /// Get the information about an asset.
    #[command(name = "asset:get")]
    AssetGet {
        /// Name of the asset
        asset: String,
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },

    /// Request the health info about a node
    #[command(name = "health:check")]
    HealthCheck {
        /// URL to connect to, or name of a known network
        #[arg(short, long, default_value = None)]
        network: Option<String>,
    },

    #[command(subcommand_help_heading = "Network management commands")]
    /// Add network
    #[command(name = "network:add")]
    NetworkAdd {
        #[arg(help = "Name of the network to add")]
        name: String,
        #[arg(short, long, default_value = Some(VSL_CLI_DEFAULT_NETWORK_URL))]
        url: Option<String>,
        #[arg(short, long, default_value = VSL_CLI_DEFAULT_NETWORK_PORT.to_string())]
        port: Option<u32>,
    },
    /// List all known networks
    #[command(name = "network:list")]
    NetworkList {
        #[arg(long, help = "Display data in a json structure.", default_value_t = false, action = ArgAction::SetTrue)]
        json: bool,
        #[arg(long, help = "Display data in a table structure.", default_value_t = false, action = ArgAction::SetTrue)]
        table: bool,
    },
    /// Use a selected network as default
    #[command(name = "network:use")]
    NetworkUse {
        #[arg(help = "The network name which is used by default")]
        name: String,
    },
    /// Prints the current default network status
    #[command(name = "network:current")]
    NetworkCurrent {
        #[arg(long, help = "Display data in a json structure.", default_value_t = false, action = ArgAction::SetTrue)]
        json: bool,
        #[arg(long, help = "Display data in a table structure.", default_value_t = false, action = ArgAction::SetTrue)]
        table: bool,
    },
    /// Update a known network
    #[command(name = "network:update")]
    NetworkUpdate {
        #[arg(help = "Name of the network to update. Default is the currently used network.")]
        name: String,
        #[arg(short, long, default_value = Some(VSL_CLI_DEFAULT_NETWORK_URL))]
        url: Option<String>,
        #[arg(short, long, default_value = VSL_CLI_DEFAULT_NETWORK_PORT.to_string())]
        port: Option<u32>,
    },
    /// Remove a network
    #[command(name = "network:remove")]
    NetworkRemove {
        #[arg(help = "Name of the network to remove")]
        name: String,
    },
    #[command(subcommand_help_heading = "Auxiliary commands")]
    /// Start a local RPC server in background.
    #[command(name = "server:launch")]
    ServerLaunch {
        /// Path to the VSL DB directory. If the value is `tmp` - create a temporary directory.
        #[arg(long, default_value = "db-data")]
        db: String,
        #[arg(
            long,
            default_value = None,
            help = "Optional path to a genesis json file. By default, this is used only if the DB is empty.",
        )]
        genesis_file: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Optional genesis json string. By default, this is used only if the DB is empty. Exactly one of genesis-json and genesis-file must be provided.",
        )]
        genesis_json: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Whether to overwrite the DB with the genesis file data.",
        )]
        force: bool,
    },
    /// Dump a local RPC server std output.
    #[command(name = "server:dump")]
    ServerDump {
        /// Number of lines of the dump, which are shown.
        #[arg(short, long, default_value_t = 128)]
        lines: u32,
        /// Show the whole dump.
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
    /// Stop a local RPC server.
    #[command(name = "server:stop")]
    ServerStop {},
    /// Start a REPL that connects to an RPC node ('localhost' at port 'vsl_utils::PORT' by default).
    Repl {
        /// Print commands into the standard output. This is useful for using REPL for pipelined batches of commands
        #[arg(long, default_value_t = false)]
        print_commands: bool,
        /// Use a temporary empty config, which won't be saved and affect the persistent config.
        #[arg(long, default_value_t = false)]
        tmp_config: bool,
    },
    /// Create a new configuration.
    #[command(name = "config:create")]
    ConfigCreate {
        /// Name of the new configuration
        name: String,
        /// Copy data from a given configuration, if provided
        #[arg(short, long, default_value = "")]
        copy: String,
        /// File path with will be used to store configuration. If not provided, the `.config/vsl/<name>.json` will be used.
        #[arg(short, long, default_value = "")]
        file: String,
        /// Overwrite the existing configuration
        #[arg(short, long, default_value_t = false)]
        overwrite: bool,
    },
    /// Use a particular configuration.
    #[command(name = "config:use")]
    ConfigUse {
        /// Name of the configuration to use
        name: String,
    },
    /// Show a current configuration.
    #[command(name = "config:current")]
    ConfigCurrent {},
    /// List all known configurations.
    #[command(name = "config:list")]
    ConfigList {
        /// Display data in a json structure
        #[arg(long, default_value_t = false, action = ArgAction::SetTrue)]
        json: bool,
        /// Display data in a table structure
        #[arg(long, default_value_t = false, action = ArgAction::SetTrue)]
        table: bool,
    },
    /// Remove an existing configuration.
    #[command(name = "config:remove")]
    ConfigRempove {
        #[arg(help = "Name of the configuration, which is going to be removed")]
        name: String,
    },
}
