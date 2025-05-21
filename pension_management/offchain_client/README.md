# Off-Chain CLI Client

## Overview

This Rust application provides a command-line interface (CLI) to interact with the `pension_manager` smart contract. It allows users to simulate various operations that would typically be performed against a live contract, such as registering entities, updating pensioner data, and querying contract state.

## Note on Simulation

**Currently, this client simulates interactions and does not connect to a live blockchain node or smart contract.** It prints the actions it would take (including parameters and simulated caller ID) and returns predefined or randomized responses. This is for demonstration and development purposes to illustrate how a client would be structured and used.

## Build

To build the off-chain client, navigate to this `offchain_client` directory:

```bash
cargo build
```
The executable will be located in `target/debug/offchain_client`.

## Example Commands

All commands are run from the `pension_management/offchain_client` directory after building the client.

1.  **Get contract owner (query):**
    This command simulates querying the smart contract for the owner's `AccountId`.
    ```bash
    ./target/debug/offchain_client get-contract-owner
    ```
    *(The `--node-url` and `--contract-address` flags can be omitted to use their default simulation values.)*

2.  **Register a company (command, admin action - simulated):**
    This command simulates the contract owner registering a new company.
    ```bash
    ./target/debug/offchain_client register-company --company-id 5FHneW46xGXgs5gUiveU4sbTyGBzmstUspZC92UhjJM694ty
    ```
    *(The `caller_id` for this admin action is assumed to be the contract owner within the simulation.)*

3.  **A pensioner gets their payout estimate (query, pensioner action - simulated):**
    This command simulates a pensioner querying their estimated future pension payout. The `--pensioner-id-as-caller` flag is used here to specify the identity of the pensioner making the call in the simulation.
    ```bash
    ./target/debug/offchain_client get-my-payout-estimate --pensioner-id-as-caller 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
    ```

4.  **An authorized company updates a pensioner's employment status (command - simulated):**
    ```bash
    ./target/debug/offchain_client update-employment --company-id-as-caller 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y --pensioner-id 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --years 15 --salary 75000 --status Active
    ```

**Note on `*_as_caller` flags:** Flags like `--pensioner-id-as-caller` or `--company-id-as-caller` are used in the simulation to specify who is contextually making the call, which is important for authorization logic in the smart contract. In a real client interacting with a live network, the caller's identity would typically be derived from a cryptographic key pair used to sign the transaction.

For a full list of commands and their options, use:
```bash
./target/debug/offchain_client --help
```
