// hooks/use-achievements.ts - Achievement system hooks

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { AchievementWithStatus, UserAchievement, Achievement } from "@focusflow/types";
import { toast } from "sonner";

// Achievement statistics response
interface AchievementStatsResponse {
  totalAchievements: number;
  unlockedCount: number;
  totalPoints: number;
  completionPercentage: number;
}

// Achievement check result
interface AchievementCheckResult {
  newlyUnlocked: Achievement[];
}

// Query keys for React Query
export const achievementQueryKeys = {
  all: ["achievements"] as const,
  stats: ["achievements", "stats"] as const,
  recent: (limit?: number) => ["achievements", "recent", limit] as const,
};

/**
 * Hook to fetch all achievements with unlock status
 */
export function useAchievements() {
  return useQuery({
    queryKey: achievementQueryKeys.all,
    queryFn: async () => {
      const achievements = await invoke<AchievementWithStatus[]>("get_achievements");
      return achievements;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

/**
 * Hook to fetch achievement statistics
 */
export function useAchievementStats() {
  return useQuery({
    queryKey: achievementQueryKeys.stats,
    queryFn: async () => {
      const stats = await invoke<AchievementStatsResponse>("get_achievement_stats");
      return stats;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

/**
 * Hook to fetch recently unlocked achievements
 */
export function useRecentAchievements(limit?: number) {
  return useQuery({
    queryKey: achievementQueryKeys.recent(limit),
    queryFn: async () => {
      const achievements = await invoke<UserAchievement[]>("get_recent_achievements", { limit });
      return achievements;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

/**
 * Hook to check for new achievements after session completion
 *
 * This will automatically invalidate achievement queries and show
 * toast notifications for newly unlocked achievements.
 */
export function useCheckAchievements() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (sessionId: string) => {
      const result = await invoke<AchievementCheckResult>("check_achievements", {
        sessionId,
      });
      return result;
    },
    onSuccess: (result) => {
      // Invalidate all achievement queries to refresh data
      queryClient.invalidateQueries({ queryKey: ["achievements"] });

      // Show toast for each newly unlocked achievement
      result.newlyUnlocked.forEach((achievement) => {
        toast.success(`Achievement Unlocked: ${achievement.name}`, {
          description: `${achievement.icon} ${achievement.description} - ${achievement.points} points!`,
          duration: 5000,
        });
      });
    },
  });
}

/**
 * Hook to get achievements by category
 */
export function useAchievementsByCategory(category?: string) {
  const { data: achievements, ...rest } = useAchievements();

  const filteredAchievements = achievements?.filter(
    (achievement) => !category || achievement.category === category
  );

  return {
    data: filteredAchievements,
    ...rest,
  };
}

/**
 * Hook to get unlocked achievements
 */
export function useUnlockedAchievements() {
  const { data: achievements, ...rest } = useAchievements();

  const unlockedAchievements = achievements?.filter((achievement) => achievement.unlocked);

  return {
    data: unlockedAchievements,
    ...rest,
  };
}

/**
 * Hook to get locked achievements
 */
export function useLockedAchievements() {
  const { data: achievements, ...rest } = useAchievements();

  const lockedAchievements = achievements?.filter((achievement) => !achievement.unlocked);

  return {
    data: lockedAchievements,
    ...rest,
  };
}

/**
 * Hook to get next achievements to unlock (closest to completion)
 */
export function useNextAchievements(limit = 3) {
  const { data: achievements, ...rest } = useAchievements();

  const nextAchievements = achievements
    ?.filter((achievement) => !achievement.unlocked && !achievement.hidden)
    .sort((a, b) => b.progress - a.progress)
    .slice(0, limit);

  return {
    data: nextAchievements,
    ...rest,
  };
}
