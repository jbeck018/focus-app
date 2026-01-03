/**
 * React hook for Enhanced Streak System
 *
 * Features:
 * - Current streak tracking with grace period
 * - Heatmap data fetching
 * - Statistics (weekly/monthly)
 * - Milestone progress
 * - Freeze management
 * - Real-time updates
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import {
  CurrentStreak,
  StreakHeatmapData,
  StreakStats,
  StreakMilestone,
  AvailableFreezes,
  StreakHistoryEntry,
  StreakFreeze,
  UseStreakFreezeRequest,
} from "@focusflow/types";

// Query keys
const STREAK_KEYS = {
  current: ["streak", "current"] as const,
  heatmap: (months?: number) => ["streak", "heatmap", months] as const,
  stats: (period: string) => ["streak", "stats", period] as const,
  milestones: ["streak", "milestones"] as const,
  freezes: ["streak", "freezes"] as const,
} as const;

// Tauri command invokers
async function getCurrentStreak(): Promise<CurrentStreak> {
  return invoke("get_current_streak");
}

async function getStreakHeatmap(months?: number): Promise<StreakHeatmapData> {
  return invoke("get_streak_heatmap", { months });
}

async function getStreakStats(period: "week" | "month"): Promise<StreakStats> {
  return invoke("get_streak_stats", { period });
}

async function getStreakMilestones(): Promise<StreakMilestone[]> {
  return invoke("get_streak_milestones");
}

async function getAvailableFreezes(): Promise<AvailableFreezes> {
  return invoke("get_available_freezes");
}

async function useStreakFreeze(request: UseStreakFreezeRequest): Promise<StreakHistoryEntry> {
  return invoke("use_streak_freeze", { request });
}

async function updateStreakHistory(): Promise<StreakHistoryEntry> {
  return invoke("update_streak_history");
}

async function createWeeklyFreeze(): Promise<StreakFreeze> {
  return invoke("create_weekly_freeze");
}

/**
 * Hook for current streak status
 */
export function useCurrentStreak() {
  return useQuery({
    queryKey: STREAK_KEYS.current,
    queryFn: getCurrentStreak,
    staleTime: 1000 * 60 * 5, // 5 minutes
    refetchInterval: 1000 * 60 * 5, // Auto-refresh every 5 minutes
  });
}

/**
 * Hook for streak heatmap data (GitHub-style contribution calendar)
 */
export function useStreakHeatmap(months: number = 12) {
  return useQuery({
    queryKey: STREAK_KEYS.heatmap(months),
    queryFn: () => getStreakHeatmap(months),
    staleTime: 1000 * 60 * 10, // 10 minutes
  });
}

/**
 * Hook for streak statistics (weekly or monthly)
 */
export function useStreakStats(period: "week" | "month" = "week") {
  return useQuery({
    queryKey: STREAK_KEYS.stats(period),
    queryFn: () => getStreakStats(period),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

/**
 * Hook for streak milestones progress
 */
export function useStreakMilestones() {
  return useQuery({
    queryKey: STREAK_KEYS.milestones,
    queryFn: getStreakMilestones,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

/**
 * Hook for available streak freezes
 */
export function useAvailableFreezes() {
  return useQuery({
    queryKey: STREAK_KEYS.freezes,
    queryFn: getAvailableFreezes,
    staleTime: 1000 * 60, // 1 minute
  });
}

/**
 * Mutation hook to use a streak freeze
 */
export function useUseStreakFreeze() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: useStreakFreeze,
    onSuccess: () => {
      // Invalidate all streak-related queries
      queryClient.invalidateQueries({ queryKey: ["streak"] });
    },
  });
}

/**
 * Mutation hook to update streak history (should be called after completing a session)
 */
export function useUpdateStreakHistory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: updateStreakHistory,
    onSuccess: () => {
      // Invalidate streak queries
      queryClient.invalidateQueries({ queryKey: STREAK_KEYS.current });
      queryClient.invalidateQueries({ queryKey: ["streak", "heatmap"] });
      queryClient.invalidateQueries({ queryKey: ["streak", "stats"] });
    },
  });
}

/**
 * Mutation hook to create/refresh weekly freeze
 */
export function useCreateWeeklyFreeze() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: createWeeklyFreeze,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: STREAK_KEYS.freezes });
    },
  });
}

/**
 * Composite hook that provides all streak functionality
 */
export function useStreaks() {
  const currentStreak = useCurrentStreak();
  const heatmap = useStreakHeatmap();
  const weekStats = useStreakStats("week");
  const monthStats = useStreakStats("month");
  const milestones = useStreakMilestones();
  const freezes = useAvailableFreezes();

  const useFreeze = useUseStreakFreeze();
  const updateHistory = useUpdateStreakHistory();
  const createFreeze = useCreateWeeklyFreeze();

  return {
    // Queries
    currentStreak: currentStreak.data,
    heatmap: heatmap.data,
    weekStats: weekStats.data,
    monthStats: monthStats.data,
    milestones: milestones.data,
    freezes: freezes.data,

    // Loading states
    isLoading:
      currentStreak.isLoading || heatmap.isLoading || weekStats.isLoading || milestones.isLoading,

    // Error states
    error: currentStreak.error ?? heatmap.error ?? weekStats.error ?? milestones.error,

    // Mutations
    useFreeze: useFreeze.mutate,
    useFreezePending: useFreeze.isPending,
    useFreezeError: useFreeze.error,

    updateHistory: updateHistory.mutate,
    updateHistoryPending: updateHistory.isPending,

    createFreeze: createFreeze.mutate,
    createFreezePending: createFreeze.isPending,

    // Refetch functions
    refetch: () => {
      currentStreak.refetch();
      heatmap.refetch();
      weekStats.refetch();
      milestones.refetch();
      freezes.refetch();
    },
  };
}

/**
 * Helper hook to check if user should be notified about streak
 */
export function useStreakNotifications() {
  const { data: currentStreak } = useCurrentStreak();

  const shouldNotifyGracePeriod =
    currentStreak?.isInGracePeriod && currentStreak.gracePeriodEndsAt !== null;

  const shouldNotifyRiskBroken =
    currentStreak &&
    currentStreak.currentCount > 0 &&
    !currentStreak.isInGracePeriod &&
    currentStreak.lastActivityDate !== null &&
    new Date(currentStreak.lastActivityDate).toDateString() !== new Date().toDateString();

  return {
    shouldNotifyGracePeriod: shouldNotifyGracePeriod ?? false,
    shouldNotifyRiskBroken: shouldNotifyRiskBroken ?? false,
    gracePeriodEndsAt: currentStreak?.gracePeriodEndsAt ?? null,
    currentCount: currentStreak?.currentCount ?? 0,
  };
}

/**
 * Helper hook to get next milestone
 */
export function useNextMilestone() {
  const { data: milestones } = useStreakMilestones();
  const { data: currentStreak } = useCurrentStreak();

  if (!milestones || !Array.isArray(milestones) || !currentStreak) {
    return null;
  }

  const nextMilestone = milestones.find((m) => !m.isAchieved);

  if (!nextMilestone) {
    return null;
  }

  const progress = (currentStreak.currentCount / nextMilestone.daysRequired) * 100;

  return {
    milestone: nextMilestone,
    progress: Math.min(progress, 100),
    daysRemaining: Math.max(0, nextMilestone.daysRequired - currentStreak.currentCount),
  };
}
