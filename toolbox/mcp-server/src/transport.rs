//! Stdio transport for JSON-RPC 2.0 communication
//!
//! This module provides async I/O over stdin/stdout for JSON-RPC 2.0 protocol,
//! used by Model Context Protocol servers.
//!
//! # Protocol
//!
//! - **Input**: Line-based JSON messages on stdin
//! - **Output**: Line-based JSON messages on stdout
//! - **Format**: One JSON-RPC message per line
//!
//! # Examples
//!
//! ```no_run
//! use solana_mcp_server::transport::StdioTransport;
//! use solana_mcp_server::jsonrpc::{JsonRpcRequest, JsonRpcResponse, RequestId};
//! use serde_json::json;
//!
//! async fn example() {
//!     let mut transport = StdioTransport::new();
//!     
//!     // Read a request from stdin
//!     if let Some(request) = transport.read_request().await.unwrap() {
//!         // Create a response
//!         let response = JsonRpcResponse::success(
//!             json!({"result": "ok"}),
//!             request.id
//!         );
//!         
//!         // Write response to stdout
//!         transport.write_response(response).await.unwrap();
//!     }
//! }
//! ```

use crate::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

/// Stdio transport for MCP protocol
///
/// Handles async reading from stdin and writing to stdout for JSON-RPC 2.0
/// messages in the Model Context Protocol.
///
/// # Examples
///
/// ```no_run
/// use insurance_mcp_server::transport::StdioTransport;
///
/// async fn run() {
///     let mut transport = StdioTransport::new();
///     
///     while let Some(request) = transport.read_request().await.unwrap() {
///         // Process request...
///     }
/// }
/// ```
pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

impl StdioTransport {
    /// Create a new stdio transport
    ///
    /// # Examples
    ///
    /// ```
    /// use insurance_mcp_server::transport::StdioTransport;
    ///
    /// let transport = StdioTransport::new();
    /// ```
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }
    }

    /// Read a JSON-RPC request from stdin
    ///
    /// Reads line-by-line from stdin, skipping empty lines and handling BOM characters.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(request))` - Successfully read a request
    /// - `Ok(None)` - EOF reached (stdin closed)
    /// - `Err(_)` - I/O error or JSON parse error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use insurance_mcp_server::transport::StdioTransport;
    ///
    /// async fn read_loop() {
    ///     let mut transport = StdioTransport::new();
    ///     
    ///     while let Some(request) = transport.read_request().await.unwrap() {
    ///         println!("Received: {}", request.method);
    ///     }
    ///     
    ///     println!("Stdin closed");
    /// }
    /// ```
    pub async fn read_request(&mut self) -> Result<Option<JsonRpcRequest>> {
        loop {
            let mut line = String::new();
            
            match self.reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF reached
                    info!("Stdin closed, shutting down");
                    return Ok(None);
                }
                Ok(n) => {
                    debug!("Read {} bytes from stdin", n);
                    
                    // Trim whitespace, newlines, and BOM
                    let trimmed = line.trim().trim_start_matches('\u{feff}');
                    
                    // Skip empty lines
                    if trimmed.is_empty() {
                        debug!("Empty line received, skipping");
                        continue;
                    }
                    
                    // Log the actual JSON for debugging
                    info!("Received JSON: {}", trimmed);
                    
                    // Parse JSON-RPC request
                    let request: JsonRpcRequest = serde_json::from_str(trimmed)
                        .context("Failed to parse JSON-RPC request")?;
                    
                    debug!("Parsed request: method={}, id={:?}", request.method, request.id);
                    return Ok(Some(request));
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    /// Write a JSON-RPC response to stdout
    ///
    /// Serializes the response to JSON and writes it to stdout with a newline,
    /// flushing immediately to ensure the client receives it.
    ///
    /// # Arguments
    ///
    /// * `response` - The JSON-RPC response to write
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or if writing to stdout fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use solana_mcp_server::transport::StdioTransport;
    /// use solana_mcp_server::jsonrpc::{JsonRpcResponse, RequestId};
    /// use serde_json::json;
    ///
    /// async fn send_response() {
    ///     let mut transport = StdioTransport::new();
    ///     let response = JsonRpcResponse::success(
    ///         json!({"status": "ok"}),
    ///         Some(RequestId::Number(1))
    ///     );
    ///     
    ///     transport.write_response(response).await.unwrap();
    /// }
    /// ```
    pub async fn write_response(&mut self, response: JsonRpcResponse) -> Result<()> {
        // Serialize response to JSON
        let json = serde_json::to_string(&response)
            .context("Failed to serialize JSON-RPC response")?;
        
        // Write to stdout with newline
        self.writer
            .write_all(json.as_bytes())
            .await
            .context("Failed to write to stdout")?;
        
        self.writer
            .write_all(b"\n")
            .await
            .context("Failed to write newline to stdout")?;
        
        // Flush to ensure immediate delivery
        self.writer
            .flush()
            .await
            .context("Failed to flush stdout")?;
        
        debug!("Sent response: id={:?}", response.id);
        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}
