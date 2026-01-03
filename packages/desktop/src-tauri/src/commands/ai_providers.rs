// commands/ai_providers.rs - Commands for managing LLM providers

use crate::ai::providers::{
    create_provider, list_available_providers, CompletionOptions, LlmProvider, Message,
    ProviderConfig, ProviderInfo,
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
#[command]
pub async fn set_active_provider(app: AppHandle, config: ProviderConfig) -> Result<()> {
    info!("Setting active provider: {}", config.provider_name());

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
