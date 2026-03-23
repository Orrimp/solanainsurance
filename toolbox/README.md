# Solana Toolbox

AI-native toolbox for this Solana workspace. It provides a Rust CLI, an MCP server, and template-driven scaffolding so developers and agents use the same workflows for generation, validation, build, test, pipeline, and local deployment.

## Components

- **CLI** — native Rust command-line interface in `toolbox/cli`
- **MCP Server** — JSON-RPC 2.0 server for AI agents in `toolbox/mcp-server`
- **Templates** — code generation templates in `toolbox/templates`
- **Docs** — architecture and workflow documentation in `toolbox/docs`

## Quick Start

### Build

```bash
cargo build --release --manifest-path toolbox/cli/Cargo.toml
cargo build --release --manifest-path toolbox/mcp-server/Cargo.toml
```

### CLI Commands

```bash
# Scaffold a new instruction
solana-toolbox new instruction Transfer --fields amount:u64 --fields recipient:Pubkey

# Generate a test template
solana-toolbox test generate test_transfer --instruction Transfer

# Validate architecture
solana-toolbox validate all

# Run CI pipeline
solana-toolbox pipeline --build --test

# Run local validator/build/deploy/client workflow
solana-toolbox deploy local --airdrop 2
```

### MCP Server

The MCP server exposes 6 tools to AI agents via JSON-RPC 2.0:

- `solana_build`
- `solana_test`
- `solana_pipeline`
- `solana_deploy_local`
- `create_instruction`
- `validate_architecture`

#### VS Code Copilot Setup

Create `.vscode/mcp.json`:

```json
{
  "servers": {
    "solana-toolbox": {
      "command": "C:/path/to/target/release/solana-mcp-server.exe",
      "args": []
    }
  }
}
```

#### Claude Desktop Setup

Edit `%APPDATA%\\Claude\\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "solana-toolbox": {
      "command": "C:/path/to/target/release/solana-mcp-server.exe",
      "args": []
    }
  }
}
```

## Directory Structure

```text
toolbox/
├── cli/                 # Human-facing command surface
├── docs/                # Toolbox architecture and workflow docs
├── mcp-server/          # Agent-facing MCP server
├── pipelines/           # Local pre-commit helpers
└── templates/           # Scaffold templates for instructions/tests/state
```

## Common Workflows

### Build and Test

```bash
solana-toolbox pipeline --check --build --test
```

### Full CI Flow

```bash
solana-toolbox pipeline --all
```

### Local Deploy Flow

```bash
solana-toolbox deploy local --keep-validator --validator-timeout 60
```

## Development

```bash
cargo build --workspace
cargo test --package solana-toolbox
cargo test --package solana-mcp-server
```

## Documentation

- [AGENTS.md](../AGENTS.md) — repository architecture guidelines
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — toolbox architecture and process diagrams
- [../.github/skills/](../.github/skills/) — agent skills and workflows
