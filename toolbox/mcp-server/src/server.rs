//! MCP Server implementation for Insurance toolbox
//!
//! This module implements the Model Context Protocol server that routes JSON-RPC
//! requests to registered tools, resources, and prompts.
//!
//! # Architecture
//!
//! The server maintains registries of:
//! - **Tools**: Executable functions AI agents can invoke
//! - **Resources**: Static content AI agents can read
//! - **Prompts**: Templates for common interactions
//!
//! # Examples
//!
//! ```no_run
//! use solana_mcp_server::server::McpServer;
//! use solana_mcp_server::tools::SolanaBuildTool;
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = McpServer::new("my-server", "1.0.0");
//!     server.register_tool(Box::new(SolanaBuildTool)).await;
//!     server.run_stdio().await.unwrap();
//! }
//! ```

use crate::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::transport::StdioTransport;
use crate::types::{Content, Prompt, Resource, Tool, ToolRequest, ToolResponse};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Trait for implementing MCP tools
///
/// Any type implementing this trait can be registered with the MCP server
/// and invoked by AI agents.
///
/// # Examples
///
/// ```
/// use insurance_mcp_server::server::ToolTrait;
/// use insurance_mcp_server::types::{ToolResponse, Content};
/// use anyhow::Result;
/// use serde_json::{json, Value};
///
/// struct MyTool;
///
/// #[async_trait::async_trait]
/// impl ToolTrait for MyTool {
///     fn name(&self) -> &str { "my_tool" }
///     fn description(&self) -> &str { "Does something useful" }
///     fn input_schema(&self) -> Value {
///         json!({"type": "object", "properties": {}})
///     }
///     async fn execute(&self, _args: Value) -> Result<ToolResponse> {
///         Ok(ToolResponse {
///             content: vec![Content::Text {
///                 text: "Success".to_string()
///             }],
///             is_error: Some(false),
///         })
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait ToolTrait: Send + Sync {
    /// Get the tool's name (used for invocation)
    fn name(&self) -> &str;
    /// Get the tool's human-readable description
    fn description(&self) -> &str;
    /// Get the JSON Schema for tool input validation
    fn input_schema(&self) -> serde_json::Value;
    /// Execute the tool with given arguments
    ///
    /// # Arguments
    ///
    /// * `args` - JSON value containing tool arguments
    ///
    /// # Returns
    ///
    /// A `ToolResponse` containing the result or error message
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResponse>;
}

/// MCP Server for the Insurance toolbox
///
/// Manages tool, resource, and prompt registries and routes JSON-RPC requests
/// from AI agents to the appropriate handlers.
///
/// # Examples
///
/// ```no_run
/// use insurance_mcp_server::server::McpServer;
/// use insurance_mcp_server::tools::*;
///
/// #[tokio::main]
/// async fn main() {
///     let server = McpServer::new("insurance-mcp", "0.1.0");
///     
///     server.register_tool(Box::new(SolanaBuildTool)).await;
///     server.register_tool(Box::new(SolanaTestTool)).await;
///     
///     println!("Registered {} tools", server.list_tools().await.len());
///     
///     server.run_stdio().await.unwrap();
/// }
/// ```
pub struct McpServer {
    name: String,
    version: String,
    tools: Arc<RwLock<HashMap<String, Box<dyn ToolTrait>>>>,
    resources: Arc<RwLock<Vec<Resource>>>,
    prompts: Arc<RwLock<Vec<Prompt>>>,
}

impl McpServer {
    /// Create a new MCP server
    ///
    /// # Arguments
    ///
    /// * `name` - Server name (e.g., "insurance-mcp-server")
    /// * `version` - Server version (e.g., "0.1.0")
    ///
    /// # Examples
    ///
    /// ```
    /// use insurance_mcp_server::server::McpServer;
    ///
    /// let server = McpServer::new("my-server", "1.0.0");
    /// ```
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(Vec::new())),
            prompts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a tool with the server
    ///
    /// Adds a tool to the registry, making it available for AI agent invocation.
    ///
    /// # Arguments
    ///
    /// * `tool` - Boxed implementation of `ToolTrait`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use insurance_mcp_server::server::McpServer;
    /// use insurance_mcp_server::tools::SolanaBuildTool;
    ///
    /// async fn register_example() {
    ///     let server = McpServer::new("server", "1.0");
    ///     server.register_tool(Box::new(SolanaBuildTool)).await;
    /// }
    /// ```
    pub async fn register_tool(&self, tool: Box<dyn ToolTrait>) {
        let name = tool.name().to_string();
        info!("Registering tool: {}", name);
        self.tools.write().await.insert(name, tool);
    }

    /// Register a resource
    pub async fn register_resource(&self, resource: Resource) {
        info!("Registering resource: {}", resource.name);
        self.resources.write().await.push(resource);
    }

    /// Register a prompt
    pub async fn register_prompt(&self, prompt: Prompt) {
        info!("Registering prompt: {}", prompt.name);
        self.prompts.write().await.push(prompt);
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<Tool> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| Tool {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }

    /// Execute a tool by name
    pub async fn execute_tool(&self, request: ToolRequest) -> Result<ToolResponse> {
        let tools = self.tools.read().await;
        
        match tools.get(&request.name) {
            Some(tool) => {
                info!("Executing tool: {}", request.name);
                tool.execute(request.arguments).await
            }
            None => {
                warn!("Tool not found: {}", request.name);
                Ok(ToolResponse {
                    content: vec![Content::Text {
                        text: format!("Tool '{}' not found", request.name),
                    }],
                    is_error: Some(true),
                })
            }
        }
    }

    /// Run the server on stdio (standard input/output)
    ///
    /// This is the primary transport for MCP servers. Reads JSON-RPC requests
    /// from stdin and writes responses to stdout.
    ///
    /// Runs until:
    /// - Stdin is closed (EOF)
    /// - A parse error occurs
    /// - An unrecoverable error happens
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use insurance_mcp_server::server::McpServer;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let server = McpServer::new("server", "1.0");
    ///     server.run_stdio().await.unwrap();
    /// }
    /// ```
    pub async fn run_stdio(&self) -> Result<()> {
        info!("Starting MCP server: {} v{}", self.name, self.version);
        info!("Running on stdio transport");
        
        let mut transport = StdioTransport::new();
        
        loop {
            // Read request from stdin
            match transport.read_request().await {
                Ok(Some(request)) => {
                    // Handle the request
                    if let Some(response) = self.handle_request(request).await {
                        // Write response to stdout
                        if let Err(e) = transport.write_response(response).await {
                            error!("Failed to write response: {}", e);
                            break;
                        }
                    }
                }
                Ok(None) => {
                    // EOF - client disconnected
                    info!("Client disconnected");
                    break;
                }
                Err(e) => {
                    error!("Error reading request: {}", e);
                    // Try to send error response
                    let error_response = JsonRpcResponse::error(
                        JsonRpcError::parse_error(),
                        None,
                    );
                    let _ = transport.write_response(error_response).await;
                    break;
                }
            }
        }
        
        info!("MCP server stopped");
        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        debug!("Handling request: {}", request.method);

        let request_id = request.id.clone();
        let is_notification = request_id.is_none();

        // Route to appropriate handler
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "notifications/initialized" => Ok(json!({})),
            "tools/list" => self.handle_tools_list(request.params).await,
            "tools/call" => self.handle_tools_call(request.params).await,
            "resources/list" => self.handle_resources_list(request.params).await,
            "prompts/list" => self.handle_prompts_list(request.params).await,
            _ => Err(JsonRpcError::method_not_found(&request.method)),
        };

        if is_notification {
            return None;
        }

        // Convert result to response
        match result {
            Ok(value) => Some(JsonRpcResponse::success(value, request_id)),
            Err(error) => Some(JsonRpcResponse::error(error, request_id)),
        }
    }

    /// Handle initialize request
    async fn handle_initialize(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        debug!("Initialize request: {:?}", params);
        
        // MCP initialize response includes server info and capabilities
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": self.name,
                "version": self.version
            }
        }))
    }

    /// Handle tools/list request
    async fn handle_tools_list(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        debug!("Listing tools");
        
        let tools = self.list_tools().await;
        
        Ok(json!({
            "tools": tools
        }))
    }

    /// Handle tools/call request
    async fn handle_tools_call(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        debug!("Calling tool");
        
        // Extract params
        let params = params.ok_or_else(|| {
            JsonRpcError::invalid_params("Missing params for tools/call")
        })?;

        // Parse tool call parameters
        let tool_name = params["name"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing tool name"))?;

        let tool_args = params.get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        // Create tool request
        let request = ToolRequest {
            name: tool_name.to_string(),
            arguments: tool_args,
        };

        // Execute tool
        let response = self
            .execute_tool(request)
            .await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

        // Convert to JSON
        Ok(serde_json::to_value(response)
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
    }

    /// Handle resources/list request
    async fn handle_resources_list(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        debug!("Listing resources");
        
        let resources = self.resources.read().await;
        
        Ok(json!({
            "resources": resources.clone()
        }))
    }

    /// Handle prompts/list request
    async fn handle_prompts_list(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        debug!("Listing prompts");
        
        let prompts = self.prompts.read().await;
        
        Ok(json!({
            "prompts": prompts.clone()
        }))
    }

    /// Get server information
    pub fn info(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "version": self.version,
            "protocol_version": "2024-11-05",
            "capabilities": {
                "tools": true,
                "resources": true,
                "prompts": true
            }
        })
    }
}
