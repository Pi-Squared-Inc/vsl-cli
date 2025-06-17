# VSL CLI Command Documentation

## Overview

The VSL CLI is a command-line interface for interacting with the VSL (Verification Service Layer) network. It provides comprehensive functionality for managing claims, accounts, assets, networks, and configurations.

## Command Categories

### Claim Management Commands

#### `claim:submit`
Request verification of a claim.

**Usage:**
```bash
vsl claim:submit <claim> [OPTIONS]
```

**Arguments:**
- `<claim>` - The claim to be submitted (required)

**Options:**
- `-t, --type <CLAIM_TYPE>` - The claim type (default: empty string)
- `-p, --proof <PROOF>` - The proof of the claim (default: empty string)
- `-e, --expires <EXPIRES>` - The expiration timestamp, when the submitted claim will be erased
- `-l, --lifetime <LIFETIME>` - How much the claim is considered alive after creation, in seconds (default: 3600 - 1 hour)
- `-f, --fee <FEE>` - The total fee for verification and claim validation. Must be non-negative integer (default: "0x1")
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl claim:submit "User is over 18" --type "age_verification" --proof "driver_license_hash" --fee "0x10"
```

#### `claim:settle`
Submit a verified claim for validation only.

**Usage:**
```bash
vsl claim:settle <claim> [OPTIONS]
```

**Arguments:**
- `<claim>` - The claim to be settled (required)

**Options:**
- `-a, --address <ADDRESS>` - Client address, to whom the claim is settled. By default the current account address is used
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl claim:settle "verified_claim_data" --address "0x1234..."
```

#### `claim:submitted`
Fetch verification request claims targeted for a verifier address since a timestamp.

**Usage:**
```bash
vsl claim:submitted [OPTIONS]
```

**Options:**
- `-a, --address <ADDRESS>` - Client address. By default the current account address is used
- `-s, --since <SINCE>` - Since the timestamp
- `-w, --within <WITHIN>` - Within a certain number seconds before now (default: 3600 - 1 hour)
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Examples:**
```bash
vsl claim:submitted --since 1640995200
vsl claim:submitted --within 7200
```

#### `claim:settled`
Fetch verified claims targeted for a client address since a timestamp.

**Usage:**
```bash
vsl claim:settled [OPTIONS]
```

**Options:**
- `-a, --address <ADDRESS>` - Client address. By default the current account address is used
- `-s, --since <SINCE>` - Since the timestamp
- `-w, --within <WITHIN>` - Within a certain number seconds before now (default: 3600 - 1 hour)
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl claim:settled --address "0x5678..." --within 86400
```

#### `claim:get`
Fetch a claim and its metadata by its ID.

**Usage:**
```bash
vsl claim:get <id> [OPTIONS]
```

**Arguments:**
- `<id>` - JSON of the claim to query (required)

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl claim:get "claim_id_123" --network mainnet
```

### Payment Commands

#### `pay`
Transfer funds to another account.

**Usage:**
```bash
vsl pay [OPTIONS]
```

**Options:**
- `-t, --to <TO>` - Recipient of the transfer (required)
- `-a, --amount <AMOUNT>` - Amount to transfer (required)
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl pay --to "0xabcd..." --amount "100" --network testnet
```

### Account Management Commands

#### `account:create`
Generates a new account in VSL.

**Usage:**
```bash
vsl account:create <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Account name (required)

**Options:**
- `-o, --overwrite` - Overwrite the existing account (default: false)

**Example:**
```bash
vsl account:create my_new_account --overwrite
```

#### `account:load`
Makes use of an existing account with a provided private key.

**Usage:**
```bash
vsl account:load <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Account name (required)

**Options:**
- `-p, --private-key <PRIVATE_KEY>` - Account private key. May be a private key itself, or a path to a file with private key
- `-o, --overwrite` - Overwrite the existing account (default: false)

**Example:**
```bash
vsl account:load imported_account --private-key "./my_key.pem" --overwrite
```

#### `account:export`
Exports the account's private key.

**Usage:**
```bash
vsl account:export [OPTIONS]
```

**Options:**
- `-n, --name <NAME>` - Account name, optional. If omitted, the current is exported (default: empty string)
- `-f, --file <FILE>` - The target file, where the private key would be written, optional. Otherwise, the private key will be shown in console (default: empty string)

**Example:**
```bash
vsl account:export --name my_account --file "./exported_key.pem"
```

#### `account:get`
Fetches the information about account.

**Usage:**
```bash
vsl account:get [account] [OPTIONS]
```

**Arguments:**
- `[account]` - Account in the form of hex string (optional)

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl account:get 0x1234567890abcdef --network mainnet
```

#### `account:balance`
Ask for the balance of an account.

**Usage:**
```bash
vsl account:balance [account] [OPTIONS]
```

**Arguments:**
- `[account]` - Account in the form of hex string (optional)

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl account:balance --network testnet
```

#### `account:state-get`
Ask for the state of an account.

**Usage:**
```bash
vsl account:state-get [account] [OPTIONS]
```

**Arguments:**
- `[account]` - Account in the form of hex string (optional)

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

#### `account:state-set`
Update for the state of an account.

**Usage:**
```bash
vsl account:state-set <state> [OPTIONS]
```

**Arguments:**
- `<state>` - The new account state, hex string (required)

**Options:**
- `-a, --account <ACCOUNT>` - Account in the form of hex string
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl account:state-set "0xdeadbeef" --account "0x1234..."
```

#### `account:use`
Switches to another account.

**Usage:**
```bash
vsl account:use <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Account name (required)

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl account:use my_main_account
```

#### `account:current`
Prints data of current account.

**Usage:**
```bash
vsl account:current [OPTIONS]
```

**Options:**
- `--json` - Display data in a json structure (default: false)
- `--table` - Display data in a table structure (default: true)

**Example:**
```bash
vsl account:current --json
```

#### `account:list`
Lists all available accounts.

**Usage:**
```bash
vsl account:list [OPTIONS]
```

**Options:**
- `--json` - Display data in a json structure (default: false)
- `--table` - Display data in a table structure (default: true)

**Example:**
```bash
vsl account:list --table
```

#### `account:remove`
Delete the account.

**Usage:**
```bash
vsl account:remove <name>
```

**Arguments:**
- `<name>` - Account name (required)

**Example:**
```bash
vsl account:remove old_account
```

### Asset Management Commands

#### `asset:balance`
Ask for the balance of an asset for the account.

**Usage:**
```bash
vsl asset:balance <asset> [OPTIONS]
```

**Arguments:**
- `<asset>` - Asset in the form of hex string (required)

**Options:**
- `-a, --account <ACCOUNT>` - Account in the form of hex string
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl asset:balance "0xasset123" --account "0xaccount456"
```

#### `asset:balances`
Ask for the balance of all assets for the account.

**Usage:**
```bash
vsl asset:balances [OPTIONS]
```

**Options:**
- `-a, --account <ACCOUNT>` - Account in the form of hex string
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl asset:balances --account "0xaccount456"
```

#### `asset:create`
Creates a new native asset.

**Usage:**
```bash
vsl asset:create [OPTIONS]
```

**Options:**
- `--symbol <SYMBOL>` - Name of the asset (required)
- `--supply <SUPPLY>` - Total number of tokens that exist (required)
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl asset:create --symbol "MYTOKEN" --supply "1000000" --network testnet
```

#### `asset:transfer`
The transfer of an asset.

**Usage:**
```bash
vsl asset:transfer [OPTIONS]
```

**Options:**
- `--asset <ASSET>` - Name of the asset (required)
- `--to <TO>` - Account name (required)
- `--amount <AMOUNT>` - Total number of tokens that exist (required)
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl asset:transfer --asset "MYTOKEN" --to "recipient_account" --amount "500"
```

#### `asset:get`
Get the information about an asset.

**Usage:**
```bash
vsl asset:get <asset> [OPTIONS]
```

**Arguments:**
- `<asset>` - Name of the asset (required)

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl asset:get "MYTOKEN" --network mainnet
```

### Health Check Commands

#### `health:check`
Request the health info about a node.

**Usage:**
```bash
vsl health:check [OPTIONS]
```

**Options:**
- `-n, --network <NETWORK>` - URL to connect to, or name of a known network

**Example:**
```bash
vsl health:check --network mainnet
```

### Network Management Commands

#### `network:add`
Add network.

**Usage:**
```bash
vsl network:add <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Name of the network to add (required)

**Options:**
- `-u, --url <URL>` - Network URL (default: uses VSL_CLI_DEFAULT_NETWORK_URL)
- `-p, --port <PORT>` - Network port (default: uses VSL_CLI_DEFAULT_NETWORK_PORT)

**Example:**
```bash
vsl network:add testnet --url "https://testnet.vsl.io" --port 8080
```

#### `network:list`
List all known networks.

**Usage:**
```bash
vsl network:list [OPTIONS]
```

**Options:**
- `--json` - Display data in a json structure (default: false)
- `--table` - Display data in a table structure (default: true)

**Example:**
```bash
vsl network:list --table
```

#### `network:use`
Use a selected network as default.

**Usage:**
```bash
vsl network:use <name>
```

**Arguments:**
- `<name>` - The network name which is used by default (required)

**Example:**
```bash
vsl network:use mainnet
```

#### `network:current`
Prints the current default network status.

**Usage:**
```bash
vsl network:current [OPTIONS]
```

**Options:**
- `--json` - Display data in a json structure (default: false)
- `--table` - Display data in a table structure (default: true)

**Example:**
```bash
vsl network:current --json
```

#### `network:update`
Update a known network.

**Usage:**
```bash
vsl network:update <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Name of the network to update. Default is the currently used network (required)

**Options:**
- `-u, --url <URL>` - Network URL (default: http://localhost)
- `-p, --port <PORT>` - Network port (default: 44444)

**Example:**
```bash
vsl network:update testnet --url "https://new-testnet.vsl.io"
```

#### `network:remove`
Remove a network.

**Usage:**
```bash
vsl network:remove <name>
```

**Arguments:**
- `<name>` - Name of the network to remove (required)

**Example:**
```bash
vsl network:remove old_testnet
```

### Auxiliary Commands

#### `server:launch`
Start a local RPC server in background.

**Usage:**
```bash
vsl server:launch [OPTIONS]
```

**Options:**
- `--db <DB>` - Path to the VSL DB directory. If omitted, use the default. If the value is `tmp` - create a temporary directory (default: empty string)
- `--log-level <LOG_LEVEL>` - The logging level of an RPC server. One of: info, warn, error, ... (default: "info")
- `--master-account <MASTER_ACCOUNT>` - Master account name (default: "master")
- `--master-balance <MASTER_BALANCE>` - Master account balance (default: "1000000")

**Example:**
```bash
vsl server:launch --db "./my_db" --log-level "debug" --master-balance "5000000"
```

#### `server:dump`
Dump a local RPC server std output.

**Usage:**
```bash
vsl server:dump
```

#### `server:stop`
Stop a local RPC server.

**Usage:**
```bash
vsl server:stop
```

#### `repl`
Start a REPL that connects to an RPC node ('localhost' at default port by default).

**Usage:**
```bash
vsl repl [OPTIONS]
```

**Options:**
- `--print-commands` - Print commands into the standard output. This is useful for using REPL for pipelined batches of commands (default: false)
- `--tmp-config` - Use a temporary empty config, which won't be saved and affect the persistent config (default: false)

**Example:**
```bash
vsl repl --print-commands --tmp-config
```

### Configuration Management Commands

#### `config:create`
Create a new configuration.

**Usage:**
```bash
vsl config:create <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Name of the new configuration (required)

**Options:**
- `-c, --copy <COPY>` - Copy data from a given configuration, if provided (default: empty string)
- `-f, --file <FILE>` - File path which will be used to store configuration. If not provided, the `.config/vsl/<name>.json` will be used (default: empty string)
- `-o, --overwrite` - Overwrite the existing configuration (default: false)

**Example:**
```bash
vsl config:create production --copy development --overwrite
```

#### `config:use`
Use a particular configuration.

**Usage:**
```bash
vsl config:use <name>
```

**Arguments:**
- `<name>` - Name of the configuration to use (required)

**Example:**
```bash
vsl config:use production
```

#### `config:current`
Show a current configuration.

**Usage:**
```bash
vsl config:current
```

#### `config:list`
List all known configurations.

**Usage:**
```bash
vsl config:list [OPTIONS]
```

**Options:**
- `--json` - Display data in a json structure (default: false)
- `--table` - Display data in a table structure (default: true)

**Example:**
```bash
vsl config:list --table
```

#### `config:remove`
Remove an existing configuration.

**Usage:**
```bash
vsl config:remove <name>
```

**Arguments:**
- `<name>` - Name of the configuration, which is going to be removed (required)

**Example:**
```bash
vsl config:remove old_config
```

## Common Patterns

### Network Parameter
Most commands accept a `-n, --network <NETWORK>` parameter that allows you to specify which network to connect to. This can be either:
- A URL (e.g., `https://mainnet.vsl.io:44444`)
- A name of a known network (e.g., `mainnet`, `testnet`)

### Output Formats
Several commands support `--json` and `--table` flags for different output formats:
- `--json`: Outputs data in JSON format for programmatic use
- `--table`: Outputs data in a human-readable table format
