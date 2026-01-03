// hooks/useKeyboardShortcuts.ts - Global keyboard shortcut handler

import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ShortcutConfig {
  key: string;
  ctrlOrCmd?: boolean;
  shift?: boolean;
  alt?: boolean;
  callback: () => void;
}

/**
 * Hook to register global keyboard shortcuts
 *
 * Handles cross-platform keyboard shortcuts with proper modifier key detection
 * (Cmd on macOS, Ctrl on Windows/Linux)
 */
export function useKeyboardShortcuts(shortcuts: ShortcutConfig[]) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        const modifierMatch =
          (!shortcut.ctrlOrCmd || event.ctrlKey || event.metaKey) &&
          (!shortcut.shift || event.shiftKey) &&
          (!shortcut.alt || event.altKey);

        const keyMatch = event.key.toLowerCase() === shortcut.key.toLowerCase();

        if (modifierMatch && keyMatch) {
          event.preventDefault();
          shortcut.callback();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [shortcuts]);
}

/**
 * Hook specifically for toggling the mini-timer window
 * Uses Cmd/Ctrl+Shift+M
 */
export function useMiniTimerShortcut() {
  useKeyboardShortcuts([
    {
      key: "m",
      ctrlOrCmd: true,
      shift: true,
      callback: () => {
        invoke("toggle_mini_timer").catch((error: unknown) => {
          console.error("Failed to toggle mini-timer:", error);
        });
      },
    },
  ]);
}
