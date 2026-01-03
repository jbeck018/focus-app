// ai/model_manager.rs - Model downloading, caching, and lifecycle management

#![cfg(feature = "local-ai")]

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn, debug};

/// Supported model configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub url: String,
    pub filename: String,
    pub size_mb: u64,
    pub context_size: u32,
    pub description: String,
}

impl ModelConfig {
    /// Phi-3.5-mini - Recommended default (3.8B params, ~2.2GB)
    /// Using bartowski's public GGUF conversion (no auth required)
    /// Context size set to 16384 for in-depth responses (model supports up to 131072)
    /// KV cache uses ~6GB VRAM at this size - suitable for most modern systems
    pub fn phi_3_5_mini() -> Self {
        Self {
            name: "phi-3.5-mini".to_string(),
            url: "https://huggingface.co/bartowski/Phi-3.5-mini-instruct-GGUF/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf".to_string(),
            filename: "phi-3.5-mini-instruct-q4.gguf".to_string(),
            size_mb: 2400,
            context_size: 16384,
            description: "Microsoft's Phi-3.5-mini (3.8B params) - Best balance of quality and speed".to_string(),
        }
    }

    /// TinyLlama - Smallest option (~600MB)
    /// Context size set to 4096 - reasonable for a smaller model
    pub fn tiny_llama() -> Self {
        Self {
            name: "tinyllama".to_string(),
            url: "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf".to_string(),
            filename: "tinyllama-1.1b-chat-q4.gguf".to_string(),
            size_mb: 600,
            context_size: 4096,
            description: "TinyLlama (1.1B params) - Fastest, smallest, lower quality".to_string(),
        }
    }

    /// Get all available models
    pub fn all_models() -> Vec<Self> {
        vec![Self::phi_3_5_mini(), Self::tiny_llama()]
    }
}

/// Current status of model availability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    NotDownloaded,
    Downloading { progress_percent: u8 },
    Downloaded,
    Loaded,
    Error { message: String },
}

/// Manages model downloading, caching, and validation
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    /// Create a new model manager with specified cache directory
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    /// Initialize models directory
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.models_dir).await?;
        info!("Models directory initialized at: {:?}", self.models_dir);
        Ok(())
    }

    /// Get path where a model would be stored
    pub fn model_path(&self, config: &ModelConfig) -> PathBuf {
        self.models_dir.join(&config.filename)
    }

    /// Check if a model is already downloaded
    pub async fn is_downloaded(&self, config: &ModelConfig) -> bool {
        let path = self.model_path(config);
        path.exists()
    }

    /// Get current status of a model
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn get_status(&self, config: &ModelConfig) -> ModelStatus {
        if self.is_downloaded(config).await {
            ModelStatus::Downloaded
        } else {
            ModelStatus::NotDownloaded
        }
    }

    /// Download a model with progress tracking
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn download_model(
        &self,
        config: &ModelConfig,
        progress_callback: Option<Box<dyn Fn(u8) + Send>>,
    ) -> Result<PathBuf> {
        let model_path = self.model_path(config);

        // Skip if already exists
        if model_path.exists() {
            info!("Model already downloaded: {:?}", model_path);
            return Ok(model_path);
        }

        info!("Downloading model: {} from {}", config.name, config.url);

        // Download to temporary file first
        let temp_path = model_path.with_extension("tmp");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout
            .build()
            .map_err(|e| Error::Network(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(&config.url)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to download model: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut file = fs::File::create(&temp_path).await?;

        // Stream download with progress tracking
        use tokio::io::AsyncWriteExt;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::Network(format!("Download error: {}", e)))?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(ref callback) = progress_callback {
                if total_size > 0 {
                    let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
                    callback(progress);
                }
            }
        }

        file.flush().await?;
        drop(file);

        // Move from temp to final location
        fs::rename(&temp_path, &model_path).await?;

        info!("Model downloaded successfully: {:?}", model_path);
        Ok(model_path)
    }

    /// Delete a downloaded model to free space
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn delete_model(&self, config: &ModelConfig) -> Result<()> {
        let path = self.model_path(config);
        if path.exists() {
            fs::remove_file(&path).await?;
            info!("Deleted model: {:?}", path);
        }
        Ok(())
    }

    /// Get list of all downloaded models
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn list_downloaded(&self) -> Result<Vec<String>> {
        let mut models = Vec::new();
        let mut entries = fs::read_dir(&self.models_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(ext) = entry.path().extension() {
                if ext == "gguf" {
                    if let Some(name) = entry.file_name().to_str() {
                        models.push(name.to_string());
                    }
                }
            }
        }

        Ok(models)
    }

    /// Validate model file integrity (basic size check)
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn validate_model(&self, config: &ModelConfig) -> Result<bool> {
        let path = self.model_path(config);

        if !path.exists() {
            return Ok(false);
        }

        let metadata = fs::metadata(&path).await?;
        let size_mb = metadata.len() / (1024 * 1024);

        // Allow 10% variance in file size
        let expected = config.size_mb;
        let min_size = (expected as f64 * 0.9) as u64;
        let max_size = (expected as f64 * 1.1) as u64;

        if size_mb < min_size || size_mb > max_size {
            warn!(
                "Model file size mismatch: expected ~{}MB, got {}MB",
                expected, size_mb
            );
            return Ok(false);
        }

        debug!("Model validation passed: {:?}", path);
        Ok(true)
    }

    /// Get total size of all downloaded models
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn total_cache_size_mb(&self) -> Result<u64> {
        let mut total = 0u64;
        let mut entries = fs::read_dir(&self.models_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(ext) = entry.path().extension() {
                if ext == "gguf" {
                    let metadata = entry.metadata().await?;
                    total += metadata.len();
                }
            }
        }

        Ok(total / (1024 * 1024))
    }

    /// Clear all downloaded models
    #[allow(dead_code)] // Public API method - used by model management features
    pub async fn clear_cache(&self) -> Result<()> {
        let mut entries = fs::read_dir(&self.models_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(ext) = entry.path().extension() {
                if ext == "gguf" || ext == "tmp" {
                    fs::remove_file(entry.path()).await?;
                }
            }
        }

        info!("Model cache cleared");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_configs() {
        let phi = ModelConfig::phi_3_5_mini();
        assert!(phi.size_mb > 0);
        assert!(phi.context_size >= 2048);

        let tiny = ModelConfig::tiny_llama();
        assert!(tiny.size_mb < phi.size_mb);
    }

    #[test]
    fn test_model_name_resolution() {
        // Test that model names can be resolved correctly for downloads
        let all_models = ModelConfig::all_models();
        let model_names: Vec<&str> = all_models.iter().map(|m| m.name.as_str()).collect();

        // Verify expected model names exist and are lowercase
        assert!(model_names.contains(&"phi-3.5-mini"), "phi-3.5-mini model not found");
        assert!(model_names.contains(&"tinyllama"), "tinyllama model not found");

        // Test lookup by name (simulating download_model_task behavior)
        for name in &["phi-3.5-mini", "tinyllama"] {
            let found = all_models.iter().find(|c| c.name == *name);
            assert!(found.is_some(), "Model {} should be found in config lookup", name);
        }
    }

    #[tokio::test]
    async fn test_model_manager_init() {
        let temp_dir = std::env::temp_dir().join("focusflow_test_models");
        let manager = ModelManager::new(temp_dir.clone());

        manager.init().await.unwrap();
        assert!(temp_dir.exists());

        // Cleanup
        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }
}
