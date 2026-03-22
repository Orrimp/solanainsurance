//! Solana Pipeline MCP Tool
//!
//! Provides MCP tool for running the full CI/CD pipeline with configurable steps.

use crate::server::ToolTrait;
use crate::types::{Content, ToolResponse};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Command;

/// Input schema for solana_pipeline tool
#[derive(Debug, Deserialize)]
pub struct PipelineInput {
    /// Run cargo check (type-check)
    #[serde(default)]
    pub check: bool,
    
    /// Run cargo build-sbf (compile program)
    #[serde(default)]
    pub build: bool,
    
    /// Run cargo test (all tests)
    #[serde(default)]
    pub test: bool,
    
    /// Deploy to local validator
    #[serde(default)]
    pub deploy: bool,
    
    /// Run example client
    #[serde(default)]
    pub client: bool,
    
    /// Run all steps (equivalent to check + build + test + deploy + client)
    #[serde(default)]
    pub all: bool,
    
    /// Validator startup timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub validator_timeout: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Result from running the pipeline
#[derive(Debug, Serialize)]
pub struct PipelineResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub steps: Vec<StepResult>,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub name: String,
    pub status: String,  // "PASS", "FAIL", or "SKIP"
}

impl PipelineResult {
    fn parse_from_output(stdout: &str, stderr: &str, success: bool) -> Self {
        let mut steps = Vec::new();
        
        // Parse step results from stdout
        for line in stdout.lines() {
            if line.contains("✅ PASS") {
                if let Some(name) = line.split("PASS").nth(1) {
                    steps.push(StepResult {
                        name: name.trim().to_string(),
                        status: "PASS".to_string(),
                    });
                }
            } else if line.contains("❌ FAIL") {
                if let Some(name) = line.split("FAIL").nth(1) {
                    steps.push(StepResult {
                        name: name.trim().to_string(),
                        status: "FAIL".to_string(),
                    });
                }
            } else if line.contains("⊝ SKIP") {
                if let Some(name) = line.split("SKIP").nth(1) {
                    steps.push(StepResult {
                        name: name.trim().to_string(),
                        status: "SKIP".to_string(),
                    });
                }
            }
        }
        
        PipelineResult {
            success,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            steps,
        }
    }
}

/// Solana Pipeline Tool
pub struct SolanaPipelineTool;

#[async_trait::async_trait]
impl ToolTrait for SolanaPipelineTool {
    fn name(&self) -> &str {
        "solana_pipeline"
    }

    fn description(&self) -> &str {
        "Run the Solana CI/CD pipeline with configurable steps. \
         Steps: cargo check → cargo build-sbf → cargo test → solana deploy → example client. \
         Each step can be enabled/disabled via flags. The pipeline auto-starts a local validator \
         when needed (deploy/client steps). Use --all to run all steps, or select specific steps \
         with individual flags."
    }

    fn input_schema(&self) -> Value {
        Self::definition()["inputSchema"].clone()
    }

    async fn execute(&self, args: Value) -> Result<ToolResponse> {
        let input: PipelineInput = serde_json::from_value(args)?;
        let result = Self::execute(input)
            .map_err(|e| anyhow::anyhow!(e))?;

        let content_text = if result.success {
            format!(
                "✅ Pipeline completed successfully\n\n{}\n\nStep Results:\n{}",
                result.stdout,
                result.steps
                    .iter()
                    .map(|s| format!("  {} {}", 
                        match s.status.as_str() {
                            "PASS" => "✅",
                            "FAIL" => "❌",
                            "SKIP" => "⊝",
                            _ => "�",
                        },
                        s.name
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            format!(
                "❌ Pipeline failed\n\n{}\n\n{}",
                result.stdout,
                result.stderr
            )
        };

        Ok(ToolResponse {
            content: vec![Content::Text { text: content_text }],
            is_error: Some(!result.success),
        })
    }
}

impl SolanaPipelineTool {
    /// Get the tool definition for MCP protocol
    pub fn definition() -> Value {
        json!({
            "name": "solana_pipeline",
            "description": "Run the Solana CI/CD pipeline with configurable steps. Steps: cargo check → cargo build-sbf → cargo test → solana deploy → example client. Each step can be enabled/disabled via flags. The pipeline auto-starts a local validator when needed (deploy/client steps). Use --all to run all steps, or select specific steps with individual flags.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "check": {
                        "type": "boolean",
                        "description": "Run cargo check (type-check without building). Fast feedback for catching compilation errors.",
                        "default": false
                    },
                    "build": {
                        "type": "boolean",
                        "description": "Run cargo build-sbf to compile the Solana program to BPF bytecode. Produces target/deploy/<program>.so required for tests and deployment.",
                        "default": false
                    },
                    "test": {
                        "type": "boolean",
                        "description": "Run cargo test to execute all tests including LiteSVM tests. Requires the program binary to exist (run build step first).",
                        "default": false
                    },
                    "deploy": {
                        "type": "boolean",
                        "description": "Deploy the program to local validator at localhost:8899. Auto-starts validator if not running. Requires build step to have run first.",
                        "default": false
                    },
                    "client": {
                        "type": "boolean",
                        "description": "Run the example client (cargo run --bin client). Requires validator to be running and program to be deployed.",
                        "default": false
                    },
                    "all": {
                        "type": "boolean",
                        "description": "Run all pipeline steps in sequence: check → build → test → deploy → client. Equivalent to enabling all individual flags.",
                        "default": false
                    },
                    "validator_timeout": {
                        "type": "integer",
                        "description": "Maximum seconds to wait for validator startup when auto-starting (default: 30). Only relevant when deploy or client steps are enabled.",
                        "default": 30,
                        "minimum": 1,
                        "maximum": 300
                    }
                },
                "required": []
            }
        })
    }

    /// Execute the pipeline
    pub fn execute(input: PipelineInput) -> Result<PipelineResult, String> {
        // Build command with flags
        let mut args = vec!["run", "--release", "--manifest-path", "toolbox/cli/Cargo.toml", "--", "pipeline"];
        
        if input.all {
            args.push("--all");
        } else {
            if input.check {
                args.push("--check");
            }
            if input.build {
                args.push("--build");
            }
            if input.test {
                args.push("--test");
            }
            if input.deploy {
                args.push("--deploy");
            }
            if input.client {
                args.push("--client");
            }
        }
        
        // Add validator timeout
        args.push("--validator-timeout");
        let timeout_str = input.validator_timeout.to_string();
        args.push(&timeout_str);

        // Execute pipeline
        let output = Command::new("cargo")
            .args(&args)
            .current_dir(".")  // Run from workspace root
            .output()
            .map_err(|e| format!("Failed to execute pipeline: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok(PipelineResult::parse_from_output(&stdout, &stderr, success))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_definition() {
        let def = SolanaPipelineTool::definition();
        assert_eq!(def["name"], "solana_pipeline");
        assert!(def["description"].is_string());
        assert!(def["inputSchema"]["properties"].is_object());
    }

    #[test]
    fn test_parse_results() {
        let stdout = r#"
════════════════════════════════════════════════════════════
  Pipeline Summary
════════════════════════════════════════════════════════════
   ✅ PASS  check
   ✅ PASS  build
   ❌ FAIL  test
   ⊝ SKIP  deploy
   ⊝ SKIP  client
════════════════════════════════════════════════════════════
"#;
        
        let result = PipelineResult::parse_from_output(stdout, "", false);
        assert_eq!(result.steps.len(), 5);
        assert_eq!(result.steps[0].name, "check");
        assert_eq!(result.steps[0].status, "PASS");
        assert_eq!(result.steps[2].name, "test");
        assert_eq!(result.steps[2].status, "FAIL");
    }
}
