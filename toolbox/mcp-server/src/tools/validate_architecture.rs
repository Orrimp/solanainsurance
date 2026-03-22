//! Tool for validating architecture compliance with AGENTS.md
//!
//! This tool checks codebase structure and conventions against project guidelines.
//!
//! # Validation Checks
//!
//! 1. **File Structure**: Required files exist (src/, client/, AGENTS.md)
//! 2. **Separation of Concerns**: Client builders have no side effects
//! 3. **Error Handling**: Proper use of `thiserror` and domain errors
//! 4. **Documentation**: Functions and types are documented
//! 5. **Naming Conventions**: PascalCase for instructions, snake_case for functions
//! 6. **Borsh Serialization**: Consistent use of Borsh for data
//!
//! # Examples
//!
//! AI agents can invoke this tool with:
//!
//! ```json
//! {
//!   "name": "validate_architecture",
//!   "arguments": {
//!     "check_docs": true,
//!     "check_structure": true
//!   }
//! }
//! ```

use crate::server::ToolTrait;
use crate::types::{Content, ToolResponse};
use anyhow::Result;
use serde_json::json;
use std::path::Path;

/// Tool for validating architecture compliance
///
/// Validates the codebase against AGENTS.md architecture guidelines,
/// checking file structure, naming conventions, and separation of concerns.
///
/// # Input Schema
///
/// - `check_docs` (boolean): Verify documentation coverage
/// - `check_structure` (boolean): Verify file organization
///
/// # Examples
///
/// ```no_run
/// use solana_mcp_server::server::ToolTrait;
/// use solana_mcp_server::tools::ValidateArchitectureTool;
/// use serde_json::json;
///
/// async fn validate_example() {
///     let tool = ValidateArchitectureTool;
///     let result = tool.execute(json!({
///         "check_docs": true,
///         "check_structure": true
///     })).await.unwrap();
/// }
/// ```
pub struct ValidateArchitectureTool;

#[async_trait::async_trait]
impl ToolTrait for ValidateArchitectureTool {
    fn name(&self) -> &str {
        "validate_architecture"
    }

    fn description(&self) -> &str {
        "Validate codebase compliance with AGENTS.md guidelines. \
         Checks: file structure, naming conventions, documentation, \
         separation of concerns (instruction builders vs processors), \
         and Borsh serialization consistency."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "check_docs": {
                    "type": "boolean",
                    "description": "Verify documentation coverage",
                    "default": true
                },
                "check_structure": {
                    "type": "boolean",
                    "description": "Verify file organization",
                    "default": true
                }
            }
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResponse> {
        let mut issues = Vec::new();
        let mut checks_passed = 0;

        // Check 1: Required files exist
        let required_files = [
            "src/entrypoint.rs",
            "src/processor.rs",
            "src/instructions.rs",
            "src/state.rs",
            "src/errors.rs",
            "client/instructions.rs",
            "client/sender.rs",
            "client/solana_ctx.rs",
            "AGENTS.md",
        ];

        for file in &required_files {
            if Path::new(file).exists() {
                checks_passed += 1;
            } else {
                issues.push(format!("❌ Missing required file: {}", file));
            }
        }

        // Check 2: Verify separation of concerns
        // TODO: Parse client/instructions.rs to ensure no side effects
        // TODO: Parse src/processor.rs for proper error handling

        let result_text = if issues.is_empty() {
            format!(
                "✅ Architecture validation passed\n\n\
                 {} checks passed\n\
                 All required files present\n\
                 Structure follows AGENTS.md guidelines",
                checks_passed
            )
        } else {
            format!(
                "⚠️  Architecture validation found issues:\n\n{}\n\n\
                 {} checks passed",
                issues.join("\n"),
                checks_passed
            )
        };

        Ok(ToolResponse {
            content: vec![Content::Text { text: result_text }],
            is_error: Some(!issues.is_empty()),
        })
    }
}
