# Pension Management System

## Overview

This project implements a blockchain-based pension management system. It consists of an ink! smart contract (`pension_manager`) responsible for the core logic and data storage, and a Rust off-chain client CLI tool for interacting with the smart contract.

## Components

*   **`contracts/`**: Contains the ink! smart contract (`pension_manager`). This contract handles all on-chain logic, including registrations, pensioner data management, payout calculations, and authorization.
*   **`offchain_client/`**: Contains a Rust command-line interface (CLI) client designed to interact with the `pension_manager` smart contract.

## Building the Smart Contract

To build the smart contract, navigate to the `contracts` directory and use `cargo-contract`. Ensure you have `cargo-contract` installed (`cargo install cargo-contract --force`).

```bash
cd pension_management/contracts
cargo contract build
```
This will produce the necessary `.contract` file (containing Wasm and metadata) in the `target/ink` directory within `contracts`.

## Building the Off-Chain Client

To build the off-chain client, navigate to the `offchain_client` directory:

```bash
cd pension_management/offchain_client
cargo build
```
The executable will be located in `target/debug/offchain_client` within the `offchain_client` directory.

## Running the Off-Chain Client

The off-chain client currently **simulates** calls to the smart contract. It prints the intended action (query or command), parameters, and simulated caller ID, then returns a predefined or randomized response. It does not actually connect to a live blockchain node.

**Example:**

To run the client (from the `pension_management/offchain_client` directory after building):
```bash
./target/debug/offchain_client --node-url <your_node_url> --contract-address <your_contract_address> get-contract-owner
```

The `--node-url` and `--contract-address` arguments have default values suitable for the current simulation mode (e.g., `http://localhost:9944` and a dummy contract address). You can omit them to use these defaults.

For more commands and options, use:
```bash
./target/debug/offchain_client --help
```
