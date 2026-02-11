// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Structured context formatters for Graph-RAG.
//!
//! Provides multiple output formats for retrieved context to optimize
//! for different LLM use cases.

use crate::retrieval::RetrievedNode;
use serde::{Deserialize, Serialize};

/// Configuration for context formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Include metadata in formatted output.
    pub include_metadata: bool,

    /// Include relation information.
    pub include_relations: bool,

    /// Include scores in output.
    pub include_scores: bool,

    /// Maximum tokens in output (approximate).
    pub max_tokens: usize,

    /// Format style.
    pub style: FormatStyle,

    /// Compression level (0.0 = no compression, 1.0 = maximum).
    pub compression: f32,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_relations: false,
            include_scores: false,
            max_tokens: 4096,
            style: FormatStyle::Structured,
            compression: 0.0,
        }
    }
}

impl FormatConfig {
    /// Create a new format config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the format style.
    pub fn with_style(mut self, style: FormatStyle) -> Self {
        self.style = style;
        self
    }

    /// Enable or disable metadata.
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set maximum tokens.
    pub fn with_max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set compression level.
    pub fn with_compression(mut self, level: f32) -> Self {
        self.compression = level.clamp(0.0, 1.0);
        self
    }
}

/// Format style for context output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatStyle {
    /// Simple flat list format.
    Flat,

    /// Structured with sections and hierarchy.
    Structured,

    /// JSON format for programmatic consumption.
    Json,

    /// Markdown format.
    Markdown,

    /// Compact format (compressed).
    Compact,
}

/// Trait for formatting retrieved context.
pub trait ContextFormatter: Send + Sync {
    /// Format nodes into a context string.
    fn format(&self, nodes: &[RetrievedNode], config: &FormatConfig) -> String;

    /// Get the formatter name.
    fn name(&self) -> &str;
}

/// Simple flat formatter - plain text list.
#[derive(Debug, Clone, Copy)]
pub struct FlatFormatter;

impl FlatFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlatFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextFormatter for FlatFormatter {
    fn format(&self, nodes: &[RetrievedNode], _config: &FormatConfig) -> String {
        nodes
            .iter()
            .filter_map(|n| {
                let content = n.node.content();
                if content.is_empty() {
                    None
                } else {
                    Some(content.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    fn name(&self) -> &str {
        "flat"
    }
}

/// Structured formatter with sections and metadata.
#[derive(Debug, Clone)]
pub struct StructuredFormatter {
    /// Section header for retrieved context.
    pub section_header: String,

    /// Include metadata in output.
    pub include_metadata: bool,

    /// Include relations in output.
    pub include_relations: bool,
}

impl StructuredFormatter {
    pub fn new() -> Self {
        Self {
            section_header: "Retrieved Context".to_string(),
            include_metadata: true,
            include_relations: false,
        }
    }

    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.section_header = header.into();
        self
    }

    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn with_relations(mut self, include: bool) -> Self {
        self.include_relations = include;
        self
    }
}

impl Default for StructuredFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextFormatter for StructuredFormatter {
    fn format(&self, nodes: &[RetrievedNode], config: &FormatConfig) -> String {
        if nodes.is_empty() {
            return format!("=== {} ===\nNo relevant context found.", self.section_header);
        }

        let mut parts = vec![format!("=== {} ===", self.section_header)];

        for (i, node) in nodes.iter().enumerate() {
            parts.push(format!("\n[{}]", i + 1));

            if config.include_scores {
                parts.push(format!("Relevance: {:.2}", node.score));
            }

            if config.include_metadata {
                parts.push(format!("Type: {}", node.node.node_type));
                if node.hop_distance > 0 {
                    parts.push(format!("Distance: {} hops", node.hop_distance));
                }
            }

            parts.push(format!("Content: {}", node.node.content()));
        }

        parts.join("\n")
    }

    fn name(&self) -> &str {
        "structured"
    }
}

/// Markdown formatter.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextFormatter for MarkdownFormatter {
    fn format(&self, nodes: &[RetrievedNode], config: &FormatConfig) -> String {
        if nodes.is_empty() {
            return "# Retrieved Context\n\n*No relevant context found.*".to_string();
        }

        let mut parts = vec!["# Retrieved Context".to_string()];

        for (i, node) in nodes.iter().enumerate() {
            parts.push(format!("\n## {}. {}\n", i + 1, node.node.node_type));

            if config.include_scores {
                parts.push(format!("**Relevance:** {:.2}\n", node.score));
            }

            let content = node.node.content();
            parts.push(content.to_string());
        }

        parts.join("\n")
    }

    fn name(&self) -> &str {
        "markdown"
    }
}

/// JSON formatter for programmatic consumption.
#[derive(Debug, Clone)]
pub struct JsonFormatter {
    /// Pretty print JSON.
    pub pretty: bool,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self { pretty: true }
    }

    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextFormatter for JsonFormatter {
    fn format(&self, nodes: &[RetrievedNode], config: &FormatConfig) -> String {
        #[derive(Debug, Serialize)]
        struct JsonNode<'a> {
            id: String,
            content: &'a str,
            node_type: String,
            score: f32,
            hop_distance: usize,
        }

        let json_nodes: Vec<JsonNode> = nodes
            .iter()
            .map(|n| JsonNode {
                id: n.node.id.to_string(),
                content: n.node.content(),
                node_type: format!("{}", n.node.node_type),
                score: n.score,
                hop_distance: n.hop_distance,
            })
            .collect();

        if self.pretty {
            serde_json::to_string_pretty(&json_nodes).unwrap_or_default()
        } else {
            serde_json::to_string(&json_nodes).unwrap_or_default()
        }
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// Compact formatter - minimal output.
#[derive(Debug, Clone, Copy)]
pub struct CompactFormatter;

impl CompactFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompactFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextFormatter for CompactFormatter {
    fn format(&self, nodes: &[RetrievedNode], _config: &FormatConfig) -> String {
        nodes
            .iter()
            .map(|n| n.node.content())
            .filter(|c| !c.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn name(&self) -> &str {
        "compact"
    }
}

/// Get a formatter by style.
pub fn get_formatter(style: FormatStyle) -> Box<dyn ContextFormatter> {
    match style {
        FormatStyle::Flat => Box::new(FlatFormatter::new()),
        FormatStyle::Structured => Box::new(StructuredFormatter::new()),
        FormatStyle::Json => Box::new(JsonFormatter::new()),
        FormatStyle::Markdown => Box::new(MarkdownFormatter::new()),
        FormatStyle::Compact => Box::new(CompactFormatter::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synton_core::NodeType;

    #[test]
    fn test_flat_formatter() {
        let formatter = FlatFormatter::new();
        let nodes = vec![
            RetrievedNode::new(
                crate::retrieval::test_node("First content"),
                0.9,
                0,
                0.9,
                true,
            ),
            RetrievedNode::new(
                crate::retrieval::test_node("Second content"),
                0.8,
                0,
                0.8,
                true,
            ),
        ];

        let result = formatter.format(&nodes, &FormatConfig::default());
        assert!(result.contains("First content"));
        assert!(result.contains("Second content"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_structured_formatter() {
        let formatter = StructuredFormatter::new().with_metadata(true);
        let nodes = vec![RetrievedNode::new(
            crate::retrieval::test_node("Test content"),
            0.9,
            1,
            0.9,
            false,
        )];

        let config = FormatConfig {
            include_metadata: true,
            include_scores: true,
            ..Default::default()
        };

        let result = formatter.format(&nodes, &config);
        assert!(result.contains("Retrieved Context"));
        assert!(result.contains("Test content"));
        assert!(result.contains("0.90"));
        assert!(result.contains("Distance: 1 hops"));
    }

    #[test]
    fn test_markdown_formatter() {
        let formatter = MarkdownFormatter::new();
        let nodes = vec![RetrievedNode::new(
            crate::retrieval::test_node("Test content"),
            0.9,
            0,
            0.9,
            true,
        )];

        let config = FormatConfig {
            include_scores: true,
            ..Default::default()
        };

        let result = formatter.format(&nodes, &config);
        assert!(result.contains("# Retrieved Context"));
        assert!(result.contains("## "));
        assert!(result.contains("Test content"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let nodes = vec![RetrievedNode::new(
            crate::retrieval::test_node("Test"),
            0.9,
            0,
            0.9,
            true,
        )];

        let result = formatter.format(&nodes, &FormatConfig::default());
        assert!(result.contains("\"content\""));
        assert!(result.contains("\"score\""));
        assert!(result.contains("0.9"));
    }

    #[test]
    fn test_compact_formatter() {
        let formatter = CompactFormatter::new();
        let nodes = vec![
            RetrievedNode::new(
                crate::retrieval::test_node("First"),
                0.9,
                0,
                0.9,
                true,
            ),
            RetrievedNode::new(
                crate::retrieval::test_node("Second"),
                0.8,
                0,
                0.8,
                true,
            ),
        ];

        let result = formatter.format(&nodes, &FormatConfig::default());
        assert_eq!(result, "First Second");
        assert!(!result.contains("\n"));
    }

    #[test]
    fn test_format_config() {
        let config = FormatConfig::new()
            .with_style(FormatStyle::Markdown)
            .with_max_tokens(2048)
            .with_compression(0.5);

        assert_eq!(config.style, FormatStyle::Markdown);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.compression, 0.5);
    }
}
