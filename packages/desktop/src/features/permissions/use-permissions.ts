// features/permissions/use-permissions.ts - Hook to access permission status

import { useContext } from "react";
import { PermissionStatusContext } from "./permission-status-context";

export function usePermissions() {
  const context = useContext(PermissionStatusContext);

  if (context === undefined) {
    throw new Error("usePermissions must be used within a PermissionStatusProvider");
  }

  return context;
}
