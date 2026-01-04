// ai/llm-engine.rs - Core LLM inference engine with streaming support

#![allow(clippy::missing_transmute_annotations)]

use crate::{Error, Result};
use crate::ai::model_manager::{ModelConfig, ModelManager};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaModel, Special};
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};

/// Response from LLM inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_generated: usize,
    pub inference_time_ms: u64,
}

/// A chunk of streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub text: String,
    pub is_final: bool,
}

/// Core LLM engine with lazy loading
pub struct LlmEngine {
    model_manager: ModelManager,
    model_config: ModelConfig,
    model: Arc<RwLock<Option<LoadedModel>>>,
    backend: Arc<LlamaBackend>,
}

/// A loaded model with its context
struct LoadedModel {
    model: Arc<LlamaModel>,
    context: LlamaContext<'static>,
}

// Safety: LoadedModel is protected by RwLock and only accessed from a single thread at a time
unsafe impl Send for LoadedModel {}
unsafe impl Sync for LoadedModel {}

impl LlmEngine {
    /// Create a new LLM engine (does not load model until first use)
    pub fn new(models_dir: PathBuf, model_config: ModelConfig) -> Result<Self> {
        // Initialize llama.cpp backend (one-time global init)
        let backend = LlamaBackend::init()
            .map_err(|e| Error::System(format!("Failed to init llama backend: {}", e)))?;

        Ok(Self {
            model_manager: ModelManager::new(models_dir),
            model_config,
            model: Arc::new(RwLock::new(None)),
            backend: Arc::new(backend),
        })
    }

    /// Check if model is currently loaded
    pub async fn is_loaded(&self) -> bool {
        self.model.read().await.is_some()
    }

    /// Ensure model is downloaded
    pub async fn ensure_model_downloaded(&self) -> Result<PathBuf> {
        self.model_manager.init().await?;

        if !self.model_manager.is_downloaded(&self.model_config).await {
            info!("Model not found, downloading: {}", self.model_config.name);
            return Err(Error::NotFound(format!(
                "Model {} is not downloaded. Please download it first.",
                self.model_config.name
            )));
        }

        Ok(self.model_manager.model_path(&self.model_config))
    }

    /// Load the model into memory (expensive operation)
    pub async fn load_model(&self) -> Result<()> {
        // Check if already loaded
        if self.is_loaded().await {
            debug!("Model already loaded");
            return Ok(());
        }

        let start = std::time::Instant::now();
        info!("Loading model: {}", self.model_config.name);

        let model_path = self.ensure_model_downloaded().await?;

        // Load model and context (must happen before any await to avoid Send issues)
        let loaded = {
            // Model parameters
            let model_params = LlamaModelParams::default();

            // Load model
            let model = LlamaModel::load_from_file(&self.backend, model_path, &model_params)
                .map_err(|e| Error::System(format!("Failed to load model: {}", e)))?;

            let model = Arc::new(model);

            // Context parameters
            // Use larger batch size to handle bigger prompts more efficiently
            // Set to 2048 to allow processing larger prompt chunks without multiple decode calls
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(std::num::NonZeroU32::new(self.model_config.context_size))
                .with_n_batch(2048);

            // Create context
            let context = model
                .new_context(&self.backend, ctx_params)
                .map_err(|e| Error::System(format!("Failed to create context: {}", e)))?;

            // Create loaded model
            LoadedModel {
                model: model.clone(),
                context: unsafe { std::mem::transmute(context) }, // Extend lifetime (safe in this context)
            }
        };

        *self.model.write().await = Some(loaded);

        let duration = start.elapsed();
        info!("Model loaded successfully in {:.2}s", duration.as_secs_f64());

        Ok(())
    }

    /// Unload model from memory to free resources
    pub async fn unload_model(&self) -> Result<()> {
        let mut model_lock = self.model.write().await;
        if model_lock.is_some() {
            *model_lock = None;
            info!("Model unloaded from memory");
        }
        Ok(())
    }

    /// Stop sequences for Phi-3.5 model
    const STOP_SEQUENCES: &'static [&'static str] = &[
        "<|end|>",      // Phi-3.5 end token
        "<|user|>",     // Stop if model tries to continue as user
        "<|system|>",   // Stop if model injects system message
        "[SYS]",        // Hallucinated format - stop immediately
        "\nUser:",      // Model hallucinating user turn
        "\n\nUser:",    // Same with double newline
    ];

    /// Check if generated text contains a stop sequence
    fn should_stop(text: &str) -> bool {
        for stop_seq in Self::STOP_SEQUENCES {
            if text.ends_with(stop_seq) || text.contains(stop_seq) {
                return true;
            }
        }
        false
    }

    /// Detect problematic generation patterns
    fn detect_bad_patterns(text: &str) -> bool {
        // Empty or near-empty code blocks
        if text.contains("```xml\n```") || text.contains("```\n```") {
            return true;
        }

        // Multiple tool calls (more than one <tool in the response)
        if text.matches("<tool ").count() > 1 {
            return true;
        }

        // Code block wrapping a tool (should be plain XML)
        if text.contains("```") && text.contains("<tool ") {
            return true;
        }

        false
    }

    /// Detect repetition loops (same phrase repeated 3+ times)
    fn detect_repetition_loop(text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 12 {
            return false;
        }

        // Check if last 4 words repeat 3 times
        let window_size = 4;
        let check_size = window_size * 3;

        if words.len() >= check_size {
            let recent = &words[words.len() - check_size..];
            let chunk1 = recent[0..window_size].join(" ");
            let chunk2 = recent[window_size..window_size * 2].join(" ");
            let chunk3 = recent[window_size * 2..window_size * 3].join(" ");

            if chunk1 == chunk2 && chunk2 == chunk3 {
                return true;
            }
        }

        false
    }

    /// Generate a response (non-streaming) with proper sampling
    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<LlmResponse> {
        // Ensure model is loaded
        if !self.is_loaded().await {
            self.load_model().await?;
        }

        let start = std::time::Instant::now();
        debug!("Generating response for prompt ({}chars)", prompt.len());

        let mut model_lock = self.model.write().await;
        let loaded = model_lock
            .as_mut()
            .ok_or_else(|| Error::System("Model not loaded".into()))?;

        // Clear KV cache to reset position tracking and avoid decode errors
        // This ensures the sequence positions start from 0 for this new inference session
        loaded.context.clear_kv_cache();
        debug!("KV cache cleared for new inference session");

        // Tokenize prompt
        let tokens = loaded
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| Error::System(format!("Tokenization failed: {}", e)))?;

        debug!("Prompt tokenized: {} tokens", tokens.len());

        // Validate prompt and generation tokens fit in context
        let context_size = self.model_config.context_size as usize;
        let total_tokens_needed = tokens.len() + max_tokens;

        if total_tokens_needed > context_size {
            warn!(
                "Prompt + generation may exceed context: {} prompt tokens + {} max_tokens = {} total > {} context size",
                tokens.len(),
                max_tokens,
                total_tokens_needed,
                context_size
            );
        }

        // Hard limit: prompt tokens alone cannot exceed context
        if tokens.len() > context_size {
            return Err(Error::InvalidInput(format!(
                "Prompt too long: {} tokens exceeds context size {}",
                tokens.len(),
                context_size
            )));
        }

        // Process prompt tokens in batches to handle prompts larger than batch size
        // We use a batch size of 2048, so split prompt into chunks if needed
        let batch_size = 2048;
        let mut batch = LlamaBatch::new(batch_size, 1);

        // Process all prompt tokens in chunks
        let num_tokens = tokens.len();
        let mut pos = 0;

        while pos < num_tokens {
            batch.clear();

            // Determine chunk size (min of remaining tokens and batch size)
            let chunk_end = (pos + batch_size).min(num_tokens);
            let is_last_chunk = chunk_end == num_tokens;

            // Add tokens for this chunk
            for (idx, &token) in tokens[pos..chunk_end].iter().enumerate() {
                let token_pos = pos + idx;
                // Only request logits for the last token of the last chunk
                let needs_logits = is_last_chunk && (token_pos == num_tokens - 1);

                batch
                    .add(token, token_pos as i32, &[0], needs_logits)
                    .map_err(|e| Error::System(format!("Failed to add token to batch: {}", e)))?;
            }

            debug!("Processing prompt chunk: tokens {}-{}/{}", pos, chunk_end - 1, num_tokens);

            // Decode this batch chunk
            loaded
                .context
                .decode(&mut batch)
                .map_err(|e| Error::System(format!("Decode failed: {}", e)))?;

            pos = chunk_end;
        }

        // After processing all prompt tokens, verify KV cache has room for generation
        let kv_cache_remaining = context_size as i32 - num_tokens as i32;
        if kv_cache_remaining < max_tokens as i32 {
            warn!(
                "KV cache may be insufficient: {} tokens remaining, {} tokens needed for generation",
                kv_cache_remaining, max_tokens
            );
        } else {
            debug!(
                "KV cache ready: {} tokens used by prompt, {} tokens available for generation (max_tokens={})",
                num_tokens, kv_cache_remaining, max_tokens
            );
        }

        // Generate tokens with stop sequence and loop detection
        // Note: llama-cpp-2 0.1.x doesn't expose repetition penalty or temperature sampling
        // We rely on stop sequences and loop detection to prevent degenerate output
        let mut generated_text = String::new();
        let mut n_generated = 0;

        // Temperature parameter reserved for future use when library supports it
        let _ = temperature;

        // Get the starting position for generation (after all prompt tokens)
        let mut current_pos = num_tokens;

        for i in 0..max_tokens {
            // Get logits for the last processed token (index -1 from batch)
            let logits = loaded.context.candidates_ith(batch.n_tokens() - 1);

            let mut candidates = LlamaTokenDataArray::from_iter(logits, false);

            // Use greedy sampling (llama-cpp-2 0.1.x limitation)
            let token = candidates.sample_token_greedy();

            // Check for end of sequence
            if token == loaded.model.token_eos() {
                debug!("EOS token encountered, stopping generation");
                break;
            }

            // Convert token to text
            let piece = loaded
                .model
                .token_to_str(token, Special::Plaintext)
                .map_err(|e| Error::System(format!("Token to string failed: {}", e)))?;

            generated_text.push_str(&piece);
            n_generated += 1;

            // Check for stop sequences every 8 tokens
            if i % 8 == 0 && Self::should_stop(&generated_text) {
                warn!("Stop sequence detected, truncating response");
                // Remove the stop sequence from output
                for stop_seq in Self::STOP_SEQUENCES {
                    if generated_text.contains(stop_seq) {
                        if let Some(pos) = generated_text.find(stop_seq) {
                            generated_text.truncate(pos);
                        }
                        break;
                    }
                }
                break;
            }

            // Check for repetition loops every 16 tokens
            if i % 16 == 0 && i > 32 && Self::detect_repetition_loop(&generated_text) {
                warn!("Repetition loop detected, stopping generation early");
                // Try to find a good stopping point
                if let Some(last_period) = generated_text.rfind('.') {
                    generated_text.truncate(last_period + 1);
                }
                break;
            }

            // Check for bad patterns (empty code blocks, multiple tools, code-wrapped tools)
            if i % 16 == 0 && i > 48 && Self::detect_bad_patterns(&generated_text) {
                warn!("Bad pattern detected, stopping early");

                // Strategy: Keep everything up to and including the first complete tool call
                // This preserves any conversational text before the tool
                if let Some(first_tool_end) = generated_text.find("/>") {
                    // Include the /> and any immediately following newline
                    let mut end_pos = first_tool_end + 2;
                    if generated_text.get(end_pos..end_pos+1) == Some("\n") {
                        end_pos += 1;
                    }
                    generated_text.truncate(end_pos);
                } else if let Some(last_period) = generated_text.rfind('.') {
                    // No tool found, truncate at last complete sentence
                    generated_text.truncate(last_period + 1);
                }
                break;
            }

            // Prepare next batch with new token at the correct position
            batch.clear();
            batch
                .add(token, current_pos as i32, &[0], true)
                .map_err(|e| Error::System(format!("Failed to add token: {}", e)))?;

            current_pos += 1;

            // Decode next token
            loaded
                .context
                .decode(&mut batch)
                .map_err(|e| Error::System(format!("Decode failed: {}", e)))?;
        }

        let duration = start.elapsed();
        let tokens_per_sec = n_generated as f64 / duration.as_secs_f64();

        info!(
            "Generated {} tokens in {:.2}s ({:.1} tok/s)",
            n_generated,
            duration.as_secs_f64(),
            tokens_per_sec
        );

        // Clean up the output - strip common LLM prefixes and markdown artifacts
        let mut cleaned = generated_text.trim().to_string();

        // Strip markdown header prefixes like "## Response:" or "# Response:"
        let header_prefixes = ["## Response:", "# Response:", "### Response:",
                               "## response:", "# response:", "### response:"];
        for prefix in header_prefixes {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].trim_start().to_string();
                break;
            }
        }

        // Strip plain prefixes
        let plain_prefixes = ["Response:", "response:", "Assistant:", "assistant:", "A:"];
        for prefix in plain_prefixes {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].trim_start().to_string();
                break;
            }
        }

        // Remove code block wrappers around tool calls
        // Pattern: ```xml\n<tool.../>\n``` or ```\n<tool.../>\n```
        if cleaned.contains("```") && cleaned.contains("<tool ") {
            // Extract just the tool call
            if let Some(tool_start) = cleaned.find("<tool ") {
                if let Some(tool_end) = cleaned[tool_start..].find("/>") {
                    let tool_call = &cleaned[tool_start..tool_start + tool_end + 2];
                    // Get text before the code block
                    let before_block = if let Some(block_start) = cleaned.find("```") {
                        cleaned[..block_start].trim()
                    } else {
                        ""
                    };
                    if before_block.is_empty() {
                        cleaned = tool_call.to_string();
                    } else {
                        cleaned = format!("{}\n\n{}", before_block, tool_call);
                    }
                }
            }
        }

        Ok(LlmResponse {
            text: cleaned,
            tokens_generated: n_generated,
            inference_time_ms: duration.as_millis() as u64,
        })
    }

    /// Generate with streaming (returns channel for chunks)
    pub async fn generate_stream(
        &self,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamChunk>>> {
        // Ensure model is loaded
        if !self.is_loaded().await {
            self.load_model().await?;
        }

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let prompt = prompt.to_string();
        let model = self.model.clone();
        let model_config = self.model_config.clone();

        // Spawn streaming task
        tokio::spawn(async move {
            let result = Self::stream_generate_internal(
                model,
                &prompt,
                max_tokens,
                temperature,
                model_config,
                tx.clone(),
            )
            .await;

            if let Err(e) = result {
                let _ = tx.send(Err(e)).await;
            }
        });

        Ok(rx)
    }

    /// Internal streaming implementation
    async fn stream_generate_internal(
        model: Arc<RwLock<Option<LoadedModel>>>,
        prompt: &str,
        max_tokens: usize,
        _temperature: f32,
        model_config: ModelConfig,
        tx: tokio::sync::mpsc::Sender<Result<StreamChunk>>,
    ) -> Result<()> {
        // Tokenize and process prompt
        let tokens = {
            let mut model_lock = model.write().await;
            let loaded = model_lock
                .as_mut()
                .ok_or_else(|| Error::System("Model not loaded".into()))?;

            // Clear KV cache to reset position tracking for new streaming session
            loaded.context.clear_kv_cache();
            debug!("KV cache cleared for new streaming session");

            // Tokenize
            let tokens = loaded
                .model
                .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
                .map_err(|e| Error::System(format!("Tokenization failed: {}", e)))?;

            // Validate prompt and generation tokens fit in context
            let context_size = model_config.context_size as usize;
            let total_tokens_needed = tokens.len() + max_tokens;

            if total_tokens_needed > context_size {
                warn!(
                    "Streaming: Prompt + generation may exceed context: {} prompt tokens + {} max_tokens = {} total > {} context size",
                    tokens.len(),
                    max_tokens,
                    total_tokens_needed,
                    context_size
                );
            }

            // Hard limit: prompt tokens alone cannot exceed context
            if tokens.len() > context_size {
                return Err(Error::InvalidInput(format!(
                    "Prompt too long: {} tokens exceeds context size {}",
                    tokens.len(),
                    context_size
                )));
            }

            // Process prompt tokens in batches (same as non-streaming)
            let batch_size = 2048;
            let mut batch = LlamaBatch::new(batch_size, 1);
            let num_tokens = tokens.len();
            let mut pos = 0;

            while pos < num_tokens {
                batch.clear();

                let chunk_end = (pos + batch_size).min(num_tokens);
                let is_last_chunk = chunk_end == num_tokens;

                for (idx, &token) in tokens[pos..chunk_end].iter().enumerate() {
                    let token_pos = pos + idx;
                    let needs_logits = is_last_chunk && (token_pos == num_tokens - 1);

                    batch
                        .add(token, token_pos as i32, &[0], needs_logits)
                        .map_err(|e| Error::System(format!("Failed to add token: {}", e)))?;
                }

                loaded
                    .context
                    .decode(&mut batch)
                    .map_err(|e| Error::System(format!("Decode failed: {}", e)))?;

                pos = chunk_end;
            }

            // After processing all prompt tokens, verify KV cache has room for generation
            let context_size = model_config.context_size as usize;
            let kv_cache_remaining = context_size as i32 - num_tokens as i32;
            if kv_cache_remaining < max_tokens as i32 {
                warn!(
                    "Streaming: KV cache may be insufficient: {} tokens remaining, {} tokens needed for generation",
                    kv_cache_remaining, max_tokens
                );
            } else {
                debug!(
                    "Streaming: KV cache ready: {} tokens used by prompt, {} tokens available for generation (max_tokens={})",
                    num_tokens, kv_cache_remaining, max_tokens
                );
            }

            tokens
        }; // Lock is dropped here

        // Generate and stream tokens
        let num_prompt_tokens = tokens.len();
        let mut current_pos = num_prompt_tokens;

        for i in 0..max_tokens {
            // Acquire lock, generate token, prepare next batch, release lock
            let (is_eos, chunk_text) = {
                let mut model_lock = model.write().await;
                let loaded = model_lock
                    .as_mut()
                    .ok_or_else(|| Error::System("Model not loaded".into()))?;

                let mut batch = LlamaBatch::new(2048, 1);

                // Get logits from the last position in the KV cache
                // The batch from prompt processing should have left us ready to sample
                let logits = loaded.context.candidates_ith(0);
                let mut candidates = LlamaTokenDataArray::from_iter(logits, false);
                let token = candidates.sample_token_greedy();

                // Check for EOS and prepare chunk
                let (is_eos, chunk_text) = if token == loaded.model.token_eos() {
                    (true, String::new())
                } else {
                    let piece = loaded
                        .model
                        .token_to_str(token, Special::Plaintext)
                        .map_err(|e| Error::System(format!("Token to string failed: {}", e)))?;
                    (false, piece)
                };

                // Prepare next batch if not EOS
                if !is_eos {
                    batch.clear();
                    batch
                        .add(token, current_pos as i32, &[0], true)
                        .map_err(|e| Error::System(format!("Failed to add token: {}", e)))?;

                    loaded
                        .context
                        .decode(&mut batch)
                        .map_err(|e| Error::System(format!("Decode failed: {}", e)))?;
                }

                (is_eos, chunk_text)
            }; // Lock is dropped here

            // Update position for next iteration
            if !is_eos {
                current_pos += 1;
            }

            // Now send the chunk (no lock held)
            let is_final = is_eos || i == max_tokens - 1;
            if tx
                .send(Ok(StreamChunk {
                    text: chunk_text,
                    is_final,
                }))
                .await
                .is_err()
            {
                break; // Receiver dropped
            }

            if is_eos {
                break;
            }
        }

        Ok(())
    }

    /// Get model info
    pub fn model_info(&self) -> &ModelConfig {
        &self.model_config
    }

    /// Health check: verify model is loaded and functional
    ///
    /// This performs a minimal inference test to ensure the model is responding.
    /// Returns Ok(()) if healthy, Err otherwise.
    pub async fn health_check(&self) -> Result<()> {
        if !self.is_loaded().await {
            return Err(Error::System("Model not loaded".into()));
        }

        debug!("Performing LLM health check");

        // Try a minimal generation to verify functionality
        let test_prompt = "Hello";
        match self.generate(test_prompt, 5, 0.7).await {
            Ok(response) => {
                if response.text.is_empty() {
                    warn!("Health check: model returned empty response");
                    Err(Error::System("Model returned empty response".into()))
                } else {
                    debug!("Health check passed: generated {} tokens", response.tokens_generated);
                    Ok(())
                }
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                Err(Error::System(format!("Health check failed: {}", e)))
            }
        }
    }
}

// Safety: LlmEngine uses Arc and RwLock internally
unsafe impl Send for LlmEngine {}
unsafe impl Sync for LlmEngine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_engine_creation() {
        let temp_dir = std::env::temp_dir().join("focusflow_llm_test");
        let config = ModelConfig::tiny_llama();

        // Should create engine without loading model
        let _engine = LlmEngine::new(temp_dir, config);
    }
}
