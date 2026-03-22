//! Tool for scaffolding new Solana instructions
//!
//! This tool generates all code layers for a new instruction following AGENTS.md guidelines.
//!
//! # Generated Code
//!
//! For an instruction named `PayPension`, generates:
//! - Instruction enum variant in `src/instructions.rs`
//! - Processor handler in `src/processor.rs`
//! - Client builder in `client/instructions.rs`
//! - LiteSVM test in `tests/`
//!
//! # Examples
//!
//! AI agents can invoke this tool with:
//!
//! ```json
//! {
//!   "name": "create_instruction",
//!   "arguments": {
//!     "name": "PayPension",
//!     "accounts": [
//!       {"name": "pension_account", "mutable": true, "signer": false},
//!       {"name": "authority", "mutable": false, "signer": true}
//!     ],
//!     "data_fields": [
//!       {"name": "amount", "rust_type": "u64"}
//!     ]
//!   }
//! }
//! ```

use crate::server::ToolTrait;
use crate::types::{Content, ToolResponse};
use anyhow::Result;
use serde_json::json;

/// Tool for creating new instruction scaffolding
///
/// Generates complete instruction implementation following project architecture:
/// - Deterministic instruction builders (no side effects)
/// - Processor handlers with validation
/// - Client helpers for transaction assembly
/// - LiteSVM tests
///
/// # Input Schema
///
/// - `name` (string): Instruction name in PascalCase
/// - `accounts` (array): Account definitions with name, mutable, signer flags
/// - `data_fields` (array, optional): Instruction data field definitions
///
/// # Examples
///
/// ```no_run
/// use insurance_mcp_server::server::ToolTrait;
/// use insurance_mcp_server::tools::CreateInstructionTool;
/// use serde_json::json;
///
/// async fn scaffold_example() {
///     let tool = CreateInstructionTool;
///     let result = tool.execute(json!({
///         "name": "PayPension",
///         "accounts": [
///             {"name": "pension", "mutable": true, "signer": false}
///         ]
///     })).await.unwrap();
/// }
/// ```
pub struct CreateInstructionTool;

#[async_trait::async_trait]
impl ToolTrait for CreateInstructionTool {
    fn name(&self) -> &str {
        "create_instruction"
    }

    fn description(&self) -> &str {
        "Scaffold a new Solana instruction with all required layers: \
         instruction enum variant, processor handler, client builder, and test template. \
         Follows AGENTS.md architecture guidelines."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Instruction name in PascalCase (e.g., 'InitializePensioner')"
                },
                "accounts": {
                    "type": "array",
                    "description": "Array of account definitions",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Account name in snake_case"
                            },
                            "mutable": {
                                "type": "boolean",
                                "description": "Is account writable?"
                            },
                            "signer": {
                                "type": "boolean",
                                "description": "Is account required to sign?"
                            }
                        },
                        "required": ["name", "mutable", "signer"]
                    }
                },
                "data_fields": {
                    "type": "array",
                    "description": "Data fields for the instruction (optional)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string"
                            },
                            "rust_type": {
                                "type": "string",
                                "description": "Rust type (e.g., 'u64', 'Pubkey')"
                            }
                        },
                        "required": ["name", "rust_type"]
                    }
                }
            },
            "required": ["name", "accounts"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResponse> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' field"))?;

        // TODO: Implement full scaffolding logic
        // For now, return a preview of what would be generated
        
        let preview = format!(
            r#"Instruction scaffolding preview for: {}

Files that would be modified:
1. src/instructions.rs - Add {} variant to InsuranceInstruction enum
2. src/processor.rs - Add handler function process_{}
3. client/instructions.rs - Add builder function {}
4. Add test in tests/ or src/lib.rs

Next steps:
- Use templates from toolbox/templates/
- Follow AGENTS.md Section 11 guidelines
- Run validation after generation

(Full implementation pending - use manual templates for now)"#,
            name, name, name.to_lowercase(), name.to_lowercase()
        );

        Ok(ToolResponse {
            content: vec![Content::Text { text: preview }],
            is_error: Some(false),
        })
    }
}
