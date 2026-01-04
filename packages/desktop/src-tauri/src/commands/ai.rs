// commands/ai.rs - AI/LLM management commands for frontend

use crate::{AppState, Error, Result};
use serde::{Deserialize, Serialize};
use tauri::{State, Emitter, Manager};
use tracing::info;

#[cfg(feature = "local-ai")]
use crate::ai::{ModelConfig, ModelStatus};

/// Model information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub description: String,
    pub size_mb: u64,
    #[cfg(feature = "local-ai")]
    pub status: ModelStatus,
    #[cfg(not(feature = "local-ai"))]
    pub status: String,
    pub is_active: bool,
}

/// LLM engine status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStatus {
    pub enabled: bool,
    pub model_loaded: bool,
    pub current_model: Option<String>,
}

/// Download progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DownloadProgress {
    pub model_name: String,
    pub progress_percent: u8,
    pub completed: bool,
}

/// Get list of available models
#[tauri::command]
pub async fn get_available_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>> {
    #[cfg(not(feature = "local-ai"))]
    {
        Ok(vec![])
    }

    #[cfg(feature = "local-ai")]
    {
        let state_clone = state.inner().clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let engine_lock = state_clone.llm_engine.read().await;
                let engine = engine_lock
                    .as_ref()
                    .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

                let active_model = engine.model_info().name.clone();
                let mut models = Vec::new();

                for config in ModelConfig::all_models() {
                    let status = if engine.is_loaded().await && config.name == active_model {
                        ModelStatus::Loaded
                    } else {
                        ModelStatus::NotDownloaded
                    };

                    models.push(ModelInfo {
                        name: config.name.clone(),
                        description: config.description.clone(),
                        size_mb: config.size_mb,
                        status,
                        is_active: config.name == active_model,
                    });
                }

                Ok::<Vec<ModelInfo>, Error>(models)
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))?
    }
}

/// Get current LLM engine status (internal use only - tauri command is in llm.rs)
///
/// NOTE: This is a duplicate of llm::get_llm_status. The version in llm.rs is the
/// registered Tauri command with caching. This version is kept for backwards compatibility
/// but should be removed in favor of using llm::get_llm_status directly.
#[allow(dead_code)]
pub async fn get_llm_status(state: State<'_, AppState>) -> Result<LlmStatus> {
    let engine_lock = state.llm_engine.read().await;

    #[cfg(not(feature = "local-ai"))]
    {
        Ok(LlmStatus {
            enabled: false,
            model_loaded: false,
            current_model: None,
        })
    }

    #[cfg(feature = "local-ai")]
    {
        match engine_lock.as_ref() {
            Some(engine) => {
                let is_loaded = engine.is_loaded().await;
                let current_model = if is_loaded {
                    Some(engine.model_info().name.clone())
                } else {
                    None
                };

                Ok(LlmStatus {
                    enabled: true,
                    model_loaded: is_loaded,
                    current_model,
                })
            }
            None => Ok(LlmStatus {
                enabled: false,
                model_loaded: false,
                current_model: None,
            }),
        }
    }
}

/// Load the current model into memory
#[tauri::command]
pub async fn load_model(state: State<'_, AppState>) -> Result<()> {
    #[cfg(not(feature = "local-ai"))]
    {
        Err(Error::NotFound("AI features not enabled in this build".into()))
    }

    #[cfg(feature = "local-ai")]
    {
        let state_clone = state.inner().clone();

        // Use spawn_blocking for CPU-intensive model loading
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let engine_lock = state_clone.llm_engine.read().await;
                let engine = engine_lock
                    .as_ref()
                    .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

                info!("Loading LLM model...");
                engine.load_model().await?;
                info!("Model loaded successfully");

                Ok::<(), Error>(())
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

/// Unload model from memory
#[tauri::command]
pub async fn unload_model(state: State<'_, AppState>) -> Result<()> {
    #[cfg(not(feature = "local-ai"))]
    {
        Ok(())
    }

    #[cfg(feature = "local-ai")]
    {
        let state_clone = state.inner().clone();

        // Use spawn_blocking for model unloading
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let engine_lock = state_clone.llm_engine.read().await;
                let engine = engine_lock
                    .as_ref()
                    .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

                info!("Unloading LLM model...");
                engine.unload_model().await?;
                info!("Model unloaded");

                Ok::<(), Error>(())
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

/// Download a model (blocking operation, use with progress events)
#[tauri::command]
pub async fn download_model(
    _state: State<'_, AppState>,
    model_name: String,
    app: tauri::AppHandle,
) -> Result<String> {
    #[cfg(not(feature = "local-ai"))]
    {
        Err(Error::NotFound("AI features not enabled in this build".into()))
    }

    #[cfg(feature = "local-ai")]
    {
        let state = _state.inner().clone();
        let app_clone = app.clone();
        let model_name_clone = model_name.clone();

        // Spawn a background task for the download
        tokio::spawn(async move {
            if let Err(e) = download_model_task(&state, &model_name_clone, &app_clone).await {
                let error_msg = format!("Model download failed: {}", e);
                tracing::error!("{}", error_msg);
                let _ = app_clone.emit("model-download-error", &serde_json::json!({
                    "error": error_msg
                }));
            }
        });

        Ok(format!("Download started for model: {}", model_name))
    }
}

/// Internal task to handle model download with progress events
#[cfg(feature = "local-ai")]
async fn download_model_task(
    _state: &AppState,
    model_name: &str,
    app: &tauri::AppHandle,
) -> Result<()> {
    use tauri::Emitter;

    // Find the model configuration
    let config = ModelConfig::all_models()
        .into_iter()
        .find(|c| c.name == model_name)
        .ok_or_else(|| Error::NotFound(format!("Model not found: {}", model_name)))?;

    let models_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Config(format!("Failed to get app data dir: {}", e)))?
        .join("models");

    let manager = crate::ai::ModelManager::new(models_dir);
    manager.init().await?;

    // Check if already downloaded
    if manager.is_downloaded(&config).await {
        info!("Model already downloaded: {}", model_name);
        let _ = app.emit("model-download-complete", &serde_json::json!({
            "model_name": model_name
        }));
        return Ok(());
    }

    // Download with progress callback
    let app_clone = app.clone();
    let model_name_clone = model_name.to_string();

    let progress_callback = Box::new(move |percent: u8| {
        let app = app_clone.clone();
        let model_name = model_name_clone.clone();

        let _ = app.emit("model-download-progress", &serde_json::json!({
            "modelName": model_name,
            "percent": percent,
            "downloadedMb": 0,
            "totalMb": 0
        }));
    });

    manager.download_model(&config, Some(progress_callback)).await?;

    // Emit completion event
    let _ = app.emit("model-download-complete", &serde_json::json!({
        "model_name": model_name
    }));

    info!("Model downloaded successfully: {}", model_name);
    Ok(())
}

/// Check if a model is downloaded
#[tauri::command]
pub async fn is_model_downloaded(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<bool> {
    #[cfg(not(feature = "local-ai"))]
    {
        Ok(false)
    }

    #[cfg(feature = "local-ai")]
    {
        let state_clone = state.inner().clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let engine_lock = state_clone.llm_engine.read().await;
                let engine = engine_lock
                    .as_ref()
                    .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

                let is_active = engine.model_info().name == model_name;
                let is_loaded = engine.is_loaded().await;

                Ok::<bool, Error>(is_active && is_loaded)
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))?
    }
}

/// Delete a downloaded model
#[tauri::command]
pub async fn delete_model(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<()> {
    #[cfg(not(feature = "local-ai"))]
    {
        Err(Error::NotFound("AI features not enabled in this build".into()))
    }

    #[cfg(feature = "local-ai")]
    {
        let state_clone = state.inner().clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let engine_lock = state_clone.llm_engine.read().await;
                let engine = engine_lock
                    .as_ref()
                    .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

                if engine.is_loaded().await && engine.model_info().name == model_name {
                    return Err(Error::InvalidInput(
                        "Cannot delete currently loaded model. Unload it first.".into(),
                    ));
                }

                info!("Model deletion not yet implemented");
                Err(Error::System("Model deletion not yet implemented".into()))
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

/// Enable/disable AI coach features
#[tauri::command]
pub async fn toggle_ai_coach(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<()> {
    #[cfg(not(feature = "local-ai"))]
    {
        if enabled {
            Err(Error::NotFound("AI features not enabled in this build".into()))
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "local-ai")]
    {
        use tauri::Manager;

        let state_clone = state.inner().clone();

        // Get models_dir outside spawn_blocking to avoid path() method issues
        let models_dir = if enabled {
            Some(
                state_clone
                    .app_handle
                    .path()
                    .app_data_dir()
                    .map_err(|e| Error::Config(format!("Failed to get app data dir: {}", e)))?
                    .join("models")
            )
        } else {
            None
        };

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut engine_lock = state_clone.llm_engine.write().await;

                if enabled {
                    if engine_lock.is_none() {
                        let models_dir = models_dir.expect("models_dir should be set when enabled is true");
                        let engine = crate::ai::LlmEngine::new(models_dir, ModelConfig::phi_3_5_mini())?;
                        *engine_lock = Some(engine);
                        info!("AI coach enabled");
                    }
                } else {
                    if let Some(engine) = engine_lock.as_ref() {
                        let _ = engine.unload_model().await;
                    }
                    *engine_lock = None;
                    info!("AI coach disabled");
                }

                Ok::<(), Error>(())
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

/// Get total cache size of all models
#[tauri::command]
pub async fn get_models_cache_size(_state: State<'_, AppState>) -> Result<u64> {
    Ok(0)
}

/// Clear all downloaded models
#[tauri::command]
pub async fn clear_models_cache(state: State<'_, AppState>) -> Result<()> {
    #[cfg(not(feature = "local-ai"))]
    {
        Ok(())
    }

    #[cfg(feature = "local-ai")]
    {
        let state_clone = state.inner().clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let engine_lock = state_clone.llm_engine.read().await;
                let engine = engine_lock
                    .as_ref()
                    .ok_or_else(|| Error::NotFound("LLM engine not available".into()))?;

                if engine.is_loaded().await {
                    return Err(Error::InvalidInput(
                        "Cannot clear cache while model is loaded. Unload it first.".into(),
                    ));
                }

                info!("Cache clearing not yet implemented");
                Err(Error::System("Cache clearing not yet implemented".into()))
            })
        })
        .await
        .map_err(|e| Error::System(format!("Task join error: {}", e)))??;

        Ok(())
    }
}
