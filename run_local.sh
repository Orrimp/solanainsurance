#!/bin/bash
# This script automates the process of building, deploying, and testing the Solana program locally.

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
# URL for the local validator
RPC_URL="http://127.0.0.1:8899"
# Path to the program's keypair
PROGRAM_KEYPAIR="./target/deploy/Insurance-keypair.json"
rustup override set 1.86.0

# --- Step 1: Clean and Build ---
echo "--- Cleaning and Building Program ---"
# Clean previous build artifacts to prevent dependency issues
cargo clean
# Build the Solana program
cargo build-sbf
echo "Build complete."


# --- Step 2: Start Local Validator ---
# Check if a validator is already running, if not, start one.
if ! pgrep -f "solana-test-validator" > /dev/null; then
    echo "--- Starting solana-test-validator ---"
    # Start a new validator in the background and wait for it to be ready
    solana-test-validator --reset > /dev/null 2>&1 &
    VALIDATOR_PID=$!
    # Add a trap to kill the validator when the script exits
    trap "echo '--- Stopping solana-test-validator (PID: $VALIDATOR_PID) ---'; kill $VALIDATOR_PID" EXIT
    # Wait a moment for the validator to initialize
    sleep 5
    echo "Validator started."
else
    echo "--- Validator already running ---"
fi


# --- Step 3: Deploy Program ---
echo "--- Deploying Program ---"
# Deploy the program to the local validator
solana program deploy ./target/deploy/Insurance.so --url $RPC_URL
# Get the program ID from the keypair file
PROGRAM_ID=$(solana address -k $PROGRAM_KEYPAIR)
echo "Program deployed with ID: $PROGRAM_ID"


# --- Step 4: Run Client ---
echo "--- Running Client ---"
# Export the program ID so the client can use it
export PROGRAM_ID
# Run the client example to interact with the deployed program
cargo run --example client
echo "Client execution finished."

echo "--- Script finished successfully ---"
