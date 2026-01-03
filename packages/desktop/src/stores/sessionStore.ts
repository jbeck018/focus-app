// stores/sessionStore.ts - Zustand store for active focus session state
// Timer state (elapsed, paused) is owned by Rust backend for cross-window sync
// Session counts are owned by authStore (synced from backend)

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { ActiveSession } from "@focusflow/types";

interface SessionState {
  // Current session (UI state only - backend is source of truth)
  activeSession: ActiveSession | null;

  // Actions
  startSession: (session: ActiveSession) => void;
  endSession: () => void;
  updateSessionDuration: (newDurationMinutes: number) => void;
}

export const useSessionStore = create<SessionState>()(
  devtools(
    (set) => ({
      activeSession: null,

      startSession: (session) =>
        set({
          activeSession: session,
        }),

      endSession: () =>
        set({
          activeSession: null,
        }),

      updateSessionDuration: (newDurationMinutes) =>
        set((state) => ({
          activeSession: state.activeSession
            ? { ...state.activeSession, plannedDurationMinutes: newDurationMinutes }
            : null,
        })),
    }),
    { name: "session-store" }
  )
);

// Selector hooks for performance
export const useActiveSession = () => useSessionStore((state) => state.activeSession);
