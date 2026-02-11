// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! MCP server implementation using stdio transport.
//!
//! This module implements the MCP server that communicates via stdin/stdout
//! using JSON-RPC 2.0 messages.

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    client::SyntonDbClient,
    protocol::{
        CallToolRequest, InitializeRequest, InitializeResponse, JsonRpcError,
        JsonRpcRequest, JsonRpcResponse, ListToolsRequest,
        ListToolsResponse, RequestId, ServerCapabilities, ServerInfo, ToolsCapability,
    },
    tools::{execute_tool, get_all_tools},
    McpError, McpResult,
};

/// MCP server state.
#[derive(Clone)]
pub struct McpServerState {
    /// SYNTON-DB client.
    client: Arc<SyntonDbClient>,
    /// Whether initialized.
    initialized: Arc<RwLock<bool>>,
}

impl McpServerState {
    /// Create a new server state.
    pub fn new(client: SyntonDbClient) -> Self {
        Self {
            client: Arc::new(client),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Check if initialized.
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    /// Mark as initialized.
    pub async fn mark_initialized(&self) {
        *self.initialized.write().await = true;
    }

    /// Get the SYNTON-DB client.
    pub fn client(&self) -> &SyntonDbClient {
        &self.client
    }
}

/// MCP server.
pub struct McpServer {
    state: McpServerState,
}

impl McpServer {
    /// Create a new MCP server.
    pub fn new(client: SyntonDbClient) -> Self {
        Self {
            state: McpServerState::new(client),
        }
    }

    /// Get the server state.
    pub fn state(&self) -> &McpServerState {
        &self.state
    }

    /// Run the server (stdio mode).
    ///
    /// This method reads JSON-RPC requests from stdin and writes responses to stdout.
    pub async fn run_stdio(&self) -> McpResult<()> {
        tracing::info!("Starting MCP server in stdio mode");

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line.map_err(|e| McpError::Io(e))?;

            // Skip empty lines and whitespace
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            tracing::debug!("Received request: {}", line);

            // Parse and handle the request
            let response = match self.handle_request_line(line).await {
                Ok(Some(resp)) => resp,
                Ok(None) => continue, // Notification, no response
                Err(e) => {
                    tracing::error!("Error handling request: {}", e);

                    // Try to extract request ID for error response
                    if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(line) {
                        JsonRpcResponse::error(req.id, JsonRpcError::internal_error(e.to_string()))
                    } else {
                        // Can't parse request, send generic error
                        JsonRpcResponse::error(
                            RequestId::Null,
                            JsonRpcError::parse_error("Failed to parse request"),
                        )
                    }
                }
            };

            // Write response
            let response_json = serde_json::to_string(&response)
                .map_err(|e| McpError::Json(e))?;
            writeln!(stdout, "{}", response_json).map_err(|e| McpError::Io(e))?;
            stdout.flush().map_err(|e| McpError::Io(e))?;

            tracing::debug!("Sent response: {}", response_json);
        }

        Ok(())
    }

    /// Handle a single request line.
    async fn handle_request_line(&self, line: &str) -> McpResult<Option<JsonRpcResponse>> {
        // Try to parse as a request (not a notification)
        let request: JsonRpcRequest = serde_json::from_str(line)?;

        let response = self.handle_request(request).await?;
        Ok(Some(response))
    }

    /// Handle a JSON-RPC request.
    async fn handle_request(&self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params, id).await,
            "notifications/initialized" => {
                self.state.mark_initialized().await;
                Ok(JsonRpcResponse::success(id, serde_json::json!({})))
            }
            "tools/list" => self.handle_list_tools(request.params, id).await,
            "tools/call" => self.handle_call_tool(request.params, id).await,
            _ => Ok(JsonRpcResponse::error(
                id,
                JsonRpcError::method_not_found(request.method),
            )),
        }
    }

    /// Handle initialize request.
    async fn handle_initialize(
        &self,
        params: serde_json::Value,
        id: RequestId,
    ) -> McpResult<JsonRpcResponse> {
        let init_req: InitializeRequest = serde_json::from_value(params)
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid initialize params: {}", e)))?;

        tracing::info!(
            "Initializing MCP server with SYNTON-DB endpoint: {}",
            self.state.client().endpoint()
        );

        // Verify connection to SYNTON-DB
        match self.state.client().health().await {
            Ok(health) => {
                tracing::info!(
                    "Connected to SYNTON-DB: status={}, version={}",
                    health.status,
                    health.version
                );
            }
            Err(e) => {
                tracing::warn!("Failed to connect to SYNTON-DB: {}", e);
                // Don't fail initialization - the server can still function
                // and will return errors when tools are called
            }
        }

        let response = InitializeResponse {
            protocol_version: init_req.protocol_version,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "synton-db-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        Ok(JsonRpcResponse::success(id, serde_json::to_value(response)?))
    }

    /// Handle tools/list request.
    async fn handle_list_tools(
        &self,
        params: serde_json::Value,
        id: RequestId,
    ) -> McpResult<JsonRpcResponse> {
        let _list_req: ListToolsRequest = serde_json::from_value(params)
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid list params: {}", e)))?;

        let tools = get_all_tools();
        let response = ListToolsResponse { tools };

        Ok(JsonRpcResponse::success(id, serde_json::to_value(response)?))
    }

    /// Handle tools/call request.
    async fn handle_call_tool(
        &self,
        params: serde_json::Value,
        id: RequestId,
    ) -> McpResult<JsonRpcResponse> {
        // Ensure initialized
        if !self.state.is_initialized().await {
            return Ok(JsonRpcResponse::error(
                id,
                JsonRpcError::invalid_request("Server not initialized yet"),
            ));
        }

        let call_req: CallToolRequest = serde_json::from_value(params).map_err(|e| {
            JsonRpcError::invalid_params(format!("Invalid call_tool params: {}", e))
        })?;

        tracing::info!("Calling tool: {}", call_req.name);

        let result = execute_tool(self.state.client(), &call_req.name, call_req.arguments).await;
        let response = result.to_response();

        Ok(JsonRpcResponse::success(id, serde_json::to_value(response)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let client = SyntonDbClient::new();
        let server = McpServer::new(client);
        assert_eq!(server.state().client().endpoint(), SyntonDbClient::new().endpoint());
    }

    #[tokio::test]
    async fn test_server_state_initialized() {
        let client = SyntonDbClient::new();
        let state = McpServerState::new(client);

        assert!(!state.is_initialized().await);
        state.mark_initialized().await;
        assert!(state.is_initialized().await);
    }
}
