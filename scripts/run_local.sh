#!/usr/bin/env bash
set -euo pipefail

# Run a local Solana validator, build + deploy the program, and run the example client.
# Works with Solana CLI localnet. Safe defaults; configurable via flags.
#
# Usage:
#   bash scripts/run_local.sh [--airdrop 2] [--keep-validator] [--skip-build] [--skip-client]
#
# Notes:
# - Requires: solana, solana-test-validator, cargo
# - Tries `cargo build-sbf` first; falls back to `solana program build`
# - Program artifact paths default to ./target/deploy/Insurance.(so|json)

AIRDROP=2
KEEP_VALIDATOR=false
SKIP_BUILD=false
SKIP_CLIENT=false
PROGRAM_BASENAME=${PROGRAM_BASENAME:-Insurance}
PROGRAM_SO=${PROGRAM_SO:-"./target/deploy/${PROGRAM_BASENAME}.so"}
PROGRAM_KEYPAIR=${PROGRAM_KEYPAIR:-"./target/deploy/${PROGRAM_BASENAME}-keypair.json"}
RPC_URL=${RPC_URL:-"http://127.0.0.1:8899"}

log() { echo "[run_local] $*"; }
err() { echo "[run_local][ERROR] $*" >&2; }

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    err "Missing required command: $1"; exit 1
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --airdrop)
        AIRDROP=${2:-2}; shift 2;;
      --keep-validator)
        KEEP_VALIDATOR=true; shift;;
      --skip-build)
        SKIP_BUILD=true; shift;;
      --skip-client)
        SKIP_CLIENT=true; shift;;
      -h|--help)
        cat <<EOF
Usage: bash scripts/run_local.sh [options]
  --airdrop N         Amount of SOL to airdrop to default keypair (default: 2)
  --keep-validator    Don't stop the validator on exit
  --skip-build        Skip building the on-chain program
  --skip-client       Skip running the example client
  -h, --help          Show this help and exit
EOF
        exit 0;;
      *)
        err "Unknown option: $1"; exit 1;;
    esac
  done
}

start_validator() {
  log "Starting local validator (resetting ledger) …"
  solana-test-validator --reset --quiet > .localnet.log 2>&1 &
  VALIDATOR_PID=$!
  log "Validator PID: ${VALIDATOR_PID} (logs: .localnet.log)"
}

stop_validator() {
  if [[ -n "${VALIDATOR_PID:-}" ]] && ps -p "$VALIDATOR_PID" >/dev/null 2>&1; then
    log "Stopping local validator (PID ${VALIDATOR_PID}) …"
    kill "$VALIDATOR_PID" >/dev/null 2>&1 || true
    wait "$VALIDATOR_PID" 2>/dev/null || true
  fi
}

wait_for_rpc() {
  log "Waiting for RPC at ${RPC_URL} …"
  local tries=0
  until solana -u "$RPC_URL" cluster-version >/dev/null 2>&1; do
    tries=$((tries+1))
    if [[ $tries -gt 60 ]]; then
      err "RPC didn't come up after 60 tries (~30s). See .localnet.log"; exit 1
    fi
    sleep 0.5
  done
  log "RPC is up."
}

ensure_keypair() {
  # Read keypair path from solana config
  local kp
  kp=$(solana config get | awk -F': ' '/Keypair Path/ {print $2}') || true
  if [[ -z "$kp" || "$kp" == "ASK" || "$kp" == prompt:* ]]; then
    kp="$HOME/.config/solana/id.json"
    log "Setting default keypair path to $kp"
    solana config set --keypair "$kp" >/dev/null
  fi
  if [[ ! -f "$kp" ]]; then
    log "Generating default keypair at $kp (no passphrase) …"
    mkdir -p "$(dirname "$kp")"
    solana-keygen new --no-bip39-passphrase -o "$kp" -f >/dev/null
  fi
  log "Using keypair: $kp"
}

fund_wallet() {
  log "Airdropping ${AIRDROP} SOL …"
  if ! solana -u "$RPC_URL" airdrop "$AIRDROP" >/dev/null 2>&1; then
    # Sometimes first airdrop fails while validator warms up; retry a few times
    local i
    for i in 1 2 3 4 5; do
      sleep 1
      if solana -u "$RPC_URL" airdrop "$AIRDROP" >/dev/null 2>&1; then
        break
      fi
      [[ $i -eq 5 ]] && { err "Airdrop failed after retries"; exit 1; }
    done
  fi
  solana -u "$RPC_URL" balance
}

build_program() {
  $SKIP_BUILD && { log "Skipping build (per flag)"; return; }
  log "Building on-chain program …"
  if command -v cargo-build-sbf >/dev/null 2>&1; then
    cargo build-sbf
  else
    log "cargo-build-sbf not found, trying 'solana program build' …"
    solana program build
  fi
  if [[ ! -f "$PROGRAM_SO" ]]; then
    err "Program artifact not found at $PROGRAM_SO"; exit 1
  fi
}

deploy_program() {
  log "Deploying program: $PROGRAM_SO …"
  local out
  if ! out=$(solana program deploy -u "$RPC_URL" "$PROGRAM_SO" 2>&1); then
    echo "$out" >&2
    err "Deploy failed"; exit 1
  fi
  echo "$out" | tee .deploy.out >/dev/null
  PROGRAM_ID=$(echo "$out" | awk '/Program Id:/ {print $3}' | tail -n1)
  if [[ -z "${PROGRAM_ID:-}" ]]; then
    err "Could not parse Program Id from deploy output"; exit 1
  fi
  log "Program Id: ${PROGRAM_ID}"
}

run_client() {
  $SKIP_CLIENT && { log "Skipping example client (per flag)"; return; }
  log "Running example client …"
  cargo run --example client
}

main() {
  parse_args "$@"
  require_cmd solana
  require_cmd solana-test-validator
  require_cmd cargo

  trap '[[ "$KEEP_VALIDATOR" == true ]] || stop_validator' EXIT

  start_validator
  wait_for_rpc
  solana config set --url "$RPC_URL" >/dev/null
  ensure_keypair
  fund_wallet
  build_program
  deploy_program
  run_client

  log "Done. ${KEEP_VALIDATOR:+Validator left running.}" 
}

main "$@"
