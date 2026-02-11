// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! MCP (Model Context Protocol) message types.
//!
//! This module defines the JSON-RPC 2.0 based protocol used for communication
//! between the MCP server and AI coding assistants.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0").
    pub jsonrpc: String,
    /// Request ID.
    pub id: RequestId,
    /// Method name.
    pub method: String,
    /// Method parameters (optional).
    #[serde(default)]
    pub params: serde_json::Value,
}

impl JsonRpcRequest {
    /// Create a new request.
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: RequestId::String(Uuid::new_v4().to_string()),
            method: method.into(),
            params,
        }
    }

    /// Create a notification (no response expected).
    pub fn notification(method: impl Into<String>, params: serde_json::Value) -> JsonRpcNotification {
        JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 notification (no ID, no response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Request ID.
    pub id: RequestId,
    /// Result (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional error data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Parse error (-32700).
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(-32700, message)
    }

    /// Invalid request (-32600).
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(-32600, message)
    }

    /// Method not found (-32601).
    pub fn method_not_found(method: String) -> Self {
        Self::new(-32601, format!("Method not found: {}", method))
    }

    /// Invalid params (-32602).
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(-32602, message)
    }

    /// Internal error (-32603).
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(-32603, message)
    }
}

/// Request identifier (can be number, string, or null).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    /// Number ID.
    Number(i64),
    /// String ID.
    String(String),
    /// Null ID (for notifications).
    Null,
}

/// Initialize request from client.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct InitializeRequest {
    /// Protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client info.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
    /// Requested trace level (optional).
    #[serde(default, rename = "traceLevel")]
    pub trace_level: Option<String>,
}

/// Client capabilities.
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct ClientCapabilities {
    /// Sampling/sagemaker capabilities (optional).
    #[serde(default)]
    pub sampling: Option<serde_json::Value>,
    /// Resources capabilities (optional).
    #[serde(default)]
    pub resources: Option<bool>,
}

/// Client information.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    pub version: String,
}

/// Initialize response to client.
#[derive(Debug, Clone, Serialize)]
pub struct InitializeResponse {
    /// Protocol version (must match client's requested version).
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server info.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server capabilities.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ServerCapabilities {
    /// Tools capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Tools capability.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ToolsCapability {
    /// Whether tools can list themselves (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Server information.
#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

/// List tools request.
#[derive(Debug, Clone, Deserialize)]
pub struct ListToolsRequest {
    /// Optional cursor for pagination.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// List tools response.
#[derive(Debug, Clone, Serialize)]
pub struct ListToolsResponse {
    /// Available tools.
    pub tools: Vec<Tool>,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Call tool request.
#[derive(Debug, Clone, Deserialize)]
pub struct CallToolRequest {
    /// Tool name to call.
    pub name: String,
    /// Tool arguments.
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Call tool response.
#[derive(Debug, Clone, Serialize)]
pub struct CallToolResponse {
    /// Tool result content.
    pub content: Vec<ToolContent>,
    /// Whether the result is an error.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_error: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

/// Tool content (can be text or image).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content.
    Text(ToolTextContent),
    /// Image content (not currently used).
    Image(ToolImageContent),
    /// Embedded resource (not currently used).
    Resource(ToolResourceContent),
}

/// Text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTextContent {
    /// Content type discriminator.
    #[serde(rename = "type")]
    pub content_type: String,
    /// Text content.
    pub text: String,
}

impl ToolTextContent {
    /// Create new text content.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: text.into(),
        }
    }
}

/// Image content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolImageContent {
    /// Content type discriminator.
    #[serde(rename = "type")]
    pub content_type: String,
    /// Image data (base64).
    pub data: String,
    /// MIME type.
    pub mime_type: String,
}

/// Resource content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResourceContent {
    /// Content type discriminator.
    #[serde(rename = "type")]
    pub content_type: String,
    /// Resource URI.
    pub uri: String,
}

/// Tool execution error.
#[derive(Debug, Clone, Serialize)]
pub struct ToolError {
    /// Error message.
    pub message: String,
    /// Additional error details (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ToolError {
    /// Create a new tool error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the error.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Convert to tool content.
    pub fn to_content(&self) -> ToolContent {
        let text = if let Some(ref details) = self.details {
            format!("{}: {}", self.message, details)
        } else {
            self.message.clone()
        };
        ToolContent::Text(ToolTextContent::new(text))
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref details) = self.details {
            write!(f, ": {}", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for ToolError {}

/// Call tool result wrapper.
#[derive(Debug)]
pub enum CallToolResult {
    /// Success with content.
    Success(Vec<ToolContent>),
    /// Error.
    Error(ToolError),
}

impl CallToolResult {
    /// Convert to response.
    pub fn to_response(self) -> CallToolResponse {
        match self {
            Self::Success(content) => CallToolResponse {
                content,
                is_error: false,
            },
            Self::Error(err) => CallToolResponse {
                content: vec![err.to_content()],
                is_error: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request() {
        let req = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test_method");
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let id = RequestId::Number(1);
        let result = serde_json::json!({"status": "ok"});
        let resp = JsonRpcResponse::success(id, result);
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error() {
        let err = JsonRpcError::method_not_found("unknown_method".to_string());
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("unknown_method"));
    }

    #[test]
    fn test_tool_text_content() {
        let content = ToolTextContent::new("Hello, world!");
        assert_eq!(content.content_type, "text");
        assert_eq!(content.text, "Hello, world!");
    }

    #[test]
    fn test_tool_error() {
        let err = ToolError::new("Test error").with_details("Additional info");
        assert_eq!(err.message, "Test error");
        assert_eq!(err.details, Some("Additional info".to_string()));
    }
}
