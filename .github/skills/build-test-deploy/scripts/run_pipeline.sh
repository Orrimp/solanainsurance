#!/usr/bin/env bash
# =============================================================================
# run_pipeline.sh — Solana Insurance CI Pipeline
# =============================================================================
# Compiles, tests, and optionally deploys the on-chain program.
# Each step only runs when its flag is passed. A failure stops the pipeline.
#
# Usage:
#   bash run_pipeline.sh [FLAGS]
#
# Flags (combine freely):
#   --check               Run `cargo check` (type-check only, no binary)
#   --build               Run `cargo build-sbf` (produces target/deploy/insurance.so)
#   --test                Run `cargo test` (LiteSVM tests; requires --build to have run first)
#   --deploy              Deploy to local validator at http://localhost:8899
#   --client              Run the example client after deploy
#   --all                 Equivalent to --check --build --test --deploy --client
#   --validator-timeout N Seconds to wait for auto-started validator (default: 30)
#
# Validator auto-start (--deploy / --client):
#   If localhost:8899 is not reachable the script starts `solana-test-validator --reset`
#   in the background, waits up to --validator-timeout seconds, then airdrops 2 SOL to
#   the default keypair.  The validator process is NOT stopped when the pipeline exits.
#
# Exit codes:
#   0  All requested steps passed
#   1  One or more steps failed (first failure stops pipeline)
#   2  Precondition not met (validator could not be started)
#
# Example:
#   bash run_pipeline.sh --build --test
#   bash run_pipeline.sh --all
# =============================================================================
set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; BOLD='\033[1m'; RESET='\033[0m'

log()     { echo -e "${BLUE}[pipeline]${RESET} $*"; }
success() { echo -e "${GREEN}[pipeline] ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}[pipeline] ⚠${RESET} $*"; }
fail()    { echo -e "${RED}[pipeline] ✗${RESET} $*"; }
header()  { echo -e "\n${BOLD}══════════════════════════════════════════${RESET}"; echo -e "${BOLD}  $*${RESET}"; echo -e "${BOLD}══════════════════════════════════════════${RESET}"; }

# ── Argument parsing ──────────────────────────────────────────────────────────
DO_CHECK=false; DO_BUILD=false; DO_TEST=false; DO_DEPLOY=false; DO_CLIENT=false
VALIDATOR_TIMEOUT=30

for arg in "$@"; do
  case "$arg" in
    --check)              DO_CHECK=true ;;
    --build)              DO_BUILD=true ;;
    --test)               DO_TEST=true ;;
    --deploy)             DO_DEPLOY=true ;;
    --client)             DO_CLIENT=true ;;
    --all)                DO_CHECK=true; DO_BUILD=true; DO_TEST=true; DO_DEPLOY=true; DO_CLIENT=true ;;
    --validator-timeout)  shift; VALIDATOR_TIMEOUT="$1" ;;
    --validator-timeout=*)VALIDATOR_TIMEOUT="${arg#*=}" ;;
    *) warn "Unknown flag: $arg (ignored)" ;;
  esac
done

# Default: if no flags given, run build + test
if ! $DO_CHECK && ! $DO_BUILD && ! $DO_TEST && ! $DO_DEPLOY && ! $DO_CLIENT; then
  DO_BUILD=true; DO_TEST=true
fi

# ── Metadata ──────────────────────────────────────────────────────────────────
PIPELINE_START=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Resolve workspace root: three levels up from this script's location
# (.github/skills/build-test-deploy/scripts/ → project root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DEPLOY_SO="$WORKSPACE_ROOT/target/deploy/insurance.so"
KEYPAIR="$WORKSPACE_ROOT/target/deploy/insurance-keypair.json"
RPC_URL="${RPC_URL:-http://localhost:8899}"

cd "$WORKSPACE_ROOT"

header "Solana Insurance — CI Pipeline"
log "Workspace : $WORKSPACE_ROOT"
log "Started   : $PIPELINE_START"
log "Steps     : $([ $DO_CHECK = true ] && echo "check ") $([ $DO_BUILD = true ] && echo "build ") $([ $DO_TEST = true ] && echo "test ") $([ $DO_DEPLOY = true ] && echo "deploy ") $([ $DO_CLIENT = true ] && echo "client")"
echo ""

# ── Tracking variables ────────────────────────────────────────────────────────
STEP_RESULTS=()   # "STEP_NAME:PASS|FAIL|SKIP"
OVERALL=0

record() { STEP_RESULTS+=("$1:$2"); }
passed() { record "$1" "PASS"; success "$1 passed"; }
failed() { record "$1" "FAIL"; fail "$1 FAILED"; OVERALL=1; }
skipped() { record "$1" "SKIP"; warn "$1 skipped"; }

# ── STEP 1: cargo check ───────────────────────────────────────────────────────
if $DO_CHECK; then
  header "Step 1 — cargo check"
  if cargo check 2>&1; then
    passed "check"
  else
    failed "check"
    fail "Type-check failed. Fix compilation errors before proceeding."
    exit 1
  fi
else
  skipped "check"
fi

# ── STEP 2: cargo build-sbf ───────────────────────────────────────────────────
if $DO_BUILD; then
  header "Step 2 — cargo build-sbf"
  if cargo build-sbf 2>&1; then
    if [[ -f "$DEPLOY_SO" ]]; then
      BIN_SIZE=$(du -k "$DEPLOY_SO" | cut -f1)
      passed "build"
      log "Binary  : $DEPLOY_SO (${BIN_SIZE} KB)"
    else
      failed "build"
      fail "Build succeeded but $DEPLOY_SO not found."
      exit 1
    fi
  else
    failed "build"
    fail "BPF build failed. Resolve compiler errors then retry."
    exit 1
  fi
else
  skipped "build"
fi

# ── STEP 3: cargo test ────────────────────────────────────────────────────────
if $DO_TEST; then
  header "Step 3 — cargo test"

  # Verify the .so exists for LiteSVM tests
  if [[ ! -f "$DEPLOY_SO" ]]; then
    fail "target/deploy/insurance.so not found — run with --build first."
    failed "test"
    exit 1
  fi

  TEST_OUTPUT=$(cargo test 2>&1)
  TEST_EXIT=$?
  echo "$TEST_OUTPUT"

  PASSED_COUNT=$(echo "$TEST_OUTPUT" | grep -Eo '[0-9]+ passed' | awk '{sum+=$1} END{print sum+0}')
  FAILED_COUNT=$(echo "$TEST_OUTPUT" | grep -Eo '[0-9]+ failed' | awk '{sum+=$1} END{print sum+0}')

  if [[ $TEST_EXIT -eq 0 ]]; then
    passed "test"
    log "Tests: ${PASSED_COUNT} passed, ${FAILED_COUNT} failed"
  else
    failed "test"
    log "Tests: ${PASSED_COUNT} passed, ${FAILED_COUNT} FAILED"
    fail "Test suite failed. Fix failing tests before deploying."
    exit 1
  fi
else
  skipped "test"
fi

# ── Validator auto-start helper ──────────────────────────────────────────────
# Starts solana-test-validator in the background and waits until it responds.
# Stores the PID in VALIDATOR_PID (0 if validator was already running).
VALIDATOR_PID=0
ensure_validator() {
  if solana cluster-version -u "$RPC_URL" &>/dev/null; then
    log "Validator already running at $RPC_URL"
    return 0
  fi

  warn "Validator not detected — starting solana-test-validator --reset in background..."
  # Redirect validator output to a log file to keep pipeline stdout clean.
  VALIDATOR_LOG="$WORKSPACE_ROOT/target/validator.log"
  mkdir -p "$WORKSPACE_ROOT/target"
  solana-test-validator --reset > "$VALIDATOR_LOG" 2>&1 &
  VALIDATOR_PID=$!
  log "Validator PID : $VALIDATOR_PID  (log: $VALIDATOR_LOG)"

  local elapsed=0
  while [[ $elapsed -lt $VALIDATOR_TIMEOUT ]]; do
    if solana cluster-version -u "$RPC_URL" &>/dev/null; then
      success "Validator ready after ${elapsed}s"
      # Fund the default keypair so deploy fees are covered.
      local airdrop_out
      airdrop_out=$(solana airdrop 2 -u "$RPC_URL" 2>&1) && \
        log "Airdrop: $airdrop_out" || \
        warn "Airdrop failed (keypair may already be funded): $airdrop_out"
      return 0
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done

  fail "Validator did not become ready within ${VALIDATOR_TIMEOUT}s."
  fail "Check $VALIDATOR_LOG for details."
  return 1
}

# ── STEP 4: solana deploy ─────────────────────────────────────────────────────
if $DO_DEPLOY; then
  header "Step 4 — solana program deploy"

  # Safety gate: tests must have run (or --test was explicitly part of this run)
  if ! $DO_TEST; then
    warn "Deploying without running tests. Pass --test to enforce test gate."
  fi

  # Ensure validator is running, auto-starting if necessary.
  if ! ensure_validator; then
    failed "deploy"
    exit 2
  fi

  DEPLOY_OUTPUT=$(solana program deploy -u "$RPC_URL" "$DEPLOY_SO" 2>&1)
  DEPLOY_EXIT=$?
  echo "$DEPLOY_OUTPUT"

  if [[ $DEPLOY_EXIT -eq 0 ]]; then
    PROGRAM_ID=$(echo "$DEPLOY_OUTPUT" | grep 'Program Id:' | awk '{print $NF}' || echo "unknown")
    passed "deploy"
    log "Program ID : $PROGRAM_ID"
    log "RPC URL    : $RPC_URL"
  else
    failed "deploy"
    fail "Deployment failed. Check validator logs and keypair funding."
    exit 1
  fi
else
  skipped "deploy"
fi

# ── STEP 5: example client ────────────────────────────────────────────────────
if $DO_CLIENT; then
  header "Step 5 — cargo run --example client"

  if ! $DO_DEPLOY; then
    warn "Running client without a fresh deploy. Program may be stale."
  fi

  if cargo run --example client 2>&1; then
    passed "client"
  else
    failed "client"
    fail "Example client failed. Check RPC connectivity and program ID."
    exit 1
  fi
else
  skipped "client"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
PIPELINE_END=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
header "Pipeline Summary"
log "Finished : $PIPELINE_END"
echo ""
for result in "${STEP_RESULTS[@]}"; do
  STEP="${result%%:*}"
  STATUS="${result##*:}"
  case "$STATUS" in
    PASS) echo -e "  ${GREEN}✓ PASS${RESET}  $STEP" ;;
    FAIL) echo -e "  ${RED}✗ FAIL${RESET}  $STEP" ;;
    SKIP) echo -e "  ${YELLOW}– SKIP${RESET}  $STEP" ;;
  esac
done
echo ""

if [[ $OVERALL -eq 0 ]]; then
  success "All steps passed."
else
  fail "Pipeline completed with failures."
fi

exit $OVERALL
