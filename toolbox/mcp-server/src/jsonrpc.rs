//! JSON-RPC 2.0 types for MCP protocol
//!
//! This module implements JSON-RPC 2.0 request/response types for Model Context Protocol
//! communication over stdio transport.
//!
//! # Examples
//!
//! ```
//! use solana_mcp_server::jsonrpc::{JsonRpcRequest, JsonRpcResponse, RequestId};
//! use serde_json::json;
//!
//! // Create a request
//! let request = JsonRpcRequest::new(
//!     "tools/list",
//!     None,
//!     Some(RequestId::Number(1))
//! );
//!
//! // Create a success response
//! let response = JsonRpcResponse::success(
//!     json!({"tools": []}),
//!     Some(RequestId::Number(1))
//! );
//! ```
//!
//! # JSON-RPC 2.0 Specification
//!
//! - Requests with `id` field expect a response
//! - Requests without `id` are notifications (no response expected)
//! - Responses must have the same `id` as the corresponding request

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
///
/// Represents a JSON-RPC 2.0 request. Requests with an `id` expect a response,
/// while requests without an `id` are notifications.
///
/// # Examples
///
/// ```
/// use insurance_mcp_server::jsonrpc::{JsonRpcRequest, RequestId};
/// use serde_json::json;
///
/// // Request expecting response
/// let request = JsonRpcRequest::new(
///     "initialize",
///     Some(json!({"protocolVersion": "2024-11-05"})),
///     Some(RequestId::Number(1))
/// );
///
/// // Notification (no response expected)
/// let notification = JsonRpcRequest::new(
///     "notifications/initialized",
///     None,
///     None
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String, // Must be "2.0"
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Optional id - if None, this is a notification (no response expected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
}

/// JSON-RPC 2.0 Response
///
/// Represents a response to a JSON-RPC 2.0 request. Contains either a `result` (success)
/// or an `error` (failure), never both.
///
/// # Examples
///
/// ```
/// use solana_mcp_server::jsonrpc::{JsonRpcResponse, JsonRpcError, RequestId};
/// use serde_json::json;
///
/// // Success response
/// let success = JsonRpcResponse::success(
///     json!({"status": "ok"}),
///     Some(RequestId::Number(1))
/// );
///
/// // Error response
/// let error = JsonRpcResponse::error(
///     JsonRpcError::method_not_found("unknown_method"),
///     Some(RequestId::Number(1))
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String, // Must be "2.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Optional id - notifications don't get responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
}

/// JSON-RPC 2.0 Error
///
/// Standard error codes per JSON-RPC 2.0 specification:
/// - `-32700`: Parse error (invalid JSON)
/// - `-32600`: Invalid request
/// - `-32601`: Method not found
/// - `-32602`: Invalid params
/// - `-32603`: Internal error
///
/// # Examples
///
/// ```
/// use solana_mcp_server::jsonrpc::JsonRpcError;
/// use serde_json::json;
///
/// // Standard errors
/// let parse_err = JsonRpcError::parse_error();
/// let method_err = JsonRpcError::method_not_found("foo");
///
/// // Custom error with data
/// let custom = JsonRpcError::new(-32000, "Custom error")
///     .with_data(json!({"details": "Additional info"}));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Request ID (can be string, number, or null)
///
/// JSON-RPC 2.0 allows request IDs to be strings, numbers, or null.
///
/// # Examples
///
/// ```
/// use solana_mcp_server::jsonrpc::RequestId;
///
/// let id1 = RequestId::String("req-123".to_string());
/// let id2 = RequestId::Number(42);
/// let id3 = RequestId::Null;
///
/// assert_ne!(id1, id2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    ///
    /// # Arguments
    ///
    /// * `method` - The method name to invoke
    /// * `params` - Optional parameters for the method
    /// * `id` - Optional request ID (None for notifications)
    ///
    /// # Examples
    ///
    /// ```
    /// use solana_mcp_server::jsonrpc::{JsonRpcRequest, RequestId};
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest::new(
    ///     "tools/call",
    ///     Some(json!({"name": "solana_build"})),
    ///     Some(RequestId::Number(1))
    /// );
    /// ```
    pub fn new(method: impl Into<String>, params: Option<Value>, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

impl JsonRpcResponse {
    /// Create a success response
    ///
    /// # Examples
    ///
    /// ```
    /// use solana_mcp_server::jsonrpc::{JsonRpcResponse, RequestId};
    /// use serde_json::json;
    ///
    /// let response = JsonRpcResponse::success(
    ///     json!({"tools": []}),
    ///     Some(RequestId::Number(1))
    /// );
    /// ```
    pub fn success(result: Value, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(error: JsonRpcError, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

impl JsonRpcError {
    // Standard JSON-RPC error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(Self::PARSE_ERROR, "Parse error")
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(Self::INVALID_REQUEST, message)
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(Self::METHOD_NOT_FOUND, format!("Method not found: {}", method))
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(Self::INVALID_PARAMS, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(Self::INTERNAL_ERROR, message)
    }

    /// Add data to the error
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}
