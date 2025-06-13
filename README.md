The VSL command-line interface
==============================

Provides the access to the VSL nodes, which may be configured with `network:<action>` commands.

The VSL CLI operates with _commands_, which have a form: `<subject>:<action>` and may have arguments.

The categories of subjects
--------------------------

Currently the categories are:
  * `claim` - commands for submission/settlement/query of claims,
  * `pay` - payment operations,
  * `account` - all about manipulation with VSL accounts,
  * `asset` - all about manupulation with VSL assets,
  * `code`
  * `network` - configurations of available network nodes,
  * other

In each category there may be several commands. For each command its aguments may explained by invoking the 
help of this particular command:
```bash
$ vsl-cli <command> --help
```

Category `claim`
----------------

| Command | Description |
|--------|-------------|
|  claim:submit    | Request verification of a claim |
|  claim:settle    | Submit a verified claim for validation only |
|  claim:submitted | Fetch verification request claims targeted for a verifier address since a timestamp |
|  claim:settled   | Fetch verified claims targeted for a client address since a timestamp |
|  claim:get       | Fetch a claim and its metadata by its ID |


Category `pay`
--------------

| Command | Description |
|--------|-------------|
| pay               | Transfer funds to another account |

Usage: `vsl-cli pay [OPTIONS] --to <TO> --amount <AMOUNT>`

Category `account`
------------------

| Command | Description |
|--------|-------------|
|  account:create   | Generates a new account in VSL | 
|  account:get      | Fetches the information about account | 
|  account:balance  | Ask for the balance of an account | 
|  account:use      | Switches to another account | 
|  account:current  | Prints data of current account | 
|  account:list     | Lists all available accounts | 
|  account:remove   | Delete the account | 

Category `asset`
----------------

| Command | Description |
|--------|-------------|
|  asset:balance    | Ask for the balance of an asset for the account |
|  asset:balances   | Ask for the balance of all assets for the account |
|  asset:create     | Creates a new native asset |
|  asset:transfer   | The transfer of an asset |
|  asset:get        | Get the information about an asset |


Category `network`
----------------

Initially and futher anytime, there's an implicit default local network: default - http://localhost:55555
Other networks may be added/used. The network information includes the current status of a network: is it
up or down. If network is down, it can't be used with `network:use` - it makes no sence.

| Command | Description |
|--------|-------------|
|  network:add     | Add network |
|  network:list    | List all known networks |
|  network:use     | Use a selected network as default |
|  network:current | Prints the current default network status |
|  network:update  | Update a known network |
|  network:remove  | Remove a network |


Catefory `server`
-----------------
The `vsl-cli` allows a user to launch the RPC server locally from withing the `vsl-cli`.

| Command | Description |
|--------|-------------|
|  server:launch   | Start a local RPC server in background |
|  server:dump     | Dump a local RPC server std output |
|  server:stop     | Stop a local RPC server |


The lifetime of the VSL server, which was launched locally from withing the `vsl-cli`:
  1. In case of a single command: `vsl-cli server:launch` the server process is launched and keeps running
  2. In case the `server:launch` is triggered in the REPL mode, the server keeps running until explicitly shut down or a REPL session is over.

The `server:launch` takes three arguments:
  * `--db` markups the directory, which VSL uses for its DB. If empty, the default directory is used. In case of a special value: `--db=tmp`, the temporary directory is created and is destroyed at the `vsl-cli` exit. 
  * `--log-level` - the level of verbosity of RPC server. Possible values: `trace`, `debug`, `info`, `warn`, `error`. Default is `info`.
  * `--master` - The amount with which to initialize the master account. If not specified, the master account is not initialized.

Other
-----

| Command | Description |
|--------|-------------|
|  health:check     | Query the servers' (nodes') state  |



REPL mode
----------
The read-eval-print loop mode is available with:
```bash
$ vsl-cli repl
```

The REPL-mode is also may be used to run `vsl-cli` for multiple requests in a batch:
```bash
$ vsl-cli repl < batch_commands_file
```
In order to see the commands themselves together with the responses from the VSL node, pass the
environment variable `VSL_CLI_PRINT_COMMANDS` to the binary:
```bash
$ VSL_CLI_PRINT_COMMANDS=1 vsl-cli repl < batch_commands_file
```


The example of the batch file may be found in [test batch commands](tests/batch_commands)

There are also several special commands in REPL mode:
| Command | Description |
|--------|-------------|
| help            | Show the help message |
| history:list    | Show command history |
| clear:screen    | Clear the screen |
| clear:history   | Clear command history |
| exit, quit, bye | Exit the REPL |

Navigation:
| Command | Description |
|--------|-------------|
| ↑/↓ arrows | Browse command history |
| Ctrl+C     | Interrupt current input |
| Ctrl+D     | Exit REPL |
| Tab        | Auto-completion of commands |





Configuration
-------------

The state of `vsl-cli` application is preserved in a config files in the `<config_dir>/vsl` directory.
In linux typically the `<config_dir>` is `~/.config`. Currently there may be two files:
  * `cli.json` - holds the list of configured accounts, networks, addresses and curernt nonce
  * `cli_history` - the history of command-line commands, entered by a user.

To pass some particular private key, which will be used as default when creaing the default initial
configuration, use the `VSL_CLI_PRIVATE_KEY` environment variable:

```bash
VSL_CLI_PRIVATE_KEY=45ae437368b0f84ad97c9c5cb86fcfa4dfff216c71376aa6f2a8a9fe0fce5772 vsl-cli
```
will create the `default_accaount` with the given private key.


For each account following data is stored:
  * the name of the accaount,
  * `signatures` - the list of addresses of verifiers for this account,
  * `address`- the address of this account in VSL
  * `quorum` - the minimum quorum of signatures
  * `private_key` - the private key of the account. 

**WARNING**: private key is stored as is, non-encrypted.

In case a user wants to use a clean new default config, which would not be loaded/saved, he should set
the `VSL_CLI_PERSISTENT_CONFIG=0` environment variable:

```bash
VSL_CLI_PERSISTENT_CONFIG=0 vsl-cli
```


Stress testing
--------------

To run stress tests, do:
```bash
cargo build --release
cd vsl-cli
ulimit -n 1000000
cargo run --release --example stress_test
```

The approximate result:

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
