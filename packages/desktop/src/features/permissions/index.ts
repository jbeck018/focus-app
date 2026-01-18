// features/permissions/index.ts - Main exports for permissions feature

export { SetupGuides, MacOSGuide, WindowsGuide, LinuxGuide, usePlatform } from "./setup-guides";
export { SetupGuidesModal, SetupGuidesButton } from "./setup-guides-modal";
export { PermissionStatusProvider } from "./permission-status-provider";
export { PermissionModal } from "./permission-modal";
export { DegradedModeBanner } from "./degraded-mode-banner";
export type { Platform } from "./setup-guides";
export type { PermissionStatus } from "./types";
