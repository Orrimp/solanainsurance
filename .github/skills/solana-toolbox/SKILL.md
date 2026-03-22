---
name: solana-toolbox
description: "Use the Solana MCP toolbox for AI-assisted development. Provides 5 tools: solana_build (compile program), solana_test (run tests), solana_pipeline (run CI/CD pipeline), create_instruction (scaffold new instruction), validate_architecture (check AGENTS.md compliance). Use for: rapid feature development, CI/CD automation, automated validation, test execution, and code generation following project architecture patterns."
argument-hint: "Specify tool name and parameters: solana_build, solana_test, solana_pipeline, create_instruction, or validate_architecture"
applyTo:
  - "**/*.rs"
  - "**/*.toml"
  - "**/AGENTS.md"
---

# solana-toolbox Skill

Interact with the Solana MCP server to build, test, run CI/CD pipelines, scaffold, and validate the on-chain program using AI-native tooling.

## Available MCP Tools

The toolbox exposes 5 tools via Model Context Protocol (JSON-RPC 2.0):

### 1. `solana_build` - Compile the Program

**Purpose:** Compile the Solana on-chain program to BPF bytecode.

**Parameters:**
- `check_only` (boolean, optional, default: false)
  - `true`: Run `cargo check` only (fast type checking)
  - `false`: Run `cargo build-sbf` (full BPF compilation)

**When to use:**
- Before running tests (full build required for LiteSVM)
- Quick syntax validation (`check_only: true`)
- Pre-deployment verification
- After editing processor or instruction handlers

**Example invocations:**

```
Build the Solana program
```

```
Run cargo check on the program
```

**Expected output:**
- Success: Build logs, binary path (`target/deploy/<program>.so`), compile time
- Failure: Compilation errors with file paths and line numbers

**Common errors:**
- Missing dependencies → Run `cargo update`
- Type mismatches → Check account validation in processor
- Borsh serialization issues → Ensure all state structs derive `BorshSerialize` + `BorshDeserialize`

---

### 2. `solana_test` - Run Test Suite

**Purpose:** Execute LiteSVM tests for the on-chain program.

**Parameters:**
- `test_name` (string, optional)
  - If provided: Runs only the specified test
  - If omitted: Runs all tests
- `nocapture` (boolean, optional, default: false)
  - `true`: Shows `msg!()` output from program logs (`--nocapture`)
  - `false`: Suppresses output (cleaner for CI)

**When to use:**
- After modifying processor logic
- Before committing code
- To debug specific instruction handlers (use `nocapture: true`)
- Continuous integration validation

**Example invocations:**

```
Run all tests
```

```
Run the test_transfer test with output
```

```
Run tests for the update_state instruction
```

**Expected output:**
- Success: `test result: ok. N passed; 0 failed`
- Failure: Failed test names, assertion errors, program logs

**Test file location:**
- `src/lib.rs` - Unit tests (under `#[cfg(test)]`)
- `tests/` - Integration tests

**Common failures:**
- Account validation errors → Check signer/mutability requirements
- Insufficient balance → Ensure test setup funds accounts
- Incorrect account ordering → Match processor expectations
- Borsh deserialization → Verify state struct layout matches

---

### 3. `solana_pipeline` - Run CI/CD Pipeline

**Purpose:** Execute a configurable CI/CD pipeline with automatic step tracking, validator management, and structured reporting.

**Parameters:**
- `check` (boolean, optional, default: false)
  - Run `cargo check` for type-checking
- `build` (boolean, optional, default: false)
  - Run `cargo build-sbf` to compile the program
- `test` (boolean, optional, default: false)
  - Run `cargo test` (all tests including LiteSVM)
- `deploy` (boolean, optional, default: false)
  - Deploy program to local validator (auto-starts validator if needed)
- `client` (boolean, optional, default: false)
  - Run the example client (`cargo run --bin client`)
- `all` (boolean, optional, default: false)
  - Run all steps in sequence (equivalent to enabling all flags)
- `validator_timeout` (integer, optional, default: 30)
  - Maximum seconds to wait for validator startup (1-300)

**Pipeline Flow:**
1. **check** → Type-check without building (fast feedback)
2. **build** → Compile to BPF bytecode (`target/deploy/<program>.so`)
3. **test** → Run all tests (requires build step first)
4. **deploy** → Deploy to `localhost:8899` (auto-starts validator)
5. **client** → Execute example client (requires validator + deployed program)

**Step Tracking:**
- Each step reports: ✅ PASS | ❌ FAIL | ⊝ SKIP
- Pipeline stops on first failure
- Structured summary with timestamps and duration

**Validator Auto-Start:**
- If `localhost:8899` is unreachable, automatically starts `solana-test-validator --reset`
- Waits up to `validator_timeout` seconds for validator to be ready
- Airdrops 2 SOL to default keypair for deployment fees
- Validator continues running after pipeline exits

**When to use:**
- **Before commit:** `check: true, test: true` (fast validation)
- **Before deploy:** `build: true, test: true, deploy: true` (full CI)
- **Full verification:** `all: true` (end-to-end smoke test)
- **Quick check:** `check: true` (syntax validation only)
- **Deploy only:** `deploy: true` (deploy existing binary)

**Example invocations:**

```
Run the full CI/CD pipeline (all steps)
```

```
Run build and test steps only
```

```
Deploy the program to local validator with 60-second timeout
```

```
Run type-check only for quick feedback
```

**Expected output:**
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

**Common issues:**
- **Build fails:** Compilation errors → Fix Rust code, re-run
- **Test fails:** Assertion errors → Debug with `solana_test` tool using `nocapture: true`
- **Deploy fails:** Validator startup timeout → Increase `validator_timeout` or check if port 8899 is blocked
- **Client fails:** Program not deployed → Run with `deploy: true` first

**Pipeline vs. Individual Tools:**
- Use `solana_pipeline` for **multi-step workflows** (CI/CD, pre-commit, full validation)
- Use individual tools (`solana_build`, `solana_test`) for **targeted operations** (debug single test, quick build)

---

### 4. `create_instruction` - Scaffold New Instruction

**Purpose:** Generate a complete instruction implementation following AGENTS.md architecture patterns.

**Parameters:**
- `name` (string, required)
  - Instruction name in **PascalCase** (e.g., `Initialize`, `Transfer`)
- `accounts` (array, required)
  - Each account has:
    - `name` (string): Account name in **snake_case** (e.g., `state_account`, `authority`)
    - `mutable` (boolean): Is account writable?
    - `signer` (boolean): Must account sign the transaction?
- `data_fields` (array, optional)
  - Each field has:
    - `name` (string): Field name in snake_case
    - `rust_type` (string): Rust type (e.g., `u64`, `Pubkey`, `i64`, `[u8; 32]`)

**Generates 4 files:**
1. **Instruction enum variant** → `src/instructions.rs`
2. **Processor handler** → `src/processor.rs`
3. **Client builder** → `client/instructions.rs`
4. **Test template** → `tests/test_{instruction_name}.rs`

**When to use:**
- Starting a new feature (story implementation)
- Adding a new on-chain operation
- Following TDD (test template generated)
- Maintaining consistency with AGENTS.md patterns

**Example invocations:**

```
Create a new instruction called Transfer with these accounts:
- authority (signer, immutable)
- state_account (mutable)
- recipient (mutable)
- system_program (immutable)

And these data fields:
- amount: u64
```

```
Scaffold a UpdateOwner instruction with old_owner (signer), 
new_owner (immutable), and state_account (mutable)
```

**Expected output:**
- 4 generated code snippets
- File paths where code should be added
- Instructions for integration

**Post-generation checklist:**
1. Copy instruction variant to `src/instructions.rs` enum
2. Copy processor handler to `src/processor.rs` match arm
3. Copy client builder to `client/instructions.rs`
4. Add test file to workspace
5. Implement business logic in processor handler
6. Fill in test assertions
7. Run `validate_architecture` to verify compliance
8. Run `solana_build` and `solana_test`

---

### 5. `validate_architecture` - Check Compliance

**Purpose:** Validate codebase compliance with AGENTS.md architectural guidelines.

**Parameters:**
- `check_structure` (boolean, optional, default: true)
  - Verify file organization (entrypoint, processor, state, errors, instructions)
- `check_docs` (boolean, optional, default: true)
  - Verify documentation coverage on public items

**Validates:**
- ✅ File structure matches official Solana patterns
- ✅ Naming conventions (snake_case, PascalCase)
- ✅ Documentation coverage (public functions, types, modules)
- ✅ Separation of concerns (instruction builders in `client/`, processors in `src/`)
- ✅ Borsh serialization consistency
- ✅ Error handling patterns (custom error types)
- ✅ Account validation in processor handlers

**When to use:**
- Before committing code
- After refactoring
- During code review
- When onboarding new contributors
- To enforce architectural patterns

**Example invocations:**

```
Validate the architecture
```

```
Check file structure compliance
```

```
Validate documentation coverage only
```

**Expected output:**
- Success: ✅ for each passed check, overall compliance percentage
- Warnings: Missing documentation, naming violations
- Errors: Missing required files, incorrect separation of concerns

**Common issues:**
- Missing `pub` items documentation → Add `///` doc comments
- Instruction builders in wrong location → Move to `client/instructions.rs`
- Mixed serialization formats → Use Borsh exclusively
- Inconsistent error types → Define in `src/errors.rs` with `thiserror`

---

## Workflow Patterns

### Pattern 1: Feature Implementation (Story-Driven)

```
1. Create a new instruction called <Name> with <accounts> and <fields>
2. [Review generated code]
3. Build the Solana program
4. Run all tests
5. Validate the architecture
```

**Result:** Complete feature with tests, validated against architectural guidelines.

---

### Pattern 2: Debugging Failed Tests

```
1. Run the test_<name> test with output
2. [Analyze logs and errors]
3. [Fix processor logic]
4. Build the Solana program
5. Run the test_<name> test with output
```

**Result:** Isolated test execution with full program logs for debugging.

---

### Pattern 3: Pre-Commit Validation

```
1. Run cargo check on the program
2. Run all tests
3. Validate the architecture
```

**Result:** Fast verification pipeline (type check + tests + compliance).

---

### Pattern 4: Full Build & Deploy Verification

```
1. Validate the architecture
2. Build the Solana program
3. Run all tests
4. [If all pass] Deploy to local validator
```

**Result:** Comprehensive verification before deployment.

---

## Error Handling

### Build Errors

**Symptom:** `solana_build` fails with compilation errors

**Diagnosis:**
1. Read error messages carefully (file paths, line numbers)
2. Check for common issues:
   - Missing imports
   - Type mismatches in account validation
   - Borsh derive macros missing
   - Incorrect account meta ordering

**Resolution:**
- Fix compilation errors in indicated files
- Re-run `solana_build`

---

### Test Failures

**Symptom:** `solana_test` reports failed assertions or panics

**Diagnosis:**
1. Re-run with `nocapture: true` to see program logs
2. Check assertion messages
3. Review account setup in test
4. Verify instruction data serialization

**Resolution:**
- Fix processor logic or test setup
- Ensure account ordering matches processor expectations
- Verify Borsh serialization/deserialization

---

### Validation Failures

**Symptom:** `validate_architecture` reports warnings or errors

**Diagnosis:**
1. Review specific failed checks
2. Locate violating files/functions
3. Consult AGENTS.md for correct patterns

**Resolution:**
- Add missing documentation (`///` doc comments)
- Move code to correct locations (builders → `client/`, processors → `src/`)
- Fix naming conventions
- Ensure consistent error handling

---

### Scaffolding Issues

**Symptom:** `create_instruction` generates incorrect code

**Diagnosis:**
1. Check parameter format (PascalCase for name, snake_case for accounts)
2. Verify account array structure
3. Ensure `rust_type` is valid Rust syntax

**Resolution:**
- Re-invoke with corrected parameters
- Manually adjust generated code if needed
- Report issues for template improvements

---

## Integration with AGENTS.md

The toolbox enforces patterns from [AGENTS.md](../../../AGENTS.md):

### Modular Program Structure
- Instruction builders separated from processors
- Client helpers in `client/` directory
- State definitions in `src/state.rs`
- Custom errors in `src/errors.rs`

### Instruction Builder Pattern
- Pure functions (no side effects)
- Deterministic account ordering
- Borsh serialization
- Centralized in `client/instructions.rs`

### Processor Validation
- Account ownership checks
- Signer validation
- Domain-specific error types
- Structured logging with `msg!()`

### Testing Standards
- LiteSVM for fast integration tests
- Unit tests for pure functions
- Test coverage for all instructions
- `--nocapture` for debugging

---

## Best Practices

### 1. Always Validate After Changes
After modifying code, run:
```
Validate the architecture and build the program
```

### 2. Use Specific Tests for Debugging
Instead of running all tests, isolate failures:
```
Run the test_initialize_pensioner test with output
```

### 3. Scaffold Before Manual Coding
Let the toolbox generate boilerplate:
```
Create a new instruction called <Name> with <details>
```
Then fill in business logic manually.

### 4. Check Before Build
Save compile time:
```
Run cargo check on the program
```
for quick validation before full BPF build.

### 5. Document As You Go
The validator checks documentation. Add `///` comments immediately:
```rust
/// Initialize a new pension account for the beneficiary
pub fn process_initialize_pensioner(
    ...
) -> ProgramResult {
```

---

## Tool Invocation Reference

### Quick Commands

**Build:**
- "Build the Solana program"
- "Run cargo check on the program"

**Test:**
- "Run all tests"
- "Run the test_X test with output"
- "Run tests for the Y instruction"

**Create:**
- "Create a new instruction called X with accounts A, B, C and field D of type T"
- "Scaffold an instruction named X"

**Validate:**
- "Validate the architecture"
- "Check file structure compliance"
- "Validate documentation coverage"

---

## Troubleshooting

### MCP Server Not Responding

**Check logs:**
- Open Output panel in VS Code (`Ctrl+Shift+U`)
- Select "MCP (solana-insurance)" from dropdown
- Look for server startup messages

**Restart server:**
- Reload VS Code window (`Ctrl+Shift+P` → "Reload Window")

**Verify configuration:**
- Check `.vscode/mcp.json` or `%APPDATA%/Code/User/mcp.json`
- Ensure binary path is correct
- Confirm binary exists: `target/release/insurance-mcp-server.exe`

### Tool Execution Hangs

**For long-running operations:**
- `solana_build` with full BPF compilation takes 10-30 seconds
- `solana_test` with many tests may take 5-15 seconds
- Be patient; check MCP logs for progress

**If truly stuck:**
- Cancel operation
- Check MCP server logs for errors
- Restart MCP server (reload VS Code window)

### Unexpected Tool Results

**Verify working directory:**
- Tools run from workspace root
- Ensure `Cargo.toml` is present
- Check relative paths in tool output

**Check tool parameters:**
- Account names should be snake_case
- Instruction names should be PascalCase
- Boolean parameters: `true` or `false` (not `1`/`0`)

---

## Advanced Usage

### Chaining Commands

Ask for multi-step workflows:
```
Create a new instruction called PayPension, then build and test it
```

The assistant will:
1. Invoke `create_instruction`
2. Invoke `solana_build`
3. Invoke `solana_test`
4. Report results

### Conditional Execution

```
Build the program. If successful, run all tests. If tests pass, validate architecture.
```

The assistant will execute steps conditionally based on results.

### Parameterized Workflows

```
For each of these instructions (PayPension, MarkDeceased, ClaimBenefits),
create the instruction, then validate it was added correctly.
```

---

## Example Session

**User:** Create a new instruction called UpdateMonthlyPayment with authority (signer), pension_account (mutable), and new_amount (u64 field).

**Assistant:** [Invokes `create_instruction` tool]

✅ Generated `UpdateMonthlyPayment` instruction:
- Instruction variant added to src/instructions.rs
- Processor handler scaffolded in src/processor.rs
- Client builder created in client/instructions.rs
- Test template: tests/test_update_monthly_payment.rs

Next steps:
1. Implement validation logic in processor
2. Add assertion in test

**User:** Build the program and run all tests.

**Assistant:** [Invokes `solana_build`, then `solana_test`]

✅ Build successful (12.3s)
Binary: target/deploy/insurance.so

✅ Tests passed: 5 passed; 0 failed

**User:** Validate the architecture.

**Assistant:** [Invokes `validate_architecture`]

✅ File structure: PASS
✅ Documentation: PASS (100% coverage)
✅ Separation of concerns: PASS
✅ Naming conventions: PASS
✅ Borsh consistency: PASS

Overall: ✅ 100% compliant with AGENTS.md

---

## Summary

The `solana-toolbox` skill provides AI-native access to the Insurance program development workflow:
- **Build:** Compile program to BPF
- **Test:** Run LiteSVM suite
- **Create:** Scaffold instructions with architecture compliance
- **Validate:** Enforce AGENTS.md patterns

Use natural language to invoke tools and chain complex workflows. The MCP server handles JSON-RPC protocol, tool execution, and result formatting automatically.

**For detailed MCP server setup, see:**
- [toolbox/mcp-server/CLAUDE_SETUP.md](../../../toolbox/mcp-server/CLAUDE_SETUP.md)
- [toolbox/COMPLETE_SUMMARY.md](../../../toolbox/COMPLETE_SUMMARY.md)
