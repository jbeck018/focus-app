# AI Settings Components

Comprehensive AI provider configuration system for managing multiple LLM providers with a consistent, accessible interface.

## Components Overview

### 1. AISettingsDialog

Main dialog for configuring AI providers and models.

```tsx
import { AISettingsDialog } from "@/components/ai-settings";

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <>
      <Button onClick={() => setSettingsOpen(true)}>
        AI Settings
      </Button>
      <AISettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
      />
    </>
  );
}
```

**Features:**
- Provider selection tabs (Local, OpenAI, Anthropic, Google, OpenRouter)
- Model selection with recommendations
- API key management for cloud providers
- Model download for local provider
- Privacy warnings when switching from local to cloud
- Real-time validation and testing

### 2. ModelIndicator

Status indicator for chat header showing current model.

```tsx
import { ModelIndicator, ModelStatusBadge } from "@/components/chat/ModelIndicator";

function ChatHeader() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div className="flex items-center justify-between">
      <h1>AI Chat</h1>

      {/* Full indicator with click action */}
      <ModelIndicator
        onOpenSettings={() => setSettingsOpen(true)}
        variant="default" // or "compact"
      />

      {/* Or simple badge */}
      <ModelStatusBadge />
    </div>
  );
}
```

**Variants:**
- `default`: Full badge with model name and status
- `compact`: Icon-only with status dot

**Status Colors:**
- 🟢 Green: Connected and working
- 🔴 Red: Error or connection failed
- ⚫ Gray: Not configured

### 3. ProviderCard

Reusable card for displaying provider information.

```tsx
import { ProviderCard, ProviderCardCompact } from "@/components/ai-settings";

function ProviderList() {
  const { providers, activeProvider } = useAIProviderManager();

  return (
    <div className="grid gap-4">
      {providers?.map((provider) => (
        <ProviderCard
          key={provider.id}
          provider={provider}
          isActive={activeProvider?.provider === provider.id}
          onConfigure={(id) => handleConfigure(id)}
          onActivate={(id) => handleActivate(id)}
        />
      ))}
    </div>
  );
}
```

**Features:**
- Connection status badges
- Provider icons (lock for local, cloud for cloud)
- Current model display
- Configure and Activate buttons
- Error messages

### 4. APIKeyInput

Secure input for API keys with validation.

```tsx
import { APIKeyInput } from "@/components/ai-settings";

function ProviderSettings() {
  const [apiKey, setApiKey] = useState("");

  return (
    <APIKeyInput
      provider="openai"
      modelId="gpt-4o"
      value={apiKey}
      onChange={setApiKey}
      onTestSuccess={() => toast.success("Connected!")}
      onTestError={(error) => toast.error(error)}
    />
  );
}
```

**Features:**
- Show/hide toggle for security
- Test connection button with loading states
- Real-time validation
- Success/error feedback
- Press Enter to test

### 5. ModelDownloadProgress

Progress indicator for local model downloads.

```tsx
import {
  ModelDownloadProgress,
  ModelDownloadProgressInline,
} from "@/components/ai-settings";

function LocalModelSetup() {
  const { data: status } = useModelDownloadStatus();

  if (status?.isDownloading && status.progress) {
    return (
      <ModelDownloadProgress
        progress={status.progress}
        onCancel={() => invoke("cancel_download")}
      />
    );
  }

  // Or inline version
  return <ModelDownloadProgressInline progress={status.progress} />;
}
```

**Features:**
- Animated progress bar
- Download speed calculation
- Estimated time remaining
- File size information
- Cancel button

## React Query Hooks

### useAIProviderManager

Comprehensive provider management hook.

```tsx
import { useAIProviderManager } from "@/hooks/useAIProviders";

function Settings() {
  const {
    providers,
    activeProvider,
    setProvider,
    testConnection,
    saveApiKey,
    deleteApiKey,
    isLoading,
  } = useAIProviderManager();

  // Use the utilities
}
```

### Individual Hooks

```tsx
import {
  useProviders,
  useActiveProvider,
  useAvailableModels,
  useSetProvider,
  useTestConnection,
  useSaveApiKey,
  useDeleteApiKey,
  useDownloadModel,
  useModelDownloadStatus,
  useModelDownloadProgress,
} from "@/hooks/useAIProviders";

// Get list of providers
const { data: providers } = useProviders();

// Get active provider
const { data: activeProvider } = useActiveProvider();

// Get models for a provider
const { data: models } = useAvailableModels("openai");

// Set active provider
const setProvider = useSetProvider();
setProvider.mutate({
  provider: "openai",
  modelId: "gpt-4o",
  apiKey: "sk-...",
});

// Test connection
const testConnection = useTestConnection();
testConnection.mutate(config, {
  onSuccess: (result) => {
    if (result.success) {
      toast.success("Connected!");
    }
  },
});

// Save API key
const saveApiKey = useSaveApiKey();
saveApiKey.mutate({ provider: "openai", apiKey: "sk-..." });

// Download model
const downloadModel = useDownloadModel();
downloadModel.mutate("phi-3.5-mini");

// Listen to download progress
useModelDownloadProgress(
  (progress) => console.log(`${progress.percent}%`),
  (modelName) => toast.success(`Downloaded ${modelName}`),
  (error) => toast.error(error)
);
```

## TypeScript Types

```typescript
import type {
  ProviderType,
  ProviderConfig,
  ProviderInfo,
  ModelInfo,
  DownloadProgress,
  DownloadStatus,
} from "@/hooks/useAIProviders";

// Provider types
type ProviderType = "local" | "openai" | "anthropic" | "google" | "openrouter";

// Provider configuration
interface ProviderConfig {
  provider: ProviderType;
  apiKey?: string;
  modelId: string;
}

// Provider information with status
interface ProviderInfo {
  id: ProviderType;
  name: string;
  description: string;
  requiresApiKey: boolean;
  status: "connected" | "disconnected" | "error";
  errorMessage?: string;
  currentModel?: string;
}

// Model information
interface ModelInfo {
  id: string;
  name: string;
  description: string;
  sizeMb?: number;
  recommended?: boolean;
}

// Download progress
interface DownloadProgress {
  percent: number;
  downloadedMb: number;
  totalMb: number;
  modelName: string;
}
```

## Tauri Commands

These components expect the following Tauri commands to be implemented:

```rust
// Provider management
#[tauri::command]
async fn list_providers() -> Result<Vec<ProviderInfo>, String>

#[tauri::command]
async fn get_active_provider() -> Result<Option<ProviderConfig>, String>

#[tauri::command]
async fn set_active_provider(config: ProviderConfig) -> Result<(), String>

// Model management
#[tauri::command]
async fn list_models(provider: ProviderType) -> Result<Vec<ModelInfo>, String>

// Connection testing
#[tauri::command]
async fn test_provider_connection(config: ProviderConfig) -> Result<ConnectionResult, String>

// API key management (stored securely)
#[tauri::command]
async fn save_api_key(provider: ProviderType, api_key: String) -> Result<(), String>

#[tauri::command]
async fn get_api_key(provider: ProviderType) -> Result<Option<String>, String>

#[tauri::command]
async fn delete_api_key(provider: ProviderType) -> Result<(), String>

// Local model download
#[tauri::command]
async fn download_model(model_name: String) -> Result<(), String>

#[tauri::command]
async fn get_model_download_status() -> Result<DownloadStatus, String>
```

## Tauri Events

Listen for these events for real-time updates:

```typescript
import { listen } from "@tauri-apps/api/event";

// Download progress
listen<DownloadProgress>("model-download-progress", (event) => {
  console.log(`Download: ${event.payload.percent}%`);
});

// Download complete
listen<{ model_name: string }>("model-download-complete", (event) => {
  console.log(`Downloaded: ${event.payload.model_name}`);
});

// Download error
listen<{ error: string }>("model-download-error", (event) => {
  console.error(`Error: ${event.payload.error}`);
});
```

## Accessibility

All components follow WCAG 2.1 Level AA guidelines:

- **Keyboard Navigation**: Full keyboard support with Tab, Enter, Escape
- **Screen Readers**: Proper ARIA labels and live regions
- **Focus Management**: Clear focus indicators and logical tab order
- **Error Handling**: Accessible error messages with `role="alert"`
- **Loading States**: Announced to screen readers with `aria-live`

## Styling

Components use shadcn/ui design system with Tailwind CSS:

- Consistent spacing and sizing
- Dark mode support
- Responsive design
- Color-coded status indicators
- Smooth animations and transitions

## Privacy Features

- **Local Provider**: Clearly marked with lock icon (🔒)
- **Cloud Providers**: Clearly marked with cloud icon (☁️)
- **Privacy Warning**: Shown when switching from local to cloud
- **Secure Storage**: API keys encrypted and stored securely
- **Masked Input**: API keys masked by default with show/hide toggle

## Best Practices

1. **Always test connections** before saving
2. **Handle errors gracefully** with user-friendly messages
3. **Show loading states** during async operations
4. **Validate inputs** before submission
5. **Provide clear feedback** for all user actions
6. **Cache appropriately** with React Query
7. **Listen to events** for real-time updates

## Example: Complete Integration

```tsx
import { useState } from "react";
import { AISettingsDialog } from "@/components/ai-settings";
import { ModelIndicator } from "@/components/chat/ModelIndicator";
import { useAIProviderManager, useModelDownloadProgress } from "@/hooks/useAIProviders";
import { toast } from "sonner";

function AIChat() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const { activeProvider, isLoading } = useAIProviderManager();

  // Listen to download events
  useModelDownloadProgress(
    undefined,
    (modelName) => toast.success(`Downloaded ${modelName}`),
    (error) => toast.error(`Download failed: ${error}`)
  );

  // Block chat if no provider configured
  const canChat = activeProvider !== null && !isLoading;

  return (
    <div className="flex flex-col h-screen">
      {/* Header with model indicator */}
      <header className="flex items-center justify-between p-4 border-b">
        <h1>AI Chat</h1>
        <ModelIndicator
          onOpenSettings={() => setSettingsOpen(true)}
        />
      </header>

      {/* Chat area */}
      <main className="flex-1">
        {canChat ? (
          <ChatInterface />
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center space-y-4">
              <p className="text-muted-foreground">
                No AI provider configured
              </p>
              <Button onClick={() => setSettingsOpen(true)}>
                Configure AI Settings
              </Button>
            </div>
          </div>
        )}
      </main>

      {/* Settings dialog */}
      <AISettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
      />
    </div>
  );
}
```

## Troubleshooting

### Provider shows as "Error"
- Check API key is valid
- Test connection to verify
- Check network connectivity
- View error message in provider card

### Model download stuck
- Check available disk space
- Verify internet connection
- Check download status with `useModelDownloadStatus()`
- Listen to error events

### Changes not saving
- Check form validation
- Look for error toasts
- Verify Tauri commands are implemented
- Check browser console for errors

### Status not updating
- React Query cache may be stale
- Force refresh with `refetch()`
- Check `refetchInterval` settings
- Verify event listeners are active
