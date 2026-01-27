// ai/providers/types.rs - Shared types for LLM providers

use serde::{Deserialize, Serialize};

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    #[allow(dead_code)] // Public API - convenience constructor for tests and future use
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
        }
    }

    #[allow(dead_code)] // Public API - convenience constructor for tests and future use
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
        }
    }

    #[allow(dead_code)] // Public API - convenience constructor for tests and future use
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
        }
    }
}

/// Message role in conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Options for completion requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl Default for CompletionOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(1024),
            temperature: Some(0.7),
            top_p: None,
            stop: None,
            stream: Some(false),
        }
    }
}

/// Response from a completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
}

/// Reason why generation finished
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    Error,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// A chunk from a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<FinishReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

/// Information about a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,
}

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
}

/// Configuration for different LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum ProviderConfig {
    OpenAI {
        api_key: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        base_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        organization: Option<String>,
    },
    Anthropic {
        api_key: String,
        model: String,
    },
    Google {
        api_key: String,
        model: String,
    },
    OpenRouter {
        api_key: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        site_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        app_name: Option<String>,
    },
    Local {
        model_path: String,
    },
}

impl ProviderConfig {
    pub fn provider_name(&self) -> &str {
        match self {
            ProviderConfig::OpenAI { .. } => "openai",
            ProviderConfig::Anthropic { .. } => "anthropic",
            ProviderConfig::Google { .. } => "google",
            ProviderConfig::OpenRouter { .. } => "openrouter",
            ProviderConfig::Local { .. } => "local",
        }
    }

    #[allow(dead_code)] // Public API for extracting model name from configuration
    pub fn model_name(&self) -> &str {
        match self {
            ProviderConfig::OpenAI { model, .. } => model,
            ProviderConfig::Anthropic { model, .. } => model,
            ProviderConfig::Google { model, .. } => model,
            ProviderConfig::OpenRouter { model, .. } => model,
            ProviderConfig::Local { model_path } => model_path,
        }
    }

    /// Create a sanitized config with sensitive data removed
    pub fn sanitize(&self) -> Self {
        match self {
            ProviderConfig::OpenAI {
                model,
                base_url,
                organization,
                ..
            } => ProviderConfig::OpenAI {
                api_key: "***".to_string(),
                model: model.clone(),
                base_url: base_url.clone(),
                organization: organization.clone(),
            },
            ProviderConfig::Anthropic { model, .. } => ProviderConfig::Anthropic {
                api_key: "***".to_string(),
                model: model.clone(),
            },
            ProviderConfig::Google { model, .. } => ProviderConfig::Google {
                api_key: "***".to_string(),
                model: model.clone(),
            },
            ProviderConfig::OpenRouter {
                model,
                site_url,
                app_name,
                ..
            } => ProviderConfig::OpenRouter {
                api_key: "***".to_string(),
                model: model.clone(),
                site_url: site_url.clone(),
                app_name: app_name.clone(),
            },
            ProviderConfig::Local { model_path } => ProviderConfig::Local {
                model_path: model_path.clone(),
            },
        }
    }
}

/// Available provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requires_api_key: bool,
    pub default_models: Vec<String>,
    /// Whether this provider is available in the current build
    /// For local provider, this depends on the `local-ai` feature flag
    #[serde(default = "default_true")]
    pub available: bool,
    /// Reason why the provider is unavailable (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

fn default_true() -> bool {
    true
}

impl ProviderInfo {
    pub fn openai() -> Self {
        Self {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            description: "GPT-4 and GPT-3.5 models from OpenAI".to_string(),
            requires_api_key: true,
            default_models: vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            available: true,
            unavailable_reason: None,
        }
    }

    pub fn anthropic() -> Self {
        Self {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            description: "Claude models from Anthropic".to_string(),
            requires_api_key: true,
            default_models: vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
            ],
            available: true,
            unavailable_reason: None,
        }
    }

    pub fn google() -> Self {
        Self {
            id: "google".to_string(),
            name: "Google".to_string(),
            description: "Gemini models from Google".to_string(),
            requires_api_key: true,
            default_models: vec![
                "gemini-2.0-flash-exp".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
            ],
            available: true,
            unavailable_reason: None,
        }
    }

    pub fn openrouter() -> Self {
        Self {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            description: "Access to multiple LLM providers through OpenRouter".to_string(),
            requires_api_key: true,
            default_models: vec![
                "anthropic/claude-3.5-sonnet".to_string(),
                "openai/gpt-4o".to_string(),
                "google/gemini-pro".to_string(),
            ],
            available: true,
            unavailable_reason: None,
        }
    }

    /// Create local provider info with availability based on feature flag.
    ///
    /// The local AI provider uses llama.cpp for privacy-first on-device inference.
    /// This requires the `local-ai` feature flag to be enabled at compile time.
    ///
    /// When available, users can download and run models like Phi-3.5-mini or TinyLlama
    /// entirely on their device without any data leaving their machine.
    pub fn local() -> Self {
        #[cfg(feature = "local-ai")]
        {
            Self {
                id: "local".to_string(),
                name: "Local (llama.cpp)".to_string(),
                description: "Privacy-first local LLM inference - runs entirely on your device".to_string(),
                requires_api_key: false,
                default_models: vec![
                    "phi-3.5-mini".to_string(),
                    "tinyllama".to_string(),
                ],
                available: true,
                unavailable_reason: None,
            }
        }
        #[cfg(not(feature = "local-ai"))]
        {
            Self {
                id: "local".to_string(),
                name: "Local (llama.cpp)".to_string(),
                description: "Privacy-first local LLM inference - not available in this build".to_string(),
                requires_api_key: false,
                default_models: vec![],
                available: false,
                unavailable_reason: Some(
                    "Local AI is not enabled in this build. The application was compiled without the 'local-ai' feature flag. Please use a cloud provider or contact the developer for a build with local AI support.".to_string()
                ),
            }
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::openai(),
            Self::anthropic(),
            Self::google(),
            Self::openrouter(),
            Self::local(),
        ]
    }
}

/// Check if the local AI feature is enabled at compile time.
///
/// This function allows runtime code to check whether local AI capabilities
/// are available without causing compilation errors.
///
/// # Returns
/// - `true` if the `local-ai` feature flag was enabled during compilation
/// - `false` otherwise
pub fn is_local_ai_enabled() -> bool {
    #[cfg(feature = "local-ai")]
    {
        true
    }
    #[cfg(not(feature = "local-ai"))]
    {
        false
    }
}
