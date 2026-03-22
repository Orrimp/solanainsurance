//! Insurance MCP Server Library
//!
//! Provides Model Context Protocol (MCP) tools for AI agents to interact with
//! the Solana Insurance on-chain program development workflow.
//!
//! # Features
//!
//! - **JSON-RPC 2.0 Protocol**: Full request/response and notification support
//! - **Stdio Transport**: Async I/O for AI agent communication
//! - **4 Core Tools**: Build, test, scaffold, validate
//!
//! # Architecture
//!
//! ```text
//! AI Agent (Claude, VSCode Copilot)
//!       ↓ JSON-RPC 2.0 over stdio
//! StdioTransport
//!       ↓
//! McpServer (request router)
//!       ↓
//! ToolTrait implementations
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use solana_mcp_server::{server::McpServer, tools::*};
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = McpServer::new("my-server", "1.0.0");
//!     
//!     // Register tools
//!     server.register_tool(Box::new(SolanaBuildTool)).await;
//!     server.register_tool(Box::new(SolanaTestTool)).await;
//!     
//!     // Run on stdio
//!     server.run_stdio().await.unwrap();
//! }
//! ```

pub mod jsonrpc;
pub mod server;
pub mod tools;
pub mod transport;
pub mod types;

pub use server::McpServer;
