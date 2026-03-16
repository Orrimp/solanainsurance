---
name: build-test-deploy
description: "Compile, test, and deploy the Solana Insurance program. Use for: running the full CI pipeline (cargo check → cargo build-sbf → cargo test → solana deploy → example client); verifying build health; producing a structured pipeline report. Works with the solana-ci agent and run_pipeline.sh script."
argument-hint: "Flags: --check | --build | --test | --deploy | --client (combine freely)"
---

# build-test-deploy Skill

Runs the full or partial CI pipeline for the Solana Insurance on-chain program and produces a
structured results report using the [pipeline report template](./templates/pipeline-report.md).

## When to Use

- Before committing: quick `--check --test` to catch regressions
- Before deploying: full `--build --test --deploy` to guarantee a clean program binary goes on-chain
- During story verification: confirm all acceptance criteria pass as LiteSVM tests
- After a refactor: `--build --test` to validate nothing broke

## Pipeline Steps

Each step is only executed when the corresponding flag is passed. Steps run in order; a failure
at any step stops the pipeline and skips all subsequent steps.

### Step 1 — `--check`: Type Check

```bash
cargo check
```

Validates Rust types and imports without producing a binary. Fast feedback (~1–5 s).
Expected outcome: `Finished dev profile` with only pre-existing warnings.

### Step 2 — `--build`: BPF Binary Build

```bash
cargo build-sbf
```

Compiles the program for the `sbpf-solana-solana` target and writes `target/deploy/insurance.so`.
Required before running LiteSVM tests that load the binary, and before any deployment.
Expected outcome: `Finished release profile`.

### Step 3 — `--test`: Run Test Suite

```bash
cargo test 2>&1
```

Runs all `#[test]` functions in `src/lib.rs` (and any other test modules). LiteSVM loads
`target/deploy/insurance.so` — the `--build` step must have run first in this session.
Expected outcome: `test result: ok. N passed; 0 failed`.

Targeted run (specific module):
```bash
cargo test initialize_pensioner -- --nocapture
```

### Step 4 — `--deploy`: Deploy to Local Validator

```bash
solana program deploy -u localhost ./target/deploy/insurance.so
```

**Validator auto-start**: if `localhost:8899` is not reachable the pipeline automatically runs
`solana-test-validator --reset` in the background, waits up to 30 seconds (configurable via
`--validator-timeout N`), and airdrops 2 SOL to the default keypair. The validator process stays
running after the pipeline exits; its output is written to `target/validator.log`.

If you prefer to manage the validator manually:
```bash
# Terminal 1 — keep open:
solana-test-validator --reset

# Terminal 2:
solana airdrop 2 -u localhost
```

Check the current program address at any time:
```bash
solana address -k ./target/deploy/insurance-keypair.json
```

Expected outcome: `Program Id: <pubkey>` followed by a deployment signature.

### Step 5 — `--client`: Run Example Client

```bash
cargo run --example client
```

Runs the off-chain demo flow in `client/client.rs` against the deployed program.
Expected outcome: a transaction signature printed to stdout.

## Execution

Run the bundled pipeline script directly:

```bash
bash .github/skills/build-test-deploy/scripts/run_pipeline.sh [FLAGS]
```

See [run_pipeline.sh](./scripts/run_pipeline.sh) for the full implementation.

## Output

After execution, fill in the [pipeline report template](./templates/pipeline-report.md) with the
actual results. Replace every `{{placeholder}}` with the real value captured from stdout/stderr.
Print the completed report as the final response.
