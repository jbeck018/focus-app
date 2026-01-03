// components/ai-settings/ProviderCard.tsx
// Reusable provider card showing status and actions

import * as React from "react";
import { Settings, CheckCircle2, Cloud, Lock, AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { ProviderInfo, ProviderType } from "@/hooks/useAIProviders";

export interface ProviderCardProps {
  provider: ProviderInfo;
  isActive?: boolean;
  onConfigure?: (providerId: ProviderType) => void;
  onActivate?: (providerId: ProviderType) => void;
  className?: string;
  disabled?: boolean;
}

/**
 * Get provider icon based on type
 */
function getProviderIcon(providerId: ProviderType): React.ReactNode {
  const iconClass = "h-5 w-5";

  switch (providerId) {
    case "local":
      return <Lock className={cn(iconClass, "text-primary")} aria-hidden="true" />;
    case "openai":
    case "anthropic":
    case "google":
    case "openrouter":
      return <Cloud className={cn(iconClass, "text-blue-500")} aria-hidden="true" />;
    default:
      return <Cloud className={cn(iconClass, "text-muted-foreground")} aria-hidden="true" />;
  }
}

/**
 * Get status badge variant and color
 */
function getStatusBadge(status: ProviderInfo["status"]) {
  switch (status) {
    case "connected":
      return {
        variant: "default" as const,
        className: "bg-green-500/10 text-green-700 dark:text-green-400 border-green-500/20",
        icon: <CheckCircle2 className="h-3 w-3" aria-hidden="true" />,
        label: "Connected",
      };
    case "disconnected":
      return {
        variant: "secondary" as const,
        className: "bg-muted text-muted-foreground",
        icon: null,
        label: "Not configured",
      };
    case "error":
      return {
        variant: "destructive" as const,
        className: "",
        icon: <AlertCircle className="h-3 w-3" aria-hidden="true" />,
        label: "Error",
      };
  }
}

/**
 * ProviderCard Component
 *
 * Displays provider information with:
 * - Provider icon and name
 * - Connection status badge
 * - Current model (if active)
 * - Configure and Activate buttons
 * - Error messages
 *
 * @example
 * ```tsx
 * <ProviderCard
 *   provider={providerInfo}
 *   isActive={activeProvider?.provider === providerInfo.id}
 *   onConfigure={(id) => setSelectedProvider(id)}
 *   onActivate={(id) => setProvider.mutate({ provider: id, modelId })}
 * />
 * ```
 */
export function ProviderCard({
  provider,
  isActive = false,
  onConfigure,
  onActivate,
  className,
  disabled = false,
}: ProviderCardProps) {
  const statusBadge = getStatusBadge(provider.status);
  const canActivate = provider.status === "connected" && !isActive;

  return (
    <Card
      className={cn(
        "transition-all duration-200",
        isActive && "border-primary bg-primary/5 shadow-md",
        disabled && "opacity-60 pointer-events-none",
        className
      )}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3 flex-1 min-w-0">
            <div
              className={cn(
                "rounded-lg p-2.5 shrink-0",
                provider.id === "local" ? "bg-primary/10" : "bg-blue-500/10"
              )}
            >
              {getProviderIcon(provider.id)}
            </div>

            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <CardTitle className="text-base leading-tight">{provider.name}</CardTitle>
                {isActive && (
                  <Badge variant="default" className="text-xs">
                    Active
                  </Badge>
                )}
              </div>

              <CardDescription className="text-xs line-clamp-2">
                {provider.description}
              </CardDescription>
            </div>
          </div>

          <Badge
            variant={statusBadge.variant}
            className={cn("shrink-0 text-xs", statusBadge.className)}
          >
            {statusBadge.icon}
            {statusBadge.label}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-3">
        {/* Current model */}
        {provider.currentModel && (
          <div className="text-xs text-muted-foreground">
            <span className="font-medium text-foreground">Model:</span> {provider.currentModel}
          </div>
        )}

        {/* Error message */}
        {provider.status === "error" && provider.errorMessage && (
          <div className="flex items-start gap-2 p-2 rounded-md bg-destructive/10 border border-destructive/20">
            <AlertCircle className="h-4 w-4 text-destructive shrink-0 mt-0.5" aria-hidden="true" />
            <p className="text-xs text-destructive flex-1">{provider.errorMessage}</p>
          </div>
        )}

        {/* Requires API key note */}
        {provider.requiresApiKey && provider.status === "disconnected" && (
          <p className="text-xs text-muted-foreground italic">
            API key required to use this provider
          </p>
        )}

        {/* Action buttons */}
        <div className="flex items-center gap-2 pt-1">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onConfigure?.(provider.id)}
            className="flex-1"
          >
            <Settings className="h-4 w-4" aria-hidden="true" />
            Configure
          </Button>

          {canActivate && onActivate && (
            <Button
              type="button"
              variant="default"
              size="sm"
              onClick={() => onActivate(provider.id)}
              className="flex-1"
            >
              Use This Provider
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

/**
 * Compact version for smaller displays
 */
export interface ProviderCardCompactProps {
  provider: ProviderInfo;
  isActive?: boolean;
  onClick?: (providerId: ProviderType) => void;
  className?: string;
}

export function ProviderCardCompact({
  provider,
  isActive = false,
  onClick,
  className,
}: ProviderCardCompactProps) {
  const statusBadge = getStatusBadge(provider.status);

  return (
    <button
      type="button"
      onClick={() => onClick?.(provider.id)}
      className={cn(
        "flex items-center gap-3 w-full p-3 rounded-lg border transition-all",
        "hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        isActive && "border-primary bg-primary/5",
        className
      )}
    >
      <div
        className={cn(
          "rounded-md p-2 shrink-0",
          provider.id === "local" ? "bg-primary/10" : "bg-blue-500/10"
        )}
      >
        {getProviderIcon(provider.id)}
      </div>

      <div className="flex-1 min-w-0 text-left">
        <div className="flex items-center gap-2 mb-0.5">
          <p className="text-sm font-medium truncate">{provider.name}</p>
          {isActive && (
            <Badge variant="default" className="text-xs px-1.5 py-0">
              Active
            </Badge>
          )}
        </div>
        <p className="text-xs text-muted-foreground truncate">
          {provider.currentModel || provider.description}
        </p>
      </div>

      <Badge
        variant={statusBadge.variant}
        className={cn("text-xs shrink-0", statusBadge.className)}
      >
        {statusBadge.icon}
      </Badge>
    </button>
  );
}

export default ProviderCard;
