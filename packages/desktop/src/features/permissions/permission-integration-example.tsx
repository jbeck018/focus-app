// features/permissions/permission-integration-example.tsx
// Example of how to integrate DegradedModeBanner with PermissionModal

import { useState } from "react";
import { DegradedModeBanner } from "./degraded-mode-banner";
import { PermissionModal } from "./permission-modal";

/**
 * Example integration showing how to wire up the banner and modal together.
 *
 * Usage in your App.tsx:
 *
 * ```tsx
 * import { PermissionIntegration } from "@/features/permissions/permission-integration-example";
 *
 * function App() {
 *   return (
 *     <PermissionStatusProvider>
 *       <YourApp />
 *       <PermissionIntegration />
 *     </PermissionStatusProvider>
 *   );
 * }
 * ```
 */
export function PermissionIntegration() {
  const [showPermissionModal, setShowPermissionModal] = useState(false);

  return (
    <>
      {/* Persistent banner shown when permissions are degraded */}
      <DegradedModeBanner onFixClick={() => setShowPermissionModal(true)} />

      {/* Permission modal for fixing issues */}
      <PermissionModal open={showPermissionModal} onOpenChange={setShowPermissionModal} />
    </>
  );
}
