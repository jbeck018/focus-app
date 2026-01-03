# Frontend Integration Guide - Multi-Provider LLM System

This guide shows how to integrate the multi-provider LLM system into the FocusFlow frontend.

## TypeScript Types

First, define the types matching the Rust backend:

```typescript
// types/llm.ts

export type MessageRole = 'system' | 'user' | 'assistant';

export interface Message {
  role: MessageRole;
  content: string;
  name?: string;
}

export interface CompletionOptions {
  max_tokens?: number;
  temperature?: number;
  top_p?: number;
  stop?: string[];
  stream?: boolean;
}

export interface StreamChunk {
  delta: string;
  finish_reason?: 'stop' | 'length' | 'content_filter' | 'tool_calls' | 'error';
  usage?: TokenUsage;
}

export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  description?: string;
  context_length?: number;
  pricing?: {
    input_per_1k: number;
    output_per_1k: number;
  };
}

export interface ProviderInfo {
  id: string;
  name: string;
  description: string;
  requires_api_key: boolean;
  default_models: string[];
}

export type ProviderConfig =
  | { provider: 'openai'; api_key: string; model: string; base_url?: string; organization?: string }
  | { provider: 'anthropic'; api_key: string; model: string }
  | { provider: 'google'; api_key: string; model: string }
  | { provider: 'openrouter'; api_key: string; model: string; site_url?: string; app_name?: string }
  | { provider: 'local'; model_path: string };

export interface CredentialInfo {
  provider: string;
  has_api_key: boolean;
}
```

## React Hooks

Create custom hooks for common operations:

```typescript
// hooks/useLLMProviders.ts
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';
import type { ProviderInfo, ProviderConfig, ModelInfo } from '../types/llm';

export function useLLMProviders() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<ProviderInfo[]>('list_providers')
      .then(setProviders)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return { providers, loading, error };
}

export function useActiveProvider() {
  const [config, setConfig] = useState<ProviderConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    setLoading(true);
    try {
      const config = await invoke<ProviderConfig | null>('get_active_provider');
      setConfig(config);
    } catch (e) {
      console.error('Failed to get active provider:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const setActiveProvider = async (newConfig: ProviderConfig) => {
    await invoke('set_active_provider', { config: newConfig });
    await refresh();
  };

  return { config, loading, setActiveProvider, refresh };
}

export function useProviderModels(provider: string, apiKey: string) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchModels = async () => {
    if (!provider || !apiKey) return;

    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ModelInfo[]>('list_models', {
        provider,
        apiKey,
        model: 'gpt-4', // Placeholder model for API call
      });
      setModels(result);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  };

  return { models, loading, error, fetchModels };
}
```

## Streaming Chat Component

```typescript
// components/StreamingChat.tsx
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useState } from 'react';
import type { Message, CompletionOptions, StreamChunk } from '../types/llm';

export function StreamingChat() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [streaming, setStreaming] = useState(false);
  const [currentResponse, setCurrentResponse] = useState('');

  const sendMessage = async () => {
    if (!input.trim() || streaming) return;

    const userMessage: Message = { role: 'user', content: input };
    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setStreaming(true);
    setCurrentResponse('');

    try {
      const streamId = await invoke<string>('stream_chat', {
        messages: [...messages, userMessage],
        options: {
          max_tokens: 1024,
          temperature: 0.7,
        } as CompletionOptions,
      });

      // Listen for chunks
      const unlisten = await listen<StreamChunk>(`chat-stream-${streamId}`, (event) => {
        setCurrentResponse((prev) => prev + event.payload.delta);

        if (event.payload.finish_reason) {
          unlisten();
        }
      });

      // Listen for completion
      await listen<{ content: string }>(`chat-stream-complete-${streamId}`, (event) => {
        const assistantMessage: Message = {
          role: 'assistant',
          content: event.payload.content,
        };
        setMessages((prev) => [...prev, assistantMessage]);
        setCurrentResponse('');
        setStreaming(false);
      });

      // Listen for errors
      await listen<{ error: string }>(`chat-stream-error-${streamId}`, (event) => {
        console.error('Stream error:', event.payload.error);
        setStreaming(false);
        setCurrentResponse('');
      });
    } catch (error) {
      console.error('Failed to start stream:', error);
      setStreaming(false);
    }
  };

  return (
    <div className="chat-container">
      <div className="messages">
        {messages.map((msg, idx) => (
          <div key={idx} className={`message ${msg.role}`}>
            <strong>{msg.role}:</strong> {msg.content}
          </div>
        ))}
        {currentResponse && (
          <div className="message assistant streaming">
            <strong>assistant:</strong> {currentResponse}
            <span className="cursor">▊</span>
          </div>
        )}
      </div>

      <div className="input-area">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          disabled={streaming}
          placeholder="Type a message..."
        />
        <button onClick={sendMessage} disabled={streaming || !input.trim()}>
          {streaming ? 'Sending...' : 'Send'}
        </button>
      </div>
    </div>
  );
}
```

## Provider Configuration Component

```typescript
// components/ProviderSettings.tsx
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { useLLMProviders, useActiveProvider } from '../hooks/useLLMProviders';
import type { ProviderConfig } from '../types/llm';

export function ProviderSettings() {
  const { providers } = useLLMProviders();
  const { config, setActiveProvider } = useActiveProvider();

  const [selectedProvider, setSelectedProvider] = useState('openai');
  const [apiKey, setApiKey] = useState('');
  const [model, setModel] = useState('gpt-4o');
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null);

  const testConnection = async () => {
    setTesting(true);
    setTestResult(null);

    const testConfig: ProviderConfig = {
      provider: selectedProvider as any,
      api_key: apiKey,
      model,
    } as ProviderConfig;

    try {
      await invoke('test_provider_connection', { config: testConfig });
      setTestResult('success');
    } catch (error) {
      console.error('Test failed:', error);
      setTestResult('error');
    } finally {
      setTesting(false);
    }
  };

  const saveProvider = async () => {
    // Save API key to keychain
    await invoke('save_api_key', {
      provider: selectedProvider,
      apiKey: apiKey,
    });

    // Set as active provider
    const providerConfig: ProviderConfig = {
      provider: selectedProvider as any,
      api_key: apiKey,
      model,
    } as ProviderConfig;

    await setActiveProvider(providerConfig);
    alert('Provider configured successfully!');
  };

  return (
    <div className="provider-settings">
      <h2>LLM Provider Settings</h2>

      <div className="form-group">
        <label>Provider</label>
        <select
          value={selectedProvider}
          onChange={(e) => setSelectedProvider(e.target.value)}
        >
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label>API Key</label>
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="Enter your API key"
        />
      </div>

      <div className="form-group">
        <label>Model</label>
        <input
          type="text"
          value={model}
          onChange={(e) => setModel(e.target.value)}
          placeholder="e.g., gpt-4o"
        />
      </div>

      <div className="actions">
        <button onClick={testConnection} disabled={testing || !apiKey}>
          {testing ? 'Testing...' : 'Test Connection'}
        </button>
        <button onClick={saveProvider} disabled={!apiKey || testResult !== 'success'}>
          Save & Activate
        </button>
      </div>

      {testResult === 'success' && (
        <div className="success">Connection successful!</div>
      )}
      {testResult === 'error' && (
        <div className="error">Connection failed. Check your API key.</div>
      )}

      {config && (
        <div className="current-provider">
          <h3>Active Provider</h3>
          <p>
            <strong>{config.provider}</strong> - {config.model || config.model_path}
          </p>
        </div>
      )}
    </div>
  );
}
```

## Credential Management

```typescript
// utils/credentials.ts
import { invoke } from '@tauri-apps/api/core';

export async function saveApiKey(provider: string, apiKey: string): Promise<void> {
  await invoke('save_api_key', { provider, apiKey });
}

export async function getApiKey(provider: string): Promise<string> {
  return await invoke('get_api_key', { provider });
}

export async function deleteApiKey(provider: string): Promise<void> {
  await invoke('delete_api_key', { provider });
}

export async function hasApiKey(provider: string): Promise<boolean> {
  return await invoke('has_api_key', { provider });
}

export async function listSavedProviders(): Promise<CredentialInfo[]> {
  return await invoke('list_saved_providers');
}
```

## Complete Example: AI Coach Integration

```typescript
// components/AICoach.tsx
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useState } from 'react';
import type { Message, StreamChunk } from '../types/llm';

export function AICoach() {
  const [advice, setAdvice] = useState('');
  const [loading, setLoading] = useState(false);

  const getCoachingAdvice = async (sessionData: any) => {
    setLoading(true);
    setAdvice('');

    const messages: Message[] = [
      {
        role: 'system',
        content: 'You are a productivity coach specializing in the Indistractable framework.',
      },
      {
        role: 'user',
        content: `Based on my session data: ${JSON.stringify(sessionData)}, what advice do you have?`,
      },
    ];

    try {
      const streamId = await invoke<string>('stream_chat', {
        messages,
        options: { max_tokens: 500, temperature: 0.7 },
      });

      await listen<StreamChunk>(`chat-stream-${streamId}`, (event) => {
        setAdvice((prev) => prev + event.payload.delta);
      });

      await listen(`chat-stream-complete-${streamId}`, () => {
        setLoading(false);
      });
    } catch (error) {
      console.error('Failed to get coaching advice:', error);
      setLoading(false);
    }
  };

  return (
    <div className="ai-coach">
      <h2>AI Coaching</h2>
      {advice && (
        <div className="advice-box">
          <p>{advice}</p>
          {loading && <span className="loading-indicator">...</span>}
        </div>
      )}
      <button onClick={() => getCoachingAdvice({})}>
        Get Advice
      </button>
    </div>
  );
}
```

## Error Handling

```typescript
// utils/errorHandling.ts

export function handleLLMError(error: any): string {
  if (typeof error === 'string') {
    return error;
  }

  if (error.type === 'Network') {
    return `Network error: ${error.message}. Check your internet connection and API key.`;
  }

  if (error.type === 'Ai') {
    return `AI error: ${error.message}. Try again or switch providers.`;
  }

  if (error.type === 'Config') {
    return `Configuration error: ${error.message}. Check your provider settings.`;
  }

  return 'An unexpected error occurred. Please try again.';
}
```

## Best Practices

1. **Always test connections** before saving provider configurations
2. **Handle streaming errors** gracefully with proper cleanup
3. **Store API keys securely** using the credential commands
4. **Provide feedback** during long-running operations
5. **Cache provider info** to avoid repeated API calls
6. **Implement retry logic** for transient network errors
7. **Respect rate limits** by throttling requests
8. **Show token usage** to help users understand costs

## Next Steps

1. Implement provider selection UI
2. Add model picker with descriptions
3. Show token usage and costs
4. Add retry logic for failed requests
5. Implement conversation history
6. Add export/import for conversations
7. Create presets for common use cases
