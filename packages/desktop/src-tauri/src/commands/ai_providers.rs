// commands/ai_providers.rs - Commands for managing LLM providers
//
// This module provides Tauri commands for managing AI providers including:
// - Listing available providers (cloud and local)
// - Setting the active provider
// - Testing provider connections
// - Streaming chat completions
//
// ## Local AI Support
//
// The local AI provider uses llama.cpp for on-device inference. This requires
// the `local-ai` feature flag to be enabled at compile time. When enabled,
// users can download and run models like Phi-3.5-mini or TinyLlama entirely
// on their device without any data leaving their machine.
//
// To check if local AI is available at runtime, use the `is_local_ai_available`
// command which returns detailed availability information.

use crate::ai::providers::{
    create_provider, is_local_ai_enabled, list_available_providers, CompletionOptions, LlmProvider,
    Message, ProviderConfig, ProviderInfo,
};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Active provider state
static ACTIVE_PROVIDER: once_cell::sync::Lazy<RwLock<Option<Box<dyn LlmProvider>>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

/// Active provider configuration (sanitized)
static ACTIVE_CONFIG: once_cell::sync::Lazy<RwLock<Option<ProviderConfig>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

/// Local AI availability status
///
/// Provides detailed information about whether local AI is available
/// and why it may not be available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAIStatus {
    /// Whether local AI is available in this build
    pub available: bool,
    /// Whether the local-ai feature flag was enabled at compile time
    pub feature_enabled: bool,
    /// Human-readable reason if unavailable
    pub reason: Option<String>,
    /// Available local models (empty if feature is disabled)
    pub available_models: Vec<String>,
}

/// Check if local AI is available in this build.
///
/// This command checks whether the application was compiled with local AI support
/// and returns detailed status information. Use this to determine whether to show
/// local AI options in the UI.
///
/// # Returns
/// - `LocalAIStatus` with availability information
///
/// # Example Response (local-ai enabled)
/// ```json
/// {
///   "available": true,
///   "feature_enabled": true,
///   "reason": null,
///   "available_models": ["phi-3.5-mini", "tinyllama"]
/// }
/// ```
///
/// # Example Response (local-ai disabled)
/// ```json
/// {
///   "available": false,
///   "feature_enabled": false,
///   "reason": "Local AI is not enabled in this build. The application was compiled without the 'local-ai' feature flag.",
///   "available_models": []
/// }
/// ```
#[command]
pub async fn is_local_ai_available() -> Result<LocalAIStatus> {
    debug!("Checking local AI availability");

    let feature_enabled = is_local_ai_enabled();

    if feature_enabled {
        #[cfg(feature = "local-ai")]
        {
            use crate::ai::ModelConfig;
            let models: Vec<String> = ModelConfig::all_models()
                .into_iter()
                .map(|m| m.name)
                .collect();

            info!("Local AI is available with {} models", models.len());
            Ok(LocalAIStatus {
                available: true,
                feature_enabled: true,
                reason: None,
                available_models: models,
            })
        }

        #[cfg(not(feature = "local-ai"))]
        {
            // This branch should never execute if feature_enabled is true,
            // but we need it for compilation when the feature is disabled
            Ok(LocalAIStatus {
                available: false,
                feature_enabled: false,
                reason: Some("Local AI is not enabled in this build.".to_string()),
                available_models: vec![],
            })
        }
    } else {
        info!("Local AI is not available (feature disabled)");
        Ok(LocalAIStatus {
            available: false,
            feature_enabled: false,
            reason: Some(
                "Local AI is not enabled in this build. The application was compiled without \
                the 'local-ai' feature flag. Please use a cloud provider or contact the \
                developer for a build with local AI support."
                    .to_string(),
            ),
            available_models: vec![],
        })
    }
}

/// List all available provider types
#[command]
pub async fn list_providers() -> Result<Vec<ProviderInfo>> {
    debug!("Listing available providers");
    Ok(list_available_providers())
}

/// List available models for a specific provider
///
/// For local provider, returns hardcoded list of available models.
/// For cloud providers, attempts to fetch from API if API key is available.
#[command]
pub async fn list_models(provider: String) -> Result<Vec<crate::ai::providers::ModelInfo>> {
    debug!("Listing models for provider: {}", provider);

    // Handle local provider separately
    if provider == "local" {
        #[cfg(feature = "local-ai")]
        {
            use crate::ai::ModelConfig;
            let models = ModelConfig::all_models()
                .into_iter()
                .map(|config| crate::ai::providers::ModelInfo {
                    id: config.name.to_lowercase().replace(" ", "-"),
                    name: config.name,
                    description: Some(config.description),
                    context_length: Some(config.context_size),
                    pricing: None,
                })
                .collect();
            return Ok(models);
        }

        #[cfg(not(feature = "local-ai"))]
        {
            return Ok(vec![]);
        }
    }

    // For cloud providers, try to get API key from keychain
    let api_key = match crate::commands::credentials::get_api_key(provider.clone()).await {
        Ok(key) => key,
        Err(_) => {
            // If no API key, return empty list
            debug!("No API key found for provider: {}", provider);
            return Ok(vec![]);
        }
    };

    // Use a dummy model ID for listing (providers don't require a specific model to list)
    let dummy_model = "gpt-4".to_string();

    // Create a temporary provider to list models
    let config = match provider.as_str() {
        "openai" => ProviderConfig::OpenAI {
            api_key,
            model: dummy_model,
            base_url: None,
            organization: None,
        },
        "anthropic" => ProviderConfig::Anthropic {
            api_key,
            model: dummy_model,
        },
        "google" => ProviderConfig::Google {
            api_key,
            model: dummy_model,
        },
        "openrouter" => ProviderConfig::OpenRouter {
            api_key,
            model: dummy_model,
            site_url: None,
            app_name: Some("FocusFlow".to_string()),
        },
        _ => {
            return Err(Error::InvalidInput(format!(
                "Unknown provider: {}",
                provider
            )))
        }
    };

    let provider_instance = create_provider(config)?;
    let models = provider_instance.list_models().await?;

    info!("Found {} models for provider", models.len());
    Ok(models)
}

/// Set the active provider
///
/// This command sets the active AI provider for the application.
/// For cloud providers, this validates the API key and model.
/// For local providers, this checks if local AI is enabled and available.
///
/// # Errors
/// - Returns an error if the local provider is selected but local AI is not available
/// - Returns an error if the provider fails the health check
/// - Returns an error if the API key is invalid (for cloud providers)
#[command]
pub async fn set_active_provider(app: AppHandle, config: ProviderConfig) -> Result<()> {
    info!("Setting active provider: {}", config.provider_name());

    // For local provider, check if local AI is available first
    if let ProviderConfig::Local { .. } = &config {
        if !is_local_ai_enabled() {
            return Err(Error::Config(
                "Local AI is not available in this build. The application was compiled without \
                the 'local-ai' feature flag. Please select a cloud provider instead."
                    .to_string(),
            ));
        }
    }

    // For local provider, resolve model ID to path using app data directory
    let resolved_config = if let ProviderConfig::Local { model_path } = &config {
        use tauri::Manager;

        // Only resolve if it's a model ID, not already a full path
        let is_model_id = !std::path::Path::new(model_path).exists();

        if is_model_id {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Error::Config(format!("Failed to get app data dir: {}", e)))?;

            let models_dir = app_data_dir.join("models");

            // Set environment variable for create_provider to use
            std::env::set_var("FOCUSFLOW_MODELS_DIR", models_dir.to_string_lossy().to_string());
        }

        config.clone()
    } else {
        config.clone()
    };

    // Create the provider
    let provider = create_provider(resolved_config)?;

    // Perform health check
    provider.health_check().await?;

    // Store the provider and sanitized config
    *ACTIVE_PROVIDER.write().await = Some(provider);
    *ACTIVE_CONFIG.write().await = Some(config.sanitize());

    info!("Active provider set successfully");
    Ok(())
}

/// Get the current active provider configuration (sanitized)
#[command]
pub async fn get_active_provider() -> Result<Option<ProviderConfig>> {
    debug!("Getting active provider");
    Ok(ACTIVE_CONFIG.read().await.clone())
}

/// Test a provider connection without setting it as active
#[command]
pub async fn test_provider_connection(config: ProviderConfig) -> Result<()> {
    info!("Testing provider connection: {}", config.provider_name());

    let provider = create_provider(config)?;
    provider.health_check().await?;

    info!("Provider connection test successful");
    Ok(())
}

/// Stream a chat completion with the active provider
///
/// Emits events to the frontend as chunks are received.
#[command]
pub async fn stream_chat(
    app: AppHandle,
    messages: Vec<Message>,
    options: Option<CompletionOptions>,
) -> Result<String> {
    debug!("Starting chat stream with {} messages", messages.len());

    // Get active provider
    let provider_lock = ACTIVE_PROVIDER.read().await;
    let provider = provider_lock
        .as_ref()
        .ok_or_else(|| Error::Config("No active provider set".to_string()))?;

    let options = options.unwrap_or_default();

    // Generate a unique stream ID
    let stream_id = uuid::Uuid::new_v4().to_string();

    // Start streaming
    let mut rx = provider.complete_stream(&messages, &options).await?;

    // Spawn a task to forward chunks to the frontend
    let stream_id_clone = stream_id.clone();
    tokio::spawn(async move {
        let mut full_content = String::new();

        while let Some(result) = rx.recv().await {
            match result {
                Ok(chunk) => {
                    full_content.push_str(&chunk.delta);

                    // Emit chunk event
                    let event_name = format!("chat-stream-{}", stream_id_clone);
                    if let Err(e) = app.emit(&event_name, &chunk) {
                        warn!("Failed to emit stream chunk: {}", e);
                        break;
                    }

                    // If this is the final chunk, emit completion event
                    if chunk.finish_reason.is_some() {
                        let completion_event = format!("chat-stream-complete-{}", stream_id_clone);
                        let _ = app.emit(
                            &completion_event,
                            ChatStreamComplete {
                                stream_id: stream_id_clone.clone(),
                                content: full_content.clone(),
                            },
                        );
                        break;
                    }
                }
                Err(e) => {
                    warn!("Stream error: {}", e);
                    let error_event = format!("chat-stream-error-{}", stream_id_clone);
                    let _ = app.emit(
                        &error_event,
                        ChatStreamError {
                            stream_id: stream_id_clone.clone(),
                            error: e.to_string(),
                        },
                    );
                    break;
                }
            }
        }
    });

    Ok(stream_id)
}

/// Generate a non-streaming completion with the active provider
#[command]
pub async fn complete_chat(
    messages: Vec<Message>,
    options: Option<CompletionOptions>,
) -> Result<String> {
    debug!("Generating chat completion with {} messages", messages.len());

    // Get active provider
    let provider_lock = ACTIVE_PROVIDER.read().await;
    let provider = provider_lock
        .as_ref()
        .ok_or_else(|| Error::Config("No active provider set".to_string()))?;

    let options = options.unwrap_or_default();

    // Generate completion
    let response = provider.complete(&messages, &options).await?;

    info!(
        "Completion generated: {} tokens",
        response.usage.completion_tokens
    );
    Ok(response.content)
}

// Event payload types

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatStreamComplete {
    stream_id: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatStreamError {
    stream_id: String,
    error: String,
}
