//! MCP Tool implementations for Solana development
//!
//! This module provides 6 core tools that AI agents can invoke:
//!
//! - [`SolanaBuildTool`]: Compile Solana programs to BPF bytecode
//! - [`SolanaTestTool`]: Run test suites with optional filters
//! - [`SolanaPipelineTool`]: Run full CI/CD pipeline with configurable steps
//! - [`SolanaDeployLocalTool`]: Run local deploy workflow from integrated script behavior
//! - [`CreateInstructionTool`]: Scaffold new instructions from templates
//! - [`ValidateArchitectureTool`]: Check AGENTS.md compliance
//!
//! # Examples
//!
//! ```no_run
//! use solana_mcp_server::server::McpServer;
//! use solana_mcp_server::tools::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = McpServer::new("server", "1.0");
//!     
//!     // Register all tools
//!     server.register_tool(Box::new(SolanaBuildTool)).await;
//!     server.register_tool(Box::new(SolanaTestTool)).await;
//!     server.register_tool(Box::new(SolanaPipelineTool)).await;
//!     server.register_tool(Box::new(SolanaDeployLocalTool)).await;
//!     server.register_tool(Box::new(CreateInstructionTool)).await;
//!     server.register_tool(Box::new(ValidateArchitectureTool)).await;
//! }
//! ```

pub mod solana_build;
pub mod solana_test;
pub mod solana_pipeline;
pub mod solana_deploy_local;
pub mod create_instruction;
pub mod validate_architecture;

pub use solana_build::SolanaBuildTool;
pub use solana_test::SolanaTestTool;
pub use solana_pipeline::SolanaPipelineTool;
pub use solana_deploy_local::SolanaDeployLocalTool;
pub use create_instruction::CreateInstructionTool;
pub use validate_architecture::ValidateArchitectureTool;
