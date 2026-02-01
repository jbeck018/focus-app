// hooks/useFocusTime.ts - Focus Time calendar-based blocking hooks

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { FocusTimeEvent, FocusTimeState, FocusTimeOverride } from "@focusflow/types";

// Response type from backend get_focus_time_events
interface FocusTimeEventsResponse {
  events: FocusTimeEvent[];
  activeEvent: FocusTimeEvent | null;
  upcomingEvents: FocusTimeEvent[];
}

// Response type from backend get_focus_time_status
interface FocusTimeStatusResponse {
  active: boolean;
  state: FocusTimeState | null;
  activeEvent: FocusTimeEvent | null;
  upcomingEvents: FocusTimeEvent[];
  secondsUntilNext: number | null;
}

// Query keys
export const focusTimeQueryKeys = {
  events: ["focusTimeEvents"] as const,
  active: ["activeFocusTime"] as const,
  allowedApps: ["focusTimeAllowedApps"] as const,
  state: ["focusTimeState"] as const,
  status: ["focusTimeStatus"] as const,
};

/**
 * Get all upcoming Focus Time events from connected calendars
 */
export function useFocusTimeEvents() {
  return useQuery({
    queryKey: focusTimeQueryKeys.events,
    queryFn: async () => {
      const response = await invoke<FocusTimeEventsResponse>("get_focus_time_events");
      return response.events;
    },
    refetchInterval: 1000 * 60, // Refetch every minute
  });
}

/**
 * Get the currently active Focus Time session
 */
export function useActiveFocusTime() {
  return useQuery({
    queryKey: focusTimeQueryKeys.active,
    queryFn: async () => {
      return invoke<FocusTimeState | null>("get_active_focus_time");
    },
    refetchInterval: 1000 * 5, // Refetch every 5 seconds when active
  });
}

/**
 * Get the complete Focus Time state (active session + allowed apps)
 */
export function useFocusTimeState() {
  return useQuery({
    queryKey: focusTimeQueryKeys.state,
    queryFn: async () => {
      const response = await invoke<FocusTimeStatusResponse>("get_focus_time_status");
      // Transform response to match expected FocusTimeState interface
      if (!response.active || !response.state) {
        return {
          active: false,
          current_event: null,
          allowed_apps: [],
          allowed_categories: [],
          remaining_seconds: 0,
          can_override: true,
        } as FocusTimeState;
      }
      return response.state;
    },
    refetchInterval: 1000 * 5, // Refetch every 5 seconds
  });
}

/**
 * Get allowed apps in current Focus Time session
 */
export function useAllowedApps() {
  return useQuery({
    queryKey: focusTimeQueryKeys.allowedApps,
    queryFn: async () => {
      return invoke<string[]>("get_allowed_apps");
    },
    enabled: false, // Only fetch when explicitly called
  });
}

/**
 * Override Focus Time rules (add/remove apps, end early)
 */
export function useFocusTimeActions() {
  const queryClient = useQueryClient();

  const addApp = useMutation({
    mutationFn: async (appName: string) => {
      // Use override_focus_time_apps with add parameter
      return invoke<string[]>("override_focus_time_apps", {
        request: { add: [appName] },
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.state });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.allowedApps });
    },
  });

  const removeApp = useMutation({
    mutationFn: async (appName: string) => {
      // Use override_focus_time_apps with remove parameter
      return invoke<string[]>("override_focus_time_apps", {
        request: { remove: [appName] },
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.state });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.allowedApps });
    },
  });

  const endEarly = useMutation({
    mutationFn: async (_reason?: string) => {
      // Backend command doesn't take reason parameter
      await invoke("end_focus_time_early");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.state });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.active });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.events });
    },
  });

  const startNow = useMutation({
    mutationFn: async (eventId: string) => {
      return invoke<FocusTimeState>("start_focus_time_now", { eventId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.state });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.active });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.events });
    },
  });

  const override = useMutation({
    mutationFn: async (overrideAction: FocusTimeOverride) => {
      // Map the override action to the backend API
      const request: { add?: string[]; remove?: string[]; reset?: boolean } = {};

      if (overrideAction.action === "add_app" && overrideAction.app_name) {
        request.add = [overrideAction.app_name];
      } else if (overrideAction.action === "remove_app" && overrideAction.app_name) {
        request.remove = [overrideAction.app_name];
      } else if (overrideAction.action === "end_early") {
        await invoke("end_focus_time_early");
        return [];
      }

      return invoke<string[]>("override_focus_time_apps", { request });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.state });
    },
  });

  return {
    addApp,
    removeApp,
    endEarly,
    startNow,
    override,
  };
}

/**
 * Refresh Focus Time events from calendar (triggers sync)
 */
export function useRefreshFocusTimeEvents() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      // Sync with calendar and then invalidate to refetch
      return invoke<boolean>("sync_focus_time_with_calendar");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.events });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.state });
      queryClient.invalidateQueries({ queryKey: focusTimeQueryKeys.active });
    },
  });
}
