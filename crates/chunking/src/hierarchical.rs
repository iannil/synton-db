// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Hierarchical chunking strategy with multi-level summaries.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    chunk::{Chunk, ChunkMetadata, ChunkType},
    error::{ChunkingError, Result},
    strategy::{ChunkingConfig, ChunkingStrategy},
    SemanticChunker,
};

/// Configuration for hierarchical chunking.
#[derive(Debug, Clone)]
pub struct HierarchicalChunkConfig {
    /// Base chunking configuration.
    pub base: ChunkingConfig,

    /// Maximum size for document-level chunks.
    pub max_document_size: usize,

    /// Maximum size for paragraph-level chunks.
    pub max_paragraph_size: usize,

    /// Maximum size for sentence-level chunks.
    pub max_sentence_size: usize,

    /// Whether to generate summaries.
    pub generate_summaries: bool,
}

impl Default for HierarchicalChunkConfig {
    fn default() -> Self {
        Self {
            base: ChunkingConfig::default(),
            max_document_size: 5000,
            max_paragraph_size: 1000,
            max_sentence_size: 200,
            generate_summaries: true,
        }
    }
}

impl HierarchicalChunkConfig {
    /// Create a new hierarchical chunk config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum document size.
    pub fn with_document_size(mut self, size: usize) -> Self {
        self.max_document_size = size;
        self
    }

    /// Set the maximum paragraph size.
    pub fn with_paragraph_size(mut self, size: usize) -> Self {
        self.max_paragraph_size = size;
        self
    }

    /// Set the maximum sentence size.
    pub fn with_sentence_size(mut self, size: usize) -> Self {
        self.max_sentence_size = size;
        self
    }

    /// Enable or disable summary generation.
    pub fn with_summaries(mut self, enable: bool) -> Self {
        self.generate_summaries = enable;
        self
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.max_document_size <= self.max_paragraph_size {
            return Err(ChunkingError::InvalidChunkSize(
                "document size must be greater than paragraph size".to_string(),
            ));
        }
        if self.max_paragraph_size <= self.max_sentence_size {
            return Err(ChunkingError::InvalidChunkSize(
                "paragraph size must be greater than sentence size".to_string(),
            ));
        }
        Ok(())
    }
}

/// Hierarchical chunking result with parent-child relationships.
#[derive(Debug, Clone)]
pub struct HierarchicalChunks {
    /// Document-level chunks (summaries).
    pub documents: Vec<Chunk>,

    /// Paragraph-level chunks.
    pub paragraphs: Vec<Chunk>,

    /// Sentence-level chunks.
    pub sentences: Vec<Chunk>,

    /// Mapping of child IDs to parent IDs.
    pub parent_map: HashMap<uuid::Uuid, uuid::Uuid>,
}

impl HierarchicalChunks {
    /// Get all chunks flattened in hierarchy order.
    pub fn all_chunks(&self) -> Vec<&Chunk> {
        let mut chunks = Vec::new();
        chunks.extend(self.documents.iter());
        chunks.extend(self.paragraphs.iter());
        chunks.extend(self.sentences.iter());
        chunks
    }

    /// Get chunks by level.
    pub fn chunks_by_level(&self, level: usize) -> Vec<&Chunk> {
        self.all_chunks()
            .into_iter()
            .filter(|c| c.level == level)
            .collect()
    }

    /// Get children of a parent chunk.
    pub fn get_children(&self, parent_id: &uuid::Uuid) -> Vec<&Chunk> {
        self.all_chunks()
            .into_iter()
            .filter(|c| c.parent_id.as_ref() == Some(parent_id))
            .collect()
    }
}

/// Hierarchical chunking strategy.
///
/// Creates a multi-level chunk structure:
/// - Level 0: Document summaries
/// - Level 1: Paragraph chunks
/// - Level 2: Sentence chunks
///
/// Each level is connected via parent-child relationships.
#[derive(Debug, Clone)]
pub struct HierarchicalChunker {
    config: HierarchicalChunkConfig,
    semantic_chunker: SemanticChunker,
}

impl HierarchicalChunker {
    /// Create a new hierarchical chunker with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(HierarchicalChunkConfig::default())
    }

    /// Create a new hierarchical chunker with custom configuration.
    pub fn with_config(config: HierarchicalChunkConfig) -> Result<Self> {
        config.validate()?;

        // Create semantic chunker with paragraph-level config
        let semantic_config = crate::semantic::SemanticChunkConfig::new(
            config.max_paragraph_size,
            0.3,
        );
        let semantic_chunker = SemanticChunker::with_config(semantic_config)?;

        Ok(Self {
            config,
            semantic_chunker,
        })
    }

    /// Create document-level summary chunk.
    async fn create_document_chunk(
        &self,
        text: &str,
        paragraphs: &[Chunk],
    ) -> Result<Chunk> {
        // For now, use a simple prefix as the "summary"
        // In production, this would use an LLM to generate actual summaries
        let summary_len = self.config.max_document_size.min(500);
        let summary = if text.len() > summary_len {
            let prefix = &text[..summary_len];
            format!("{}...", prefix.split_whitespace().last().unwrap_or(prefix))
        } else {
            text.to_string()
        };

        let summary_len = summary.len();
        let mut chunk = Chunk::new(
            summary,
            0,
            (0, summary_len),
            ChunkType::Document,
        )
        .with_level(0);

        // Link all paragraphs as children
        for paragraph in paragraphs {
            chunk.add_child(paragraph.id);
        }

        Ok(chunk)
    }

    /// Create sentence-level chunks from a paragraph.
    fn create_sentence_chunks(
        &self,
        paragraph: &Chunk,
    ) -> Result<Vec<Chunk>> {
        let sentences = crate::strategy::split_into_sentences(&paragraph.content);
        let mut chunks = Vec::new();

        for (i, sentence) in sentences.iter().enumerate() {
            if sentence.len() > self.config.max_sentence_size {
                // Split long sentences
                let chunks_needed = (sentence.len() + self.config.max_sentence_size - 1)
                    / self.config.max_sentence_size;
                for j in 0..chunks_needed {
                    let start = j * self.config.max_sentence_size;
                    let end = ((j + 1) * self.config.max_sentence_size).min(sentence.len());
                    let part = &sentence[start..end];
                    chunks.push(Chunk::new(
                        part.to_string(),
                        i,
                        (start, end),
                        ChunkType::Sentence,
                    )
                    .with_parent(paragraph.id)
                    .with_level(2));
                }
            } else {
                chunks.push(
                    Chunk::new(
                        sentence.clone(),
                        i,
                        (0, sentence.len()),
                        ChunkType::Sentence,
                    )
                    .with_parent(paragraph.id)
                    .with_level(2),
                );
            }
        }

        Ok(chunks)
    }

    /// Build parent-child mapping.
    fn build_parent_map(chunks: &[Chunk]) -> HashMap<uuid::Uuid, uuid::Uuid> {
        let mut map = HashMap::new();
        for chunk in chunks {
            if let Some(parent_id) = chunk.parent_id {
                map.insert(chunk.id, parent_id);
            }
        }
        map
    }
}

impl Default for HierarchicalChunker {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[async_trait]
impl ChunkingStrategy for HierarchicalChunker {
    async fn chunk(&self, text: &str, _metadata: ChunkMetadata) -> Result<Vec<Chunk>> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ChunkingError::EmptyInput);
        }

        // Step 1: Create paragraph-level chunks using semantic chunking
        let paragraphs = self
            .semantic_chunker
            .chunk(text, ChunkMetadata::default())
            .await?
            .into_iter()
            .map(|mut p| {
                p.level = 1;
                p
            })
            .collect::<Vec<_>>();

        if paragraphs.is_empty() {
            return Err(ChunkingError::BoundaryDetectionFailed(
                "No paragraphs found".to_string(),
            ));
        }

        // Step 2: Create sentence-level chunks from each paragraph
        let mut sentences = Vec::new();
        for paragraph in &paragraphs {
            let paragraph_sentences = self.create_sentence_chunks(paragraph)?;
            sentences.extend(paragraph_sentences);
        }

        // Step 3: Create document-level summary
        let document = self.create_document_chunk(text, &paragraphs).await?;

        // Step 4: Build parent-child relationships
        let mut all_chunks = Vec::new();
        all_chunks.push(document);
        all_chunks.extend(paragraphs);
        all_chunks.extend(sentences);

        Ok(all_chunks)
    }

    fn name(&self) -> &str {
        "hierarchical"
    }

    fn config(&self) -> &ChunkingConfig {
        &self.config.base
    }
}

/// Extension trait for creating hierarchical chunks.
pub trait HierarchicalChunking {
    /// Create a hierarchical chunk structure.
    async fn chunk_hierarchical(&self, text: &str, metadata: ChunkMetadata)
        -> Result<HierarchicalChunks>;
}

impl HierarchicalChunking for HierarchicalChunker {
    async fn chunk_hierarchical(
        &self,
        text: &str,
        metadata: ChunkMetadata,
    ) -> Result<HierarchicalChunks> {
        let all_chunks = self.chunk(text, metadata).await?;

        let mut documents = Vec::new();
        let mut paragraphs = Vec::new();
        let mut sentences = Vec::new();

        for chunk in all_chunks {
            match chunk.chunk_type {
                ChunkType::Document => documents.push(chunk),
                ChunkType::Paragraph => paragraphs.push(chunk),
                ChunkType::Sentence => sentences.push(chunk),
                _ => sentences.push(chunk),
            }
        }

        let parent_map = HierarchicalChunker::build_parent_map(
            &documents.iter().chain(&paragraphs).chain(&sentences).cloned().collect::<Vec<_>>(),
        );

        Ok(HierarchicalChunks {
            documents,
            paragraphs,
            sentences,
            parent_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hierarchical_chunking() {
        let chunker = HierarchicalChunker::new().unwrap();
        let text = "This is the first paragraph. It has multiple sentences. \
                     This is the second paragraph. It also has sentences. \
                     And a third paragraph for good measure.";

        let chunks = chunker.chunk(text, ChunkMetadata::default()).await.unwrap();

        // Should have document, paragraphs, and sentences
        assert!(!chunks.is_empty());

        // Check hierarchy levels
        let has_document = chunks.iter().any(|c| c.level == 0);
        let has_paragraph = chunks.iter().any(|c| c.level == 1);
        let has_sentence = chunks.iter().any(|c| c.level == 2);

        assert!(has_document, "Should have document-level chunk");
        assert!(has_paragraph, "Should have paragraph-level chunks");
        assert!(has_sentence, "Should have sentence-level chunks");
    }

    #[tokio::test]
    async fn test_hierarchical_chunking_structure() {
        let chunker = HierarchicalChunker::new().unwrap();
        let text = "This is the first paragraph with multiple sentences to make it long enough. \
                     It has several sentences to ensure proper chunking behavior. \
                     This is the second paragraph which also needs to be sufficiently long. \
                     It contains additional sentences for testing the hierarchical structure. \
                     Here is a third paragraph to add more content to the test. \
                     This ensures we have enough text for proper semantic chunking.";

        let result = chunker.chunk_hierarchical(text, ChunkMetadata::default()).await.unwrap();

        // Should have one document
        assert_eq!(result.documents.len(), 1);

        // Should have paragraphs
        assert!(!result.paragraphs.is_empty());

        // Should have sentences
        assert!(!result.sentences.is_empty());

        // Check parent relationships
        for sentence in &result.sentences {
            assert!(sentence.parent_id.is_some());
        }
    }

    #[tokio::test]
    async fn test_hierarchical_chunking_empty_input() {
        let chunker = HierarchicalChunker::new().unwrap();
        let result = chunker.chunk("", ChunkMetadata::default()).await;
        assert!(matches!(result, Err(ChunkingError::EmptyInput)));
    }

    #[test]
    fn test_config_validation() {
        let valid = HierarchicalChunkConfig::new();
        assert!(valid.validate().is_ok());

        let invalid = HierarchicalChunkConfig {
            max_document_size: 100,
            max_paragraph_size: 200, // Larger than document
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_get_children() {
        let parent_id = uuid::Uuid::new_v4();
        let child1_id = uuid::Uuid::new_v4();
        let child2_id = uuid::Uuid::new_v4();

        let mut parent = Chunk::new("parent".to_string(), 0, (0, 6), ChunkType::Document);
        parent.id = parent_id;
        parent.add_child(child1_id);
        parent.add_child(child2_id);

        let mut child1 = Chunk::new("child1".to_string(), 0, (0, 6), ChunkType::Sentence)
            .with_parent(parent_id);
        child1.id = child1_id;

        let mut child2 = Chunk::new("child2".to_string(), 0, (0, 6), ChunkType::Sentence)
            .with_parent(parent_id);
        child2.id = child2_id;

        let all_chunks = vec![parent.clone(), child1, child2];
        let parent_map = HierarchicalChunker::build_parent_map(&all_chunks);

        assert_eq!(parent_map.get(&child1_id), Some(&parent_id));
        assert_eq!(parent_map.get(&child2_id), Some(&parent_id));
    }
}
