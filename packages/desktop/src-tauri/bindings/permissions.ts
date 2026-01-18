// TypeScript bindings for permission checking commands
// Auto-generated type definitions for the Rust permissions module

/**
 * Overall permission status categorization
 */
export type OverallPermissionStatus =
  | "fully_functional"  // All blocking features work perfectly
  | "degraded"          // Some features work, but not all
  | "non_functional";   // No privileged features work

/**
 * Comprehensive permission status for all blocking capabilities
 */
export interface PermissionStatus {
  /** Can we modify the hosts file for website blocking? */
  hosts_file_writable: boolean;

  /** Detailed error if hosts file is not writable */
  hosts_file_error: string | null;

  /** Path to the hosts file for reference */
  hosts_file_path: string;

  /** Can we enumerate and monitor running processes? */
  process_monitoring_available: boolean;

  /** Detailed error if process monitoring is not available */
  process_monitoring_error: string | null;

  /** Can we terminate blocked processes? */
  process_termination_available: boolean;

  /** Detailed error if process termination is not available */
  process_termination_error: string | null;

  /** Overall assessment of the blocking system's functionality */
  overall_status: OverallPermissionStatus;

  /** Recommendations for the user */
  recommendations: string[];

  /** Current platform */
  platform: string;
}

/**
 * A specific method for granting permissions
 */
export interface PermissionMethod {
  /** Name of the method (e.g., "Full Disk Access", "Run as Administrator") */
  name: string;

  /** Step-by-step instructions */
  steps: string[];

  /** Whether this is a permanent solution or temporary */
  is_permanent: boolean;

  /** Whether this method is recommended (vs alternative) */
  is_recommended: boolean;

  /** Specific capability this method grants */
  grants: string[];
}

/**
 * Platform-specific instructions for granting permissions
 */
export interface PlatformInstructions {
  /** Operating system name */
  platform: string;

  /** Primary recommended method for granting permissions */
  primary_method: PermissionMethod;

  /** Alternative methods available */
  alternative_methods: PermissionMethod[];

  /** Whether app needs restart after granting permissions */
  requires_restart: boolean;

  /** General security notes about the permissions */
  security_notes: string[];
}

/**
 * Tauri command bindings
 */
export namespace Commands {
  /**
   * Check all permissions and return detailed status
   *
   * This command provides a comprehensive overview of what blocking features
   * are currently available based on the app's permissions.
   *
   * @returns Promise resolving to detailed permission status
   * @example
   * ```typescript
   * const status = await checkPermissions();
   * if (status.overall_status === "fully_functional") {
   *   console.log("All blocking features are available!");
   * } else {
   *   console.log("Recommendations:", status.recommendations);
   * }
   * ```
   */
  export function checkPermissions(): Promise<PermissionStatus> {
    // @ts-expect-error - Tauri command will be injected
    return window.__TAURI__.core.invoke("check_permissions");
  }

  /**
   * Get platform-specific instructions for granting permissions
   *
   * Returns detailed, step-by-step instructions tailored to the user's
   * operating system. Pass an empty string to auto-detect platform.
   *
   * @param platform Platform name ("macos", "windows", "linux", or "" for auto-detect)
   * @returns Promise resolving to platform-specific instructions
   * @example
   * ```typescript
   * // Auto-detect platform
   * const instructions = await getPermissionInstructions("");
   * console.log("Primary method:", instructions.primary_method.name);
   * console.log("Steps:", instructions.primary_method.steps);
   *
   * // Or specify platform explicitly
   * const macInstructions = await getPermissionInstructions("macos");
   * ```
   */
  export function getPermissionInstructions(platform: string): Promise<PlatformInstructions> {
    // @ts-expect-error - Tauri command will be injected
    return window.__TAURI__.core.invoke("get_permission_instructions", { platform });
  }
}

/**
 * Helper functions for working with permissions
 */
export namespace PermissionHelpers {
  /**
   * Check if all permissions are granted
   */
  export function isFullyFunctional(status: PermissionStatus): boolean {
    return status.overall_status === "fully_functional";
  }

  /**
   * Check if any permissions are missing
   */
  export function needsPermissions(status: PermissionStatus): boolean {
    return status.overall_status !== "fully_functional";
  }

  /**
   * Get a human-readable status message
   */
  export function getStatusMessage(status: PermissionStatus): string {
    switch (status.overall_status) {
      case "fully_functional":
        return "All blocking features are available and working correctly.";
      case "degraded":
        return "Some blocking features are available, but full functionality requires additional permissions.";
      case "non_functional":
        return "Blocking features require elevated permissions to function. Please follow the setup instructions.";
    }
  }

  /**
   * Get the most important missing permission
   */
  export function getPrimaryIssue(status: PermissionStatus): string | null {
    if (!status.hosts_file_writable && status.hosts_file_error) {
      return `Hosts file not writable: ${status.hosts_file_error}`;
    }
    if (!status.process_monitoring_available && status.process_monitoring_error) {
      return `Process monitoring unavailable: ${status.process_monitoring_error}`;
    }
    if (!status.process_termination_available && status.process_termination_error) {
      return `Process termination unavailable: ${status.process_termination_error}`;
    }
    return null;
  }

  /**
   * Format permission method steps as HTML
   */
  export function formatStepsAsHTML(method: PermissionMethod): string {
    return `
      <div class="permission-method">
        <h3>${method.name}</h3>
        <p class="method-type">${method.is_permanent ? "Permanent" : "Temporary"} solution</p>
        <ol>
          ${method.steps.map(step => `<li>${step}</li>`).join("\n")}
        </ol>
        <div class="grants">
          <strong>Grants:</strong>
          <ul>
            ${method.grants.map(grant => `<li>${grant}</li>`).join("\n")}
          </ul>
        </div>
      </div>
    `;
  }
}

/**
 * React hooks for permission checking (optional)
 */
export namespace ReactHooks {
  /**
   * Example React hook for permission checking
   *
   * @example
   * ```typescript
   * function PermissionStatus() {
   *   const [status, setStatus] = useState<PermissionStatus | null>(null);
   *   const [loading, setLoading] = useState(true);
   *
   *   useEffect(() => {
   *     Commands.checkPermissions()
   *       .then(setStatus)
   *       .finally(() => setLoading(false));
   *   }, []);
   *
   *   if (loading) return <div>Checking permissions...</div>;
   *   if (!status) return <div>Error loading permissions</div>;
   *
   *   return (
   *     <div>
   *       <h2>Status: {status.overall_status}</h2>
   *       {PermissionHelpers.needsPermissions(status) && (
   *         <div>
   *           <h3>Recommendations:</h3>
   *           <ul>
   *             {status.recommendations.map((rec, i) => <li key={i}>{rec}</li>)}
   *           </ul>
   *         </div>
   *       )}
   *     </div>
   *   );
   * }
   * ```
   */
  export function usePermissionStatus() {
    // Implementation would go here in actual React codebase
    throw new Error("This is an example hook signature - implement in your React app");
  }
}
