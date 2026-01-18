// features/permissions/types.ts - Permission types

export interface PermissionStatus {
  hosts_file_writable: boolean;
  hosts_file_error: string | null;
  process_monitoring_available: boolean;
  process_monitoring_error: string | null;
  overall_status: "fully_functional" | "degraded" | "non_functional";
}

export interface PermissionContextValue {
  permissionStatus: PermissionStatus | null;
  isLoading: boolean;
  hasFullPermissions: boolean;
  isDegraded: boolean;
  recheckPermissions: () => Promise<void>;
}
