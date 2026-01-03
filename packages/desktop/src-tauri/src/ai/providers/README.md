# Multi-Provider LLM Abstraction System

This module provides a unified interface for interacting with multiple LLM providers in the FocusFlow desktop app.

## Architecture

### Core Components

1. **`traits.rs`** - Defines the `LlmProvider` trait that all providers must implement
2. **`types.rs`** - Shared types for messages, completions, streaming, and configuration
3. **Provider implementations**:
   - `openai.rs` - OpenAI/Azure OpenAI using `async-openai`
   - `anthropic.rs` - Anthropic Claude using direct API
   - `google.rs` - Google Gemini using OpenAI-compatible endpoint
   - `openrouter.rs` - OpenRouter unified API access
   - `local.rs` - Local llama.cpp integration wrapper
4. **`mod.rs`** - Module exports and provider factory function

### Provider Trait

All providers implement the `LlmProvider` trait:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn health_check(&self) -> Result<()>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn complete(&self, messages: &[Message], options: &CompletionOptions) -> Result<CompletionResponse>;
    async fn complete_stream(&self, messages: &[Message], options: &CompletionOptions) -> Result<mpsc::Receiver<Result<StreamChunk>>>;
}
```

## Usage

### Tauri Commands

The system exposes several Tauri commands for frontend integration:

#### List Available Providers

```typescript
const providers = await invoke<ProviderInfo[]>('list_providers');
// Returns: [{ id: "openai", name: "OpenAI", ... }, ...]
```

#### Set Active Provider

```typescript
await invoke('set_active_provider', {
  config: {
    provider: 'openai',
    api_key: 'sk-...',
    model: 'gpt-4o'
  }
});
```

#### Test Connection

```typescript
await invoke('test_provider_connection', {
  config: {
    provider: 'anthropic',
    api_key: 'sk-ant-...',
    model: 'claude-3-5-sonnet-20241022'
  }
});
```

#### Stream Chat

```typescript
const streamId = await invoke<string>('stream_chat', {
  messages: [
    { role: 'user', content: 'Hello!' }
  ],
  options: {
    max_tokens: 1024,
    temperature: 0.7
  }
});

// Listen for chunks
listen(`chat-stream-${streamId}`, (event) => {
  console.log('Chunk:', event.payload.delta);
});

// Listen for completion
listen(`chat-stream-complete-${streamId}`, (event) => {
  console.log('Complete:', event.payload.content);
});
```

#### Complete Chat (Non-streaming)

```typescript
const response = await invoke<string>('complete_chat', {
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'What is the capital of France?' }
  ],
  options: {
    max_tokens: 100
  }
});
```

### Credential Management

API keys are stored securely in the OS keychain using the `keyring` crate.

#### Save API Key

```typescript
await invoke('save_api_key', {
  provider: 'openai',
  api_key: 'sk-...'
});
```

#### Get API Key

```typescript
const apiKey = await invoke<string>('get_api_key', {
  provider: 'openai'
});
```

#### Delete API Key

```typescript
await invoke('delete_api_key', {
  provider: 'openai'
});
```

#### List Saved Providers

```typescript
const saved = await invoke<CredentialInfo[]>('list_saved_providers');
// Returns: [{ provider: "openai", has_api_key: true }, ...]
```

## Supported Providers

### OpenAI

```rust
ProviderConfig::OpenAI {
    api_key: "sk-...".to_string(),
    model: "gpt-4o".to_string(),
    base_url: None, // Optional: for Azure OpenAI
    organization: None, // Optional: organization ID
}
```

**Models**: gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo

### Anthropic

```rust
ProviderConfig::Anthropic {
    api_key: "sk-ant-...".to_string(),
    model: "claude-3-5-sonnet-20241022".to_string(),
}
```

**Models**: claude-3-5-sonnet-20241022, claude-3-5-haiku-20241022, claude-3-opus-20240229

### Google Gemini

```rust
ProviderConfig::Google {
    api_key: "AIza...".to_string(),
    model: "gemini-2.0-flash-exp".to_string(),
}
```

**Models**: gemini-2.0-flash-exp, gemini-1.5-pro, gemini-1.5-flash

### OpenRouter

```rust
ProviderConfig::OpenRouter {
    api_key: "sk-or-...".to_string(),
    model: "anthropic/claude-3.5-sonnet".to_string(),
    site_url: Some("https://focusflow.app".to_string()),
    app_name: Some("FocusFlow".to_string()),
}
```

**Access to**: Multiple providers through a single API

### Local (llama.cpp)

```rust
ProviderConfig::Local {
    model_path: "/path/to/model.gguf".to_string(),
}
```

**Privacy-first**: All inference happens locally, no network calls

## Implementation Details

### Type Conversions

Each provider handles type conversions between our unified types and provider-specific types:

- **OpenAI/OpenRouter**: Uses `async-openai` types with conversion layer
- **Anthropic**: Direct API with custom request/response types
- **Google**: OpenAI-compatible endpoint with custom base URL
- **Local**: Wraps existing `LlmEngine` with message-to-prompt conversion

### Streaming

Streaming is implemented using Tokio channels:

1. Provider creates a streaming response
2. Spawns a task to forward chunks to a channel
3. Returns the receiver to the caller
4. Chunks are emitted as Tauri events to the frontend

### Error Handling

All providers use the application's `Result<T>` type with comprehensive error variants:

- `Error::Network` - API/network errors
- `Error::Ai` - LLM-specific errors
- `Error::Config` - Configuration errors
- `Error::NotFound` - Missing API keys or models

### Security

API keys are:
- Never logged or exposed in error messages
- Stored in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Sanitized before being sent to frontend
- Only accessed when needed for API calls

## Testing

Run tests with:

```bash
cargo test --features local-ai
```

Check compilation:

```bash
cargo check --features local-ai
```

## Future Enhancements

Potential improvements:

1. **Tool/Function Calling**: Add support for structured outputs and function calling
2. **Cost Tracking**: Monitor token usage and estimate costs per provider
3. **Rate Limiting**: Implement per-provider rate limiting
4. **Caching**: Cache responses for identical requests
5. **Fallback**: Automatic fallback to alternative providers on failure
6. **Model Selection UI**: Help users choose optimal models for their use case
7. **Streaming Indicators**: Visual feedback for streaming progress
8. **Provider-specific Features**: Expose advanced features (system prompts, JSON mode, etc.)

## Dependencies

```toml
async-openai = "0.24"       # OpenAI API client
eventsource-stream = "0.2"  # SSE parsing for Anthropic streaming
keyring = "2.0"             # Secure credential storage
reqwest = "0.12"            # HTTP client for direct APIs
futures-util = "0.3"        # Stream utilities
async-trait = "0.1"         # Async trait support
tokio = "1"                 # Async runtime
```

## License

Part of FocusFlow desktop application.
