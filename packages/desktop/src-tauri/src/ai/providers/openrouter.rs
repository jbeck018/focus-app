// ai/providers/openrouter.rs - OpenRouter provider (OpenAI-compatible wrapper)

use super::traits::LlmProvider;
use super::types::{
    CompletionOptions, CompletionResponse, FinishReason, Message, MessageRole, ModelInfo,
    StreamChunk, TokenUsage,
};
use crate::{Error, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tracing::{debug, info};

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// OpenRouter provider implementation
///
/// OpenRouter provides access to multiple LLM providers through a single API.
/// It's OpenAI-compatible, so we use the async-openai client with custom headers.
pub struct OpenRouterProvider {
    client: Client<OpenAIConfig>,
    model: String,
    #[allow(dead_code)] // Reserved for future OpenRouter-specific headers
    site_url: Option<String>,
    #[allow(dead_code)] // Reserved for future OpenRouter-specific headers
    app_name: Option<String>,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub fn new(
        api_key: String,
        model: String,
        site_url: Option<String>,
        app_name: Option<String>,
    ) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(OPENROUTER_BASE_URL);

        let client = Client::with_config(config);

        Self {
            client,
            model,
            site_url,
            app_name,
        }
    }

    /// Convert our Message type to OpenAI's message type
    fn convert_message(message: &Message) -> ChatCompletionRequestMessage {
        match message.role {
            MessageRole::System => ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: message.content.clone().into(),
                    name: message.name.clone(),
                },
            ),
            MessageRole::User => ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: message.content.clone().into(),
                    name: message.name.clone(),
                },
            ),
            MessageRole::Assistant => {
                use async_openai::types::ChatCompletionRequestAssistantMessageContent;
                ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                            message.content.clone(),
                        )),
                        name: message.name.clone(),
                        tool_calls: None,
                        refusal: None,
                        ..Default::default()
                    },
                )
            }
        }
    }

    /// Build a chat completion request
    fn build_request(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<CreateChatCompletionRequest> {
        let mut builder = CreateChatCompletionRequestArgs::default();

        builder.model(&self.model);
        builder.messages(
            messages
                .iter()
                .map(Self::convert_message)
                .collect::<Vec<_>>(),
        );

        if let Some(max_tokens) = options.max_tokens {
            builder.max_tokens(max_tokens);
        }

        if let Some(temperature) = options.temperature {
            builder.temperature(temperature);
        }

        if let Some(top_p) = options.top_p {
            builder.top_p(top_p);
        }

        if let Some(stop) = &options.stop {
            builder.stop(stop.clone());
        }

        builder
            .build()
            .map_err(|e| Error::Ai(format!("Failed to build request: {}", e)))
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing OpenRouter health check");

        // Try to list models as a health check
        self.client
            .models()
            .list()
            .await
            .map_err(|e| Error::Network(format!("OpenRouter health check failed: {}", e)))?;

        info!("OpenRouter health check passed");
        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        debug!("Fetching OpenRouter models");

        let response = self
            .client
            .models()
            .list()
            .await
            .map_err(|e| Error::Network(format!("Failed to list models: {}", e)))?;

        let models = response
            .data
            .into_iter()
            .map(|model| ModelInfo {
                id: model.id.clone(),
                name: model.id,
                description: None,
                context_length: None,
                pricing: None,
            })
            .collect();

        Ok(models)
    }

    async fn complete(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        debug!(
            "Generating OpenRouter completion with {} messages",
            messages.len()
        );

        let request = self.build_request(messages, options)?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::Network(format!("OpenRouter API error: {}", e)))?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| Error::Ai("No choices in response".to_string()))?;

        let content = choice.message.content.clone().unwrap_or_default();

        let finish_reason = match &choice.finish_reason {
            Some(async_openai::types::FinishReason::Stop) => FinishReason::Stop,
            Some(async_openai::types::FinishReason::Length) => FinishReason::Length,
            Some(async_openai::types::FinishReason::ContentFilter) => FinishReason::ContentFilter,
            Some(async_openai::types::FinishReason::ToolCalls) => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        };

        let usage = response
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_default();

        Ok(CompletionResponse {
            content,
            model: response.model,
            finish_reason,
            usage,
        })
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        debug!(
            "Starting OpenRouter streaming completion with {} messages",
            messages.len()
        );

        let request = self.build_request(messages, options)?;

        let mut stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| Error::Network(format!("OpenRouter streaming error: {}", e)))?;

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        if let Some(choice) = response.choices.first() {
                            let delta = choice.delta.content.clone().unwrap_or_default();
                            let finish_reason = choice.finish_reason.as_ref().map(|r| {
                                match r {
                                    async_openai::types::FinishReason::Stop => FinishReason::Stop,
                                    async_openai::types::FinishReason::Length => FinishReason::Length,
                                    async_openai::types::FinishReason::ContentFilter => FinishReason::ContentFilter,
                                    async_openai::types::FinishReason::ToolCalls => FinishReason::ToolCalls,
                                    _ => FinishReason::Stop,
                                }
                            });

                            let chunk = StreamChunk {
                                delta,
                                finish_reason,
                                usage: None,
                            };

                            if tx.send(Ok(chunk)).await.is_err() {
                                break;
                            }

                            if finish_reason.is_some() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(Error::Network(format!("Stream error: {}", e))))
                            .await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
