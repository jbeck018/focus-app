// ai/providers/traits.rs - Core LlmProvider trait definition

use super::types::{CompletionOptions, CompletionResponse, Message, ModelInfo, StreamChunk};
use crate::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Core trait for LLM providers
///
/// All providers must implement this trait to be used in the application.
/// The trait provides both streaming and non-streaming interfaces.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name (e.g., "openai", "anthropic")
    #[allow(dead_code)] // Public API for provider identification, used in dynamic dispatch
    fn name(&self) -> &str;

    /// Get the current model identifier
    fn model(&self) -> &str;

    /// Perform a health check to verify the provider is accessible
    ///
    /// This should make a minimal API call to verify credentials and connectivity.
    async fn health_check(&self) -> Result<()>;

    /// List available models for this provider
    ///
    /// Returns a list of models that can be used with this provider.
    /// For API-based providers, this may require an API call.
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Generate a completion (non-streaming)
    ///
    /// Takes a list of messages and returns a single complete response.
    async fn complete(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<CompletionResponse>;

    /// Generate a completion with streaming
    ///
    /// Returns a receiver that yields chunks as they are generated.
    /// The receiver will be closed when generation is complete.
    async fn complete_stream(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>>;
}

/// Trait for providers that support temperature-based sampling
#[allow(dead_code)] // Reserved for future capability detection pattern
#[async_trait]
pub trait SupportsTemperature {
    /// Set the temperature for generation (0.0 - 2.0)
    fn set_temperature(&mut self, temperature: f32);
}

/// Trait for providers that support custom system prompts
#[allow(dead_code)] // Reserved for future capability detection pattern
#[async_trait]
pub trait SupportsSystemPrompt {
    /// Set a custom system prompt
    fn set_system_prompt(&mut self, prompt: String);
}

/// Trait for providers that support tools/function calling
#[allow(dead_code)] // Reserved for future capability detection pattern
#[async_trait]
pub trait SupportsTools {
    /// Add a tool definition
    fn add_tool(&mut self, tool: serde_json::Value);

    /// Clear all tools
    fn clear_tools(&mut self);
}
