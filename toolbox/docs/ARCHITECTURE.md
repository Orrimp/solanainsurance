# Toolbox Architecture

## Overview

```
AI Agents (Claude, VSCode Copilot)
         ↓
   MCP Server (JSON-RPC 2.0)
         ↓
   4 Tools: build, test, create_instruction, validate_architecture
         ↓
   Native CLI Implementation
         ↓
   Templates + Code Generation
         ↓
   On-Chain Program (src/)
```

## Components

### MCP Server (`mcp-server/`)
- **Protocol**: JSON-RPC 2.0 over stdio
- **Transport**: StdioTransport (async tokio)
- **Tools**: 4 registered tools implementing ToolTrait
- **Purpose**: Expose development tools to AI agents

### CLI (`cli/`)
- **Commands**: new, test, validate, deploy, story, optimize
- **Framework**: clap-based argument parsing
- **Templates**: Reads from `templates/` directory
- **Purpose**: Human-friendly interface to toolbox features

### Templates (`templates/`)
- Instruction, processor, client builder, test, error, state
- Placeholder substitution: `{{PLACEHOLDER}}`
- Ensures AGENTS.md compliance

### File Flow

1. AI agent calls MCP tool or developer runs CLI command
2. Tool reads template from `templates/`
3. Substitutes placeholders (name, fields, etc.)
4. Outputs generated code
5. Developer integrates into `src/`, `client/`, or `tests/`

## Key Principles

- **MCP-First**: AI agents are first-class users
- **Template-Driven**: Consistent code patterns
- **Validation**: Architecture checks at every step
- **Cross-Platform**: Works on Windows, Linux, macOS
│  └──────────────────────────────────────────────────────────┘               │
│                             │                                               │
│                    cargo build-sbf                                          │
│                             ▼                                               │
│  ┌──────────────────────────────────────────────────────────┐               │
│  │         target/deploy/program.so                         │               │
│  │              (BPF bytecode)                              │               │
│  └──────────────────────────────────────────────────────────┘               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         Off-Chain Client Layer                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────┐               │
│  │                  client/ (Off-chain)                     │               │
│  │  ┌─────────────────────────────────────────────────────┐ │               │
│  │  │  • instructions.rs - Instruction builders (pure)    │ │               │
│  │  │  • sender.rs       - Transaction assembly & submit  │ │               │
│  │  │  • solana_ctx.rs   - RPC client, environment        │ │               │
│  │  │  • client.rs       - Demo orchestration flow        │ │               │
│  │  └─────────────────────────────────────────────────────┘ │               │
│  └──────────────────────────────────────────────────────────┘               │
│                             │                                               │
│                    cargo run --example client                               │
│                             │                                               │
└─────────────────────────────┼───────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Solana Network  │
                    │   (localhost or  │
                    │    devnet/main)  │
                    └──────────────────┘

═══════════════════════════════════════════════════════════════════════════════

Workflow Example: AI-Assisted Feature Development

1. Developer/AI: "Add Transfer instruction"
   │
   ▼
2. AI Agent uses MCP → create_instruction tool
   │
   ▼
3. Templates generate code snippets
   │
   ▼
4. Developer reviews & integrates into src/, client/
   │
   ▼
5. AI Agent uses MCP → solana_build
   │
   ▼
6. AI Agent uses MCP → solana_test
   │
   ▼
7. AI Agent uses MCP → validate_architecture
   │
   ▼
8. ✅ Feature complete with tests & validation
   │
   ▼
9. Deploy: solana program deploy

═══════════════════════════════════════════════════════════════════════════════

Key Design Principles:

1. MCP-First: AI agents are first-class citizens
2. Template-Driven: Consistent patterns enforced
3. Layered Architecture: Clear separation of concerns
4. Validation Built-In: Architecture compliance automated
5. Idiomatic Rust: Follows official guidelines
6. Borsh Serialization: On-chain + client consistency
7. LiteSVM Testing: Fast in-process tests

═══════════════════════════════════════════════════════════════════════════════
