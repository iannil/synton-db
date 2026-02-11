// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! MCP tool definitions for SYNTON-DB.
//!
//! This module defines all available tools that can be called through the MCP protocol.

use serde_json::json;
use uuid::Uuid;

use crate::{
    client::SyntonDbClient, protocol::Tool, CallToolResult, ToolContent, ToolError,
    ToolTextContent,
};
use synton_core::{NodeType, Relation};

/// Get all available MCP tools.
pub fn get_all_tools() -> Vec<Tool> {
    vec![
        absorb_tool(),
        query_tool(),
        hybrid_search_tool(),
        get_node_tool(),
        traverse_tool(),
        add_edge_tool(),
        stats_tool(),
        list_nodes_tool(),
    ]
}

/// Tool: synton_absorb
///
/// Absorb knowledge into the cognitive database.
/// Automatically vectorizes content and creates semantic nodes.
fn absorb_tool() -> Tool {
    Tool {
        name: "synton_absorb".to_string(),
        description: "Absorb knowledge into SYNTON-DB cognitive database. \
                     Automatically creates semantic nodes with optional vectorization. \
                     Use this to store information for later retrieval and reasoning. \
                     Ideal for capturing project context, architectural decisions, \
                     code concepts, and any knowledge worth remembering.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Content to absorb into the database"
                },
                "node_type": {
                    "type": "string",
                    "description": "Type of semantic node",
                    "enum": ["entity", "concept", "fact", "raw_chunk"],
                    "default": "concept"
                }
            },
            "required": ["content"]
        })
    }
}

/// Tool: synton_query
///
/// Query the database using natural language (PaQL).
fn query_tool() -> Tool {
    Tool {
        name: "synton_query".to_string(),
        description: "Execute a natural language query against SYNTON-DB. \
                     Supports semantic search, text matching, and PaQL (Prompt as Query Language). \
                     Ideal for retrieving related code concepts, architectural decisions, \
                     project context, and any previously absorbed knowledge. \
                     Returns nodes ranked by relevance and access frequency.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural language query (PaQL - Prompt as Query Language)"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of results to return",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["query"]
        })
    }
}

/// Tool: synton_hybrid_search
///
/// Hybrid search combining vector similarity and graph traversal (Graph-RAG).
fn hybrid_search_tool() -> Tool {
    Tool {
        name: "synton_hybrid_search".to_string(),
        description: "Perform hybrid Graph-RAG search combining vector similarity \
                     and graph traversal for context-aware retrieval. \
                     Returns semantically related nodes along with their graph neighbors. \
                     Best for discovering related concepts and exploring knowledge connections.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query for semantic search"
                },
                "k": {
                    "type": "number",
                    "description": "Number of results to return",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 50
                }
            },
            "required": ["query"]
        })
    }
}

/// Tool: synton_get_node
///
/// Get a specific node by ID.
fn get_node_tool() -> Tool {
    Tool {
        name: "synton_get_node".to_string(),
        description: "Retrieve a specific node from SYNTON-DB by its UUID. \
                     Returns the full node including content, metadata, and attributes. \
                     Use this when you have a node ID and need its complete information.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Node UUID (e.g., '550e8400-e29b-41d4-a716-446655440000')",
                    "format": "uuid"
                }
            },
            "required": ["id"]
        })
    }
}

/// Tool: synton_traverse
///
/// Traverse the knowledge graph from a starting node.
fn traverse_tool() -> Tool {
    Tool {
        name: "synton_traverse".to_string(),
        description: "Traverse the knowledge graph starting from a specific node. \
                     Explores related concepts through semantic relationships (IS_A, PART_OF, CAUSES, etc.). \
                     Ideal for exploring connections between concepts and understanding context. \
                     Returns both nodes and edges discovered during traversal.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "start_id": {
                    "type": "string",
                    "description": "Starting node UUID",
                    "format": "uuid"
                },
                "max_depth": {
                    "type": "number",
                    "description": "Maximum traversal depth (hops from start)",
                    "default": 2,
                    "minimum": 1,
                    "maximum": 5
                },
                "max_nodes": {
                    "type": "number",
                    "description": "Maximum number of nodes to return",
                    "default": 50,
                    "minimum": 1,
                    "maximum": 500
                }
            },
            "required": ["start_id"]
        })
    }
}

/// Tool: synton_add_edge
///
/// Create a relationship edge between two nodes.
fn add_edge_tool() -> Tool {
    Tool {
        name: "synton_add_edge".to_string(),
        description: "Create a semantic relationship (edge) between two nodes in SYNTON-DB. \
                     Use this to explicitly model relationships like 'A causes B', 'A is part of B', etc. \
                     Relationships enable graph traversal and contextual reasoning.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Source node UUID",
                    "format": "uuid"
                },
                "target": {
                    "type": "string",
                    "description": "Target node UUID",
                    "format": "uuid"
                },
                "relation": {
                    "type": "string",
                    "description": "Type of semantic relationship",
                    "enum": [
                        "is_a",
                        "part_of",
                        "causes",
                        "similar_to",
                        "contradicts",
                        "happened_after",
                        "belongs_to"
                    ],
                    "default": "similar_to"
                },
                "weight": {
                    "type": "number",
                    "description": "Relationship strength (0.0 - 1.0)",
                    "default": 1.0,
                    "minimum": 0.0,
                    "maximum": 1.0
                }
            },
            "required": ["source", "target"]
        })
    }
}

/// Tool: synton_stats
///
/// Get database statistics.
fn stats_tool() -> Tool {
    Tool {
        name: "synton_stats".to_string(),
        description: "Get SYNTON-DB statistics including node count, edge count, \
                     embedded nodes, and memory metrics. Useful for understanding \
                     the current state and growth of the knowledge base.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }
}

/// Tool: synton_list_nodes
///
/// List all nodes in the database.
fn list_nodes_tool() -> Tool {
    Tool {
        name: "synton_list_nodes".to_string(),
        description: "List all nodes currently stored in SYNTON-DB. \
                     Returns all nodes with their content, types, and metadata. \
                     Useful for exploring what knowledge has been absorbed.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }
}

/// Execute a tool call.
pub async fn execute_tool(
    client: &SyntonDbClient,
    name: &str,
    args: serde_json::Value,
) -> CallToolResult {
    match name {
        "synton_absorb" => execute_absorb(client, args).await,
        "synton_query" => execute_query(client, args).await,
        "synton_hybrid_search" => execute_hybrid_search(client, args).await,
        "synton_get_node" => execute_get_node(client, args).await,
        "synton_traverse" => execute_traverse(client, args).await,
        "synton_add_edge" => execute_add_edge(client, args).await,
        "synton_stats" => execute_stats(client).await,
        "synton_list_nodes" => execute_list_nodes(client).await,
        _ => CallToolResult::Error(ToolError::new(format!("Unknown tool: {}", name))),
    }
}

// Tool implementations

async fn execute_absorb(client: &SyntonDbClient, args: serde_json::Value) -> CallToolResult {
    // Parse arguments
    let content = match args.get("content") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new(
                "Missing or invalid 'content' argument",
            ))
        }
    };

    let node_type_str = args
        .get("node_type")
        .and_then(|v| v.as_str())
        .unwrap_or("concept");

    let node_type = match node_type_str {
        "entity" => NodeType::Entity,
        "fact" => NodeType::Fact,
        "raw_chunk" => NodeType::RawChunk,
        _ => NodeType::Concept,
    };

    // Call the API
    match client.add_node(content.clone(), node_type).await {
        Ok(response) => {
            let node = response.node;
            let created = if response.created { "created" } else { "existing" };

            let text = format!(
                "Successfully {} node:\n\
                 - ID: {}\n\
                 - Type: {:?}\n\
                 - Content: {}\n\
                 - Confidence: {:.2}",
                created,
                node.id,
                node.node_type,
                truncate(&node.content, 200),
                node.meta.confidence
            );
            CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Failed to absorb: {}", e))),
    }
}

async fn execute_query(client: &SyntonDbClient, args: serde_json::Value) -> CallToolResult {
    let query = match args.get("query") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new(
                "Missing or invalid 'query' argument",
            ))
        }
    };

    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    match client.query(query.clone(), Some(limit)).await {
        Ok(result) => {
            if result.nodes.is_empty() {
                let text = format!(
                    "Query returned no results.\n\
                     Query: '{}'\n\
                     Execution time: {}ms",
                    query, result.execution_time_ms
                );
                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
            } else {
                let mut output = format!(
                    "Found {} result(s) for '{}':\n\
                     Execution time: {}ms\n\n",
                    result.total_count,
                    query,
                    result.execution_time_ms
                );

                for (i, node) in result.nodes.iter().enumerate().take(limit) {
                    output.push_str(&format!(
                        "{}. [{}] {}\n\
                           ID: {}\n\
                           Score: {:.2}\n\n",
                        i + 1,
                        format!("{:?}", node.node_type),
                        truncate(&node.content, 150),
                        node.id,
                        node.meta.access_score
                    ));
                }

                if result.truncated {
                    output.push_str(&format!(
                        "\n(... results truncated, showing {} of {} total)\n",
                        result.nodes.len(),
                        result.total_count
                    ));
                }

                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(output))])
            }
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Query failed: {}", e))),
    }
}

async fn execute_hybrid_search(
    client: &SyntonDbClient,
    args: serde_json::Value,
) -> CallToolResult {
    let query = match args.get("query") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new(
                "Missing or invalid 'query' argument",
            ))
        }
    };

    let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    match client.hybrid_search(query.clone(), k).await {
        Ok(nodes) => {
            if nodes.is_empty() {
                let text = format!("Hybrid search returned no results for: '{}'", query);
                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
            } else {
                let mut output = format!("Graph-RAG hybrid search results for '{}':\n\n", query);

                for (i, node) in nodes.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. [{}] {}\n\
                           ID: {}\n\
                           Score: {:.2}\n\n",
                        i + 1,
                        format!("{:?}", node.node_type),
                        truncate(&node.content, 150),
                        node.id,
                        node.meta.access_score
                    ));
                }

                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(output))])
            }
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!(
            "Hybrid search failed: {}",
            e
        ))),
    }
}

async fn execute_get_node(client: &SyntonDbClient, args: serde_json::Value) -> CallToolResult {
    let id_str = match args.get("id") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new("Missing or invalid 'id' argument"))
        }
    };

    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return CallToolResult::Error(ToolError::new(format!(
                "Invalid UUID format: '{}'",
                id_str
            )))
        }
    };

    match client.get_node(id).await {
        Ok(Some(node)) => {
            let text = format!(
                "Node found:\n\
                 - ID: {}\n\
                 - Type: {:?}\n\
                 - Content: {}\n\
                 - Created: {}\n\
                 - Updated: {}\n\
                 - Access Score: {:.2}\n\
                 - Confidence: {:.2}\n\
                 - Source: {:?}\n\
                 - Has Embedding: {}\n\
                 - Attributes: {}",
                node.id,
                node.node_type,
                node.content,
                node.meta.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                node.meta.updated_at.format("%Y-%m-%d %H:%M:%S UTC"),
                node.meta.access_score,
                node.meta.confidence,
                node.meta.source,
                node.has_embedding(),
                node.attributes
            );
            CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
        }
        Ok(None) => {
            let text = format!("Node not found: {}", id);
            CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Failed to get node: {}", e))),
    }
}

async fn execute_traverse(client: &SyntonDbClient, args: serde_json::Value) -> CallToolResult {
    let start_id_str = match args.get("start_id") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new(
                "Missing or invalid 'start_id' argument",
            ))
        }
    };

    let start_id = match Uuid::parse_str(&start_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return CallToolResult::Error(ToolError::new(format!(
                "Invalid UUID format: '{}'",
                start_id_str
            )))
        }
    };

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(2) as usize;

    let max_nodes = args
        .get("max_nodes")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;

    match client.traverse(start_id, max_depth, max_nodes).await {
        Ok(result) => {
            if result.nodes.is_empty() {
                let text = format!("Graph traversal returned no nodes from: {}", start_id);
                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
            } else {
                let mut output = format!(
                    "Graph traversal from {} (depth: {}, {} nodes, {} edges):\n\n",
                    start_id, result.depth, result.nodes.len(), result.edges.len()
                );

                // Show nodes
                output.push_str("Nodes:\n");
                for (i, node) in result.nodes.iter().enumerate().take(20) {
                    output.push_str(&format!(
                        "  {}. [{}] {}\n\
                           ID: {}\n\n",
                        i + 1,
                        format!("{:?}", node.node_type),
                        truncate(&node.content, 100),
                        node.id
                    ));
                }

                if result.nodes.len() > 20 {
                    output.push_str(&format!("  ... and {} more nodes\n\n", result.nodes.len() - 20));
                }

                // Show edges
                output.push_str("Edges:\n");
                for (i, edge) in result.edges.iter().enumerate().take(10) {
                    output.push_str(&format!(
                        "  {}. {} --[{:?}]--> {} (weight: {:.2})\n",
                        i + 1,
                        edge.source,
                        edge.relation,
                        edge.target,
                        edge.weight
                    ));
                }

                if result.edges.len() > 10 {
                    output.push_str(&format!("  ... and {} more edges\n", result.edges.len() - 10));
                }

                if result.truncated {
                    output.push_str("\n(Traversal was truncated due to limits)\n");
                }

                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(output))])
            }
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Traversal failed: {}", e))),
    }
}

async fn execute_add_edge(client: &SyntonDbClient, args: serde_json::Value) -> CallToolResult {
    let source_str = match args.get("source") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new(
                "Missing or invalid 'source' argument",
            ))
        }
    };

    let target_str = match args.get("target") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return CallToolResult::Error(ToolError::new(
                "Missing or invalid 'target' argument",
            ))
        }
    };

    let source = match Uuid::parse_str(&source_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return CallToolResult::Error(ToolError::new(format!(
                "Invalid source UUID: '{}'",
                source_str
            )))
        }
    };

    let target = match Uuid::parse_str(&target_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return CallToolResult::Error(ToolError::new(format!(
                "Invalid target UUID: '{}'",
                target_str
            )))
        }
    };

    let relation_str = args
        .get("relation")
        .and_then(|v| v.as_str())
        .unwrap_or("similar_to");

    let relation = match relation_str {
        "is_a" => Relation::IsA,
        "part_of" => Relation::IsPartOf,
        "causes" => Relation::Causes,
        "similar_to" => Relation::SimilarTo,
        "contradicts" => Relation::Contradicts,
        "happened_after" => Relation::HappenedAfter,
        "belongs_to" => Relation::BelongsTo,
        _ => Relation::SimilarTo,
    };

    let weight = args
        .get("weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0) as f32;

    match client.add_edge(source, target, relation, weight).await {
        Ok(edge) => {
            let text = format!(
                "Successfully created edge:\n\
                 - {} --[{:?} (weight: {:.2})]--> {}",
                edge.source, edge.relation, edge.weight, edge.target
            );
            CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Failed to add edge: {}", e))),
    }
}

async fn execute_stats(client: &SyntonDbClient) -> CallToolResult {
    match client.stats().await {
        Ok(stats) => {
            let text = format!(
                "SYNTON-DB Statistics:\n\
                 - Total Nodes: {}\n\
                 - Total Edges: {}\n\
                 - Embedded Nodes: {}\n\
                 - Active Nodes: {}\n\
                 - Decayed Nodes: {}\n\
                 - Average Access Score: {:.2}\n\
                 - Memory Load Factor: {:.2}",
                stats.node_count,
                stats.edge_count,
                stats.embedded_count,
                stats.memory_stats.active_nodes,
                stats.memory_stats.decayed_nodes,
                stats.memory_stats.average_score,
                stats.memory_stats.load_factor
            );
            CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Failed to get stats: {}", e))),
    }
}

async fn execute_list_nodes(client: &SyntonDbClient) -> CallToolResult {
    match client.get_all_nodes().await {
        Ok(nodes) => {
            if nodes.is_empty() {
                let text = "No nodes in database. Use synton_absorb to add knowledge.".to_string();
                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(text))])
            } else {
                let mut output = format!("Database contains {} node(s):\n\n", nodes.len());

                // Group by type
                let mut by_type = std::collections::HashMap::new();
                for node in &nodes {
                    by_type
                        .entry(format!("{:?}", node.node_type))
                        .or_insert_with(Vec::new)
                        .push(node);
                }

                for (type_name, type_nodes) in by_type.iter() {
                    output.push_str(&format!("{} ({}):\n", type_name, type_nodes.len()));
                    for node in type_nodes.iter().take(10) {
                        output.push_str(&format!(
                            "  - {} | {}\n",
                            node.id,
                            truncate(&node.content, 80)
                        ));
                    }
                    if type_nodes.len() > 10 {
                        output.push_str(&format!("  ... and {} more\n", type_nodes.len() - 10));
                    }
                    output.push('\n');
                }

                CallToolResult::Success(vec![ToolContent::Text(ToolTextContent::new(output))])
            }
        }
        Err(e) => CallToolResult::Error(ToolError::new(format!("Failed to list nodes: {}", e))),
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len.saturating_sub(3)).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_tools() {
        let tools = get_all_tools();
        assert_eq!(tools.len(), 8);

        let tool_names: Vec<_> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"synton_absorb".to_string()));
        assert!(tool_names.contains(&"synton_query".to_string()));
        assert!(tool_names.contains(&"synton_hybrid_search".to_string()));
        assert!(tool_names.contains(&"synton_get_node".to_string()));
        assert!(tool_names.contains(&"synton_traverse".to_string()));
        assert!(tool_names.contains(&"synton_add_edge".to_string()));
        assert!(tool_names.contains(&"synton_stats".to_string()));
        assert!(tool_names.contains(&"synton_list_nodes".to_string()));
    }

    #[test]
    fn test_tool_schemas() {
        let tools = get_all_tools();

        // Check that all tools have valid JSON schemas
        for tool in &tools {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.input_schema.is_object());
        }
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("", 10), "");
    }
}
