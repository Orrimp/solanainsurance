# Solana Insurance Program

Native Solana on-chain program for pension/insurance management with AI-accelerated development toolbox.

## 🚀 Quick Links

- **[AI Toolbox Quick Start](toolbox/QUICKSTART.md)** - Get started with AI-assisted development
- **[Toolbox Documentation](toolbox/README.md)** - Full toolbox guide
- **[AGENTS.md](AGENTS.md)** - Architecture guidelines for AI development
- **[Build-Test-Deploy Skill](.github/skills/build-test-deploy/SKILL.md)** - CI pipeline

## 🧰 AI Development Toolbox

This project includes an AI-first development toolbox for accelerating feature development:

- **MCP Server** - Model Context Protocol server for AI agents (Claude, Cursor)
- **CLI Tool** - `solana-toolbox` command-line interface
- **Templates** - Code scaffolding for instructions, tests, processors
- **Validators** - Architecture compliance checking
- **Scripts** - Automation for common workflows

**Get started:** See [toolbox/QUICKSTART.md](toolbox/QUICKSTART.md)

---

## 🛠️ Development Setup

## Run a local validator (local devnet)

Run a local Solana cluster on your machine and point the CLI to it.

1) Start the validator (keep this terminal open):
```bash
solana-test-validator --reset
```

2) In a new terminal, point your CLI to localhost:
```bash
solana config set --url localhost
solana config get
```

3) Ensure your default keypair exists (see next section) and fund it:
```bash
solana airdrop 2
solana balance
```

4) Deploiy and run program
```bash
solana program deploy -u localhost ./target/deploy/Insurance.so
cargo run --example client
```

## Generate Default Keypair
```bash
solana-keygen new -o /Users/Vitaliy.Schreibmann/.config/solana/id.json --force
Generating a new keypair

For added security, enter a BIP39 passphrase

NOTE! This passphrase improves security of the recovery seed phrase NOT the
keypair file itself, which is stored as insecure plain text

BIP39 Passphrase (empty for none): (Vitaliy518)
Enter same passphrase again: 

Wrote new keypair to /Users/Vitaliy.Schreibmann/.config/solana/id.json
=============================================================================
pubkey: CMVaCbvZhh2yEtS47993FXkrh7augXnr9XDNoesw9YDp
=============================================================================
Save this seed phrase and your BIP39 passphrase to recover your new keypair:
bulb kick thunder fiction armed market moment turkey length polar spare model
=============================================================================
```

## Solana Airdrop

```bash

❯ solana airdrop 2
Requesting airdrop of 2 SOL

Signature: 4xLip5XFMWQzn8Mvmfu6cAnWPpzDvH2DW9yAdb8XQ9RfjRuUKDoaZkE3Ja4UyuLq6MKUQ26Bi2giBEfJtMqwuAWz

2 SOL

```

## Check Dependencies
Finds truly unused deps across targets (lib/bin/tests/examples). Installed with `cargo install cargo-udeps`
```bash
cargo +nightly udeps --all-targets --workspace
```


## Build
Build the on-chain program for SBF and produce `./target/deploy/Insurance.so`.
Requires Solana CLI 1.16+ and Rust (edition 2021).
```bash
# Option A: Use Solana helper (produces target/deploy/*.so)
solana program build

# Option B: Direct cargo (equivalent)
cargo build-sbf
```


## Check Address
```bash
solana address -k ./target/deploy/Insurance-keypair.json
```

## Deploy and execute
```bash
# With the local validator running and CLI set to localhost
solana program deploy -u localhost ./target/deploy/Insurance.so
```
Program Id: B4yfzKC4NsUsYCetguU7tiewFWi8EDrQA9fEJFiYagVw

Signature: 5G2ZCbZYS3x8S62UdKrkQUgyTc1SdJqzgpUos7T8RxkZT3x1wtN5x4eSUzFgYDMrgLn3KrCFQYDHcrRCMLLTTpmf

## Run the Example client (example is here command from cargo)
```bash
cargo run --example client   
```

Transaction Signature: 2BbZtwt8KkA3oHpXqpt4hpR8CDgqjcbTHd4dbvtERNg9xyMiLkjdnZxRMCnhDSeko2VYUupmtMEmXr3d7QVKGhWU
https://explorer.solana.com/tx/2BbZtwt8KkA3oHpXqpt4hpR8CDgqjcbTHd4dbvtERNg9xyMiLkjdnZxRMCnhDSeko2VYUupmtMEmXr3d7QVKGhWU?cluster=custom


## Close Program and retrieve the SOL (delete the program)
```bash
solana program close B4yfzKC4NsUsYCetguU7tiewFWi8EDrQA9fEJFiYagVw --bypass-warning
```

## Stop the local validator
Stop the process in the terminal running `solana-test-validator` (Ctrl+C). The `--reset` flag wipes the local ledger on the next start.


# Use 

Of course. Here is the summary formatted with markdown.

***

## Solana Pensioner Insurance Smart Contract Summary

The user was provided with a **Solana smart contract** for a pensioner insurance system, written in **Rust** using the **Anchor framework**.

---

### Core Concept & Data Structure

The contract's primary function is to manage the state of a pension. It defines a data structure, **`PensionAccount`**, to store information for each pensioner. This account includes:

* **`authority`**: The managing entity's public key.
* **`pensioner`**: The recipient's public key.
* **`monthly_payment`**: The pension amount.
* **`last_payment_timestamp`**: A Unix timestamp of the last payout.
* **`status`**: An enum with two states: **`Active`** or **`Deceased`**.

---

### Key Instructions (Functions)

The contract exposes two main instructions as requested:

1.  **`mark_deceased`**: This instruction allows the designated **`authority`** to change the pensioner's **`status`** from `Active` to `Deceased`. Security is enforced using Anchor's **`has_one = authority`** constraint, ensuring only the authorized party can call this function.
2.  **`calculate_due_payment`**: This instruction reads the **`PensionAccount`**'s state, calculates the amount owed based on the time elapsed, and outputs the result to the transaction logs using **`msg!`**.

An `initialize_pensioner` instruction is also included to create new accounts.

---

### Architecture & Design

The overall design follows the standard Solana model where the program is **stateless**, and all data resides in separate **accounts**. This separation of logic and state is a fundamental concept in Solana development.