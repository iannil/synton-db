// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Model loader for local ML models using Candle.
//!
//! Provides utilities for loading embedding models from Hugging Face Hub
//! or local cache.

use std::path::PathBuf;
use thiserror::Error;

use crate::error::{MlError, Result};

#[cfg(feature = "candle")]
use candle::{Device as CandleDevice, DType as CandleDType};


/// Configuration for model loading.
#[derive(Debug, Clone)]
pub struct ModelLoaderConfig {
    /// Cache directory for downloaded models.
    pub cache_dir: PathBuf,

    /// Whether to use offline mode (don't download).
    pub offline: bool,

    /// Device to use for inference.
    pub device: DeviceType,
}

/// Device type for model inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// CPU device.
    Cpu,
    /// CUDA GPU (if available).
    Cuda(usize),
    /// Metal (Apple Silicon, if available).
    Metal,
}

impl Default for ModelLoaderConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("./models"),
            offline: false,
            device: DeviceType::Cpu,
        }
    }
}

/// Loaded model with tokenizer and device.
#[cfg(feature = "candle")]
pub struct LoadedModel {
    /// The model wrapper.
    pub model: ModelWrapper,

    /// The tokenizer.
    pub tokenizer: tokenizers::Tokenizer,

    /// The device for computation.
    pub device: CandleDevice,

    /// Model name for reference.
    pub model_name: String,
}

/// Wrapper for different model types.
#[cfg(feature = "candle")]
pub enum ModelWrapper {
    /// BERT-style model (sentence transformers).
    Bert(candle_transformers::models::bert::BertModel),
}

/// Model loader for local embedding models.
pub struct ModelLoader {
    config: ModelLoaderConfig,
}

impl ModelLoader {
    /// Create a new model loader with default config.
    pub fn new() -> Self {
        Self {
            config: ModelLoaderConfig::default(),
        }
    }

    /// Create a new model loader with custom config.
    pub fn with_config(config: ModelLoaderConfig) -> Self {
        Self { config }
    }

    /// Get the cache directory for a model.
    pub fn model_cache_path(&self, model_name: &str) -> PathBuf {
        // Sanitize model name for filesystem
        let sanitized = model_name.replace('/', "--");
        self.config.cache_dir.join(sanitized)
    }

    /// Check if a model is cached locally.
    pub fn is_model_cached(&self, model_name: &str) -> bool {
        let cache_path = self.model_cache_path(model_name);
        cache_path.exists() && cache_path.join("tokenizer.json").exists()
    }
}

impl Default for ModelLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "candle")]
impl ModelLoader {
    /// Load an embedding model by name.
    ///
    /// This will load from local cache if available, or download from Hugging Face.
    pub async fn load_embedding_model(&self, model_name: &str) -> Result<LoadedModel> {
        use hf_hub::{api::sync::ApiBuilder, Repo};

        let cache_path = self.model_cache_path(model_name);

        // Set up the device
        let device = match self.config.device {
            DeviceType::Cpu => CandleDevice::Cpu,
            DeviceType::Cuda(idx) => CandleDevice::new_cuda(idx).unwrap_or(CandleDevice::Cpu),
            DeviceType::Metal => CandleDevice::new_metal(0).unwrap_or(CandleDevice::Cpu),
        };

        // Build the API for Hugging Face
        let api = ApiBuilder::new()
            .with_cache_dir(cache_path.clone())
            .build()
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to create HF API: {}", e)))?;

        let repo = Repo::model(model_name.to_string());

        // Download/load tokenizer
        let tokenizer_filename = api
            .repo(repo.clone())
            .get("tokenizer.json")
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to get tokenizer: {}", e)))?;

        let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to load tokenizer: {}", e)))?;

        // Load model weights
        let model_file = api
            .repo(repo.clone())
            .get("model.safetensors")
            .or_else(|_| api.repo(repo.clone()).get("pytorch_model.bin"))
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to get model weights: {}", e)))?;

        // Load the model based on type
        let model = self.load_bert_model(&model_file, &device).await?;

        Ok(LoadedModel {
            model: ModelWrapper::Bert(model),
            tokenizer,
            device,
            model_name: model_name.to_string(),
        })
    }

    /// Load a BERT-style model from weights file.
    async fn load_bert_model(
        &self,
        weights_path: &std::path::Path,
        device: &CandleDevice,
    ) -> Result<candle_transformers::models::bert::BertModel> {
        use candle_transformers::models::bert::Config;

        // Load configuration
        let config_file = weights_path
            .parent()
            .map(|p| p.join("config.json"))
            .ok_or_else(|| MlError::ModelLoadFailed("Invalid weights path".to_string()))?;

        let config_content = std::fs::read_to_string(&config_file)
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to read config: {}", e)))?;

        let config: Config = serde_json::from_str(&config_content)
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to parse config: {}", e)))?;

        // Load weights - Candle 0.9.2 API
        // Use safetensors crate directly to load weights
        let vb = if weights_path.ends_with(".safetensors") {
            // Load safetensors file
            let safetensor_content = std::fs::read(weights_path)
                .map_err(|e| MlError::ModelLoadFailed(format!("Failed to read safetensors file: {}", e)))?;

            // Parse and load tensors using safetensors crate
            let tensors = safetensors::SafeTensors::deserialize(&safetensor_content)
                .map_err(|e| MlError::ModelLoadFailed(format!("Failed to deserialize safetensors: {}", e)))?;

            // Convert to HashMap of tensors
            let mut tensor_map = std::collections::HashMap::new();
            for (name, info) in tensors.tensors() {
                let tensor_view = tensors.tensor(&name)
                    .map_err(|e| MlError::ModelLoadFailed(format!("Failed to get tensor: {}", e)))?;
                let data = tensor_view.data();

                // Create tensor from raw bytes
                let shape: Vec<usize> = info.shape().iter().map(|&d| d as usize).collect();
                let tensor = candle::Tensor::from_raw_buffer(
                    data,
                    candle::DType::F32,
                    &shape,
                    device,
                ).map_err(|e| MlError::ModelLoadFailed(format!("Failed to create tensor: {}", e)))?;

                tensor_map.insert(name.clone(), tensor);
            }

            candle_nn::VarBuilder::from_tensors(tensor_map, CandleDType::F32, device)
        } else {
            // Fallback to pytorch format
            candle_nn::VarBuilder::from_pth(weights_path, CandleDType::F32, device)
                .map_err(|e| MlError::ModelLoadFailed(format!("Failed to load pytorch weights: {}", e)))?
        };

        // Create the model - Candle 0.9.2 uses load() instead of new()
        let model = candle_transformers::models::bert::BertModel::load(vb, &config)
            .map_err(|e| MlError::ModelLoadFailed(format!("Failed to create BERT model: {}", e)))?;

        Ok(model)
    }
}

/// Errors specific to model loading.
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid model configuration: {0}")]
    InvalidConfig(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_cache_path() {
        let loader = ModelLoader::new();
        let path = loader.model_cache_path("sentence-transformers/all-MiniLM-L6-v2");
        assert!(path.ends_with("models/sentence-transformers--all-MiniLM-L6-v2"));
    }

    #[test]
    fn test_device_type() {
        assert_eq!(DeviceType::Cpu, DeviceType::Cpu);
    }
}
