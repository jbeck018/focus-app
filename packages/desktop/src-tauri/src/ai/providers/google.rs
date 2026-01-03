// ai/providers/google.rs - Google Gemini provider (using OpenAI-compatible endpoint)

use super::openai::OpenAiProvider;
use super::traits::LlmProvider;
use super::types::{CompletionOptions, CompletionResponse, Message, ModelInfo, StreamChunk};
use crate::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

const GOOGLE_OPENAI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";

/// Google Gemini provider using OpenAI-compatible endpoint
///
/// Google provides an OpenAI-compatible API for Gemini models, so we can
/// reuse the OpenAI provider implementation with a different base URL.
pub struct GoogleProvider {
    inner: OpenAiProvider,
}

impl GoogleProvider {
    /// Create a new Google provider
    pub fn new(api_key: String, model: String) -> Self {
        let inner = OpenAiProvider::with_base_url(
            api_key,
            model,
            GOOGLE_OPENAI_BASE_URL.to_string(),
            None,
        );

        Self { inner }
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    fn model(&self) -> &str {
        self.inner.model()
    }

    async fn health_check(&self) -> Result<()> {
        self.inner.health_check().await
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Google doesn't expose models via OpenAI endpoint, return known models
        Ok(vec![
            ModelInfo {
                id: "gemini-2.0-flash-exp".to_string(),
                name: "Gemini 2.0 Flash (Experimental)".to_string(),
                description: Some("Latest experimental model with multimodal capabilities".to_string()),
                context_length: Some(1_000_000),
                pricing: None,
            },
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                description: Some("Most capable model with 1M token context".to_string()),
                context_length: Some(1_000_000),
                pricing: None,
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                description: Some("Fast and efficient model".to_string()),
                context_length: Some(1_000_000),
                pricing: None,
            },
        ])
    }

    async fn complete(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        self.inner.complete(messages, options).await
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        self.inner.complete_stream(messages, options).await
    }
}
