//! Insurance MCP Server - Model Context Protocol server for AI-assisted development
//!
//! This server exposes tools that AI agents can use to interact with the
//! Solana Insurance on-chain program development workflow.
//!
//! # Usage
//!
//! Run the server on stdio (for Claude Desktop, VSCode Copilot, etc.):
//!
//! ```bash
//! cargo run --release --bin insurance-mcp-server
//! ```
//!
//! # Configuration
//!
//! For **Claude Desktop**, add to `claude_desktop_config.json`:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "solana-insurance": {
//!       "command": "/path/to/insurance-mcp-server",
//!       "args": []
//!     }
//!   }
//! }
//! ```
//!
//! For **VSCode Copilot**, add to `.vscode/mcp.json`:
//!
//! ```json
//! {
//!   "servers": {
//!     "solana-insurance": {
//!       "command": "/path/to/insurance-mcp-server",
//!       "args": []
//!     }
//!   }
//! }
//! ```
//!
//! # Available Tools
//!
//! - **solana_build**: Compile the program to BPF bytecode
//! - **solana_test**: Run the test suite with optional filters
//! - **solana_pipeline**: Run full CI/CD pipeline with configurable steps
//! - **create_instruction**: Scaffold a new instruction with templates
//! - **validate_architecture**: Check AGENTS.md compliance
//!
//! # Logging
//!
//! Set `RUST_LOG` environment variable to control log level:
//!
//! ```bash
//! RUST_LOG=debug cargo run --bin solana-mcp-server
//! ```
//!
//! Logs are written to stderr (stdout is reserved for JSON-RPC).

use solana_mcp_server::{server::McpServer, tools::*};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Derive the workspace root from the binary's own path.
///
/// In this workspace the binary lives at `<workspace>/target/release/solana-mcp-server`.
/// Walking up three parent directories reaches the workspace root.
/// Falls back to the inherited CWD if derivation fails.
fn derive_workspace_root() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            // release/ → target/ → workspace root
            p.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (stdout is for JSON-RPC)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Auto-detect workspace root from the binary path so that all child
    // processes (cargo build-sbf, cargo test, …) run from the correct directory
    // regardless of where the MCP host launches the server from.
    let workspace_root = derive_workspace_root();
    std::env::set_current_dir(&workspace_root)?;
    // Export for tools that want an explicit path rather than inheriting CWD.
    std::env::set_var("WORKSPACE_ROOT", workspace_root.to_string_lossy().as_ref());

    info!("Starting Solana MCP Server");
    info!("Workspace root: {}", workspace_root.display());

    // Create server
    let server = McpServer::new("solana-mcp-server", "0.1.0");

    // Register tools
    server.register_tool(Box::new(SolanaBuildTool)).await;
    server.register_tool(Box::new(SolanaTestTool)).await;
    server.register_tool(Box::new(SolanaPipelineTool)).await;
    server.register_tool(Box::new(CreateInstructionTool)).await;
    server.register_tool(Box::new(ValidateArchitectureTool)).await;

    info!("Registered {} tools", server.list_tools().await.len());

    // Run on stdio
    server.run_stdio().await?;

    Ok(())
}
