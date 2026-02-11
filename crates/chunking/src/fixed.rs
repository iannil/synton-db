// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Fixed-size chunking strategy.

use async_trait::async_trait;

use crate::{
    chunk::{Chunk, ChunkMetadata, ChunkType},
    error::Result,
    strategy::{ChunkingConfig, ChunkingStrategy},
};

/// Configuration for fixed-size chunking.
#[derive(Debug, Clone)]
pub struct FixedChunkConfig {
    /// Base chunking configuration.
    pub base: ChunkingConfig,

    /// Chunk size in characters.
    pub chunk_size: usize,

    /// Overlap between chunks in characters.
    pub overlap_size: usize,
}

impl Default for FixedChunkConfig {
    fn default() -> Self {
        Self {
            base: ChunkingConfig::default(),
            chunk_size: 1000,
            overlap_size: 100,
        }
    }
}

impl FixedChunkConfig {
    /// Create a new fixed chunk config.
    pub fn new(chunk_size: usize, overlap_size: usize) -> Self {
        Self {
            base: ChunkingConfig {
                max_chunk_size: chunk_size,
                min_chunk_size: chunk_size / 10,
                overlap: overlap_size,
                ..Default::default()
            },
            chunk_size,
            overlap_size,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.chunk_size == 0 {
            return Err(crate::ChunkingError::InvalidChunkSize(
                "chunk_size must be greater than 0".to_string(),
            ));
        }
        if self.overlap_size >= self.chunk_size {
            return Err(crate::ChunkingError::InvalidOverlap(
                "overlap_size must be less than chunk_size".to_string(),
            ));
        }
        Ok(())
    }
}

/// Fixed-size chunking strategy.
///
/// Splits text into chunks of approximately equal size with optional overlap.
/// This is the simplest chunking strategy and works well for most use cases.
#[derive(Debug, Clone)]
pub struct FixedChunker {
    config: FixedChunkConfig,
}

impl FixedChunker {
    /// Create a new fixed chunker with default configuration.
    pub fn new() -> Self {
        Self {
            config: FixedChunkConfig::default(),
        }
    }

    /// Create a new fixed chunker with custom configuration.
    pub fn with_config(config: FixedChunkConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Create a new fixed chunker with specified chunk and overlap sizes.
    pub fn new_with_sizes(chunk_size: usize, overlap_size: usize) -> Result<Self> {
        Self::with_config(FixedChunkConfig::new(chunk_size, overlap_size))
    }
}

impl Default for FixedChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChunkingStrategy for FixedChunker {
    async fn chunk(&self, text: &str, _metadata: ChunkMetadata) -> Result<Vec<Chunk>> {
        let text = text.trim();
        if text.is_empty() {
            return Err(crate::ChunkingError::EmptyInput);
        }

        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_len = chars.len();
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.overlap_size;
        let step = chunk_size - overlap;

        let mut start = 0;
        let mut index = 0;

        while start < total_len {
            let end = (start + chunk_size).min(total_len);

            // Try to find a good breaking point (space, newline, punctuation)
            let actual_end = if end < total_len {
                find_breaking_point(&chars, start, end)
            } else {
                end
            };

            let chunk_text: String = chars[start..actual_end].iter().collect();
            chunks.push(Chunk::new(
                chunk_text,
                index,
                (start, actual_end),
                ChunkType::Custom,
            ));

            index += 1;
            start = actual_end;

            // Move forward by step size (accounting for overlap)
            if start + step < total_len {
                start = actual_end - overlap;
            } else if start < total_len {
                // Last chunk might be small
                start = actual_end;
            }
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "fixed"
    }

    fn config(&self) -> &ChunkingConfig {
        &self.config.base
    }
}

/// Find a good breaking point between start and end.
/// Looks for spaces, newlines, or sentence boundaries.
fn find_breaking_point(chars: &[char], start: usize, end: usize) -> usize {
    const SEARCH_WINDOW: usize = 100;

    // First, look for a paragraph break
    let search_start = end.saturating_sub(SEARCH_WINDOW);
    for i in search_start..end {
        if chars[i] == '\n' {
            return i + 1;
        }
    }

    // Then look for a space
    for i in (start + 1)..end {
        if chars[end - i + start] == ' ' {
            return end - i + start;
        }
    }

    // Last resort: break at exact end
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixed_chunking() {
        let chunker = FixedChunker::new_with_sizes(20, 5).unwrap();
        let text = "The quick brown fox jumps over the lazy dog. The dog was sleeping.";
        let chunks = chunker.chunk(text, ChunkMetadata::default()).await.unwrap();

        assert!(!chunks.is_empty());
        // Verify chunks are within size limits
        for chunk in &chunks {
            assert!(chunk.len() <= 25); // Allow some flexibility for word boundaries
        }
    }

    #[tokio::test]
    async fn test_fixed_chunking_empty_input() {
        let chunker = FixedChunker::new();
        let result = chunker.chunk("", ChunkMetadata::default()).await;
        assert!(matches!(result, Err(crate::ChunkingError::EmptyInput)));
    }

    #[tokio::test]
    async fn test_fixed_chunking_single_chunk() {
        let chunker = FixedChunker::new_with_sizes(100, 10).unwrap();
        let text = "Short text.";
        let chunks = chunker.chunk(text, ChunkMetadata::default()).await.unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Short text.");
    }

    #[test]
    fn test_config_validation() {
        let valid = FixedChunkConfig::new(100, 10);
        assert!(valid.validate().is_ok());

        let invalid = FixedChunkConfig::new(0, 10);
        assert!(invalid.validate().is_err());

        let invalid_overlap = FixedChunkConfig::new(100, 100);
        assert!(invalid_overlap.validate().is_err());
    }
}
