// components/ai-settings/APIKeyInput.tsx
// Secure API key input with validation and masking

import * as React from "react";
import { Eye, EyeOff, Loader2, CheckCircle2, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import type { ProviderType, ProviderConfig } from "@/hooks/useAIProviders";
import { useTestConnection } from "@/hooks/useAIProviders";

export interface APIKeyInputProps {
  provider: ProviderType;
  modelId: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  label?: string;
  className?: string;
  onTestSuccess?: () => void;
  onTestError?: (error: string) => void;
}

/**
 * APIKeyInput Component
 *
 * Secure API key input with:
 * - Show/hide toggle for masked input
 * - Test connection button with loading states
 * - Validation feedback with icons
 * - Accessible labels and descriptions
 *
 * @example
 * ```tsx
 * <APIKeyInput
 *   provider="openai"
 *   modelId="gpt-4o"
 *   value={apiKey}
 *   onChange={setApiKey}
 *   onTestSuccess={() => toast.success("Connected!")}
 *   onTestError={(err) => toast.error(err)}
 * />
 * ```
 */
export function APIKeyInput({
  provider,
  modelId,
  value,
  onChange,
  placeholder = "Enter your API key",
  label = "API Key",
  className,
  onTestSuccess,
  onTestError,
}: APIKeyInputProps) {
  const [showKey, setShowKey] = React.useState(false);
  const [testStatus, setTestStatus] = React.useState<"idle" | "testing" | "success" | "error">(
    "idle"
  );
  const [errorMessage, setErrorMessage] = React.useState<string>("");

  const testConnection = useTestConnection();

  const handleTestConnection = React.useCallback(() => {
    if (!value.trim()) {
      setTestStatus("error");
      setErrorMessage("API key is required");
      onTestError?.("API key is required");
      return;
    }

    setTestStatus("testing");
    setErrorMessage("");

    let config: ProviderConfig;
    switch (provider) {
      case "openai":
        config = { provider: "openai", api_key: value, model: modelId };
        break;
      case "anthropic":
        config = { provider: "anthropic", api_key: value, model: modelId };
        break;
      case "google":
        config = { provider: "google", api_key: value, model: modelId };
        break;
      case "openrouter":
        config = { provider: "openrouter", api_key: value, model: modelId, app_name: "FocusFlow" };
        break;
      default:
        setTestStatus("error");
        setErrorMessage("Invalid provider for API key testing");
        return;
    }

    testConnection.mutate(config, {
      onSuccess: (result) => {
        if (result.success) {
          setTestStatus("success");
          setErrorMessage("");
          onTestSuccess?.();
        } else {
          setTestStatus("error");
          setErrorMessage(result.error ?? "Connection failed");
          onTestError?.(result.error ?? "Connection failed");
        }
      },
      onError: (error) => {
        setTestStatus("error");
        const message = error instanceof Error ? error.message : "Connection test failed";
        setErrorMessage(message);
        onTestError?.(message);
      },
    });
  }, [value, provider, modelId, testConnection, onTestSuccess, onTestError]);

  const handleKeyChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(e.target.value);
      // Reset test status when value changes
      if (testStatus !== "idle") {
        setTestStatus("idle");
        setErrorMessage("");
      }
    },
    [onChange, testStatus]
  );

  const toggleShowKey = React.useCallback(() => {
    setShowKey((prev) => !prev);
  }, []);

  const handleKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      // Allow Enter key to trigger test
      if (e.key === "Enter" && !testConnection.isPending) {
        e.preventDefault();
        handleTestConnection();
      }
    },
    [handleTestConnection, testConnection.isPending]
  );

  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <Label htmlFor={`api-key-${provider}`} className="text-sm font-medium">
        {label}
      </Label>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Input
            id={`api-key-${provider}`}
            type={showKey ? "text" : "password"}
            value={value}
            onChange={handleKeyChange}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            className={cn(
              "pr-10",
              testStatus === "success" && "border-green-500 focus-visible:ring-green-500/50",
              testStatus === "error" && "border-destructive"
            )}
            aria-invalid={testStatus === "error"}
            aria-describedby={testStatus === "error" ? `api-key-${provider}-error` : undefined}
            autoComplete="off"
            spellCheck={false}
          />

          <button
            type="button"
            onClick={toggleShowKey}
            className={cn(
              "absolute right-2 top-1/2 -translate-y-1/2",
              "rounded-sm p-1 hover:bg-accent transition-colors",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            )}
            aria-label={showKey ? "Hide API key" : "Show API key"}
            tabIndex={-1}
          >
            {showKey ? (
              <EyeOff className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
            ) : (
              <Eye className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
            )}
          </button>
        </div>

        <Button
          type="button"
          variant="outline"
          onClick={handleTestConnection}
          disabled={!value.trim() || testConnection.isPending}
          className="shrink-0"
          aria-label="Test API key connection"
        >
          {testConnection.isPending ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
              Testing...
            </>
          ) : testStatus === "success" ? (
            <>
              <CheckCircle2 className="h-4 w-4 text-green-500" aria-hidden="true" />
              Connected
            </>
          ) : testStatus === "error" ? (
            <>
              <XCircle className="h-4 w-4 text-destructive" aria-hidden="true" />
              Failed
            </>
          ) : (
            "Test Connection"
          )}
        </Button>
      </div>

      {testStatus === "success" && (
        <p
          className="text-sm text-green-600 dark:text-green-400 flex items-center gap-1.5"
          role="status"
          aria-live="polite"
        >
          <CheckCircle2 className="h-3.5 w-3.5" aria-hidden="true" />
          Connection successful! API key is valid.
        </p>
      )}

      {testStatus === "error" && errorMessage && (
        <p
          id={`api-key-${provider}-error`}
          className="text-sm text-destructive flex items-center gap-1.5"
          role="alert"
          aria-live="assertive"
        >
          <XCircle className="h-3.5 w-3.5" aria-hidden="true" />
          {errorMessage}
        </p>
      )}

      <p className="text-xs text-muted-foreground">
        Your API key is stored securely and encrypted locally. Press Enter to test.
      </p>
    </div>
  );
}

export default APIKeyInput;
