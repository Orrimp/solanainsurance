//! Solana local deployment workflow MCP tool
//!
//! Exposes script-equivalent local workflow (validator + build + deploy + client)
//! through the toolbox CLI deploy subcommand.

use crate::server::ToolTrait;
use crate::types::{Content, ToolResponse};
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct DeployLocalInput {
    #[serde(default = "default_airdrop")]
    pub airdrop: u64,
    #[serde(default)]
    pub keep_validator: bool,
    #[serde(default)]
    pub skip_build: bool,
    #[serde(default)]
    pub skip_client: bool,
    #[serde(default = "default_timeout")]
    pub validator_timeout: u64,
    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,
    #[serde(default)]
    pub program_so: Option<String>,
}

fn default_airdrop() -> u64 {
    2
}

fn default_timeout() -> u64 {
    30
}

fn default_rpc_url() -> String {
    "http://127.0.0.1:8899".to_string()
}

pub struct SolanaDeployLocalTool;

#[async_trait::async_trait]
impl ToolTrait for SolanaDeployLocalTool {
    fn name(&self) -> &str {
        "solana_deploy_local"
    }

    fn description(&self) -> &str {
        "Run local workflow equivalent to scripts/run_local.sh: ensure validator, optional build, deploy program, and optional client run."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "airdrop": {
                    "type": "integer",
                    "description": "Amount of SOL to airdrop to default keypair.",
                    "default": 2,
                    "minimum": 0
                },
                "keep_validator": {
                    "type": "boolean",
                    "description": "Keep validator process running after command exits.",
                    "default": false
                },
                "skip_build": {
                    "type": "boolean",
                    "description": "Skip build step and deploy existing binary.",
                    "default": false
                },
                "skip_client": {
                    "type": "boolean",
                    "description": "Skip running example client after deploy.",
                    "default": false
                },
                "validator_timeout": {
                    "type": "integer",
                    "description": "Maximum seconds to wait for validator startup.",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 300
                },
                "rpc_url": {
                    "type": "string",
                    "description": "RPC URL for local deployment.",
                    "default": "http://127.0.0.1:8899"
                },
                "program_so": {
                    "type": "string",
                    "description": "Optional override path to program .so binary."
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResponse> {
        let input: DeployLocalInput = serde_json::from_value(args)?;

        let mut cmd_args = vec![
            "run".to_string(),
            "--release".to_string(),
            "--manifest-path".to_string(),
            "toolbox/cli/Cargo.toml".to_string(),
            "--".to_string(),
            "deploy".to_string(),
            "local".to_string(),
            "--airdrop".to_string(),
            input.airdrop.to_string(),
            "--validator-timeout".to_string(),
            input.validator_timeout.to_string(),
            "--rpc-url".to_string(),
            input.rpc_url,
        ];

        if input.keep_validator {
            cmd_args.push("--keep-validator".to_string());
        }
        if input.skip_build {
            cmd_args.push("--skip-build".to_string());
        }
        if input.skip_client {
            cmd_args.push("--skip-client".to_string());
        }
        if let Some(program_so) = input.program_so {
            cmd_args.push("--program-so".to_string());
            cmd_args.push(program_so);
        }

        let output = Command::new("cargo").args(&cmd_args).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        let text = if success {
            format!("✅ Local deploy workflow completed\n\n{}\n{}", stdout, stderr)
        } else {
            format!("❌ Local deploy workflow failed\n\n{}\n{}", stdout, stderr)
        };

        Ok(ToolResponse {
            content: vec![Content::Text { text }],
            is_error: Some(!success),
        })
    }
}
