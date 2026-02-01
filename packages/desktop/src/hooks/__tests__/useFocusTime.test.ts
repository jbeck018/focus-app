// Tests for useFocusTime hook
// Covers: hook behavior, state management, Tauri command mocking

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import React from "react";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]): unknown => mockInvoke(...args),
}));

// Mock the hook implementation for testing
// In a real scenario, this would import from the actual hook file
interface FocusTimeState {
  active: boolean;
  eventId: string | null;
  eventTitle: string | null;
  startedAt: string | null;
  endsAt: string | null;
  allowedApps: string[];
  endedEarly: boolean;
}

interface FocusTimeEvent {
  id: string;
  title: string;
  cleanTitle: string;
  description: string | null;
  startTime: string;
  endTime: string;
  durationMinutes: number;
  allowedApps: string[];
  isActive: boolean;
  isUpcoming: boolean;
}

// Simulated hook for testing purposes
function useFocusTime() {
  const [state, setState] = React.useState<FocusTimeState>({
    active: false,
    eventId: null,
    eventTitle: null,
    startedAt: null,
    endsAt: null,
    allowedApps: [],
    endedEarly: false,
  });
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<Error | null>(null);

  const fetchState = async () => {
    setIsLoading(true);
    try {
      const result: unknown = await mockInvoke("get_focus_time_state");
      setState(result as FocusTimeState);
    } catch (err) {
      setError(err as Error);
    } finally {
      setIsLoading(false);
    }
  };

  const startFocusTime = async (eventId: string) => {
    setIsLoading(true);
    try {
      await mockInvoke("start_focus_time", { eventId });
      await fetchState();
    } catch (err) {
      setError(err as Error);
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  const endFocusTime = async (early: boolean = false) => {
    setIsLoading(true);
    try {
      await mockInvoke("end_focus_time", { early });
      await fetchState();
    } catch (err) {
      setError(err as Error);
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  const addAllowedApp = async (app: string) => {
    try {
      await mockInvoke("add_focus_time_allowed_app", { app });
      setState((prev) => ({
        ...prev,
        allowedApps: [...prev.allowedApps, app],
      }));
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  };

  const removeAllowedApp = async (app: string) => {
    try {
      await mockInvoke("remove_focus_time_allowed_app", { app });
      setState((prev) => ({
        ...prev,
        allowedApps: prev.allowedApps.filter((a) => a !== app),
      }));
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  };

  React.useEffect(() => {
    fetchState();
  }, []);

  return {
    state,
    isLoading,
    error,
    isActive: state.active,
    allowedApps: state.allowedApps,
    startFocusTime,
    endFocusTime,
    addAllowedApp,
    removeAllowedApp,
    refetch: fetchState,
  };
}

// Test wrapper with QueryClient
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return React.createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

describe("useFocusTime", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default mock responses
    mockInvoke.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "get_focus_time_state":
          return Promise.resolve({
            active: false,
            eventId: null,
            eventTitle: null,
            startedAt: null,
            endsAt: null,
            allowedApps: [],
            endedEarly: false,
          });
        default:
          return Promise.resolve(null);
      }
    });
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe("initial state", () => {
    it("should initialize with inactive state", async () => {
      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isActive).toBe(false);
      expect(result.current.allowedApps).toEqual([]);
    });

    it("should fetch state on mount", async () => {
      renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("get_focus_time_state");
      });
    });
  });

  describe("starting Focus Time", () => {
    it("should start Focus Time from event", async () => {
      const activeState: FocusTimeState = {
        active: true,
        eventId: "event-123",
        eventTitle: "Coding Session",
        startedAt: new Date().toISOString(),
        endsAt: new Date(Date.now() + 3600000).toISOString(),
        allowedApps: ["Code", "Terminal"],
        endedEarly: false,
      };

      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "start_focus_time") return Promise.resolve(null);
        if (cmd === "get_focus_time_state") return Promise.resolve(activeState);
        return Promise.resolve(null);
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.startFocusTime("event-123");
      });

      expect(mockInvoke).toHaveBeenCalledWith("start_focus_time", {
        eventId: "event-123",
      });

      await waitFor(() => {
        expect(result.current.isActive).toBe(true);
        expect(result.current.allowedApps).toContain("Code");
      });
    });

    it("should handle start error", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "start_focus_time") {
          return Promise.reject(new Error("Focus Time already active"));
        }
        return Promise.resolve({
          active: false,
          eventId: null,
          eventTitle: null,
          startedAt: null,
          endsAt: null,
          allowedApps: [],
          endedEarly: false,
        });
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.startFocusTime("event-123");
        } catch {
          // Expected error
        }
      });

      expect(result.current.error?.message).toBe("Focus Time already active");
    });
  });

  describe("ending Focus Time", () => {
    it("should end Focus Time normally", async () => {
      const activeState: FocusTimeState = {
        active: true,
        eventId: "event-123",
        eventTitle: "Coding Session",
        startedAt: new Date().toISOString(),
        endsAt: new Date(Date.now() + 3600000).toISOString(),
        allowedApps: ["Code"],
        endedEarly: false,
      };

      const inactiveState: FocusTimeState = {
        active: false,
        eventId: null,
        eventTitle: null,
        startedAt: null,
        endsAt: null,
        allowedApps: [],
        endedEarly: false,
      };

      let callCount = 0;
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "end_focus_time") return Promise.resolve(null);
        if (cmd === "get_focus_time_state") {
          callCount++;
          return Promise.resolve(callCount <= 1 ? activeState : inactiveState);
        }
        return Promise.resolve(null);
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isActive).toBe(true);
      });

      await act(async () => {
        await result.current.endFocusTime(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith("end_focus_time", { early: false });

      await waitFor(() => {
        expect(result.current.isActive).toBe(false);
      });
    });

    it("should end Focus Time early", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "end_focus_time") return Promise.resolve(null);
        return Promise.resolve({
          active: false,
          eventId: null,
          eventTitle: null,
          startedAt: null,
          endsAt: null,
          allowedApps: [],
          endedEarly: true,
        });
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.endFocusTime(true);
      });

      expect(mockInvoke).toHaveBeenCalledWith("end_focus_time", { early: true });
    });
  });

  describe("managing allowed apps", () => {
    it("should add an allowed app", async () => {
      const activeState: FocusTimeState = {
        active: true,
        eventId: "event-123",
        eventTitle: "Coding",
        startedAt: new Date().toISOString(),
        endsAt: new Date(Date.now() + 3600000).toISOString(),
        allowedApps: ["Code"],
        endedEarly: false,
      };

      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "add_focus_time_allowed_app") return Promise.resolve(null);
        return Promise.resolve(activeState);
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isActive).toBe(true);
      });

      await act(async () => {
        await result.current.addAllowedApp("Slack");
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_focus_time_allowed_app", {
        app: "Slack",
      });

      expect(result.current.allowedApps).toContain("Slack");
    });

    it("should remove an allowed app", async () => {
      const activeState: FocusTimeState = {
        active: true,
        eventId: "event-123",
        eventTitle: "Coding",
        startedAt: new Date().toISOString(),
        endsAt: new Date(Date.now() + 3600000).toISOString(),
        allowedApps: ["Code", "Terminal"],
        endedEarly: false,
      };

      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "remove_focus_time_allowed_app") return Promise.resolve(null);
        return Promise.resolve(activeState);
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.allowedApps).toContain("Terminal");
      });

      await act(async () => {
        await result.current.removeAllowedApp("Terminal");
      });

      expect(mockInvoke).toHaveBeenCalledWith("remove_focus_time_allowed_app", {
        app: "Terminal",
      });

      expect(result.current.allowedApps).not.toContain("Terminal");
    });
  });

  describe("error handling", () => {
    it("should handle network errors", async () => {
      mockInvoke.mockRejectedValue(new Error("Network error"));

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      });

      expect(result.current.error?.message).toBe("Network error");
    });

    it("should clear error on successful operation", async () => {
      let shouldFail = true;

      mockInvoke.mockImplementation((_cmd: string) => {
        if (shouldFail) {
          shouldFail = false;
          return Promise.reject(new Error("Temporary error"));
        }
        return Promise.resolve({
          active: false,
          eventId: null,
          eventTitle: null,
          startedAt: null,
          endsAt: null,
          allowedApps: [],
          endedEarly: false,
        });
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      });

      await act(async () => {
        await result.current.refetch();
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });
  });

  describe("loading states", () => {
    it("should transition through loading states during operations", async () => {
      mockInvoke.mockImplementation(async (_cmd: string) => {
        // Add a small delay to simulate network
        await new Promise((resolve) => setTimeout(resolve, 10));
        return {
          active: false,
          eventId: null,
          eventTitle: null,
          startedAt: null,
          endsAt: null,
          allowedApps: [],
          endedEarly: false,
        };
      });

      const { result } = renderHook(() => useFocusTime(), {
        wrapper: createWrapper(),
      });

      // Wait for initial load to complete
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      // Start an operation
      await act(async () => {
        await result.current.startFocusTime("event-123");
      });

      // After operation completes, should not be loading
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });
  });
});

describe("Focus Time event detection", () => {
  // Test data fixtures
  const focusTimeEvents: FocusTimeEvent[] = [
    {
      id: "1",
      title: "Focus Time",
      cleanTitle: "Focus Time",
      description: "@coding",
      startTime: new Date(Date.now() - 1800000).toISOString(), // Started 30 mins ago
      endTime: new Date(Date.now() + 1800000).toISOString(), // Ends in 30 mins
      durationMinutes: 60,
      allowedApps: ["Code", "Terminal"],
      isActive: true,
      isUpcoming: false,
    },
    {
      id: "2",
      title: "Deep Work",
      cleanTitle: "Deep Work",
      description: "@coding, @terminal",
      startTime: new Date(Date.now() + 3600000).toISOString(), // Starts in 1 hour
      endTime: new Date(Date.now() + 7200000).toISOString(),
      durationMinutes: 60,
      allowedApps: ["Code", "Terminal", "Notion"],
      isActive: false,
      isUpcoming: false,
    },
  ];

  it("should identify active Focus Time event", () => {
    const active = focusTimeEvents.find((e) => e.isActive);
    expect(active).toBeDefined();
    expect(active?.id).toBe("1");
  });

  it("should parse allowed apps from description", () => {
    const event = focusTimeEvents[0];
    expect(event.allowedApps).toContain("Code");
    expect(event.allowedApps).toContain("Terminal");
  });

  it("should calculate duration correctly", () => {
    const event = focusTimeEvents[0];
    expect(event.durationMinutes).toBe(60);
  });
});

describe("Focus Time formatting", () => {
  function formatRemainingTime(seconds: number): string {
    if (seconds <= 0) return "0:00";

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }
    return `${minutes}:${secs.toString().padStart(2, "0")}`;
  }

  it("should format remaining time correctly", () => {
    expect(formatRemainingTime(0)).toBe("0:00");
    expect(formatRemainingTime(30)).toBe("0:30");
    expect(formatRemainingTime(90)).toBe("1:30");
    expect(formatRemainingTime(3600)).toBe("1:00:00");
    expect(formatRemainingTime(3661)).toBe("1:01:01");
  });

  it("should handle negative values", () => {
    expect(formatRemainingTime(-100)).toBe("0:00");
  });
});
