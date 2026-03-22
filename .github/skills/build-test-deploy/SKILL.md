---
name: build-test-deploy
description: "Compile, test, and deploy the Solana program. Use for: running the full CI pipeline (cargo check → cargo build-sbf → cargo test → solana deploy → example client); verifying build health; producing a structured pipeline report. Uses the Solana Toolbox pipeline command."
argument-hint: "Flags: --check | --build | --test | --deploy | --client | --all (combine freely)"
---

# build-test-deploy Skill

Runs the full or partial CI pipeline for the Solana on-chain program using the **Solana Toolbox**
`pipeline` command. Produces a structured results report with step tracking (PASS/FAIL/SKIP) and
colored output.

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

Compiles the program for the `sbpf-solana-solana` target and writes `target/deploy/<program>.so`.
Required before running LiteSVM tests that load the binary, and before any deployment.
Expected outcome: `Finished release profile`.

### Step 3 — `--test`: Run Test Suite

```bash
cargo test 2>&1
```

Runs all `#[test]` functions in `src/lib.rs` (and any other test modules). LiteSVM loads
`target/deploy/<program>.so` — the `--build` step must have run first in this session.
Expected outcome: `test result: ok. N passed; 0 failed`.

Targeted run (specific module):
```bash
cargo test initialize -- --nocapture
```

### Step 4 — `--deploy`: Deploy to Local Validator

```bash
solana program deploy -u localhost ./target/deploy/<program>.so
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
solana address -k ./target/deploy/<program>-keypair.json
```

Expected outcome: `Program Id: <pubkey>` followed by a deployment signature.

### Step 5 — `--client`: Run Example Client

```bash
cargo run --example client
```

Runs the off-chain demo flow in `client/client.rs` against the deployed program.
Expected outcome: a transaction signature printed to stdout.

## Execution

### Using Solana Toolbox (Recommended)

The toolbox provides a unified `pipeline` command that replaces the shell script:

```bash
# Build the toolbox first (one-time setup)
cargo build --release --manifest-path toolbox/cli/Cargo.toml

# Run pipeline with selected steps
cargo run --release --manifest-path toolbox/cli/Cargo.toml -- pipeline [FLAGS]
```

Or install the toolbox globally:
```bash
cargo install --path toolbox/cli
solana-toolbox pipeline [FLAGS]
```

#### Common Workflows

**Build + Test** (Before commit):
```bash
solana-toolbox pipeline --build --test
```

**Full CI/CD** (All steps):
```bash
solana-toolbox pipeline --all
```
Equivalent to: `--check --build --test --deploy --client`

**Type Check Only** (Fast feedback):
```bash
solana-toolbox pipeline --check
```

**Build + Test + Deploy**:
```bash
solana-toolbox pipeline --build --test --deploy
```

**Deploy Existing Binary**:
```bash
solana-toolbox pipeline --deploy
```

**Custom Validator Timeout**:
```bash
solana-toolbox pipeline --deploy --validator-timeout 60
```

### Direct Cargo Commands (Fallback)

If the toolbox is unavailable, use cargo commands directly:

```bash
# Step 1: Check
cargo check

# Step 2: Build
cargo build-sbf

# Step 3: Test
cargo test

# Step 4: Deploy (start validator first)
solana-test-validator --reset &  # Terminal 1
solana airdrop 2 -u localhost
solana program deploy -u localhost target/deploy/<program>.so

# Step 5: Client
cargo run --bin client
```

## Output

The `pipeline` command automatically produces:
- **Colored step-by-step progress** with emoji indicators
- **Structured summary** showing PASS/FAIL/SKIP for each step
- **Timestamps** and duration
- **Binary size** (for build step)
- **Test count** (for test step)
- **Program ID** (for deploy step)

Example output:
```
════════════════════════════════════════════════════════════
  Solana CI/CD Pipeline
════════════════════════════════════════════════════════════
   Started: 2026-03-22 10:00:00 UTC
   Steps  : build test deploy

──────────────────────────────────────────────────────────
Step 2 — cargo build-sbf
──────────────────────────────────────────────────────────
   🔨 Building Solana program...
   ✅ Build successful
   📦 Binary: target/deploy/insurance.so (45 KB)

──────────────────────────────────────────────────────────
Step 3 — cargo test
──────────────────────────────────────────────────────────
   🧪 Running cargo test...
   ✅ Tests passed: 12

──────────────────────────────────────────────────────────
Step 4 — solana program deploy
──────────────────────────────────────────────────────────
   🔍 Checking validator status...
   ⚠️ Validator not detected — starting solana-test-validator...
   ✅ Validator started (PID: 12345)
   ⏳ Waiting for validator to be ready (timeout: 30s)...
   ✅ Validator ready after 2.5s
   💰 Funding default keypair...
   ✅ Airdrop successful (2 SOL)
   📦 Deploying program...
   ✅ Program deployed
   📍 Program ID: 8X5Yfj...K9Zw
   🌐 RPC: http://localhost:8899

════════════════════════════════════════════════════════════
  Pipeline Summary
════════════════════════════════════════════════════════════
   Finished: 2026-03-22 10:00:15 UTC
   Duration: 15.234s

   ⊝ SKIP  check
   ✅ PASS  build
   ✅ PASS  test
   ✅ PASS  deploy
   ⊝ SKIP  client

════════════════════════════════════════════════════════════
  ✅ All Steps Passed
════════════════════════════════════════════════════════════
```

## Agent Workflow

When invoked as `@solana-ci`:

1. **Load this skill** — read `.github/skills/build-test-deploy/SKILL.md`
2. **Determine steps** — analyze user request for which flags to pass
3. **Execute pipeline** — run `solana-toolbox pipeline [FLAGS]`
4. **Report results** — copy the summary section (PASS/FAIL/SKIP table) into response
5. **Handle failures** — if any step fails, diagnose from the error output and suggest fixes

### Reporting Options

**Option 1: Direct Copy (Recommended)**

The toolbox pipeline output is already well-formatted. Simply copy the summary section:

```
════════════════════════════════════════════════════════════
  Pipeline Summary
════════════════════════════════════════════════════════════
   Finished: 2026-03-22 10:15:30 UTC
   Duration: 12.456s

   ✅ PASS  check
   ✅ PASS  build
   ✅ PASS  test
   ✅ PASS  deploy
   ⊝ SKIP  client

════════════════════════════════════════════════════════════
  ✅ All Steps Passed
════════════════════════════════════════════════════════════
```

**Option 2: Structured Report**

For detailed analysis, use the [toolbox-pipeline-report.md](./templates/toolbox-pipeline-report.md) template:

1. Capture full pipeline output
2. Fill template placeholders with extracted values
3. Include relevant sections (test results, deploy info, etc.)

### Common Failure Scenarios

**Build Failure:**
```
❌ Build failed
Stdout: [compilation errors...]
```
→ Review Rust errors, fix code, re-run with `--build`

**Test Failure:**
```
❌ Tests failed: 2 passed, 1 failed
```
→ Run specific test with `solana_test` MCP tool using `nocapture: true` for detailed logs

**Deploy Failure:**
```
❌ Validator startup failed: Validator did not become ready within 30s
```
→ Increase timeout: `solana-toolbox pipeline --deploy --validator-timeout 60`

**Client Failure:**
```
❌ Client failed: Program account not found
```
→ Ensure deployment succeeded first: `solana-toolbox pipeline --deploy --client`

---

## Examples

### Quick Pre-Commit Check

**User says:** "Check if my changes compile and pass tests"

**Agent response:**
```
Running build and test pipeline...

[Execute: solana-toolbox pipeline --build --test]

✅ Pipeline completed successfully

   ✅ PASS  build (45 KB binary)
   ✅ PASS  test (12 tests passed)

Your changes are ready to commit!
```

### Full CI/CD Deployment

**User says:** "Deploy the program and run the client"

**Agent response:**
```
Running full CI/CD pipeline...

[Execute: solana-toolbox pipeline --all]

✅ All steps passed

   ✅ PASS  check
   ✅ PASS  build
   ✅ PASS  test
   ✅ PASS  deploy (Program ID: 8X5Yfj...K9Zw)
   ✅ PASS  client

Program deployed to http://localhost:8899
Client execution successful
```

### Debugging Failed Tests

**User says:** "Run the tests"

**Agent response:**
```
Running test suite...

[Execute: solana-toolbox pipeline --test]

❌ Pipeline failed at step: test

Tests: 11 passed, 1 failed

To debug the failing test, I'll run it with full output:
[Execute: solana_test tool with nocapture: true, test_name: "test_transfer"]

The test fails because the account balance is insufficient. 
You need to fund the account in the test setup.
```

---

## Migration from Shell Script

Legacy shell workflow has been removed.
Use the Rust-based toolbox pipeline command for all CI/CD operations:

| Workflow | Toolbox Command |
|----------|-----------------|
| Check only | `solana-toolbox pipeline --check` |
| Full pipeline | `solana-toolbox pipeline --all` |
| Custom validator timeout | `solana-toolbox pipeline --validator-timeout 60` |

**Benefits of toolbox:**
- ✅ Cross-platform (Windows, Linux, macOS)
- ✅ MCP tool integration (agents can call directly)
- ✅ Structured error handling
- ✅ Better output formatting
- ✅ No bash/shell dependencies
- ✅ Type-safe configuration
