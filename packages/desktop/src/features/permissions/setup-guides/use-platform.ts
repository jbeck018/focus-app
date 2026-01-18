// features/permissions/setup-guides/use-platform.ts - Platform detection hook

import { useEffect, useState } from "react";
import { platform } from "@tauri-apps/plugin-os";

export type Platform = "macos" | "windows" | "linux";

// Helper hook for detecting platform
export function usePlatform() {
  const [detectedPlatform, setDetectedPlatform] = useState<Platform | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    function detectPlatform() {
      try {
        const platformName = platform();

        let detected: Platform;
        switch (platformName) {
          case "macos":
          case "ios":
            detected = "macos";
            break;
          case "windows":
            detected = "windows";
            break;
          case "linux":
          case "android":
            detected = "linux";
            break;
          default:
            detected = "linux";
        }

        setDetectedPlatform(detected);
      } catch (error) {
        console.error("Failed to detect platform:", error);
        setDetectedPlatform(null);
      } finally {
        setIsLoading(false);
      }
    }

    detectPlatform();
  }, []);

  return { platform: detectedPlatform, isLoading };
}
