// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! SYNTON-DB MCP Server
//!
//! Model Context Protocol server for SYNTON-DB cognitive database.
//! Enables AI coding assistants to use SYNTON-DB as persistent memory.

use clap::Parser;
use synton_mcp_server::{McpServer, SyntonDbClient};

/// SYNTON-DB MCP Server
///
/// MCP (Model Context Protocol) server that exposes SYNTON-DB's
/// cognitive database capabilities as tools for AI coding assistants.
#[derive(Parser, Debug)]
#[command(name = "synton-mcp-server")]
#[command(author = "SYNTON-DB Team")]
#[command(version)]
#[command(about = "MCP server for SYNTON-DB cognitive database", long_about = None)]
struct Args {
    /// SYNTON-DB REST API endpoint
    #[arg(
        long,
        env = "SYNTONDB_ENDPOINT",
        default_value = "http://localhost:8080",
        global = true
    )]
    endpoint: String,

    /// Enable verbose logging
    #[arg(long, short, env = "VERBOSE", global = true)]
    verbose: bool,

    /// Enable trace logging
    #[arg(long, env = "TRACE", global = true)]
    trace: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let env_filter = if args.trace {
        tracing::Level::TRACE
    } else if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(env_filter)
        .with_target(false)
        .init();

    tracing::info!("SYNTON-DB MCP Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Connecting to SYNTON-DB at: {}", args.endpoint);

    // Create client and server
    let client = SyntonDbClient::with_endpoint(args.endpoint);
    let server = McpServer::new(client);

    // Run server (stdio mode)
    server.run_stdio().await?;

    Ok(())
}
