// commands/llm.rs - LLM status and health check commands with caching

use crate::{AppState, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Health check cache to avoid spamming checks
static HEALTH_CACHE: once_cell::sync::Lazy<Arc<RwLock<HealthCache>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HealthCache::default())));

const CACHE_DURATION: Duration = Duration::from_secs(30);

/// Cached health check result
#[derive(Debug, Clone)]
struct HealthCache {
    last_check: Option<Instant>,
    last_result: Option<LlmStatus>,
}

impl Default for HealthCache {
    fn default() -> Self {
        Self {
            last_check: None,
            last_result: None,
        }
    }
}

impl HealthCache {
    fn is_valid(&self) -> bool {
        if let Some(last_check) = self.last_check {
            last_check.elapsed() < CACHE_DURATION
        } else {
            false
        }
    }

    fn update(&mut self, status: LlmStatus) {
        self.last_check = Some(Instant::now());
        self.last_result = Some(status);
    }

    fn get(&self) -> Option<LlmStatus> {
        if self.is_valid() {
            self.last_result.clone()
        } else {
            None
        }
    }
}

/// Comprehensive LLM status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStatus {
    /// Whether LLM features are available
    pub available: bool,
    /// Provider type: "local-llama", "none", etc.
    pub provider: String,
    /// Currently loaded model name
    pub model: Option<String>,
    /// Model status (loaded, downloading, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_status: Option<String>,
    /// Connection/health error if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether the model is currently loaded in memory
    pub model_loaded: bool,
    /// Feature flag status
    pub feature_enabled: bool,
}

/// Get comprehensive LLM status with caching
///
/// This command checks:
/// - Whether the local-ai feature is enabled
/// - Whether the LLM engine is initialized
/// - Whether a model is loaded
/// - Model health status
///
/// Results are cached for 30 seconds to avoid performance impact.
#[tauri::command]
pub async fn get_llm_status(state: State<'_, AppState>) -> Result<LlmStatus> {
    // Check cache first
    let cache = HEALTH_CACHE.read().await;
    if let Some(cached_status) = cache.get() {
        debug!("Returning cached LLM status");
        return Ok(cached_status);
    }
    drop(cache); // Release read lock before expensive operations

    let status = check_llm_status_internal(&state).await?;

    // Update cache
    let mut cache = HEALTH_CACHE.write().await;
    cache.update(status.clone());

    Ok(status)
}

/// Internal status check without caching
async fn check_llm_status_internal(state: &AppState) -> Result<LlmStatus> {
    #[cfg(not(feature = "local-ai"))]
    {
        debug!("LLM features disabled (local-ai feature not compiled)");
        Ok(LlmStatus {
            available: false,
            provider: "none".to_string(),
            model: None,
            model_status: None,
            error: Some("LLM features not enabled in this build".to_string()),
            model_loaded: false,
            feature_enabled: false,
        })
    }

    #[cfg(feature = "local-ai")]
    {
        let engine_lock = state.llm_engine.read().await;

        match engine_lock.as_ref() {
            Some(engine) => {
                let is_loaded = engine.is_loaded().await;
                let model_info = engine.model_info();

                // Perform health check
                let (available, error) = if is_loaded {
                    // Model is loaded, try a simple health check
                    match engine.health_check().await {
                        Ok(_) => (true, None),
                        Err(e) => {
                            warn!("LLM health check failed: {}", e);
                            (false, Some(format!("Health check failed: {}", e)))
                        }
                    }
                } else {
                    // Model not loaded but engine exists
                    (true, None)
                };

                let current_model = if is_loaded {
                    Some(model_info.name.clone())
                } else {
                    None
                };

                let model_status_str = if is_loaded {
                    Some("loaded".to_string())
                } else {
                    Some("not_loaded".to_string())
                };

                info!(
                    "LLM status check: available={}, loaded={}, model={:?}",
                    available, is_loaded, current_model
                );

                Ok(LlmStatus {
                    available,
                    provider: "local-llama".to_string(),
                    model: current_model,
                    model_status: model_status_str,
                    error,
                    model_loaded: is_loaded,
                    feature_enabled: true,
                })
            }
            None => {
                debug!("LLM engine not initialized");
                Ok(LlmStatus {
                    available: false,
                    provider: "local-llama".to_string(),
                    model: None,
                    model_status: Some("engine_not_initialized".to_string()),
                    error: Some("LLM engine not initialized".to_string()),
                    model_loaded: false,
                    feature_enabled: true,
                })
            }
        }
    }
}

/// Force refresh LLM status (bypasses cache)
///
/// Use this when you need immediate status after a configuration change.
#[tauri::command]
pub async fn refresh_llm_status(state: State<'_, AppState>) -> Result<LlmStatus> {
    info!("Force refreshing LLM status");

    // Clear cache
    let mut cache = HEALTH_CACHE.write().await;
    cache.last_check = None;
    cache.last_result = None;
    drop(cache);

    // Perform fresh check
    let status = check_llm_status_internal(&state).await?;

    // Update cache with new result
    let mut cache = HEALTH_CACHE.write().await;
    cache.update(status.clone());

    Ok(status)
}

/// Check LLM connection health (simple boolean check)
///
/// Returns true if LLM is available and healthy, false otherwise.
/// This is a lighter-weight check than get_llm_status for simple availability queries.
#[tauri::command]
pub async fn check_llm_connection(state: State<'_, AppState>) -> Result<bool> {
    let status = get_llm_status(state).await?;
    Ok(status.available && status.error.is_none())
}

/// Get detailed model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub name: String,
    pub description: String,
    pub size_mb: u64,
    pub status: String,
    pub is_loaded: bool,
    pub supports_streaming: bool,
}

/// Get details about the current model configuration
#[tauri::command]
pub async fn get_model_details(state: State<'_, AppState>) -> Result<ModelDetails> {
    #[cfg(not(feature = "local-ai"))]
    {
        Err(Error::NotFound("LLM features not enabled".into()))
    }

    #[cfg(feature = "local-ai")]
    {
        let engine_lock = state.llm_engine.read().await;
        let engine = engine_lock
            .as_ref()
            .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

        let model_info = engine.model_info();
        let is_loaded = engine.is_loaded().await;

        Ok(ModelDetails {
            name: model_info.name.clone(),
            description: model_info.description.clone(),
            size_mb: model_info.size_mb,
            status: if is_loaded {
                "loaded".to_string()
            } else {
                "not_loaded".to_string()
            },
            is_loaded,
            supports_streaming: true, // llama.cpp supports streaming
        })
    }
}

/// Clear the health check cache
///
/// Useful for debugging or forcing immediate re-checks
#[tauri::command]
pub async fn clear_llm_cache() -> Result<()> {
    info!("Clearing LLM status cache");
    let mut cache = HEALTH_CACHE.write().await;
    cache.last_check = None;
    cache.last_result = None;
    Ok(())
}
