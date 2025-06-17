# VSL CLI

[![License](https://img.shields.io/badge/license-BSD-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![VSL Network](https://img.shields.io/badge/VSL-Network-green.svg)](https://pisquared.network)

A powerful command-line interface for interacting with the Verifiable Settlement Layer (VSL) network - Pi Squared's infrastructure for scalable, affordable, and customizable verifiability across Web3 protocols.

## Overview

VSL CLI enables developers and users to interact with the VSL network through a comprehensive set of commands for:

- **Claim Management**: Submit, verify, and settle claims across platforms
- **Account Operations**: Create and manage VSL network accounts
- **Asset Management**: Handle custom tokens and digital assets
- **Cross-Chain Operations**: Facilitate verifiable cross-platform communication
- **Network Administration**: Connect to different VSL networks
- **Configuration Management**: Manage multiple CLI configurations
- **Development Tools**: Local server management and testing utilities

## Quick Start

### Installation

```bash
# build from source
git clone https://github.com/Pi-Squared-Inc/vsl-cli
cd vsl-cli
cargo build --release
cargo install --path .
```

### Basic Usage

```bash
# Create your first account
vsl-cli account:create myaccount

# Check account balance
vsl-cli account:balance

# Submit a claim for verification
vsl-cli claim:submit "User verification completed" --type "kyc" --fee 0x1

# Start local development server
vsl-cli server:launch --db tmp
```

## Core Concepts

### Claims
Claims are the fundamental building blocks of VSL - they encode payments, transactions, computations, assets, and data that need to be verified and settled across platforms.

### Verifiable Settlement
VSL provides a unified layer where claims are verified against proofs and settled permanently, enabling secure cross-platform communication.

### Multi-Proof Support
Claims can be verified using various methods: digital signatures, re-execution, formal proofs, zero-knowledge proofs, and more.

## Command Categories

### 🔐 Account Management
```bash
vsl-cli account:create <name>          # Create new account
vsl-cli account:load <name> <key>      # Load an existing account with particular private key
vsl-cli account:export <file>          # Export the accounts private key to a file
vsl-cli account:list                   # List all accounts
vsl-cli account:balance [address]      # Check account balance
vsl-cli account:use <name>             # Switch active account
vsl-cli account:current                # Show current account
vsl-cli account:get [address]          # Get account information
vsl-cli account:state-get [address]    # Get account state
vsl-cli account:state-set <state>      # Set account state
vsl-cli account:remove <name>          # Delete account
```

### 📝 Claim Operations
```bash
vsl-cli claim:submit <claim>           # Submit claim for verification
vsl-cli claim:settle <claim>           # Settle verified claim
vsl-cli claim:submitted                # List submitted claims
vsl-cli claim:settled                  # List settled claims
vsl-cli claim:get <id>                 # Get claim by ID
```

### 🪙 Asset Management
```bash
vsl-cli asset:create --symbol <SYM>    # Create new asset
vsl-cli asset:balance <asset>          # Check asset balance
vsl-cli asset:balances                 # Check all asset balances
vsl-cli asset:transfer --asset <ASSET> # Transfer assets
vsl-cli asset:get <asset>              # Get asset information
```

### 🌐 Network Management
```bash
vsl-cli network:add <name>             # Add network configuration
vsl-cli network:list                   # List available networks
vsl-cli network:use <name>             # Switch to network
vsl-cli network:current                # Show current network
vsl-cli network:update <name>          # Update network configuration
vsl-cli network:remove <name>          # Remove network
```

### ⚙️ Configuration Management
```bash
vsl-cli config:create <name>           # Create new configuration
vsl-cli config:use <name>              # Switch to configuration
vsl-cli config:current                 # Show current configuration
vsl-cli config:list                    # List all configurations
vsl-cli config:remove <name>           # Remove configuration
```

### 💸 Payments
```bash
vsl-cli pay --to <address> --amount <amt> # Transfer funds
```

### 🖥️ Server Management
```bash
vsl-cli server:launch                  # Start local VSL server
vsl-cli server:stop                    # Stop local server
vsl-cli server:dump                    # View server logs
```

### 🔍 Monitoring
```bash
vsl-cli health:check                   # Check network health
vsl-cli repl                           # Interactive mode
```

## Configuration

VSL CLI supports multiple configuration profiles, allowing you to manage different environments, networks, and account setups separately.

### Configuration Storage
VSL CLI stores configurations in your system's standard config directory:

- **Linux**: `~/.config/vsl-cli/`
- **macOS**: `~/Library/Application Support/vsl-cli/`
- **Windows**: `%APPDATA%\vsl-cli\`

or in a user-provided place, in case such place is provided.

### Configuration Management

#### Creating Configurations
```bash
# Create a new configuration
vsl-cli config:create development

# Create a configuration based on an existing one
vsl-cli config:create production --copy development

# Create a configuration with custom file path
vsl-cli config:create custom --file /path/to/custom/config.json

# Overwrite existing configuration
vsl-cli config:create development --overwrite
```

#### Switching Configurations
```bash
# Switch to a different configuration
vsl-cli config:use production

# View current configuration
vsl-cli config:current

# List all available configurations
vsl-cli config:list
vsl-cli config:list --json    # JSON output
vsl-cli config:list --table   # Table format
```

#### Removing Configurations
```bash
# Remove a configuration
vsl-cli config:remove old-config
```

### Network Configuration
```bash
# Add mainnet
vsl-cli network:add mainnet --url https://mainnet.vsl.network

# Add testnet
vsl-cli network:add testnet --url https://testnet.vsl.network

# Add local development
vsl-cli network:add local --url http://localhost --port 44444
```

## Development Workflow

### 1. Environment Setup with Configurations
```bash
# Create separate configurations for different environments
vsl-cli config:create development
vsl-cli config:create testing
vsl-cli config:create production

# Switch to development configuration
vsl-cli config:use development

# Start local VSL server with temporary database
vsl-cli server:launch --db tmp --log-level debug --master-balance 1000000000

# Create development account with initial balance
vsl-cli account:create dev --balance 0x989680

# Switch to development account
vsl-cli account:use dev

# Add local network
vsl-cli network:add local --url http://localhost
vsl-cli network:use local
```

### 2. Multi-Environment Management
```bash
# Development environment
vsl-cli config:use development
vsl-cli network:use local
vsl-cli account:use dev-account

# Testing environment
vsl-cli config:use testing
vsl-cli network:use testnet
vsl-cli account:use test-account

# Production environment
vsl-cli config:use production
vsl-cli network:use mainnet
vsl-cli account:use prod-account
```

### 3. Claim Submission Example
```bash
# Submit a claim with proof
vsl-cli claim:submit "Cross-chain token transfer: 100 USDC" \
  --type "bridge_transfer" \
  --proof "0x1234567890abcdef..." \
  --fee 0x5 \
  --lifetime 3600

# Check submission status
vsl-cli claim:submitted --within 3600

# Once verified, settle the claim
vsl-cli claim:settle '{"claim": "verified_data", "certificate": {...}}'
```

### 4. Asset Management Example
```bash
# Create a new token
vsl-cli asset:create --symbol MYTOKEN --supply 1000000

# Check asset details
vsl-cli asset:get MYTOKEN

# Transfer tokens
vsl-cli asset:transfer --asset MYTOKEN --to 0x742d35Cc6634C0532925a3b8D46e7Cdc2F5 --amount 1000
```

## Advanced Features

### Interactive REPL Mode
```bash
# Start interactive session
vsl-cli repl

# REPL with command echo (useful for scripting)
vsl-cli repl --print-commands

# Temporary configuration mode
vsl-cli repl --tmp-config
```

The REPL-mode is also may be used to run `vsl-cli` for multiple requests in a batch:
```bash
# Run the batch of commands from a file
$ vsl-cli repl < batch_commands_file
```

In order to see the commands themselves together with the responses from the VSL node, pass the
environment variable `VSL_CLI_PRINT_COMMANDS` to the binary:
```bash
# Run the batch of commands from a file which looks like enterging commands one by one by hand
$ vsl-cli repl --print-commands < batch_commands_file
```

The example of the batch file may be found in [test batch commands](tests/batch_commands)

#### 🧰 REPL Special Commands
There are also several special commands in REPL mode:
```
┌──────────────────────────────┬───────────────────────────────┐
│ Command                      │ Description                   │
├──────────────────────────────┼───────────────────────────────┤
│ help                         │ Show the help message         │
│ history:list                 │ Show command history          │
│ clear:screen                 │ Clear the screen              │
│ clear:history                │ Clear command history         │
│ exit, quit, bye              │ Exit the REPL                 │
└──────────────────────────────┴───────────────────────────────┘
```

#### ⌨️ REPL Navigation Shortcuts
```
┌─────────────┬───────────────────────────────────────────────┐
│ Shortcut    │ Action                                        │
├─────────────┼───────────────────────────────────────────────┤
│ ↑ / ↓       │ Browse command history                        │
│ Ctrl+C      │ Interrupt current input                       │
│ Ctrl+D      │ Exit REPL                                     │
│ Tab         │ Auto-complete command names                   │
└─────────────┴───────────────────────────────────────────────┘
```

### JSON Output Support
Many commands support structured output:
```bash
vsl-cli account:list --json
vsl-cli network:current --json
vsl-cli config:list --json
vsl-cli claim:submitted --json
```

### Scripting and Automation
```bash
#!/bin/bash
# Example automation script with configuration management

# Set up environment-specific configuration
vsl-cli config:use production
vsl-cli account:use bot

# Submit automated claim
CLAIM_ID=$(vsl-cli claim:submit "Automated verification" --type "bot")
echo "Submitted claim: $CLAIM_ID"

# Monitor and settle
vsl-cli claim:submitted --json | jq '.[] | select(.id == "'$CLAIM_ID'")'
```

## API Integration

VSL CLI interfaces with VSL network nodes through JSON-RPC APIs. Claims are submitted in structured formats:

### Claim Submission Format
```json
{
  "claim": "The claim content",
  "claim_type": "verification_type",
  "proof": "0xproof_data",
  "nonce": "unique_nonce",
  "verifiers": ["0xverifier1", "0xverifier2"],
  "quorum": 2,
  "client": "0xclient_address",
  "expires": 1640995200,
  "fee": 100
}
```

### Settlement Format
```json
{
  "claim": "verified_claim_data",
  "certificate": {
    "verifiers": ["0xverifier1", "0xverifier2"],
    "signature": "0xaggregated_signature"
  },
  "nonce": "client_nonce",
  "client": "0xclient_address"
}
```

## Security Considerations

- **Private Keys**: Never share or commit private keys. Use hardware wallets for production
- **Network Selection**: Use appropriate networks (local → testnet → mainnet)
- **Configuration Management**: Keep production configurations secure and separate from development
- **Fee Management**: Set adequate fees for timely processing
- **Proof Validation**: Ensure proofs are properly formatted and valid
- **Account Backup**: Maintain secure backups of account configurations

## Troubleshooting

### Common Issues

**Connection Problems**
```bash
# Check network health
vsl-cli health:check

# Verify network configuration
vsl-cli network:current --json
```

**Configuration Issues**
```bash
# Verify current configuration
vsl-cli config:current

# List all available configurations
vsl-cli config:list

# Switch to a different configuration
vsl-cli config:use <config-name>
```

**Account Issues**
```bash
# Verify current account
vsl-cli account:current

# Check account balance
vsl-cli account:balance
```

**Claim Failures**
```bash
# Review submitted claims
vsl-cli claim:submitted --within 7200

# Check claim format and proof validity
vsl-cli claim:get '{"claim_id": "your_claim_id"}'
```

**Debug Mode**
```bash
# Enable detailed logging
RUST_LOG=debug vsl-cli <command>
```

**Clear Config Mode**
```bash
# Use a temporary clear config that would be disposed of right after execution ends.
VSL_CLI_PERSISTENT_CONFIG=0 vsl-cli <command>
```

## Examples Repository

Find comprehensive examples and use cases in our [examples repository](https://github.com/pi-squared/vsl-examples):

- Cross-chain asset transfers
- Smart contract verification
- Identity verification systems
- Supply chain tracking
- DeFi protocol integration
- Multi-environment configuration setups

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
```bash
git clone https://github.com/Pi-Squared-Inc/vsl-cli
cd vsl-cli
cargo build
cargo test
```

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test end_to_end

# Stress testing
To run stress tests, do:
```bash
cargo build --release
ulimit -n 1000000
cargo run --release --example stress_test
```

The approximate result looks like:
```
========================================================================================================================
                                                  STRESS TEST RESULTS                                                   
========================================================================================================================
Concurrency  Total    Success  Failed   Success%     Avg Time     Min Time     Max Time     RPS          Errors  
------------------------------------------------------------------------------------------------------------------------
8            960      960      0        100.0        818μs        110μs        2.0ms        9586.3       0       
32           3840     3840     0        100.0        2.8ms        115μs        15.3ms       10191.9      0       
128          15360    15360    0        100.0        11.7ms       110μs        41.2ms       9323.0       0       
512          61440    61440    0        100.0        44.8ms       97μs         500.8ms      9121.4       0       
2048         245760   245760   0        100.0        152.2ms      100μs        936.6ms      8671.7       0       
========================================================================================================================

📊 SUMMARY:
Best RPS: 10191.9 at concurrency 32
```

## Documentation

- **[Full Tutorial](docs/tutorial.md)**: Comprehensive usage guide
- **[API Reference](docs/commands.md)**: Detailed command documentation
- **[VSL Protocol](https://docs.pi2.network/verifiable-settlement-layer/what-is-vsl)**: Protocol specification
- **[Examples](examples/)**: Real-world use cases

## Community

- **Discord**: [Join our community](https://discord.gg/pisquared)
- **Twitter**: [@PiSquaredNetwork](https://x.com/Pi_Squared_Pi2)

## License

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Pi Squared team for the VSL protocol design
- Rust community for excellent tooling
- Contributors and early adopters

---

**Ready to build verifiable cross-chain applications?** Start with `vsl-cli account:create` and join the future of Web3 interoperability!