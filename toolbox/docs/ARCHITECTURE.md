# Toolbox Architecture

## Overview

```
AI Agents (Copilot, Claude, Cursor)
         |
         v
MCP Server (JSON-RPC 2.0 over stdio)
         |
         v
Tools: solana_build, solana_test, solana_pipeline,
       solana_deploy_local, create_instruction,
       validate_architecture
         |
         v
Rust CLI + Templates
         |
         v
On-chain program + off-chain client
```

## Components

### MCP Server (`toolbox/mcp-server`)

- Protocol: JSON-RPC 2.0 over stdio
- Transport: async `tokio` stdin/stdout
- Registered tools:
  - `solana_build`
  - `solana_test`
  - `solana_pipeline`
  - `solana_deploy_local`
  - `create_instruction`
  - `validate_architecture`
- Purpose: expose deterministic development operations to AI agents

### CLI (`toolbox/cli`)

- Framework: `clap`
- Command surface mirrors toolbox workflows (build/test/validate/deploy/story/optimization)
- Purpose: human-friendly entrypoint for the same operational flows

### Templates (`toolbox/templates`)

- Instruction, processor handler, client builder, test, error variant, state struct templates
- Placeholder format: `{{PLACEHOLDER}}`
- Purpose: scaffold code that follows AGENTS.md patterns

## Interaction Flow

1. Developer starts from a story, requirement, or bug report.
2. Agent or developer invokes toolbox actions via CLI or MCP.
3. Code is scaffolded (`create_instruction`) and implemented in on-chain/off-chain layers.
4. Structural checks run with `validate_architecture`.
5. Build/test/deploy confidence is established via `solana_build`, `solana_test`, or `solana_pipeline`.
6. Local end-to-end execution uses `solana_deploy_local` or pipeline deploy/client steps.
7. Results are reviewed, refined, and merged when quality gates pass.

## Development Process Diagram

```mermaid
flowchart TD
  A[Requirement or Story] --> B[Design and Account Model]
  B --> C{Implementation Path}

  C -->|New instruction| D[create_instruction]
  C -->|Modify existing flow| E[Edit src and client modules]

  D --> F[Implement processor and state logic]
  E --> F

  F --> G[validate_architecture]
  G --> H{Validation OK?}
  H -->|No| F
  H -->|Yes| I[solana_build or cargo check]

  I --> J{Build OK?}
  J -->|No| F
  J -->|Yes| K[solana_test]

  K --> L{Tests OK?}
  L -->|No| F
  L -->|Yes| M{Execution Mode}

  M -->|Full CI| N[solana_pipeline --all]
  M -->|Local run| O[solana_deploy_local]

  N --> P{Pipeline green?}
  O --> Q{Local flow green?}

  P -->|No| F
  Q -->|No| F

  P -->|Yes| R[Review and PR]
  Q -->|Yes| R

  R --> S[Merge]
  S --> T[Post-merge monitoring and iteration]
```

## Tool Interaction Diagram

```mermaid
flowchart LR
  Dev[Developer or Agent] --> Choice{Interaction Surface}

  Choice -->|Editor or chat tool call| MCP[MCP Server]
  Choice -->|Terminal command| CLI[solana-toolbox CLI]

  MCP --> BuildTool[solana_build]
  MCP --> TestTool[solana_test]
  MCP --> PipelineTool[solana_pipeline]
  MCP --> DeployTool[solana_deploy_local]
  MCP --> ScaffoldTool[create_instruction]
  MCP --> ValidateTool[validate_architecture]

  CLI --> BuildCmd[build and check flows]
  CLI --> TestCmd[test flows]
  CLI --> PipelineCmd[pipeline workflow]
  CLI --> DeployCmd[deploy local workflow]
  CLI --> StoryCmd[story and generation flows]
  CLI --> ValidateCmd[architecture validation]

  ScaffoldTool --> Templates[toolbox templates]
  StoryCmd --> Templates

  BuildTool --> Cargo[Cargo and Solana CLI]
  TestTool --> Cargo
  PipelineTool --> Cargo
  DeployTool --> Cargo
  BuildCmd --> Cargo
  TestCmd --> Cargo
  PipelineCmd --> Cargo
  DeployCmd --> Cargo

  Templates --> Codebase[src and client modules]
  Cargo --> Codebase
  Cargo --> Validator[Local validator or RPC target]

  ValidateTool --> Report[Structured output and diagnostics]
  ValidateCmd --> Report
  Cargo --> Report
  Codebase --> Report
  Report --> Dev
```

## Runtime Layers

- On-chain layer (`src/`): instruction enum, processor handlers, state, errors, entrypoint.
- Off-chain layer (`client/`): instruction builders, transaction sender, RPC context, demo client.
- Network layer: local validator (`localhost`) or remote cluster.

## Key Principles

1. MCP-first: AI tools are first-class interfaces.
2. Template-driven consistency: predictable, reviewable codegen output.
3. Clear separation of concerns: on-chain core vs off-chain orchestration.
4. Validation by default: architecture checks alongside build/test.
5. Cross-platform operation: Windows, Linux, macOS.
