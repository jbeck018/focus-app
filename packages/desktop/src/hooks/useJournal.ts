// hooks/useJournal.ts - Journal hooks for Tauri commands

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  JournalEntry,
  CreateJournalEntryRequest,
  TriggerInsight,
  PeakTimes,
} from "@focusflow/types";

// Query keys
export const journalQueryKeys = {
  recentEntries: (limit: number) => ["journalEntries", "recent", limit] as const,
  sessionEntries: (sessionId: string) => ["journalEntries", "session", sessionId] as const,
  triggerInsights: ["triggerInsights"] as const,
  peakTimes: ["peakTimes"] as const,
};

// Get recent journal entries
export function useRecentJournalEntries(limit = 10) {
  return useQuery({
    queryKey: journalQueryKeys.recentEntries(limit),
    queryFn: async () => {
      return invoke<JournalEntry[]>("get_recent_journal_entries", { limit });
    },
  });
}

// Get journal entries for a session
export function useSessionJournalEntries(sessionId: string) {
  return useQuery({
    queryKey: journalQueryKeys.sessionEntries(sessionId),
    queryFn: async () => {
      return invoke<JournalEntry[]>("get_session_journal_entries", { sessionId });
    },
    enabled: !!sessionId,
  });
}

// Get trigger insights
export function useTriggerInsights() {
  return useQuery({
    queryKey: journalQueryKeys.triggerInsights,
    queryFn: async () => {
      return invoke<TriggerInsight[]>("get_trigger_insights");
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Get peak distraction times
export function usePeakTimes() {
  return useQuery({
    queryKey: journalQueryKeys.peakTimes,
    queryFn: async () => {
      return invoke<PeakTimes>("get_peak_distraction_times");
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Create journal entry mutation
export function useCreateJournalEntry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: CreateJournalEntryRequest) => {
      return invoke<JournalEntry>("create_journal_entry", { request });
    },
    onSuccess: () => {
      // Invalidate related queries
      queryClient.invalidateQueries({
        queryKey: ["journalEntries"],
      });
      queryClient.invalidateQueries({
        queryKey: journalQueryKeys.triggerInsights,
      });
      queryClient.invalidateQueries({
        queryKey: journalQueryKeys.peakTimes,
      });
    },
  });
}
