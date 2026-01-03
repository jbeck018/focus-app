// hooks/useTauriCommands.ts - Type-safe Tauri command invocations

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  ActiveSession,
  BlockedItemsResponse,
  DailyAnalytics,
  Session,
  StartSessionRequest,
  SessionResponse,
} from "@focusflow/types";

// Query keys for React Query
export const queryKeys = {
  activeSession: ["activeSession"] as const,
  sessionHistory: (days: number) => ["sessionHistory", days] as const,
  blockedItems: ["blockedItems"] as const,
  dailyStats: (date: string) => ["dailyStats", date] as const,
  weeklyStats: ["weeklyStats"] as const,
};

// Session Commands
export function useActiveSession() {
  return useQuery({
    queryKey: queryKeys.activeSession,
    queryFn: async () => {
      const session = await invoke<ActiveSession | null>("get_active_session");
      return session;
    },
    refetchInterval: 1000, // Poll every second during active session
  });
}

export function useStartSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: StartSessionRequest) => {
      const response = await invoke<SessionResponse>("start_focus_session", {
        request,
      });
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.activeSession });
    },
  });
}

export function useEndSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (completed: boolean) => {
      await invoke("end_focus_session", { completed });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.activeSession });
      queryClient.invalidateQueries({ queryKey: ["sessionHistory"] });
      queryClient.invalidateQueries({ queryKey: ["dailyStats"] });
    },
  });
}

export function useSessionHistory(days = 7) {
  return useQuery({
    queryKey: queryKeys.sessionHistory(days),
    queryFn: async () => {
      const sessions = await invoke<Session[]>("get_session_history", { days });
      return sessions;
    },
  });
}

// Blocking Commands
export function useBlockedItems() {
  return useQuery({
    queryKey: queryKeys.blockedItems,
    queryFn: async () => {
      const response = await invoke<BlockedItemsResponse>("get_blocked_items");
      return response;
    },
  });
}

export function useAddBlockedApp() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (appName: string) => {
      await invoke("add_blocked_app", { request: { value: appName } });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.blockedItems });
    },
  });
}

export function useRemoveBlockedApp() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (appName: string) => {
      await invoke("remove_blocked_app", { request: { value: appName } });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.blockedItems });
    },
  });
}

export function useAddBlockedWebsite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (domain: string) => {
      await invoke("add_blocked_website", { request: { value: domain } });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.blockedItems });
    },
  });
}

export function useRemoveBlockedWebsite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (domain: string) => {
      await invoke("remove_blocked_website", { request: { value: domain } });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.blockedItems });
    },
  });
}

export function useToggleBlocking() {
  return useMutation({
    mutationFn: async (enabled: boolean) => {
      await invoke("toggle_blocking", { enable: enabled });
    },
  });
}

// Analytics Commands
export function useDailyStats(date?: string) {
  const today = date ?? new Date().toISOString().split("T")[0];

  return useQuery({
    queryKey: queryKeys.dailyStats(today),
    queryFn: async () => {
      const stats = await invoke<DailyAnalytics | null>("get_daily_stats", {
        date: today,
      });
      return stats;
    },
  });
}

export function useWeeklyStats() {
  return useQuery({
    queryKey: queryKeys.weeklyStats,
    queryFn: async () => {
      const stats = await invoke<DailyAnalytics[]>("get_weekly_stats");
      return stats;
    },
  });
}

export function useProductivityScore() {
  return useQuery({
    queryKey: ["productivityScore"],
    queryFn: async () => {
      const score = await invoke<number>("get_productivity_score");
      return score;
    },
  });
}
