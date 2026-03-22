# Custom Agents Guide

This directory contains specialized AI agents for Solana development. Each agent has access to skills and the MCP toolbox to streamline development workflows.

## Available Agents

### 🔧 solana-rust
**Purpose:** Write, review, and debug Solana Rust code

**When to use:**
- Implementing new instructions
- Writing processor handlers
- Creating client builders
- Writing LiteSVM tests
- Fixing compilation errors or failing tests

**Skills:** `solana-toolbox`

**Example invocations:**
```
@solana-rust implement story 003

@solana-rust create a new instruction called UpdateSettings with authority (signer) and settings_account (mutable) accounts

@solana-rust fix the failing test_contribute test
```

### 🧪 solana-ci
**Purpose:** Run build/test/deploy pipelines and produce reports

**When to use:**
- Verifying the build succeeds
- Running the full test suite
- Deploying to local validator
- Getting a pipeline health report
- Pre-commit checks

**Skills:** `build-test-deploy`

**Example invocations:**
```
@solana-ci run the full pipeline

@solana-ci build and test the program

@solana-ci check if the program compiles
```

## Agent + Skill Integration

Both agents are configured to automatically load their respective skill files:

| Agent | Skill | Location |
|-------|-------|----------|
| `solana-rust` | `solana-toolbox` | `.github/skills/solana-toolbox/SKILL.md` |
| `solana-ci` | `build-test-deploy` | `.github/skills/build-test-deploy/SKILL.md` |

### How Skills Work

1. **Skill files** contain workflow patterns, tool documentation, and best practices
2. **Agents read the skill** at the start of their workflow
3. **MCP tools** (if available) are invoked following skill patterns
4. **Fallback**: If MCP unavailable, agents use cargo commands directly

## MCP Toolbox

The `solana-toolbox` MCP server provides 4 tools:

| Tool | Purpose | Agent |
|------|---------|-------|
| `solana_build` | Compile program (cargo check/build-sbf) | solana-rust, solana-ci |
| `solana_test` | Run test suite | solana-rust, solana-ci |
| `create_instruction` | Scaffold new instruction (4 layers) | solana-rust |
| `validate_architecture` | Check AGENTS.md compliance | solana-rust |

### MCP Server Status

Check if MCP is active:
- VSCode: Status bar shows "solana-toolbox" if connected
- Test: Ask an agent to use `solana_build`

If MCP is unavailable, agents will fall back to direct cargo commands.

## Quick Start Workflows

### Feature Development (solana-rust)

```
1. @solana-rust read story 003 and plan the implementation
2. [Agent loads solana-toolbox skill, reads story, creates plan]
3. @solana-rust implement the first task
4. [Agent uses create_instruction MCP tool to scaffold code]
5. @solana-rust verify the build and tests
6. [Agent uses solana_build and solana_test MCP tools]
```

### CI/CD Workflow (solana-ci)

```
1. @solana-ci run build and test
2. [Agent loads build-test-deploy skill]
3. [Agent executes pipeline, captures output]
4. [Agent produces structured report with results]
```

### Debugging Workflow (solana-rust)

```
1. @solana-rust run test_contribute with output
2. [Agent uses solana_test with nocapture: true]
3. [Agent analyzes logs and suggests fix]
4. @solana-rust apply the fix
5. @solana-rust verify the test passes
```

## Updating Agent Configurations

Agent files are located here:
- **solana-rust.agent.md** - Rust development agent
- **solana-ci.agent.md** - CI pipeline agent

To modify agent behavior:
1. Edit the `.agent.md` file
2. Update the `skills:` array if adding new skills
3. Reload VSCode window to apply changes

## Creating New Agents

To create a new specialized agent:

1. Create `new-agent.agent.md` in this directory
2. Add YAML frontmatter with `description`, `tools`, `skills`
3. Write clear instructions for the agent's role
4. Reference relevant skill files
5. Test by invoking `@new-agent`

## Skill Development

Skills are located in `.github/skills/`. To create a new skill:

1. Create a folder: `.github/skills/my-skill/`
2. Add `SKILL.md` with workflow patterns
3. Optionally add templates, scripts, examples
4. Reference the skill in agent configuration
5. Update this README

---

**Last updated:** March 22, 2026
**MCP Server:** solana-toolbox v0.1.0
**Agents:** solana-rust, solana-ci
