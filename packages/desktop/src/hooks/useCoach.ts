// hooks/useCoach.ts - AI Coach hooks for Tauri commands

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery } from "@tanstack/react-query";
import type { CoachResponse, UserContext } from "@focusflow/types";

// Query keys
export const coachQueryKeys = {
  dailyTip: ["coach", "dailyTip"] as const,
  patterns: ["coach", "patterns"] as const,
};

// Get daily coaching tip
export function useDailyTip() {
  return useQuery({
    queryKey: coachQueryKeys.dailyTip,
    queryFn: async () => {
      return invoke<CoachResponse>("get_daily_tip");
    },
    staleTime: 1000 * 60 * 60, // 1 hour
  });
}

// Analyze user patterns
export function usePatternAnalysis() {
  return useQuery({
    queryKey: coachQueryKeys.patterns,
    queryFn: async () => {
      return invoke<CoachResponse>("analyze_patterns");
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Get coach response for a message
export function useCoachChat() {
  return useMutation({
    mutationFn: async ({
      message,
      context,
    }: {
      message: string;
      context?: UserContext;
    }) => {
      return invoke<CoachResponse>("get_coach_response", { message, context });
    },
  });
}

// Get session planning advice
export function useSessionAdvice() {
  return useMutation({
    mutationFn: async (plannedDurationMinutes: number) => {
      return invoke<CoachResponse>("get_session_advice", { plannedDurationMinutes });
    },
  });
}

// Get post-session reflection prompt
export function useReflectionPrompt() {
  return useMutation({
    mutationFn: async ({
      sessionCompleted,
      actualDurationMinutes,
    }: {
      sessionCompleted: boolean;
      actualDurationMinutes: number;
    }) => {
      return invoke<CoachResponse>("get_reflection_prompt", {
        sessionCompleted,
        actualDurationMinutes,
      });
    },
  });
}
