# LLM Status Integration Examples

This document shows how to integrate the LLM status detection system into your React components.

## Overview

The LLM status system provides:
- Real-time health monitoring with 30-second caching
- Automatic polling to detect status changes
- Graceful degradation when LLM is unavailable
- Type-safe TypeScript hooks
- Multiple levels of detail (simple boolean check or full status)

## Backend Architecture

### Commands Available

1. **`get_llm_status`** - Comprehensive status with caching (30s cache)
2. **`refresh_llm_status`** - Force refresh, bypasses cache
3. **`check_llm_connection`** - Simple boolean health check
4. **`get_model_details`** - Detailed model information
5. **`clear_llm_cache`** - Clear status cache (debugging)

### Status Response Structure

```typescript
interface LlmStatus {
  available: boolean;           // Overall availability
  provider: string;             // "local-llama" or "none"
  model?: string;               // Currently loaded model name
  model_status?: string;        // "loaded", "not_loaded", etc.
  error?: string;               // Error message if any
  model_loaded: boolean;        // Whether model is in memory
  feature_enabled: boolean;     // Whether local-ai feature is compiled
}
```

## Frontend Hooks

### 1. Basic Status Check - `useLlmStatus`

Use this for full status information with automatic polling.

```tsx
import { useLlmStatus } from "@/hooks/useLlmStatus";

function AIStatusIndicator() {
  const { status, isLoading, isAvailable, refetch } = useLlmStatus({
    refetchInterval: 60000, // Poll every minute
    staleTime: 30000,       // Cache for 30 seconds
  });

  if (isLoading) {
    return <div>Checking AI status...</div>;
  }

  if (!isAvailable) {
    return (
      <div className="ai-offline">
        <AlertCircle className="icon" />
        <span>AI Offline</span>
        {status?.error && <p>{status.error}</p>}
        <button onClick={() => refetch()}>Retry</button>
      </div>
    );
  }

  return (
    <div className="ai-online">
      <CheckCircle className="icon" />
      <span>AI Ready</span>
      {status?.model && <p>Model: {status.model}</p>}
    </div>
  );
}
```

### 2. Simple Connection Check - `useLlmConnection`

Use this when you only need a boolean yes/no answer.

```tsx
import { useLlmConnection } from "@/hooks/useLlmStatus";

function AIFeatureToggle() {
  const { isConnected, isLoading } = useLlmConnection();

  return (
    <Switch
      checked={isConnected}
      disabled={isLoading || !isConnected}
      label="Enable AI Coach"
    />
  );
}
```

### 3. Full Status Manager - `useLlmStatusManager`

Use this for comprehensive status management with convenience helpers.

```tsx
import { useLlmStatusManager } from "@/hooks/useLlmStatus";

function FocusSuggestions() {
  const {
    isAvailable,
    isEnabled,
    isModelLoaded,
    modelName,
    errorMessage,
    refresh,
  } = useLlmStatusManager();

  // Feature not compiled into this build
  if (!isEnabled) {
    return (
      <div className="feature-disabled">
        <p>AI features are not available in this version.</p>
        <p>Download the AI-enabled build to use focus suggestions.</p>
      </div>
    );
  }

  // LLM not available (model not loaded, connection error, etc.)
  if (!isAvailable) {
    return (
      <div className="ai-setup-required">
        <h3>AI Coach Setup Required</h3>
        <p>{errorMessage}</p>

        {!isModelLoaded && (
          <div className="setup-instructions">
            <h4>To enable AI features:</h4>
            <ol>
              <li>Go to Settings → AI Coach</li>
              <li>Download a model (recommended: Phi-3.5-mini)</li>
              <li>Click "Load Model" to activate</li>
            </ol>
          </div>
        )}

        <button onClick={refresh}>Check Again</button>
      </div>
    );
  }

  // AI is available and ready
  return (
    <div className="focus-suggestions">
      <h3>AI Focus Suggestions</h3>
      <p className="model-badge">Powered by {modelName}</p>
      <SuggestionsContent />
    </div>
  );
}
```

### 4. Conditional Feature Rendering

```tsx
import { useLlmStatus } from "@/hooks/useLlmStatus";
import { useCoachChat } from "@/hooks/useCoach";

function CoachChatPanel() {
  const { isAvailable } = useLlmStatus();
  const { mutate: sendMessage, isPending } = useCoachChat();

  if (!isAvailable) {
    return (
      <div className="coach-unavailable">
        <p>AI Coach is currently offline.</p>
        <p>Please check your AI settings to enable this feature.</p>
      </div>
    );
  }

  return (
    <div className="coach-chat">
      <ChatMessages />
      <ChatInput
        onSend={(msg) => sendMessage({ message: msg })}
        disabled={isPending}
      />
    </div>
  );
}
```

### 5. Status Badge Component

```tsx
import { useLlmStatusManager } from "@/hooks/useLlmStatus";

function AIStatusBadge() {
  const { isAvailable, isModelLoaded, modelName, errorMessage } =
    useLlmStatusManager();

  if (!isAvailable) {
    return (
      <Badge variant="destructive" title={errorMessage}>
        AI Offline
      </Badge>
    );
  }

  if (!isModelLoaded) {
    return (
      <Badge variant="warning" title="Model not loaded">
        AI Standby
      </Badge>
    );
  }

  return (
    <Badge variant="success" title={`Using ${modelName}`}>
      AI Ready
    </Badge>
  );
}
```

### 6. Force Refresh After Model Changes

```tsx
import { useRefreshLlmStatus } from "@/hooks/useLlmStatus";
import { invoke } from "@tauri-apps/api/core";

function ModelLoadButton() {
  const { mutate: refresh } = useRefreshLlmStatus();
  const [loading, setLoading] = useState(false);

  const handleLoadModel = async () => {
    setLoading(true);
    try {
      // Load the model via backend command
      await invoke("load_model");

      // Force refresh status immediately (bypasses cache)
      refresh();

      toast.success("Model loaded successfully!");
    } catch (error) {
      toast.error(`Failed to load model: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <button onClick={handleLoadModel} disabled={loading}>
      {loading ? "Loading..." : "Load AI Model"}
    </button>
  );
}
```

### 7. Graceful Feature Degradation

```tsx
import { useLlmStatus } from "@/hooks/useLlmStatus";

function SessionPlanner() {
  const { isAvailable } = useLlmStatus();

  return (
    <div className="session-planner">
      <h2>Plan Your Focus Session</h2>

      <DurationSelector />
      <BlockingOptions />

      {/* Show AI suggestions only if available */}
      {isAvailable ? (
        <AISmartSuggestions />
      ) : (
        <ManualTips />
      )}

      <StartSessionButton />
    </div>
  );
}

function AISmartSuggestions() {
  return (
    <div className="ai-suggestions">
      <h3>AI Recommendations</h3>
      <p>Based on your patterns, we suggest...</p>
      {/* AI-powered content */}
    </div>
  );
}

function ManualTips() {
  return (
    <div className="manual-tips">
      <h3>Quick Tips</h3>
      <ul>
        <li>Start with 25-minute sessions</li>
        <li>Block social media for deeper focus</li>
        <li>Take 5-minute breaks between sessions</li>
      </ul>
    </div>
  );
}
```

### 8. Polling Control

```tsx
import { useLlmStatus } from "@/hooks/useLlmStatus";

function AIMonitor() {
  const [autoCheck, setAutoCheck] = useState(true);

  // Disable polling when auto-check is off
  const { status, isAvailable } = useLlmStatus({
    enabled: autoCheck,
    refetchInterval: autoCheck ? 60000 : false,
  });

  return (
    <div>
      <label>
        <input
          type="checkbox"
          checked={autoCheck}
          onChange={(e) => setAutoCheck(e.target.checked)}
        />
        Auto-check AI status
      </label>

      {status && (
        <pre>{JSON.stringify(status, null, 2)}</pre>
      )}
    </div>
  );
}
```

## Common Patterns

### Pattern 1: Show Setup Instructions When Unavailable

```tsx
function AIFeatureGate({ children }: { children: React.ReactNode }) {
  const { isAvailable, isEnabled, errorMessage } = useLlmStatusManager();

  if (!isEnabled) {
    return <FeatureNotAvailable />;
  }

  if (!isAvailable) {
    return <SetupInstructions error={errorMessage} />;
  }

  return <>{children}</>;
}

// Usage
<AIFeatureGate>
  <AICoachPanel />
</AIFeatureGate>
```

### Pattern 2: Optimistic UI with Status Check

```tsx
function QuickAIQuery() {
  const { isAvailable } = useLlmStatus();
  const { mutate, isPending } = useCoachChat();

  const handleQuickTip = () => {
    if (!isAvailable) {
      toast.error("AI is currently offline");
      return;
    }

    mutate({ message: "Give me a quick focus tip" });
  };

  return (
    <button onClick={handleQuickTip} disabled={isPending || !isAvailable}>
      Get AI Tip
    </button>
  );
}
```

### Pattern 3: Status-Aware Form Validation

```tsx
function AISettingsForm() {
  const { isModelLoaded, modelName } = useLlmStatusManager();
  const [settings, setSettings] = useState({
    enableAICoach: isModelLoaded,
    model: modelName,
  });

  // Disable AI features if model not loaded
  useEffect(() => {
    if (!isModelLoaded && settings.enableAICoach) {
      setSettings(prev => ({ ...prev, enableAICoach: false }));
    }
  }, [isModelLoaded]);

  return (
    <form>
      <Switch
        checked={settings.enableAICoach}
        disabled={!isModelLoaded}
        label="Enable AI Coach"
        helperText={
          !isModelLoaded
            ? "Load a model first to enable AI features"
            : undefined
        }
      />
    </form>
  );
}
```

## Testing

### Mock LLM Status for Tests

```tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";

function mockLlmStatus(status: Partial<LlmStatus>) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  queryClient.setQueryData(["llm", "status"], {
    available: false,
    provider: "none",
    model_loaded: false,
    feature_enabled: false,
    ...status,
  });

  return queryClient;
}

// Test with AI available
test("shows AI features when available", () => {
  const queryClient = mockLlmStatus({
    available: true,
    provider: "local-llama",
    model: "phi-3.5-mini",
    model_loaded: true,
    feature_enabled: true,
  });

  render(
    <QueryClientProvider client={queryClient}>
      <AIFeatureComponent />
    </QueryClientProvider>
  );

  expect(screen.getByText("AI Ready")).toBeInTheDocument();
});

// Test with AI offline
test("shows setup instructions when unavailable", () => {
  const queryClient = mockLlmStatus({
    available: false,
    error: "Model not loaded",
  });

  render(
    <QueryClientProvider client={queryClient}>
      <AIFeatureComponent />
    </QueryClientProvider>
  );

  expect(screen.getByText(/setup required/i)).toBeInTheDocument();
});
```

## Performance Considerations

1. **Caching**: Status is cached for 30 seconds on the backend to avoid performance impact
2. **Polling**: Default refetch interval is 60 seconds (1 minute)
3. **Lazy Loading**: Use `enabled: false` option to disable queries until needed
4. **Selective Queries**: Use `useLlmConnection()` for simple checks instead of full status

## Troubleshooting

### Status Always Shows Unavailable

1. Check if `local-ai` feature is enabled in build
2. Verify model is downloaded and path is correct
3. Check backend logs for initialization errors

### Status Not Updating After Model Load

Use `useRefreshLlmStatus()` to force a cache bypass:

```tsx
const { mutate: refresh } = useRefreshLlmStatus();
// After loading model:
refresh();
```

### High CPU Usage

Reduce polling frequency:

```tsx
useLlmStatus({
  refetchInterval: 300000, // 5 minutes instead of 1 minute
});
```

## Next Steps

- See `src/hooks/useLlmStatus.ts` for full hook implementation
- See `src-tauri/src/commands/llm.rs` for backend command details
- See `src-tauri/src/ai/llm_engine.rs` for LLM engine health checks
