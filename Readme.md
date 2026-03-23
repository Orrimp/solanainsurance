# Solana Insurance Program

Native Solana on-chain program for pension/insurance management, with an AI-first toolbox for build, test, scaffolding, and architecture validation.

## Quick Links

- [AI Toolbox Quick Start](toolbox/QUICKSTART.md)
- [Toolbox Documentation](toolbox/README.md)
- [Architecture Guidelines](AGENTS.md)
- [Build-Test-Deploy Skill](.github/skills/build-test-deploy/SKILL.md)

## Project Stack

- On-chain program: native Solana (no Anchor), Borsh serialization
- Off-chain example client: `cargo run --example client`
- AI tooling:
	- MCP server (`toolbox/mcp-server`)
	- CLI (`toolbox/cli`)
	- Templates (`toolbox/templates`)

## Compact Team Workflow

### Feature Development

1. Implement feature code with `solana-rust` (instruction/state/processor/client).
2. If adding an instruction, scaffold with `create_instruction`.
3. Run quick validation: `solana_build` (or check-only) and targeted `solana_test`.
4. Run full validation before merge: `solana_pipeline` (`--build --test` or `--all`).
5. Run `validate_architecture` for structural or convention-heavy changes.

### Agent Selection Rules

- `solana-rust`: use for all Rust code changes and test logic.
- `solana-ci`: use for compile/test/deploy confidence checks and reports.
- `Explore`: use for fast codebase discovery before editing.
- `solana-toolbox` skill: default for day-to-day implementation/testing automation.
- `build-test-deploy` skill: use when you need structured CI/CD execution and reporting.

### Definition Of Done

- Code compiles successfully.
- Relevant tests pass (targeted + full scope as needed).
- Architecture validation passes when applicable.
- Required pipeline scope is green (`--build --test` minimum; `--all` for deploy/client changes).
- Documentation/config updated when behavior or workflow changes.

## Development Setup

### 1. Start local validator

```bash
solana-test-validator --reset
```

### 2. Configure CLI to localhost

```bash
solana config set --url localhost
solana config get
```

### 3. Create/fund default keypair

If you do not already have a default keypair:

```bash
solana-keygen new --outfile ~/.config/solana/id.json --force
```

Use your own passphrase/seed phrase. Never commit secrets to this repository.

Fund the keypair:

```bash
solana airdrop 2
solana balance
```

## Build

Build the on-chain SBF program and produce `target/deploy/insurance.so`:

```bash
# Option A
solana program build

# Option B
cargo build-sbf
```

## Check Program Address

```bash
solana address -k ./target/deploy/insurance-keypair.json
```

## Deploy

```bash
solana program deploy -u localhost ./target/deploy/insurance.so
```

## Run Example Client

```bash
cargo run --example client
```

## Local Workflow (Script-Equivalent)

Run the integrated local workflow (validator + optional build + deploy + optional client):

```bash
solana-toolbox deploy local --airdrop 2
```

Useful flags:

```bash
solana-toolbox deploy local --skip-build --skip-client
solana-toolbox deploy local --keep-validator --validator-timeout 60
solana-toolbox deploy local --program-so target/deploy/insurance.so
```

## Run Tests

```bash
# Requires built .so for LiteSVM tests
cargo build-sbf
cargo test
```

## Optional: Dependency Audit

```bash
cargo +nightly udeps --all-targets --workspace
```

## Stop Local Validator

Stop the terminal running `solana-test-validator` (Ctrl+C). With `--reset`, local ledger state is wiped on next startup.