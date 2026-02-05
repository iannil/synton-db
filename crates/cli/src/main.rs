// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! SYNTON-DB CLI Tool
//!
//! Command-line interface for managing SYNTON-DB cognitive database.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod client;
mod commands;
mod output;

use clap::{Parser, Subcommand};
use commands::{EdgeCommand, NodeCommand, QueryCommand, StatsCommand};

use crate::client::SyntonClient;

/// SYNTON-DB CLI - Command-line interface for SYNTON-DB cognitive database
#[derive(Parser, Debug)]
#[command(name = "synton-cli")]
#[command(author = "SYNTON-DB Team")]
#[command(version)]
#[command(about = "Command-line interface for SYNTON-DB cognitive database", long_about = None)]
struct Cli {
    /// Server host
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// REST API port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Quiet mode (minimal output)
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Node operations
    #[command(subcommand)]
    Node(NodeCommand),

    /// Edge operations
    #[command(subcommand)]
    Edge(EdgeCommand),

    /// Query operations
    #[command(subcommand)]
    Query(QueryCommand),

    /// Database statistics
    Stats(StatsCommand),

    /// Export data
    Export {
        /// Export format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import data
    Import {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        input: Option<String>,

        /// Import format (json)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Continue on error
        #[arg(long)]
        continue_on_error: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing
    if !cli.quiet {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_level(false)
            .init();
    }

    // Create client
    let base_url = format!("http://{}:{}", cli.host, cli.port);
    let client = SyntonClient::new(base_url);

    // Execute command
    match cli.command {
        Commands::Node(cmd) => commands::execute_node(cmd, client, &cli.format).await?,
        Commands::Edge(cmd) => commands::execute_edge(cmd, client, &cli.format).await?,
        Commands::Query(cmd) => commands::execute_query(cmd, client, &cli.format).await?,
        Commands::Stats(cmd) => commands::execute_stats(cmd, client, &cli.format).await?,
        Commands::Export { format, output } => {
            commands::execute_export(client, &format, output).await?
        }
        Commands::Import {
            input,
            format,
            continue_on_error,
        } => commands::execute_import(client, input, &format, continue_on_error).await?,
    }

    Ok(())
}
