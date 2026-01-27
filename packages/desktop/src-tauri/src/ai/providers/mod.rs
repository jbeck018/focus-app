// ai/providers/mod.rs - Multi-provider LLM abstraction system

pub mod anthropic;
pub mod google;
pub mod local;
pub mod openai;
pub mod openrouter;
pub mod traits;
pub mod types;

pub use anthropic::AnthropicProvider;
pub use google::GoogleProvider;
pub use local::LocalProvider;
pub use openai::OpenAiProvider;
pub use openrouter::OpenRouterProvider;
pub use traits::LlmProvider;
pub use types::{CompletionOptions, Message, ModelInfo, ProviderConfig, ProviderInfo, is_local_ai_enabled};

use crate::Result;

/// Factory function to create a provider from configuration
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn LlmProvider>> {
    match config {
        ProviderConfig::OpenAI {
            api_key,
            model,
            base_url,
            organization,
        } => {
            let provider = if let Some(base_url) = base_url {
                OpenAiProvider::with_base_url(api_key, model, base_url, organization)
            } else {
                OpenAiProvider::new(api_key, model)
            };
            Ok(Box::new(provider))
        }
        ProviderConfig::Anthropic { api_key, model } => {
            let provider = AnthropicProvider::new(api_key, model)?;
            Ok(Box::new(provider))
        }
        ProviderConfig::Google { api_key, model } => {
            let provider = GoogleProvider::new(api_key, model);
            Ok(Box::new(provider))
        }
        ProviderConfig::OpenRouter {
            api_key,
            model,
            site_url,
            app_name,
        } => {
            let provider = OpenRouterProvider::new(api_key, model, site_url, app_name);
            Ok(Box::new(provider))
        }
        ProviderConfig::Local { model_path } => {
            #[cfg(feature = "local-ai")]
            {
                use crate::ai::{LlmEngine, ModelConfig};
                use std::path::PathBuf;

                // Resolve model ID to config
                // model_path can be either a model ID (like "phi-3.5-mini") or an actual path
                let model_config = match model_path.as_str() {
                    "phi-3.5-mini" => ModelConfig::phi_3_5_mini(),
                    "tinyllama" => ModelConfig::tiny_llama(),
                    // If it's a full path, use it directly (for custom models)
                    path if PathBuf::from(path).exists() => {
                        // For custom paths, we need to extract the models directory
                        let path_buf = PathBuf::from(path);
                        let models_dir = path_buf
                            .parent()
                            .unwrap_or(&PathBuf::from("."))
                            .to_path_buf();

                        // Use default config for custom models
                        let mut config = ModelConfig::phi_3_5_mini();
                        config.name = path_buf
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("custom-model")
                            .to_string();
                        config.filename = path_buf
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("model.gguf")
                            .to_string();

                        return {
                            let engine = LlmEngine::new(models_dir, config.clone())?;
                            let provider = LocalProvider::new(engine, config.name);
                            Ok(Box::new(provider))
                        };
                    }
                    _ => {
                        return Err(crate::Error::InvalidInput(format!(
                            "Unknown model ID: {}. Supported models: phi-3.5-mini, tinyllama",
                            model_path
                        )));
                    }
                };

                // Get models directory from environment variable set by command layer
                let models_dir = std::env::var("FOCUSFLOW_MODELS_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("./models"));

                let engine = LlmEngine::new(models_dir, model_config.clone())?;
                let provider = LocalProvider::new(engine, model_config.name);
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "local-ai"))]
            {
                Err(crate::Error::Config(
                    "Local AI is disabled. Enable the 'local-ai' feature.".to_string(),
                ))
            }
        }
    }
}

/// List all available provider types
pub fn list_available_providers() -> Vec<ProviderInfo> {
    ProviderInfo::all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::types::MessageRole;

    #[test]
    fn test_provider_config_sanitize() {
        let config = ProviderConfig::OpenAI {
            api_key: "sk-secret123".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
            organization: None,
        };

        let sanitized = config.sanitize();

        match sanitized {
            ProviderConfig::OpenAI { api_key, .. } => {
                assert_eq!(api_key, "***");
            }
            _ => panic!("Expected OpenAI config"),
        }
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");

        let msg = Message::system("You are a helpful assistant");
        assert_eq!(msg.role, MessageRole::System);
    }

    #[test]
    fn test_completion_options_default() {
        let options = CompletionOptions::default();
        assert_eq!(options.max_tokens, Some(1024));
        assert_eq!(options.temperature, Some(0.7));
    }
}
