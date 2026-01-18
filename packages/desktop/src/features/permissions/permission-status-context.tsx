// features/permissions/permission-status-context.tsx - Permission status context provider

import { createContext, useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PermissionStatus, PermissionContextValue } from "./types";

export const PermissionStatusContext = createContext<PermissionContextValue | undefined>(
  undefined
);

interface PermissionStatusProviderProps {
  children: React.ReactNode;
}

export function PermissionStatusProvider({ children }: PermissionStatusProviderProps) {
  const [permissionStatus, setPermissionStatus] = useState<PermissionStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const checkPermissions = useCallback(async () => {
    try {
      setIsLoading(true);
      const status = await invoke<PermissionStatus>("check_permissions");
      setPermissionStatus(status);
    } catch (error) {
      console.error("Failed to check permissions:", error);
      // Set a degraded state on error
      setPermissionStatus({
        hosts_file_writable: false,
        hosts_file_error: "Failed to check permissions",
        process_monitoring_available: false,
        process_monitoring_error: "Failed to check permissions",
        overall_status: "non_functional",
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  const recheckPermissions = useCallback(async () => {
    await checkPermissions();
  }, [checkPermissions]);

  // Check permissions on mount
  useEffect(() => {
    checkPermissions();
  }, [checkPermissions]);

  const hasFullPermissions =
    permissionStatus?.overall_status === "fully_functional" || false;
  const isDegraded =
    permissionStatus?.overall_status === "degraded" ||
    permissionStatus?.overall_status === "non_functional";

  const value: PermissionContextValue = {
    permissionStatus,
    isLoading,
    hasFullPermissions,
    isDegraded,
    recheckPermissions,
  };

  return (
    <PermissionStatusContext.Provider value={value}>
      {children}
    </PermissionStatusContext.Provider>
  );
}
