// components/ai-settings/ModelDownloadProgress.tsx
// Model download progress component with progress bar and controls

import * as React from "react";
import { Download, X, Clock, HardDrive, AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import type { DownloadProgress } from "@/hooks/useAIProviders";

export interface ModelDownloadProgressProps {
  progress?: DownloadProgress;
  error?: string;
  isDownloading?: boolean;
  modelName?: string;
  onCancel?: () => void;
  onRetry?: () => void;
  className?: string;
}

/**
 * Format bytes to human-readable format
 */
function formatBytes(mb: number): string {
  if (mb < 1024) {
    return `${mb.toFixed(1)} MB`;
  }
  return `${(mb / 1024).toFixed(2)} GB`;
}

/**
 * Calculate estimated time remaining
 */
function calculateETA(downloadedMb: number, totalMb: number, startTime: number): string {
  const elapsed = Date.now() - startTime;
  const progress = downloadedMb / totalMb;

  if (progress === 0) return "Calculating...";

  const totalTime = elapsed / progress;
  const remaining = totalTime - elapsed;

  const seconds = Math.floor(remaining / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `~${hours}h ${minutes % 60}m remaining`;
  }
  if (minutes > 0) {
    return `~${minutes}m ${seconds % 60}s remaining`;
  }
  return `~${seconds}s remaining`;
}

/**
 * ModelDownloadProgress Component
 *
 * Displays download progress with:
 * - Animated progress bar
 * - Download speed and size information
 * - Estimated time remaining
 * - Cancel button
 * - Error state with retry option
 *
 * @example
 * ```tsx
 * const { data: status } = useModelDownloadStatus();
 *
 * return (
 *   <ModelDownloadProgress
 *     progress={status?.progress}
 *     error={status?.error}
 *     isDownloading={status?.isDownloading}
 *     modelName={status?.modelName}
 *     onCancel={() => invoke("cancel_download")}
 *     onRetry={() => invoke("download_model", { modelName: status.modelName })}
 *   />
 * );
 * ```
 */
export function ModelDownloadProgress({
  progress,
  error,
  isDownloading = false,
  modelName,
  onCancel,
  onRetry,
  className,
}: ModelDownloadProgressProps) {
  const [startTime] = React.useState(() => Date.now());
  const [eta, setEta] = React.useState<string>("Calculating...");

  // Update ETA every second
  React.useEffect(() => {
    if (progress && isDownloading) {
      const interval = setInterval(() => {
        setEta(calculateETA(progress.downloadedMb, progress.totalMb, startTime));
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [progress, isDownloading, startTime]);

  // Show error state
  if (error) {
    return (
      <Alert variant="destructive" className={className}>
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>Download Failed</AlertTitle>
        <AlertDescription className="space-y-2">
          <p className="text-sm">{error}</p>
          {onRetry && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={onRetry}
              className="mt-2"
            >
              Retry Download
            </Button>
          )}
        </AlertDescription>
      </Alert>
    );
  }

  // Show downloading state
  if (isDownloading && progress) {
    const percentComplete = Math.round(progress.percent);
    const downloadedFormatted = formatBytes(progress.downloadedMb);
    const totalFormatted = formatBytes(progress.totalMb);

    return (
      <Card className={cn("border-primary/20 bg-primary/5", className)}>
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between gap-4">
            <div className="flex items-start gap-3 flex-1 min-w-0">
              <div className="rounded-lg bg-primary/10 p-2 shrink-0">
                <Download className="h-4 w-4 text-primary animate-pulse" aria-hidden="true" />
              </div>
              <div className="flex-1 min-w-0">
                <CardTitle className="text-base leading-tight">
                  Downloading {progress.modelName || modelName}
                </CardTitle>
                <p className="text-xs text-muted-foreground mt-1">
                  This may take a few minutes depending on your connection
                </p>
              </div>
            </div>

            {onCancel && (
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                onClick={onCancel}
                aria-label="Cancel download"
                className="shrink-0"
              >
                <X className="h-4 w-4" aria-hidden="true" />
              </Button>
            )}
          </div>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Progress bar */}
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium text-foreground" aria-live="polite">
                {percentComplete}%
              </span>
              <span className="text-muted-foreground text-xs">
                {downloadedFormatted} / {totalFormatted}
              </span>
            </div>

            <Progress
              value={progress.percent}
              className="h-2"
              aria-label={`Download progress: ${percentComplete}%`}
            />
          </div>

          {/* Download stats */}
          <div className="grid grid-cols-2 gap-3 pt-2">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Clock className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
              <span className="truncate">{eta}</span>
            </div>

            <div className="flex items-center gap-2 text-xs text-muted-foreground justify-end">
              <HardDrive className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
              <span className="truncate">
                {progress.downloadedMb > 0 && Date.now() - startTime > 0
                  ? ((progress.downloadedMb / ((Date.now() - startTime) / 1000))).toFixed(1)
                  : "0.0"}{" "}
                MB/s
              </span>
            </div>
          </div>
        </CardContent>

        {/* Accessibility: Live region for screen readers */}
        <div className="sr-only" role="status" aria-live="polite" aria-atomic="true">
          Downloading {progress.modelName || modelName}: {percentComplete}% complete. {eta}
        </div>
      </Card>
    );
  }

  // No active download or error
  return null;
}

/**
 * Compact version for inline display
 */
export interface ModelDownloadProgressInlineProps {
  progress: DownloadProgress;
  className?: string;
}

export function ModelDownloadProgressInline({
  progress,
  className,
}: ModelDownloadProgressInlineProps) {
  const percentComplete = Math.round(progress.percent);

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <Download className="h-4 w-4 text-primary animate-pulse shrink-0" aria-hidden="true" />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-sm font-medium truncate">
            Downloading {progress.modelName}
          </span>
          <span className="text-xs text-muted-foreground shrink-0">
            {percentComplete}%
          </span>
        </div>
        <Progress
          value={progress.percent}
          className="h-1"
          aria-label={`Download progress: ${percentComplete}%`}
        />
      </div>
    </div>
  );
}

export default ModelDownloadProgress;
