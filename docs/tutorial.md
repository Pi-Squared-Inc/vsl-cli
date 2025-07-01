# VSL CLI Tutorial

This tutorial walks you through using the `vsl-cli` to configure a network, manage accounts, create and transfer assets, and submit a claim.

---

## ⌨️ Step 0: Enter the REPL mode

```bash
vsl-cli repl --tmp-config
```

## 🔌 Step 1: Connect to a Network

In case the VSL node is installed locally, start a local VSL node:

```bash
vsl> server:launch --db tmp --init "genesis.json"
```

Or use some external node, in case there's no local VSL node:

```bash
vsl> network:add remote --url http://144.76.7.152
vsl> network:use remote
```

Verify connection:

```bash
vsl> health:check
```

---

## 👤 Step 2: Create and Use an Account

Create two accounts: `alice` and `bob`

```bash
vsl> account:create alice
vsl> account:create bob
```

Supply the accounts `alice` and `bob` with funds
from the so called `master` account.

```bash
vsl> account:use master
vsl> pay --to alice --amount 0x1000
vsl> pay --to bob --amount 0x500
```

Set `alice` as the active account:

```bash
vsl> account:use alice
```

Check balances:

```bash
vsl> account:balance
vsl> account:use bob
vsl> account:balance
```

---

## 🪙 Step 3: Create and Transfer Assets

Switch back to `alice` and create a custom asset:

```bash
vsl> account:use alice
vsl> asset:create --symbol DEMO --supply 0x1000
```

Transfer 100 units of DEMO asset to `bob`:

```bash
vsl> asset:transfer --to bob --amount 0x64 --symbol DEMO
```

Note: You can find Bob’s address with:

```bash
vsl> account:get bob
```

Check asset balances:

```bash
vsl> account:use bob
vsl> asset:balances
```

---

## 💸 Step 4: Transfer Native Tokens

Transfer 32 VSL from `alice` to `bob`:

```bash
vsl> account:use alice
vsl> pay --to bob --amount 0x20
```

Check that 32 VSL has beed transferred:

```bash
vsl> account:balance
vsl> account:use bob
vsl> account:balance
```

---

## 🧶 Step 5: Submit and Settle a Claim

Submit a claim:

```bash
vsl> account:use bob
vsl> claim:submit '{"email":"bob@example.com"}' --type identity --proof 0xabc123
```

List submitted claims:

```bash
vsl> claim:submitted
```

Settle a verified claim:

```bash
vsl> claim:settle <claim_id>
```

---

## 🧠 Bonus: Use the Interactive REPL

Start the REPL to try commands interactively:

```bash
vsl-cli repl --print-commands
```

Try entering:

```bash
account:current
asset:balances
```

---

## 🧹 Step 6: Clean Up

Stop the local server:

```bash
vsl> server:stop
```

Remove accounts and networks:

```bash
vsl> account:remove alice
vsl> account:remove bob
vsl> network:remove local
```

---

## 🐚 Running this tutorial as a batch or a script.

Running as a batch file:
```bash
vsl-cli repl --print-commands < docs/tutorial.vsl
```

Running in a shell script:
```bash
docs/tutorial.sh 
```

---

## ✅ You're Done!

You've successfully:

- Launched a local VSL server
- Managed accounts and balances
- Created and transferred assets
- Submitted and settled claims
- Explored the REPL

For more, run:

```bash
vsl-cli <command> --help
```

