#!/bin/bash
# VSL CLI Tutorial

# This tutorial walks you through using the `vsl-cli` to configure a network, manage accounts, create and # transfer assets, and submit a claim.

# Enable printing of commands
export VSL_CLI_PRINT_COMMANDS=1

## 🔌 Step 0: Create a temporary configuration `local_test`

vsl-cli config:create local_test --overwrite

## 🔌 Step 1: Connect to a Network
# In case the VSL node is installed locally, start a local VSL node:

vsl-cli server:launch --db tmp --genesis-json='{"accounts":[{"id":"0x749ab3318b74907f6e5856ce9ce1f3b55e3bb38a","balance":"1000000000000000000000000"}],"tokens":[]}' --force

# Or use some external node, in case there's no local VSL node:
# Uncomment these lines in case of remote network
#network:add remote --url http://144.76.7.152
#network:use remote


# Verify connection:

vsl-cli health:check

# initialize the master account (it should be one of the genesis accounts):
vsl-cli account:load master -p 0xb6dd863bea551b5bb27ce9917316a01ea4c331f24e0e4fe56e28eb430f175ed7

## 👤 Step 2: Create and Use an Account

# Create two accounts: `alice` and `bob`

vsl-cli account:create alice
vsl-cli account:create bob

# Supply the accounts `alice` and `bob` with funds
# from the so called `master` account.

vsl-cli account:use master
vsl-cli pay --to alice --amount 0x1000
vsl-cli pay --to bob --amount 0x500

# Set `alice` as the active account:

vsl-cli account:use alice

# Check balances:

vsl-cli account:balance
vsl-cli account:use bob
vsl-cli account:balance


## 🪙 Step 3: Create and Transfer Assets

# Switch back to `alice` and create a custom asset:
vsl-cli account:use alice
vsl-cli asset:create --symbol DEMO --supply 0x1000

# Transfer 100 units of DEMO asset to `bob`:

vsl-cli asset:transfer --asset DEMO --to bob --amount 0x64

# Note: You can find Bob’s address with:

vsl-cli account:get bob

# Check asset balances:

vsl-cli account:use bob
vsl-cli asset:balances


## 💸 Step 4: Transfer Native Tokens

# Transfer 32 VSL from `alice` to `bob`:

vsl-cli account:use alice
vsl-cli pay --to bob --amount 0x20
vsl-cli account:balance
vsl-cli account:use bob
vsl-cli account:balance


## 🧶 Step 5: Submit and Settle a Claim

# Submit a claim:

vsl-cli account:use bob
vsl-cli claim:submit '{"email":"bob@example.com"}' --type identity --proof 0xabc123

# List submitted claims:

vsl-cli claim:submitted

# Settle a verified claim:

vsl-cli claim:settle '{"email":"bob@example.com"}'


## 🧠 Bonus: Some more commands in REPL mode:

#vsl-cli repl < 'account:current asset:balances'

## 🧹 Step 6: Clean Up

# Stop the local server:

vsl-cli server:stop

# Remove accounts and networks:

vsl-cli account:remove alice
vsl-cli account:remove bob

vsl-cli account:remove master

# Finally, remove the test configuration
vsl-cli config:remove local_test


## ✅ You're Done!

# You've successfully:

# - Launched a local VSL server
# - Managed accounts and balances
# - Created and transferred assets
# - Submitted and settled claims
# - Explored the REPL
# For more, run:
# vsl-cli <command> --help


