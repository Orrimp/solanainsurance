//! Solana build tool for MCP
//!
//! This tool compiles Solana programs to BPF bytecode using `cargo build-sbf`.
//!
//! # Examples
//!
//! AI agents can invoke this tool with:
//!
//! ```json
//! {
//!   "name": "solana_build",
//!   "arguments": {
//!     "check_only": false
//!   }
//! }
//! ```
//!
//! Or via the CLI:
//!
//! ```bash
//! toolbox build
//! ```

use crate::server::ToolTrait;
use crate::types::{Content, ToolResponse};
use anyhow::Result;
use serde_json::json;
use std::process::Command;

/// Tool for building the Solana program
///
/// Executes `cargo build-sbf` to compile the program to BPF bytecode.
/// The compiled binary is output to `target/deploy/insurance.so`.
///
/// # Input Schema
///
/// - `check_only` (boolean): If true, only run `cargo check` instead of full build
///
/// # Examples
///
/// ```no_run
/// use insurance_mcp_server::server::ToolTrait;
/// use insurance_mcp_server::tools::SolanaBuildTool;
/// use serde_json::json;
///
/// async fn build_example() {
///     let tool = SolanaBuildTool;
///     let result = tool.execute(json!({"check_only": false})).await.unwrap();
/// }
/// ```
pub struct SolanaBuildTool;

#[async_trait::async_trait]
impl ToolTrait for SolanaBuildTool {
    fn name(&self) -> &str {
        "solana_build"
    }

    fn description(&self) -> &str {
        "Compile the Solana on-chain program to BPF bytecode. \
         Runs 'cargo build-sbf' and produces target/deploy/insurance.so binary. \
         Required before running tests or deploying."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "check_only": {
                    "type": "boolean",
                    "description": "If true, only run cargo check instead of full build",
                    "default": false
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResponse> {
        let check_only = args.get("check_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let output = if check_only {
            Command::new("cargo")
                .arg("check")
                .output()?
        } else {
            Command::new("cargo")
                .arg("build-sbf")
                .output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let success = output.status.success();
        let result_text = if success {
            format!(
                "✅ Build successful\n\nStdout:\n{}\n\nStderr:\n{}",
                stdout, stderr
            )
        } else {
            format!(
                "❌ Build failed\n\nStdout:\n{}\n\nStderr:\n{}",
                stdout, stderr
            )
        };

        Ok(ToolResponse {
            content: vec![Content::Text { text: result_text }],
            is_error: Some(!success),
        })
    }
}
