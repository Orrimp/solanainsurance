//! MCP protocol types for the Insurance toolbox
//!
//! This module defines the core Model Context Protocol types used for AI agent
//! communication, including tools, resources, and prompts.
//!
//! # Examples
//!
//! ```
//! use insurance_mcp_server::types::{Tool, ToolRequest, Content};
//! use serde_json::json;
//!
//! // Define a tool
//! let tool = Tool {
//!     name: "solana_build".to_string(),
//!     description: "Compile Solana program".to_string(),
//!     input_schema: json!({
//!         "type": "object",
//!         "properties": {}
//!     }),
//! };
//!
//! // Create a tool request
//! let request = ToolRequest {
//!     name: "solana_build".to_string(),
//!     arguments: json!({}),
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Represents an MCP tool that can be invoked by AI agents
///
/// Tools have a name, description, and JSON Schema defining their input parameters.
///
/// # Examples
///
/// ```
/// use solana_mcp_server::types::Tool;
/// use serde_json::json;
///
/// let tool = Tool {
///     name: "solana_test".to_string(),
///     description: "Run Solana program tests".to_string(),
///     input_schema: json!({
///         "type": "object",
///         "properties": {
///             "test_name": {"type": "string"}
///         }
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool invocation request
///
/// Sent by AI agents to invoke a registered tool with specific arguments.
///
/// # Examples
///
/// ```
/// use solana_mcp_server::types::ToolRequest;
/// use serde_json::json;
///
/// let request = ToolRequest {
///     name: "validate_architecture".to_string(),
///     arguments: json!({"check_docs": true}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool invocation response
///
/// Contains the result of tool execution, with optional error flag.
///
/// # Examples
///
/// ```
/// use solana_mcp_server::types::{ToolResponse, Content};
///
/// // Success response
/// let response = ToolResponse {
///     content: vec![Content::Text {
///         text: "Build successful".to_string()
///     }],
///     is_error: Some(false),
/// };
///
/// // Error response
/// let error_response = ToolResponse {
///     content: vec![Content::Text {
///         text: "Build failed".to_string()
///     }],
///     is_error: Some(true),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content type for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "resource")]
    Resource { uri: String, text: String },
}

/// MCP Resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP Prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
