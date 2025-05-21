# Pension Manager Smart Contract

## Overview

This directory contains the `pension_manager` ink! smart contract, which forms the core of the blockchain-based pension management system. It is responsible for all on-chain logic and data storage related to pensioners, employing companies, banks, tax offices, and the pension lifecycle itself.

## Key Features

*   **Pensioner Data Management:** Securely stores and manages comprehensive data for each pensioner, including employment history, salary, status, and eligibility flags.
*   **Role-Based Authorizations:** Implements a system for authorizing different entities (companies, banks, tax offices) to perform specific actions. The contract owner has administrative privileges.
*   **Pension Calculation Engine:** Calculates estimated pension payouts based on years worked, salary, and any supplementary insurance benefits.
*   **Insurance and Tax Integration:** Allows authorized banks to add insurance policies for pensioners and authorized tax offices to apply tax rates, which are factored into final payout calculations.
*   **Payout Lifecycle Management:** Supports the initiation of pension payouts for eligible pensioners and handles the process.
*   **Death Benefit Processing:** Manages the designation of spouse beneficiaries and calculates/assigns death benefits upon a pensioner's reported death.
*   **Error Handling:** Provides clear error types for various operational failures.

## Build

To build the smart contract, ensure you have `cargo-contract` installed:
```bash
cargo install cargo-contract --force
```
Then, navigate to this `contracts` directory and run:
```bash
cargo contract build
```
This command compiles the smart contract to Wasm and generates a `.contract` file (which includes the Wasm and metadata) in the `target/ink/` subdirectory. This `.contract` file is used for deploying the contract to a Substrate-based blockchain.

For more details on the specific messages and data structures, please refer to the Rustdoc comments within the `src/lib.rs` file.
