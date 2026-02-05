// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Configuration management for SYNTON-DB server.
//!
//! Loads configuration from TOML files with environment variable override support.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ML / Embedding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MlConfig {
    /// Enable ML features.
    pub enabled: bool,

    /// Backend type: local, openai, ollama.
    pub backend: String,

    /// Local model configuration.
    pub local_model: String,

    /// Local model device: cpu, cuda, metal.
    pub device: String,

    /// Maximum sequence length for local model.
    pub max_length: usize,

    /// API endpoint for OpenAI/Ollama.
    pub api_endpoint: String,

    /// API key (for OpenAI).
    pub api_key: Option<String>,

    /// Model name for API backends.
    pub api_model: String,

    /// Request timeout in seconds.
    pub timeout_secs: u64,

    /// Enable embedding cache.
    pub cache_enabled: bool,

    /// Cache size.
    pub cache_size: usize,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "local".to_string(),
            local_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            device: "cpu".to_string(),
            max_length: 512,
            api_endpoint: "https://api.openai.com/v1".to_string(),
            api_key: None,
            api_model: "text-embedding-3-small".to_string(),
            timeout_secs: 30,
            cache_enabled: true,
            cache_size: 10000,
        }
    }
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Host address to bind to.
    pub host: String,

    /// gRPC server port.
    pub grpc_port: u16,

    /// REST API server port.
    pub rest_port: u16,

    /// Enable gRPC server.
    pub grpc_enabled: bool,

    /// Enable REST API server.
    pub rest_enabled: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            grpc_port: 50051,
            rest_port: 8080,
            grpc_enabled: true,
            rest_enabled: true,
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Path to RocksDB data directory.
    pub rocksdb_path: PathBuf,

    /// Path to Lance data directory.
    pub lance_path: PathBuf,

    /// Maximum open files for RocksDB.
    pub max_open_files: i32,

    /// Cache size for RocksDB (in MB).
    pub cache_size_mb: usize,

    /// Enable write-ahead log.
    pub wal_enabled: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            rocksdb_path: PathBuf::from("./data/rocksdb"),
            lance_path: PathBuf::from("./data/lance"),
            max_open_files: 5000,
            cache_size_mb: 256,
            wal_enabled: true,
        }
    }
}

/// Memory management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    /// Decay scale for the forgetting curve (days).
    pub decay_scale: f64,

    /// Retention threshold (0.0-1.0). Nodes below this score are candidates for cleanup.
    pub retention_threshold: f32,

    /// Initial access score for new nodes.
    pub initial_access_score: f32,

    /// Access score boost per access.
    pub access_boost: f32,

    /// Enable periodic decay calculation.
    pub periodic_decay_enabled: bool,

    /// Interval for decay calculation (in seconds).
    pub decay_interval_secs: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            decay_scale: 20.0,
            retention_threshold: 0.1,
            initial_access_score: 5.0,
            access_boost: 0.5,
            periodic_decay_enabled: false,
            decay_interval_secs: 3600, // 1 hour
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error.
    pub level: String,

    /// Enable JSON formatted logs.
    pub json_format: bool,

    /// Enable tracing output.
    pub tracing_enabled: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json_format: false,
            tracing_enabled: true,
        }
    }
}

/// Graph-RAG configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphRagConfig {
    /// Maximum depth for graph traversal during retrieval.
    pub max_traversal_depth: usize,

    /// Maximum nodes to return from hybrid search.
    pub max_results: usize,

    /// Weight for vector similarity (0.0-1.0).
    pub vector_weight: f32,

    /// Weight for graph proximity (0.0-1.0).
    pub graph_weight: f32,

    /// Enable confidence scoring.
    pub confidence_scoring: bool,
}

impl Default for GraphRagConfig {
    fn default() -> Self {
        Self {
            max_traversal_depth: 3,
            max_results: 10,
            vector_weight: 0.7,
            graph_weight: 0.3,
            confidence_scoring: true,
        }
    }
}

/// Complete server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Server settings.
    #[serde(rename = "server")]
    pub server: ServerConfig,

    /// Storage settings.
    #[serde(rename = "storage")]
    pub storage: StorageConfig,

    /// Memory management settings.
    #[serde(rename = "memory")]
    pub memory: MemoryConfig,

    /// Logging settings.
    #[serde(rename = "logging")]
    pub logging: LoggingConfig,

    /// Graph-RAG settings.
    #[serde(rename = "graphrag")]
    pub graphrag: GraphRagConfig,

    /// ML / Embedding settings.
    #[serde(rename = "ml")]
    pub ml: MlConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            memory: MemoryConfig::default(),
            logging: LoggingConfig::default(),
            graphrag: GraphRagConfig::default(),
            ml: MlConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&content)?;

        // Apply environment variable overrides
        config.apply_env_overrides();

        Ok(config)
    }

    /// Apply environment variable overrides to configuration.
    ///
    /// Environment variables use the prefix `SYNTON_` and double underscores
    /// for nesting:
    ///
    /// - `SYNTON_SERVER_HOST` overrides server.host
    /// - `SYNTON_SERVER_GRPC_PORT` overrides server.grpc_port
    /// - `SYNTON_STORAGE_ROCKSDB_PATH` overrides storage.rocksdb_path
    fn apply_env_overrides(&mut self) {
        // Server overrides
        if let Ok(host) = std::env::var("SYNTON_SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("SYNTON_SERVER_GRPC_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.server.grpc_port = port_num;
            }
        }
        if let Ok(port) = std::env::var("SYNTON_SERVER_REST_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.server.rest_port = port_num;
            }
        }

        // Storage overrides
        if let Ok(path) = std::env::var("SYNTON_STORAGE_ROCKSDB_PATH") {
            self.storage.rocksdb_path = PathBuf::from(path);
        }
        if let Ok(path) = std::env::var("SYNTON_STORAGE_LANCE_PATH") {
            self.storage.lance_path = PathBuf::from(path);
        }

        // Logging overrides
        if let Ok(level) = std::env::var("SYNTON_LOG_LEVEL") {
            self.logging.level = level;
        }

        // ML overrides
        if let Ok(backend) = std::env::var("SYNTON_ML_BACKEND") {
            self.ml.backend = backend;
        }
        if let Ok(api_key) = std::env::var("SYNTON_ML_API_KEY") {
            self.ml.api_key = Some(api_key);
        }
        if let Ok(api_endpoint) = std::env::var("SYNTON_ML_API_ENDPOINT") {
            self.ml.api_endpoint = api_endpoint;
        }
    }

    /// Validate the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate ports
        if self.server.grpc_port == self.server.rest_port {
            return Err(ConfigError::DuplicatePorts {
                grpc: self.server.grpc_port,
                rest: self.server.rest_port,
            });
        }

        // Validate memory settings
        if !(0.0..=10.0).contains(&self.memory.initial_access_score) {
            return Err(ConfigError::InvalidAccessScore {
                score: self.memory.initial_access_score,
            });
        }

        if !(0.0..=1.0).contains(&self.memory.retention_threshold) {
            return Err(ConfigError::InvalidRetentionThreshold {
                threshold: self.memory.retention_threshold,
            });
        }

        // Validate Graph-RAG weights
        let total_weight = self.graphrag.vector_weight + self.graphrag.graph_weight;
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(ConfigError::InvalidWeights {
                vector: self.graphrag.vector_weight,
                graph: self.graphrag.graph_weight,
            });
        }

        Ok(())
    }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Duplicate port configuration.
    #[error("gRPC and REST ports cannot be the same: both are {grpc}")]
    DuplicatePorts { grpc: u16, rest: u16 },

    /// Invalid access score (must be 0.0-10.0).
    #[error("Invalid access score: {score}. Must be between 0.0 and 10.0")]
    InvalidAccessScore { score: f32 },

    /// Invalid retention threshold (must be 0.0-1.0).
    #[error("Invalid retention threshold: {threshold}. Must be between 0.0 and 1.0")]
    InvalidRetentionThreshold { threshold: f32 },

    /// Invalid weights (must sum to 1.0).
    #[error("Invalid weights: vector={vector}, graph={graph}. Must sum to 1.0")]
    InvalidWeights { vector: f32, graph: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.grpc_port, 50051);
        assert_eq!(config.server.rest_port, 8080);
    }

    #[test]
    fn test_config_validation_duplicate_ports() {
        let mut config = Config::default();
        config.server.grpc_port = 8080;
        config.server.rest_port = 8080;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_invalid_retention_threshold() {
        let mut config = Config::default();
        config.memory.retention_threshold = 1.5;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_invalid_weights() {
        let mut config = Config::default();
        config.graphrag.vector_weight = 0.8;
        config.graphrag.graph_weight = 0.5;

        let result = config.validate();
        assert!(result.is_err());
    }
}
