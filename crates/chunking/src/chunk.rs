// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Chunk data structures for adaptive document splitting.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A chunk of text from a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier for this chunk.
    pub id: Uuid,

    /// The text content of this chunk.
    pub content: String,

    /// Index of this chunk in the sequence.
    pub index: usize,

    /// Byte range in the original text (start, end).
    pub range: (usize, usize),

    /// Boundary score indicating how good of a split point this is.
    /// Higher values indicate better semantic boundaries.
    pub boundary_score: f32,

    /// Type of chunk based on its content structure.
    pub chunk_type: ChunkType,

    /// Parent chunk ID (for hierarchical chunks).
    pub parent_id: Option<Uuid>,

    /// Child chunk IDs (for hierarchical chunks).
    pub child_ids: Vec<Uuid>,

    /// Level in the hierarchy (0 = document, 1 = paragraph, 2 = sentence).
    pub level: usize,
}

impl Chunk {
    /// Create a new chunk.
    pub fn new(
        content: String,
        index: usize,
        range: (usize, usize),
        chunk_type: ChunkType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            index,
            range,
            boundary_score: 0.0,
            chunk_type,
            parent_id: None,
            child_ids: Vec::new(),
            level: 0,
        }
    }

    /// Create a chunk with a boundary score.
    pub fn with_score(
        content: String,
        index: usize,
        range: (usize, usize),
        chunk_type: ChunkType,
        boundary_score: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            index,
            range,
            boundary_score,
            chunk_type,
            parent_id: None,
            child_ids: Vec::new(),
            level: 0,
        }
    }

    /// Set the parent ID for hierarchical storage.
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self.level = 1; // Default to level 1 when has a parent
        self
    }

    /// Set the hierarchy level.
    pub fn with_level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    /// Add a child chunk ID.
    pub fn add_child(&mut self, child_id: Uuid) {
        self.child_ids.push(child_id);
    }

    /// Get the character length of this chunk.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if this chunk is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Type of chunk based on content structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    /// Document-level chunk (highest level summary).
    Document,

    /// Paragraph-level chunk.
    Paragraph,

    /// Sentence-level chunk.
    Sentence,

    /// Custom chunk type.
    Custom,
}

impl ChunkType {
    /// Get the hierarchy level for this chunk type.
    pub fn level(&self) -> usize {
        match self {
            Self::Document => 0,
            Self::Paragraph => 1,
            Self::Sentence => 2,
            Self::Custom => 3,
        }
    }

    /// Get the name of this chunk type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Paragraph => "paragraph",
            Self::Sentence => "sentence",
            Self::Custom => "custom",
        }
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

use std::fmt;

/// Metadata for chunking operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Source identifier (e.g., file path, URL).
    pub source: Option<String>,

    /// Document title.
    pub title: Option<String>,

    /// MIME type of the source content.
    pub content_type: Option<String>,

    /// Additional custom metadata.
    pub custom: serde_json::Value,
}

impl ChunkMetadata {
    /// Create new chunk metadata.
    pub fn new() -> Self {
        Self {
            source: None,
            title: None,
            content_type: None,
            custom: serde_json::Value::Null,
        }
    }

    /// Set the source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the content type.
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

impl Default for ChunkMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new(
            "Hello world".to_string(),
            0,
            (0, 11),
            ChunkType::Sentence,
        );

        assert_eq!(chunk.content, "Hello world");
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.range, (0, 11));
        assert_eq!(chunk.chunk_type, ChunkType::Sentence);
        assert!(!chunk.is_empty());
        assert_eq!(chunk.len(), 11);
    }

    #[test]
    fn test_chunk_with_parent() {
        let parent_id = Uuid::new_v4();
        let chunk = Chunk::new("test".to_string(), 0, (0, 4), ChunkType::Sentence)
            .with_parent(parent_id);

        assert_eq!(chunk.parent_id, Some(parent_id));
        assert_eq!(chunk.level, 1);
    }

    #[test]
    fn test_chunk_type_levels() {
        assert_eq!(ChunkType::Document.level(), 0);
        assert_eq!(ChunkType::Paragraph.level(), 1);
        assert_eq!(ChunkType::Sentence.level(), 2);
        assert_eq!(ChunkType::Custom.level(), 3);
    }
}
