// ai/providers/anthropic.rs - Anthropic Claude provider (direct API)

use super::traits::LlmProvider;
use super::types::{
    CompletionOptions, CompletionResponse, FinishReason, Message, MessageRole, ModelInfo,
    StreamChunk, TokenUsage,
};
use crate::{Error, Result};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic provider implementation
pub struct AnthropicProvider {
    client: reqwest::Client,
    #[allow(dead_code)] // Stored for potential future use (e.g., key rotation)
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key)
                .map_err(|e| Error::Config(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::Network(format!("Failed to create client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            model,
        })
    }

    /// Convert messages to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_prompt = None;
        let mut converted_messages = Vec::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    // Anthropic uses a separate system parameter
                    system_prompt = Some(msg.content.clone());
                }
                MessageRole::User => {
                    converted_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                MessageRole::Assistant => {
                    converted_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        (system_prompt, converted_messages)
    }

    /// Build request body
    fn build_request_body(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> AnthropicRequest {
        let (system, messages) = self.convert_messages(messages);

        AnthropicRequest {
            model: self.model.clone(),
            messages,
            system,
            max_tokens: options.max_tokens.unwrap_or(4096),
            temperature: options.temperature,
            top_p: options.top_p,
            stop_sequences: options.stop.clone(),
            stream: options.stream,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing Anthropic health check");

        // Make a minimal request to verify credentials
        let url = format!("{}/messages", ANTHROPIC_API_URL);

        let test_request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: None,
            max_tokens: 10,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        let response = self
            .client
            .post(&url)
            .json(&test_request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Anthropic health check failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Network(format!(
                "Anthropic health check failed: {} - {}",
                status, error_text
            )));
        }

        info!("Anthropic health check passed");
        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Anthropic doesn't have a models endpoint, so return known models
        Ok(vec![
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                description: Some("Most intelligent model".to_string()),
                context_length: Some(200_000),
                pricing: None,
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                description: Some("Fast and efficient".to_string()),
                context_length: Some(200_000),
                pricing: None,
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                description: Some("Top-level performance".to_string()),
                context_length: Some(200_000),
                pricing: None,
            },
        ])
    }

    async fn complete(
        &self,
        messages: &[Message],
        options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        debug!(
            "Generating Anthropic completion with {} messages",
            messages.len()
        );

        let url = format!("{}/messages", ANTHROPIC_API_URL);
        let request_body = self.build_request_body(messages, options);

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Anthropic API error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Network(format!(
                "Anthropic API error: {} - {}",
                status, error_text
            )));
        }

        let response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(format!("Failed to parse response: {}", e)))?;

        let content = response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("stop_sequence") => FinishReason::Stop,
            _ => FinishReason::Stop,
        };

        let usage = TokenUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        };

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
            "Starting Anthropic streaming completion with {} messages",
            messages.len()
        );

        let url = format!("{}/messages", ANTHROPIC_API_URL);
        let mut request_body = self.build_request_body(messages, options);
        request_body.stream = Some(true);

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Anthropic streaming error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Network(format!(
                "Anthropic streaming error: {} - {}",
                status, error_text
            )));
        }

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream().eventsource();

            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => {
                        if event.event == "message_stop" {
                            let _ = tx
                                .send(Ok(StreamChunk {
                                    delta: String::new(),
                                    finish_reason: Some(FinishReason::Stop),
                                    usage: None,
                                }))
                                .await;
                            break;
                        }

                        if event.event == "content_block_delta" {
                            if let Ok(delta_event) =
                                serde_json::from_str::<AnthropicStreamDelta>(&event.data)
                            {
                                if let Some(text) = delta_event.delta.text {
                                    let chunk = StreamChunk {
                                        delta: text,
                                        finish_reason: None,
                                        usage: None,
                                    };

                                    if tx.send(Ok(chunk)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Stream error: {}", e);
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

// Anthropic API types

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for deserialization, may not all be read
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    #[allow(dead_code)] // Required for deserialization to distinguish content block types
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    delta: DeltaContent,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    #[serde(rename = "type")]
    #[allow(dead_code)] // Required for deserialization to distinguish delta types
    delta_type: String,
    text: Option<String>,
}
