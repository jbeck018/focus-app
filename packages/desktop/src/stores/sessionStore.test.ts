// tests for sessionStore
// Note: Timer state (elapsed, paused) is now owned by Rust backend
// Session counts are now owned by authStore (synced from backend)

import { describe, it, expect, beforeEach } from "vitest";
import { useSessionStore } from "./sessionStore";
import type { ActiveSession } from "@focusflow/types";

describe("sessionStore", () => {
  beforeEach(() => {
    // Reset the store before each test
    useSessionStore.setState({
      activeSession: null,
    });
  });

  it("starts with null session", () => {
    const state = useSessionStore.getState();
    expect(state.activeSession).toBeNull();
  });

  it("starts a new session", () => {
    const session: ActiveSession = {
      id: "test-id",
      startTime: "2024-01-01T10:00:00Z",
      plannedDurationMinutes: 25,
      sessionType: "focus",
      blockedApps: [],
      blockedWebsites: [],
    };

    useSessionStore.getState().startSession(session);

    const state = useSessionStore.getState();
    expect(state.activeSession).toEqual(session);
  });

  it("ends a session", () => {
    const session: ActiveSession = {
      id: "test-id",
      startTime: "2024-01-01T10:00:00Z",
      plannedDurationMinutes: 25,
      sessionType: "focus",
      blockedApps: [],
      blockedWebsites: [],
    };

    useSessionStore.getState().startSession(session);
    useSessionStore.getState().endSession();

    const state = useSessionStore.getState();
    expect(state.activeSession).toBeNull();
  });

  it("updates session duration", () => {
    const session: ActiveSession = {
      id: "test-id",
      startTime: "2024-01-01T10:00:00Z",
      plannedDurationMinutes: 25,
      sessionType: "focus",
      blockedApps: [],
      blockedWebsites: [],
    };

    useSessionStore.getState().startSession(session);
    useSessionStore.getState().updateSessionDuration(30);

    const state = useSessionStore.getState();
    expect(state.activeSession?.plannedDurationMinutes).toBe(30);
  });
});
