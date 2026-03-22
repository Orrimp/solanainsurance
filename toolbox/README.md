# Solana Toolbox

AI-powered development toolbox for Solana on-chain programs. Provides code generation, validation, and automation through CLI and MCP server.

## Components

- **CLI** - Native Rust command-line interface
- **MCP Server** - JSON-RPC 2.0 server for AI agents (Claude, VSCode Copilot)
- **Templates** - Code generation templates for instructions, tests, state
- **Pipelines** - CI/CD scripts

## Quick Start

### Setup

```bash
# Build toolbox (one-time)
cargo build --release --package solana-toolbox

# Optional: Create alias (Linux/macOS - add to ~/.bashrc)
alias toolbox="./target/release/solana-toolbox"

# Optional: Create alias (Windows PowerShell - add to $PROFILE)
function toolbox { & ".\target\release\solana-toolbox.exe" @args }
```

### CLI Commands

```bash
# Scaffold new instruction (generates 4 files: instruction, processor, client, test)
toolbox new instruction Transfer --fields 'amount:u64' --fields 'recipient:Pubkey'

# Generate test
toolbox test generate test_transfer --instruction Transfer

# Validate architecture (AGENTS.md compliance)
toolbox validate all

# Deploy to local validator
toolbox deploy local

# Story-driven development
toolbox story implement 001

# Optimization analysis
toolbox optimize analyze
```

### MCP Server (AI Integration)

The MCP server exposes 4 tools to AI agents via JSON-RPC 2.0:

**Build:**
```bash
cargo build --release --package solana-mcp-server
```

**VSCode Copilot Setup:**

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

**Claude Desktop Setup:**

Edit `%APPDATA%\Claude\claude_desktop_config.json`:
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

**Available MCP Tools:**

- ✅ `solana_build` - Compile the Solana program to BPF bytecode
- ✅ `solana_test` - Run the test suite with LiteSVM
- ✅ `create_instruction` - Scaffold new instruction with all layers
- ✅ `validate_architecture` - Check AGENTS.md compliance

**Status:** All 4 tools integrated with JSON-RPC 2.0 protocol and tested end-to-end!

## 📁 Directory Structure

```
toolbox/
├── mcp-server/          # MCP protocol server
│   ├── src/
│   │   ├── main.rs      # Server entry point
│   │   ├── server.rs    # MCP server implementation
│   │   ├── types.rs     # MCP protocol types
│   │   └── tools/       # Tool implementations
│   └── Cargo.toml
│
├── cli/                 # Command-line interface
│   ├── src/
│   │   ├── main.rs      # CLI entry point
│   │   └── commands/    # Command implementations
│   └── Cargo.toml
│
├── templates/           # Code generation templates
│   ├── instruction.rs.template
│   ├── processor_handler.rs.template
│   ├── test.rs.template
│   ├── client_builder.rs.template
│   ├── error_variant.rs.template
│   └── state_struct.rs.template
│
├── scripts/             # Automation scripts
│   ├── new_instruction.sh
│   ├── new_test.sh
│   └── validate_architecture.sh
│
├── pipelines/           # CI/CD pipelines
│   └── pre-commit.sh
│
├── validators/          # Code validators (future)
├── testing/             # Test utilities (future)
├── profiling/           # Performance tools (future)
└── docs/                # Documentation (future)
```

## 🎯 Use Cases

### For AI Agents

AI agents can use the MCP server to:
- Build and test the program
- Scaffold new features
- Validate architecture compliance
- Get structured feedback on code quality

### For Developers

Developers can use the CLI/scripts to:
- Quickly scaffold boilerplate code
- Ensure consistency with AGENTS.md guidelines
- `solana_build` - Compile Solana program
- `solana_test` - Run test suite
- `create_instruction` - Scaffold new instruction
- `validate_architecture` - Check AGENTS.md compliance

## Templates

Templates in `templates/` generate code following AGENTS.md guidelines:

- **instruction.rs.template** - Instruction enum variant
- **processor_handler.rs.template** - Processor handler function
- **client_builder.rs.template** - Client instruction builder
- **test.rs.template** - LiteSVM test
- **error_variant.rs.template** - Error enum variant
- **state_struct.rs.template** - Borsh account struct

## Development

```bash
# Build all
cargo build --workspace

# Test
cargo test --package solana-mcp-server
cargo test --package solana-toolbox
```

## Documentation

- [AGENTS.md](../AGENTS.md) - Architecture guidelines
- [.github/skills/](../.github/skills/) - AI skills
- [docs/](docs/) - Additional documentation
