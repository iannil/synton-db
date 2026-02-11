// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! MCP (Model Context Protocol) server for SYNTON-DB.
//!
//! This crate provides an MCP server implementation that exposes SYNTON-DB's
//! cognitive database capabilities as tools for AI coding assistants like
//! Claude Code, Gemini CLI, Cursor, and Continue.
//!
//! # Features
//!
//! - Natural language query via PaQL (Prompt as Query Language)
//! - Knowledge absorption with automatic vectorization
//! - Graph traversal for exploring relationships
//! - Hybrid Graph-RAG retrieval
//! - Cross-session persistent memory
//!
//! # Configuration
//!
//! The MCP server connects to a running SYNTON-DB instance via REST API.
//! Configure the endpoint using the `--endpoint` argument or
//! `SYNTONDB_ENDPOINT` environment variable.
//!
//! # Example
//!
//! ```bash
//! # Start the MCP server
//! synton-mcp-server --endpoint http://localhost:8080
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod client;
mod protocol;
mod server;
mod tools;

pub use client::SyntonDbClient;
pub use protocol::{
    CallToolRequest, CallToolResponse, CallToolResult, JsonRpcError, JsonRpcNotification,
    JsonRpcRequest, JsonRpcResponse, ListToolsRequest, ListToolsResponse, Tool, ToolContent,
    ToolError, ToolTextContent,
};
pub use server::McpServer;
pub use tools::get_all_tools;

/// Result type for MCP operations.
pub type McpResult<T> = Result<T, McpError>;

/// Errors that can occur in the MCP server.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    Http(String),

    /// SYNTON-DB API error.
    #[error("SYNTON-DB API error: {0}")]
    Api(String),

    /// Invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Tool execution error.
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Not initialized.
    #[error("MCP server not initialized")]
    NotInitialized,
}

// Convert JsonRpcError to McpError for use with `?` operator
impl From<JsonRpcError> for McpError {
    fn from(err: JsonRpcError) -> Self {
        McpError::InvalidRequest(format!("JSON-RPC error: {} (code: {})", err.message, err.code))
    }
}

// Convert reqwest::Error to McpError for use with `?` operator
impl From<reqwest::Error> for McpError {
    fn from(err: reqwest::Error) -> Self {
        McpError::Http(format!("HTTP error: {}", err))
    }
}
