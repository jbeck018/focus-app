// features/permissions/degraded-mode-banner.tsx - Persistent banner for degraded permissions

import { useState, useEffect } from "react";
import { AlertTriangle, XCircle, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import { usePermissions } from "./use-permissions";
import { cn } from "@/lib/utils";

interface DegradedModeBannerProps {
  onFixClick?: () => void;
}

export function DegradedModeBanner({ onFixClick }: DegradedModeBannerProps) {
  const { permissionStatus, isDegraded } = usePermissions();
  const [isVisible, setIsVisible] = useState(false);

  // Animate in after a short delay when degraded, otherwise reset visibility
  useEffect(() => {
    if (!isDegraded) {
      // Reset visibility when no longer degraded - use timer to batch state update
      const timer = setTimeout(() => setIsVisible(false), 0);
      return () => clearTimeout(timer);
    }

    // Animate in after a short delay when degraded
    const timer = setTimeout(() => setIsVisible(true), 300);
    return () => clearTimeout(timer);
  }, [isDegraded]);

  // Don't render if permissions are fully functional
  if (!permissionStatus || !isDegraded) {
    return null;
  }

  const isNonFunctional = permissionStatus.overall_status === "non_functional";
  const isDegradedOnly = permissionStatus.overall_status === "degraded";

  // Determine what's not working
  const getMissingFeatures = () => {
    const missing: string[] = [];
    if (!permissionStatus.hosts_file_writable) {
      missing.push("website blocking");
    }
    if (!permissionStatus.process_monitoring_available) {
      missing.push("app blocking");
    }
    return missing;
  };

  const missingFeatures = getMissingFeatures();

  return (
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className={cn(
        "fixed bottom-4 left-4 right-4 z-50 transition-all duration-500 ease-out",
        isVisible ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0 pointer-events-none"
      )}
    >
      <div
        className={cn(
          "mx-auto max-w-2xl rounded-lg border shadow-lg backdrop-blur-sm transition-colors",
          isNonFunctional
            ? "bg-destructive/10 border-destructive/50 dark:bg-destructive/20"
            : "bg-amber-500/10 border-amber-500/50 dark:bg-amber-500/20"
        )}
      >
        <div className="flex items-start gap-3 p-4">
          {/* Icon */}
          <div className="shrink-0 mt-0.5">
            {isNonFunctional ? (
              <XCircle className="h-5 w-5 text-destructive" aria-hidden="true" />
            ) : (
              <AlertTriangle
                className="h-5 w-5 text-amber-600 dark:text-amber-500"
                aria-hidden="true"
              />
            )}
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0 space-y-1">
            <p
              className={cn(
                "font-semibold text-sm",
                isNonFunctional
                  ? "text-destructive dark:text-red-400"
                  : "text-amber-900 dark:text-amber-300"
              )}
            >
              {isNonFunctional
                ? "Blocking features are unavailable"
                : "Blocking features are limited"}
            </p>
            <p
              className={cn(
                "text-sm",
                isNonFunctional
                  ? "text-destructive/90 dark:text-red-400/90"
                  : "text-amber-800 dark:text-amber-400"
              )}
            >
              Missing permissions for {missingFeatures.join(" and ")}
              {permissionStatus.hosts_file_error && (
                <span className="block mt-1 text-xs opacity-80">
                  {permissionStatus.hosts_file_error}
                </span>
              )}
            </p>
          </div>

          {/* Action Button */}
          <div className="shrink-0">
            <Button
              size="sm"
              variant={isNonFunctional ? "destructive" : "default"}
              onClick={onFixClick}
              className={cn(
                "shadow-sm",
                isDegradedOnly &&
                  "bg-amber-600 hover:bg-amber-700 text-white dark:bg-amber-500 dark:hover:bg-amber-600"
              )}
            >
              <Settings className="h-4 w-4 mr-2" aria-hidden="true" />
              Fix This
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
