//! Solana test tool for MCP
//!
//! This tool runs Solana program test suites with optional filtering.
//!
//! # Examples
//!
//! AI agents can invoke this tool with:
//!
//! ```json
//! {
//!   "name": "solana_test",
//!   "arguments": {
//!     "test_name": "test_initialize_pensioner",
//!     "nocapture": true
//!   }
//! }
//! ```
//!
//! Or run all tests:
//!
//! ```json
//! {
//!   "name": "solana_test",
//!   "arguments": {}
//! }
//! ```

use crate::server::ToolTrait;
use crate::types::{Content, ToolResponse};
use anyhow::Result;
use serde_json::json;
use std::process::Command;

/// Tool for running Solana program tests
///
/// Executes the test suite using `cargo test`. Supports filtering by test name
/// and showing test output with the `nocapture` flag.
///
/// # Input Schema
///
/// - `test_name` (string, optional): Specific test to run
/// - `nocapture` (boolean, optional): Show test output (--nocapture)
///
/// # Examples
///
/// ```no_run
/// use insurance_mcp_server::server::ToolTrait;
/// use insurance_mcp_server::tools::SolanaTestTool;
/// use serde_json::json;
///
/// async fn test_example() {
///     let tool = SolanaTestTool;
///     
///     // Run all tests
///     let result = tool.execute(json!({})).await.unwrap();
///     
///     // Run specific test
///     let result = tool.execute(json!({
///         "test_name": "test_initialize",
///         "nocapture": true
///     })).await.unwrap();
/// }
/// ```
pub struct SolanaTestTool;

#[async_trait::async_trait]
impl ToolTrait for SolanaTestTool {
    fn name(&self) -> &str {
        "solana_test"
    }

    fn description(&self) -> &str {
        "Run the test suite for the Solana program. \
         Executes all LiteSVM tests. Ensure 'cargo build-sbf' was run first."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "test_name": {
                    "type": "string",
                    "description": "Optional: specific test name to run. If omitted, runs all tests."
                },
                "nocapture": {
                    "type": "boolean",
                    "description": "Show output from tests (--nocapture flag)",
                    "default": false
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResponse> {
        let test_name = args.get("test_name")
            .and_then(|v| v.as_str());
        
        let nocapture = args.get("nocapture")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        
        if let Some(name) = test_name {
            cmd.arg(name);
        }
        
        if nocapture {
            cmd.arg("--");
            cmd.arg("--nocapture");
        }

        let output = cmd.output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let success = output.status.success();
        let result_text = if success {
            format!(
                "✅ Tests passed\n\n{}\n{}",
                stdout, stderr
            )
        } else {
            format!(
                "❌ Tests failed\n\n{}\n{}",
                stdout, stderr
            )
        };

        Ok(ToolResponse {
            content: vec![Content::Text { text: result_text }],
            is_error: Some(!success),
        })
    }
}
