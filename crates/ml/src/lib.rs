// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! SYNTON-DB ML module.
//!
//! Provides embedding generation capabilities with multiple backend support:
//! - Local models using Candle
//! - OpenAI API
//! - Ollama local API

pub mod error;
pub mod backend;
pub mod config;
pub mod local;
pub mod openai;
pub mod ollama;
pub mod service;

pub use backend::{BackendType, DeviceType, EmbeddingBackend};
pub use config::{ApiConfig, EmbeddingConfig, LocalModelConfig};
pub use error::{MlError, Result as MlResult};
pub use service::{EmbeddingService, EmbeddingStats};
