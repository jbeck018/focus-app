// components/chat/ModelIndicator.tsx
// Small indicator for chat header showing current AI model

import * as React from "react";
import { Cloud, Lock, AlertCircle, Settings, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { ProviderType, ProviderConfig } from "@/hooks/useAIProviders";
import { useActiveProvider } from "@/hooks/useAIProviders";
import { useLlmStatus } from "@/hooks/useLlmStatus";

/**
 * Extract model name from provider config
 */
function getModelFromConfig(config: ProviderConfig | null | undefined): string | undefined {
  if (!config) return undefined;

  if (config.provider === "local") {
    return config.model_path;
  }

  return config.model;
}

export interface ModelIndicatorProps {
  onOpenSettings?: () => void;
  className?: string;
  variant?: "default" | "compact";
}

/**
 * Get provider icon based on type
 */
function getProviderIcon(
  providerId: ProviderType | undefined,
  className?: string
): React.ReactNode {
  const iconClass = cn("h-3.5 w-3.5", className);

  if (!providerId) {
    return <AlertCircle className={iconClass} aria-hidden="true" />;
  }

  switch (providerId) {
    case "local":
      return <Lock className={iconClass} aria-hidden="true" />;
    case "openai":
    case "anthropic":
    case "google":
    case "openrouter":
      return <Cloud className={iconClass} aria-hidden="true" />;
    default:
      return <AlertCircle className={iconClass} aria-hidden="true" />;
  }
}

/**
 * Get status color and label
 */
function getStatusInfo(
  isAvailable: boolean,
  hasError: boolean,
  isLoading: boolean
): {
  color: string;
  bgColor: string;
  label: string;
} {
  if (isLoading) {
    return {
      color: "text-muted-foreground",
      bgColor: "bg-muted",
      label: "Checking...",
    };
  }

  if (hasError) {
    return {
      color: "text-destructive",
      bgColor: "bg-destructive/10",
      label: "Error",
    };
  }

  if (isAvailable) {
    return {
      color: "text-green-600 dark:text-green-400",
      bgColor: "bg-green-500/10",
      label: "Connected",
    };
  }

  return {
    color: "text-muted-foreground",
    bgColor: "bg-muted",
    label: "Not configured",
  };
}

/**
 * Format model name for display
 */
function formatModelName(modelId: string | undefined): string {
  if (!modelId) return "No model";

  // Remove common prefixes
  const cleaned = modelId
    .replace(/^(gpt-|claude-|gemini-|phi-)/i, "")
    .replace(/-latest$/i, "")
    .replace(/-\d{8}$/i, ""); // Remove date suffixes

  // Capitalize
  return cleaned
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

/**
 * ModelIndicator Component
 *
 * Displays current AI model and provider status in chat header with:
 * - Provider icon (lock for local, cloud for cloud providers)
 * - Model name
 * - Connection status badge
 * - Click to open settings
 * - Tooltip with detailed information
 *
 * @example
 * ```tsx
 * <ModelIndicator
 *   onOpenSettings={() => setSettingsOpen(true)}
 *   variant="default"
 * />
 * ```
 */
export function ModelIndicator({
  onOpenSettings,
  className,
  variant = "default",
}: ModelIndicatorProps) {
  const { data: activeProvider, isLoading: providerLoading } = useActiveProvider();
  const { status: llmStatus, isLoading: statusLoading } = useLlmStatus();

  const isLoading = providerLoading || statusLoading;
  const hasError = Boolean(llmStatus?.error);
  const isAvailable = Boolean(llmStatus?.available) && !hasError;

  const statusInfo = getStatusInfo(isAvailable, hasError, isLoading);

  // Compact variant - just icon and status dot
  if (variant === "compact") {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              onClick={onOpenSettings}
              className={cn("relative", className)}
              aria-label="AI model settings"
            >
              {getProviderIcon(activeProvider?.provider, statusInfo.color)}

              {/* Status indicator dot */}
              <span
                className={cn(
                  "absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full border-2 border-background",
                  isAvailable ? "bg-green-500" : hasError ? "bg-destructive" : "bg-muted-foreground"
                )}
                aria-hidden="true"
              />
            </Button>
          </TooltipTrigger>

          <TooltipContent side="bottom" className="max-w-xs">
            <div className="space-y-1">
              <p className="font-medium">{formatModelName(getModelFromConfig(activeProvider))}</p>
              <p className="text-xs text-muted-foreground">
                {activeProvider?.provider || "No provider"} - {statusInfo.label}
              </p>
              {hasError && llmStatus?.error && (
                <p className="text-xs text-destructive">{llmStatus.error}</p>
              )}
            </div>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  // Default variant - full badge
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onOpenSettings}
            className={cn("h-auto px-2.5 py-1.5 gap-2 hover:bg-accent", className)}
            aria-label="AI model settings - Click to configure"
          >
            {/* Provider icon */}
            <div
              className={cn(
                "rounded-md p-1 shrink-0",
                activeProvider?.provider === "local" ? "bg-primary/10" : "bg-blue-500/10"
              )}
            >
              {getProviderIcon(activeProvider?.provider, statusInfo.color)}
            </div>

            {/* Model info */}
            <div className="flex flex-col items-start min-w-0">
              <span className="text-xs font-medium truncate max-w-[120px]">
                {isLoading ? (
                  <span className="flex items-center gap-1.5">
                    <Loader2 className="h-3 w-3 animate-spin" aria-hidden="true" />
                    Loading...
                  </span>
                ) : (
                  formatModelName(getModelFromConfig(activeProvider))
                )}
              </span>

              {!isLoading && (
                <Badge
                  variant={isAvailable ? "default" : hasError ? "destructive" : "secondary"}
                  className={cn(
                    "text-[10px] px-1.5 py-0 h-4 mt-0.5",
                    isAvailable &&
                      "bg-green-500/10 text-green-700 dark:text-green-400 border-green-500/20"
                  )}
                >
                  {statusInfo.label}
                </Badge>
              )}
            </div>

            {/* Settings icon */}
            <Settings className="h-3.5 w-3.5 text-muted-foreground shrink-0" aria-hidden="true" />
          </Button>
        </TooltipTrigger>

        <TooltipContent side="bottom" align="start" className="max-w-xs">
          <div className="space-y-2">
            <div>
              <p className="font-medium">
                {formatModelName(getModelFromConfig(activeProvider)) || "No model configured"}
              </p>
              <p className="text-xs text-muted-foreground mt-0.5">
                Provider: {activeProvider?.provider || "None"}
              </p>
            </div>

            {llmStatus?.model && (
              <p className="text-xs text-muted-foreground">Model: {llmStatus.model}</p>
            )}

            {hasError && llmStatus?.error && (
              <p className="text-xs text-destructive">Error: {llmStatus.error}</p>
            )}

            {!hasError && !activeProvider && (
              <p className="text-xs text-muted-foreground italic">Click to configure AI provider</p>
            )}

            <p className="text-xs text-muted-foreground border-t pt-2 mt-2">
              {activeProvider?.provider === "local"
                ? "🔒 Private - runs on your device"
                : activeProvider?.provider
                  ? "☁️ Cloud - uses external API"
                  : "Configure to enable AI features"}
            </p>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

/**
 * Simple status-only indicator (no click action)
 */
export interface ModelStatusBadgeProps {
  className?: string;
}

export function ModelStatusBadge({ className }: ModelStatusBadgeProps) {
  const { data: activeProvider } = useActiveProvider();
  const { status: llmStatus } = useLlmStatus();

  const hasError = Boolean(llmStatus?.error);
  const isAvailable = Boolean(llmStatus?.available) && !hasError;

  return (
    <Badge
      variant={isAvailable ? "default" : hasError ? "destructive" : "secondary"}
      className={cn(
        "text-xs gap-1.5",
        isAvailable && "bg-green-500/10 text-green-700 dark:text-green-400 border-green-500/20",
        className
      )}
    >
      {getProviderIcon(activeProvider?.provider, "h-3 w-3")}
      <span>{formatModelName(getModelFromConfig(activeProvider))}</span>
    </Badge>
  );
}

export default ModelIndicator;
