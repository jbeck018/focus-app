// ai/providers/local.rs - Wrapper around existing llama.cpp LlmEngine

use super::traits::LlmProvider;
use super::types::{
    CompletionOptions, CompletionResponse, FinishReason, Message, MessageRole, ModelInfo,
    StreamChunk, TokenUsage,
};
use crate::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::debug;

#[cfg(feature = "local-ai")]
use crate::ai::LlmEngine;

/// Local provider using llama.cpp
///
/// This wraps the existing LlmEngine to provide a consistent interface
/// with other cloud providers.
pub struct LocalProvider {
    #[cfg(feature = "local-ai")]
    engine: LlmEngine,
    model_name: String,
}

impl LocalProvider {
    /// Create a new local provider
    #[cfg(feature = "local-ai")]
    pub fn new(engine: LlmEngine, model_name: String) -> Self {
        Self { engine, model_name }
    }

    /// Create a new local provider (stub for when local-ai is disabled)
    #[cfg(not(feature = "local-ai"))]
    pub fn new(_model_path: String) -> Result<Self> {
        Err(Error::Config(
            "Local AI is disabled. Enable the 'local-ai' feature to use local models.".to_string(),
        ))
    }

    /// Convert messages to Phi-3.5 chat format
    /// Phi-3.5 uses special tokens: <|system|>, <|user|>, <|assistant|>, <|end|>
    fn messages_to_prompt(messages: &[Message]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    // Phi-3.5 system message format
                    prompt.push_str("<|system|>\n");
                    prompt.push_str(&message.content);
                    prompt.push_str("<|end|>\n");
                }
                MessageRole::User => {
                    prompt.push_str("<|user|>\n");
                    prompt.push_str(&message.content);
                    prompt.push_str("<|end|>\n");
                }
                MessageRole::Assistant => {
                    prompt.push_str("<|assistant|>\n");
                    prompt.push_str(&message.content);
                    prompt.push_str("<|end|>\n");
                }
            }
        }

        // Add assistant prefix for generation
        prompt.push_str("<|assistant|>\n");
        prompt
    }
}

#[async_trait]
impl LlmProvider for LocalProvider {
    fn name(&self) -> &str {
        "local"
    }

    fn model(&self) -> &str {
        &self.model_name
    }

    #[cfg(feature = "local-ai")]
    async fn health_check(&self) -> Result<()> {
        debug!("Performing local LLM health check");
        self.engine.health_check().await
    }

    #[cfg(not(feature = "local-ai"))]
    async fn health_check(&self) -> Result<()> {
        Err(Error::Config("Local AI is disabled".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Return information about the current model
        Ok(vec![ModelInfo {
            id: self.model_name.clone(),
            name: self.model_name.clone(),
            description: Some("Local model running via llama.cpp".to_string()),
            context_length: None,
            pricing: None,
        }])
    }

    #[cfg(feature = "local-ai")]
    async fn complete(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        debug!(
            "Generating local completion with {} messages",
            messages.len()
        );

        let prompt = Self::messages_to_prompt(messages);
        let max_tokens = options.max_tokens.unwrap_or(1024) as usize;
        let temperature = options.temperature.unwrap_or(0.7);

        let response = self.engine.generate(&prompt, max_tokens, temperature).await?;

        Ok(CompletionResponse {
            content: response.text,
            model: self.model_name.clone(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: response.tokens_generated as u32,
                total_tokens: response.tokens_generated as u32,
            },
        })
    }

    #[cfg(not(feature = "local-ai"))]
    async fn complete(
        &self,
        _messages: &[Message],
        _options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        Err(Error::Config("Local AI is disabled".to_string()))
    }

    #[cfg(feature = "local-ai")]
    async fn complete_stream(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        debug!(
            "Starting local streaming completion with {} messages",
            messages.len()
        );

        let prompt = Self::messages_to_prompt(messages);
        let max_tokens = options.max_tokens.unwrap_or(1024) as usize;
        let temperature = options.temperature.unwrap_or(0.7);

        let mut engine_rx = self
            .engine
            .generate_stream(&prompt, max_tokens, temperature)
            .await?;

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(result) = engine_rx.recv().await {
                match result {
                    Ok(chunk) => {
                        let stream_chunk = StreamChunk {
                            delta: chunk.text,
                            finish_reason: if chunk.is_final {
                                Some(FinishReason::Stop)
                            } else {
                                None
                            },
                            usage: None,
                        };

                        if tx.send(Ok(stream_chunk)).await.is_err() {
                            break;
                        }

                        if chunk.is_final {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    #[cfg(not(feature = "local-ai"))]
    async fn complete_stream(
        &self,
        _messages: &[Message],
        _options: &CompletionOptions,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        Err(Error::Config("Local AI is disabled".to_string()))
    }
}
