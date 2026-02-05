// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Output formatting for CLI.

use serde::Serialize;
use synton_core::{Edge, Node};

use crate::client::QueryResponse;

/// Output format for CLI.
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            _ => Self::Text,
        }
    }

    fn print_json<T: Serialize + ?Sized>(&self, value: &T) {
        if let Ok(json) = serde_json::to_string_pretty(value) {
            println!("{}", json);
        }
    }

    pub fn print_node(&self, node: &Node) {
        match self {
            Self::Json => self.print_json(node),
            Self::Text => {
                println!("Node:");
                println!("  ID:        {}", node.id);
                println!("  Type:      {:?}", node.node_type);
                println!("  Content:   {}", node.content());
                println!("  Created:   {}", node.meta.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!("  Access:    {:.2}", node.meta.access_score);
            }
        }
    }

    pub fn print_nodes(&self, nodes: &[Node]) {
        match self {
            Self::Json => self.print_json(nodes),
            Self::Text => {
                println!("Nodes ({}):", nodes.len());
                for node in nodes {
                    println!("  {} | {:?} | {}",
                        node.id,
                        node.node_type,
                        truncate(node.content(), 40)
                    );
                }
            }
        }
    }

    pub fn print_edge(&self, edge: &Edge) {
        match self {
            Self::Json => self.print_json(edge),
            Self::Text => {
                println!("Edge:");
                println!("  Source:   {}", edge.source);
                println!("  Target:   {}", edge.target);
                println!("  Relation: {}", edge.relation);
                println!("  Weight:   {}", edge.weight);
            }
        }
    }

    pub fn print_stats(&self, stats: &crate::client::StatsResponse, detailed: bool) {
        match self {
            Self::Json => self.print_json(stats),
            Self::Text => {
                println!("Database Statistics:");
                println!("  Nodes:           {}", stats.node_count);
                println!("  Edges:           {}", stats.edge_count);
                println!("  Embedded:        {}", stats.embedded_count);

                if detailed {
                    println!("\nMemory Statistics:");
                    // These would be in memory_stats field
                    println!("  (Detailed stats not yet available via REST API)");
                }
            }
        }
    }

    pub fn print_query_response(&self, response: &QueryResponse) {
        match self {
            Self::Json => self.print_json(response),
            Self::Text => {
                println!("Query Results ({} nodes, {}ms):",
                    response.total_count,
                    response.execution_time_ms
                );

                for (i, node) in response.nodes.iter().enumerate() {
                    let id = node["id"].as_str().unwrap_or("");
                    let content = node["content"].as_str().unwrap_or("");
                    let node_type = node["node_type"].as_str().unwrap_or("");
                    println!("  {}. {} | {} | {}",
                        i + 1,
                        id,
                        node_type,
                        truncate(content, 50)
                    );
                }

                if response.truncated {
                    println!("  (results truncated)");
                }
            }
        }
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
