// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! CLI command implementations.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::io::Read;
use uuid::Uuid;

use crate::client::SyntonClient;
use crate::output::OutputFormat;
use synton_core::{NodeType, Relation};

/// Node commands
#[derive(Subcommand, Debug)]
pub enum NodeCommand {
    /// Create a new node
    Create {
        /// Node content
        content: String,

        /// Node type (entity, concept, fact, raw_chunk)
        #[arg(short, long, default_value = "concept")]
        node_type: String,
    },

    /// Get a node by ID
    Get {
        /// Node ID
        id: String,
    },

    /// Delete a node by ID
    Delete {
        /// Node ID
        id: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List all nodes
    List {
        /// Maximum number of nodes to return
        #[arg(short, long, default_value = "100")]
        limit: usize,
    },
}

/// Query commands
#[derive(Subcommand, Debug)]
pub enum QueryCommand {
    /// Execute a PaQL query
    Execute {
        /// Query string
        query: String,

        /// Maximum number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

/// Stats command arguments
#[derive(Args, Debug)]
pub struct StatsCommand {
    /// Show detailed statistics
    #[arg(short, long)]
    pub detailed: bool,
}

/// Edge command arguments
#[derive(Args, Debug)]
pub struct EdgeCreateCommand {
    /// Source node ID
    pub source: String,

    /// Target node ID
    pub target: String,

    /// Relation type (is_a, is_part_of, causes, similar_to, contradicts, happened_after, belongs_to)
    pub relation: String,

    /// Edge weight (0.0-1.0)
    #[arg(short, long, default_value = "1.0")]
    pub weight: f32,
}

/// Edge list command arguments
#[derive(Args, Debug)]
pub struct EdgeListCommand {
    /// Node ID
    pub id: String,

    /// Maximum number of edges to return
    #[arg(short, long, default_value = "100")]
    pub limit: usize,
}

/// Edge commands
#[derive(Subcommand, Debug)]
pub enum EdgeCommand {
    /// Create a new edge
    Create(EdgeCreateCommand),

    /// List edges for a node
    List(EdgeListCommand),
}

/// Execute a node command.
pub async fn execute_node(
    cmd: NodeCommand,
    client: SyntonClient,
    format: &str,
) -> Result<()> {
    let output = OutputFormat::from_str(format);

    match cmd {
        NodeCommand::Create { content, node_type } => {
            let node_type = parse_node_type(&node_type)?;
            let node = client.create_node(content, node_type).await?;

            output.print_node(&node);
        }
        NodeCommand::Get { id } => {
            let uuid = Uuid::parse_str(&id)?;
            match client.get_node(uuid).await? {
                Some(node) => output.print_node(&node),
                None => {
                    eprintln!("Node not found: {}", id);
                    std::process::exit(1);
                }
            }
        }
        NodeCommand::Delete { id, force } => {
            let uuid = Uuid::parse_str(&id)?;

            if !force {
                eprintln!("Are you sure you want to delete node {}? (y/N)", id);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    eprintln!("Aborted");
                    return Ok(());
                }
            }

            let deleted = client.delete_node(uuid).await?;
            if deleted {
                println!("Node {} deleted", id);
            } else {
                eprintln!("Node not found: {}", id);
                std::process::exit(1);
            }
        }
        NodeCommand::List { limit } => {
            let nodes = client.list_nodes().await?;
            let nodes: Vec<_> = nodes.into_iter().take(limit).collect();
            output.print_nodes(&nodes);
        }
    }

    Ok(())
}

/// Execute an edge command.
pub async fn execute_edge(
    cmd: EdgeCommand,
    client: SyntonClient,
    format: &str,
) -> Result<()> {
    let output = OutputFormat::from_str(format);

    match cmd {
        EdgeCommand::Create(args) => {
            let source_uuid = Uuid::parse_str(&args.source)?;
            let target_uuid = Uuid::parse_str(&args.target)?;
            let relation = parse_relation(&args.relation)?;

            let edge = client
                .create_edge(source_uuid, target_uuid, relation, args.weight)
                .await?;
            output.print_edge(&edge);
        }
        EdgeCommand::List(args) => {
            // Note: This endpoint isn't fully implemented in the current API
            eprintln!("Edge listing not yet implemented in REST API");
            eprintln!("Use gRPC client for edge traversal");
            eprintln!("Node ID: {}", args.id);
        }
    }

    Ok(())
}

/// Execute a query command.
pub async fn execute_query(
    cmd: QueryCommand,
    client: SyntonClient,
    format: &str,
) -> Result<()> {
    let output = OutputFormat::from_str(format);

    match cmd {
        QueryCommand::Execute { query, limit } => {
            let response = client.query(query, limit).await?;
            output.print_query_response(&response);
        }
    }

    Ok(())
}

/// Execute a stats command.
pub async fn execute_stats(
    cmd: StatsCommand,
    client: SyntonClient,
    format: &str,
) -> Result<()> {
    let output = OutputFormat::from_str(format);

    let stats = client.stats().await?;
    output.print_stats(&stats, cmd.detailed);

    Ok(())
}

/// Execute an export command.
pub async fn execute_export(
    client: SyntonClient,
    format_name: &str,
    output: Option<String>,
) -> anyhow::Result<()> {
    let nodes = client.list_nodes().await?;

    let data = if format_name == "json" {
        serde_json::to_string_pretty(&nodes)?
    } else {
        anyhow::bail!("Unsupported export format: {}", format_name);
    };

    if let Some(path) = output {
        std::fs::write(&path, data)?;
        eprintln!("Exported {} nodes to {}", nodes.len(), path);
    } else {
        println!("{}", data);
    }

    Ok(())
}

/// Execute an import command.
pub async fn execute_import(
    client: SyntonClient,
    input: Option<String>,
    format_name: &str,
    continue_on_error: bool,
) -> anyhow::Result<()> {
    let data = if let Some(path) = input {
        std::fs::read_to_string(path)?
    } else {
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    if format_name == "json" {
        let nodes: Vec<serde_json::Value> = serde_json::from_str(&data)?;

        let mut success = 0;
        let mut failed = 0;

        for node in nodes {
            let content = node["content"].as_str().unwrap_or("");
            let node_type_str = node["node_type"].as_str().unwrap_or("concept");
            let node_type = parse_node_type(node_type_str).unwrap_or(NodeType::Concept);

            match client.create_node(content.to_string(), node_type).await {
                Ok(_) => success += 1,
                Err(e) => {
                    failed += 1;
                    eprintln!("Failed to create node: {}", e);
                    if !continue_on_error {
                        return Err(e.into());
                    }
                }
            }
        }

        eprintln!("Imported {} nodes successfully", success);
        if failed > 0 {
            eprintln!("Failed to import {} nodes", failed);
        }
    } else {
        anyhow::bail!("Unsupported import format: {}", format_name);
    }

    Ok(())
}

/// Parse node type from string.
fn parse_node_type(s: &str) -> Result<NodeType> {
    match s.to_lowercase().as_str() {
        "entity" => Ok(NodeType::Entity),
        "concept" => Ok(NodeType::Concept),
        "fact" => Ok(NodeType::Fact),
        "raw_chunk" => Ok(NodeType::RawChunk),
        _ => anyhow::bail!("Unknown node type: {}", s),
    }
}

/// Parse relation from string.
fn parse_relation(s: &str) -> Result<Relation> {
    match s.to_lowercase().replace('-', "_").as_str() {
        "is_a" => Ok(Relation::IsA),
        "is_part_of" => Ok(Relation::IsPartOf),
        "causes" => Ok(Relation::Causes),
        "similar_to" => Ok(Relation::SimilarTo),
        "contradicts" => Ok(Relation::Contradicts),
        "happened_after" => Ok(Relation::HappenedAfter),
        "belongs_to" => Ok(Relation::BelongsTo),
        _ => anyhow::bail!("Unknown relation: {}", s),
    }
}
