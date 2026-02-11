// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Core chunking strategy trait and configuration.

use async_trait::async_trait;

use crate::{chunk::ChunkMetadata, error::Result, Chunk, ChunkingError};

/// Configuration for chunking operations.
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    /// Maximum chunk size in characters.
    pub max_chunk_size: usize,

    /// Minimum chunk size in characters.
    pub min_chunk_size: usize,

    /// Overlap between chunks in characters.
    pub overlap: usize,

    /// Whether to respect sentence boundaries.
    pub respect_sentences: bool,

    /// Whether to respect paragraph boundaries.
    pub respect_paragraphs: bool,

    /// Threshold for semantic boundary detection (0.0-1.0).
    /// Lower values mean more aggressive splitting.
    pub semantic_threshold: f32,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 1000,
            min_chunk_size: 100,
            overlap: 100,
            respect_sentences: true,
            respect_paragraphs: true,
            semantic_threshold: 0.3,
        }
    }
}

impl ChunkingConfig {
    /// Create a new chunking config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum chunk size.
    pub fn with_max_chunk_size(mut self, size: usize) -> Self {
        self.max_chunk_size = size;
        self
    }

    /// Set the minimum chunk size.
    pub fn with_min_chunk_size(mut self, size: usize) -> Self {
        self.min_chunk_size = size;
        self
    }

    /// Set the overlap size.
    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.overlap = overlap;
        self
    }

    /// Set the semantic threshold.
    pub fn with_semantic_threshold(mut self, threshold: f32) -> Self {
        self.semantic_threshold = threshold;
        self
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
        if self.overlap >= self.max_chunk_size {
            return Err(ChunkingError::InvalidOverlap(
                "overlap must be less than max_chunk_size".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.semantic_threshold) {
            return Err(ChunkingError::ConfigError(
                "semantic_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Trait for document chunking strategies.
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    /// Split the input text into chunks.
    ///
    /// # Arguments
    ///
    /// * `text` - The input text to chunk
    /// * `metadata` - Optional metadata about the input
    ///
    /// # Returns
    ///
    /// A vector of chunks with their positions and metadata.
    async fn chunk(&self, text: &str, metadata: ChunkMetadata) -> Result<Vec<Chunk>>;

    /// Get the name of this chunking strategy.
    fn name(&self) -> &str;

    /// Get the configuration for this strategy.
    fn config(&self) -> &ChunkingConfig;
}

/// Helper function to split text into sentences.
///
/// This is a simple implementation that uses common sentence delimiters.
/// For production, consider using a more sophisticated NLP library.
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        current.push(c);

        // Check for sentence endings
        if c == '.' || c == '!' || c == '?' {
            // Look ahead to see if there's a space or end of string
            match chars.peek() {
                Some(&' ') | Some(&'\n') | None => {
                    // End of sentence
                    let sentence = current.trim().to_string();
                    if !sentence.is_empty() {
                        sentences.push(sentence);
                    }
                    current = String::new();
                    // Consume the space if present
                    if let Some(&' ') = chars.peek() {
                        chars.next();
                    }
                }
                _ => {}
            }
        }
    }

    // Add remaining text as last sentence
    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        sentences.push(remaining);
    }

    sentences
}

/// Helper function to split text into paragraphs.
pub fn split_into_paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Helper function to create chunks from ranges.
pub fn chunks_from_ranges(text: &str, ranges: &[(usize, usize)], chunk_type: crate::ChunkType) -> Vec<Chunk> {
    ranges
        .iter()
        .enumerate()
        .map(|(i, &(start, end))| {
            let content = text[start..end].trim().to_string();
            Chunk::new(content, i, (start, end), chunk_type)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_sentences() {
        let text = "Hello world. How are you? I'm fine!";
        let sentences = split_into_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
        assert_eq!(sentences[2], "I'm fine!");
    }

    #[test]
    fn test_split_into_paragraphs() {
        let text = "First paragraph.\n\nSecond paragraph.\n\n\nThird paragraph.";
        let paragraphs = split_into_paragraphs(text);
        assert_eq!(paragraphs.len(), 3);
        assert_eq!(paragraphs[0], "First paragraph.");
        assert_eq!(paragraphs[1], "Second paragraph.");
        assert_eq!(paragraphs[2], "Third paragraph.");
    }

    #[test]
    fn test_config_validation() {
        let config = ChunkingConfig::new();
        assert!(config.validate().is_ok());

        let invalid_config = ChunkingConfig {
            max_chunk_size: 0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }
}
