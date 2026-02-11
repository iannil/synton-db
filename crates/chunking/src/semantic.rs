// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Semantic-aware chunking strategy.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    chunk::{Chunk, ChunkMetadata, ChunkType},
    error::{ChunkingError, Result},
    strategy::{ChunkingConfig, ChunkingStrategy},
};

/// Configuration for semantic chunking.
#[derive(Debug, Clone)]
pub struct SemanticChunkConfig {
    /// Base chunking configuration.
    pub base: ChunkingConfig,

    /// Maximum chunk size in characters.
    pub max_chunk_size: usize,

    /// Minimum chunk size in characters.
    pub min_chunk_size: usize,

    /// Similarity threshold for boundary detection (0.0-1.0).
    /// Lower values create more chunks at semantic boundaries.
    pub boundary_threshold: f32,

    /// Number of sentences to consider when calculating similarity.
    pub window_size: usize,
}

impl Default for SemanticChunkConfig {
    fn default() -> Self {
        Self {
            base: ChunkingConfig::default(),
            max_chunk_size: 1000,
            min_chunk_size: 100,
            boundary_threshold: 0.3,
            window_size: 2,
        }
    }
}

impl SemanticChunkConfig {
    /// Create a new semantic chunk config.
    pub fn new(max_chunk_size: usize, boundary_threshold: f32) -> Self {
        Self {
            max_chunk_size,
            min_chunk_size: max_chunk_size / 10,
            boundary_threshold,
            window_size: 2,
            base: ChunkingConfig {
                max_chunk_size,
                min_chunk_size: max_chunk_size / 10,
                semantic_threshold: boundary_threshold,
                ..Default::default()
            },
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.max_chunk_size == 0 {
            return Err(ChunkingError::InvalidChunkSize(
                "max_chunk_size must be greater than 0".to_string(),
            ));
        }
        if self.min_chunk_size > self.max_chunk_size {
            return Err(ChunkingError::InvalidChunkSize(
                "min_chunk_size cannot be greater than max_chunk_size".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.boundary_threshold) {
            return Err(ChunkingError::ConfigError(
                "boundary_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Semantic chunking strategy.
///
/// Splits text at semantic boundaries by analyzing embedding similarity
/// between adjacent sentences. Creates chunks where content is semantically
/// coherent.
#[derive(Debug, Clone)]
pub struct SemanticChunker {
    config: SemanticChunkConfig,
    // Note: In production, this would use an embedding backend
    // For now, we use a simple heuristic approach
}

impl SemanticChunker {
    /// Create a new semantic chunker with default configuration.
    pub fn new() -> Self {
        Self {
            config: SemanticChunkConfig::default(),
        }
    }

    /// Create a new semantic chunker with custom configuration.
    pub fn with_config(config: SemanticChunkConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Calculate semantic similarity between two text segments.
    ///
    /// This is a simplified version that uses word overlap as a proxy.
    /// In production, this would use actual embeddings.
    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        // Simple word overlap similarity as a placeholder
        let words1: std::collections::HashSet<&str> =
            text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> =
            text2.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Find semantic boundaries in the text.
    fn find_boundaries(&self, sentences: &[String]) -> Vec<usize> {
        if sentences.len() <= 1 {
            return Vec::new();
        }

        let mut boundaries = Vec::new();
        let threshold = self.config.boundary_threshold;

        // Calculate similarity between adjacent sentence windows
        for i in 1..sentences.len() {
            let window_before = i.saturating_sub(self.config.window_size)..i;
            let window_after = i..(i + self.config.window_size).min(sentences.len());

            let text_before: String = window_before
                .map(|j| sentences[j].as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let text_after: String = window_after
                .map(|j| sentences[j].as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let similarity = self.calculate_similarity(&text_before, &text_after);

            // Low similarity indicates a semantic boundary
            if similarity < threshold {
                boundaries.push(i);
            }
        }

        boundaries
    }

    /// Group sentences into chunks based on boundaries and size constraints.
    fn group_into_chunks(
        &self,
        sentences: &[String],
        boundaries: &[usize],
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut current_start = 0;
        let mut current_size = 0;
        let mut chunk_index = 0;
        let mut boundary_iter = boundaries.iter().peekable();

        for (i, sentence) in sentences.iter().enumerate() {
            let sentence_len = sentence.len();

            // Check if we should start a new chunk
            let should_split = if current_size > 0 {
                // Check size constraints
                let exceeds_max = current_size + sentence_len > self.config.max_chunk_size;
                let at_boundary = boundary_iter.peek().map(|&&b| b == i).unwrap_or(false);
                let meets_min = current_size >= self.config.min_chunk_size;

                exceeds_max || (at_boundary && meets_min)
            } else {
                false
            };

            if should_split && current_size >= self.config.min_chunk_size {
                // Create chunk from current_start to i
                let chunk_text: String = sentences[current_start..i].join(" ");
                let byte_start = sentences[..current_start]
                    .iter()
                    .map(|s| s.len() + 1) // +1 for space
                    .sum::<usize>();
                let byte_end = sentences[..i]
                    .iter()
                    .map(|s| s.len() + 1)
                    .sum::<usize>();

                chunks.push(Chunk::with_score(
                    chunk_text,
                    chunk_index,
                    (byte_start, byte_end),
                    ChunkType::Paragraph,
                    0.8, // Default boundary score
                ));

                chunk_index += 1;
                current_start = i;
                current_size = 0;
            }

            current_size += sentence_len;
        }

        // Add final chunk
        if current_start < sentences.len() {
            let chunk_text: String = sentences[current_start..].join(" ");
            let byte_start = if current_start > 0 {
                sentences[..current_start]
                    .iter()
                    .map(|s| s.len() + 1)
                    .sum::<usize>()
            } else {
                0
            };
            let byte_end = sentences
                .iter()
                .map(|s| s.len() + 1)
                .sum::<usize>()
                .saturating_sub(1);

            chunks.push(Chunk::with_score(
                chunk_text,
                chunk_index,
                (byte_start, byte_end),
                ChunkType::Paragraph,
                0.8,
            ));
        }

        Ok(chunks)
    }
}

impl Default for SemanticChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChunkingStrategy for SemanticChunker {
    async fn chunk(&self, text: &str, _metadata: ChunkMetadata) -> Result<Vec<Chunk>> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ChunkingError::EmptyInput);
        }

        if text.len() < self.config.min_chunk_size {
            return Err(ChunkingError::InputTooShort {
                min: self.config.min_chunk_size,
                actual: text.len(),
            });
        }

        // Split into sentences
        let sentences: Vec<String> = crate::strategy::split_into_sentences(text)
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return Err(ChunkingError::BoundaryDetectionFailed(
                "No sentences found in text".to_string(),
            ));
        }

        // Find semantic boundaries
        let boundaries = self.find_boundaries(&sentences);

        // Group sentences into chunks
        let chunks = self.group_into_chunks(&sentences, &boundaries)?;

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "semantic"
    }

    fn config(&self) -> &ChunkingConfig {
        &self.config.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_chunking() {
        let chunker = SemanticChunker::with_config(
            SemanticChunkConfig::new(200, 0.5)
        ).unwrap();
        let text = "This is about cats. Cats are furry animals. \
                     Now let's talk about dogs. Dogs are loyal pets. \
                     Finally, birds can fly.";
        let chunks = chunker.chunk(text, ChunkMetadata::default()).await.unwrap();

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
        }
    }

    #[tokio::test]
    async fn test_semantic_chunking_empty_input() {
        let chunker = SemanticChunker::new();
        let result = chunker.chunk("", ChunkMetadata::default()).await;
        assert!(matches!(result, Err(ChunkingError::EmptyInput)));
    }

    #[tokio::test]
    async fn test_semantic_chunking_short_input() {
        let chunker = SemanticChunker::with_config(
            SemanticChunkConfig::new(1000, 0.3)
        ).unwrap();
        let result = chunker.chunk("Short.", ChunkMetadata::default()).await;
        assert!(matches!(result, Err(ChunkingError::InputTooShort { .. })));
    }

    #[test]
    fn test_similarity_calculation() {
        let chunker = SemanticChunker::new();

        // Identical texts should have high similarity
        let sim1 = chunker.calculate_similarity("cat dog", "cat dog");
        assert!(sim1 > 0.8);

        // Different texts should have low similarity
        let sim2 = chunker.calculate_similarity("cat dog", "bird fish");
        assert!(sim2 < 0.5);

        // Some overlap should give medium similarity
        let sim3 = chunker.calculate_similarity("cat dog mouse", "cat bird fish");
        assert!(sim3 > 0.0 && sim3 < 1.0);
    }
}
