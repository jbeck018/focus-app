// hooks/use-blocking-advanced.ts - React hooks for advanced blocking features

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  BlockingSchedule,
  CreateScheduleRequest,
  UpdateScheduleRequest,
  BlockingCategory,
  CreateCategoryRequest,
  UpdateCategoryRequest,
  StrictModeState,
  NuclearOption,
  ActivateNuclearOptionRequest,
  BlockStatistics,
  RecordAttemptRequest,
} from "@focusflow/types";

// ============================================================================
// Query Keys
// ============================================================================

export const advancedBlockingKeys = {
  schedules: ["blocking", "schedules"] as const,
  categories: ["blocking", "categories"] as const,
  strictMode: ["blocking", "strictMode"] as const,
  nuclearOption: ["blocking", "nuclearOption"] as const,
  statistics: (days?: number) => ["blocking", "statistics", days] as const,
};

// ============================================================================
// Blocking Schedules
// ============================================================================

export function useBlockingSchedules() {
  return useQuery({
    queryKey: advancedBlockingKeys.schedules,
    queryFn: async () => {
      const schedules = await invoke<BlockingSchedule[]>("get_blocking_schedules");
      return schedules;
    },
  });
}

export function useCreateSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: CreateScheduleRequest) => {
      const schedule = await invoke<BlockingSchedule>("create_blocking_schedule", { request });
      return schedule;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.schedules });
    },
  });
}

export function useUpdateSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: UpdateScheduleRequest) => {
      await invoke("update_blocking_schedule", { request });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.schedules });
    },
  });
}

export function useDeleteSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: number) => {
      await invoke("delete_blocking_schedule", { id });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.schedules });
    },
  });
}

// ============================================================================
// Blocking Categories
// ============================================================================

interface CategoryResponse {
  id: number;
  name: string;
  description: string | null;
  items: string[];
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export function useBlockingCategories() {
  return useQuery({
    queryKey: advancedBlockingKeys.categories,
    queryFn: async () => {
      const categories = await invoke<CategoryResponse[]>("get_blocking_categories");
      // Convert to BlockingCategory format
      return categories.map((cat) => ({
        id: cat.id,
        name: cat.name,
        description: cat.description,
        items: cat.items,
        enabled: cat.enabled,
        createdAt: cat.createdAt,
        updatedAt: cat.updatedAt,
      })) as BlockingCategory[];
    },
  });
}

export function useCreateCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: CreateCategoryRequest) => {
      const category = await invoke<CategoryResponse>("create_blocking_category", { request });
      return category;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.categories });
    },
  });
}

export function useUpdateCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: UpdateCategoryRequest) => {
      await invoke("update_blocking_category", { request });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.categories });
    },
  });
}

export function useToggleCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, enabled }: { id: number; enabled: boolean }) => {
      await invoke("toggle_blocking_category", { id, enabled });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.categories });
    },
  });
}

// ============================================================================
// Strict Mode
// ============================================================================

export function useStrictModeState() {
  return useQuery({
    queryKey: advancedBlockingKeys.strictMode,
    queryFn: async () => {
      const state = await invoke<StrictModeState>("get_strict_mode_state");
      return state;
    },
    refetchInterval: 5000, // Poll every 5 seconds to check if session ended
  });
}

export function useEnableStrictMode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (sessionId: string) => {
      await invoke("enable_strict_mode", { sessionId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.strictMode });
    },
  });
}

export function useDisableStrictMode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      await invoke("disable_strict_mode");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.strictMode });
    },
  });
}

// ============================================================================
// Nuclear Option
// ============================================================================

export function useNuclearOptionState() {
  return useQuery({
    queryKey: advancedBlockingKeys.nuclearOption,
    queryFn: async () => {
      const state = await invoke<NuclearOption>("get_nuclear_option_state");
      return state;
    },
    refetchInterval: (query) => {
      // If active, poll every second; otherwise every 30 seconds
      return query.state.data?.active ? 1000 : 30000;
    },
  });
}

export function useActivateNuclearOption() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: ActivateNuclearOptionRequest) => {
      const result = await invoke<NuclearOption>("activate_nuclear_option", { request });
      return result;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: advancedBlockingKeys.nuclearOption });
    },
  });
}

// ============================================================================
// Block Statistics
// ============================================================================

export function useBlockStatistics(days = 7) {
  return useQuery({
    queryKey: advancedBlockingKeys.statistics(days),
    queryFn: async () => {
      const stats = await invoke<BlockStatistics>("get_block_statistics", { days });
      return stats;
    },
  });
}

export function useRecordBlockAttempt() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: RecordAttemptRequest) => {
      await invoke("record_block_attempt", { request });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["blocking", "statistics"] });
    },
  });
}

// ============================================================================
// Helper Hooks
// ============================================================================

/**
 * Check if current time matches any active schedule
 */
export function useIsScheduledBlockingActive() {
  const { data: schedules } = useBlockingSchedules();

  const now = new Date();
  const currentDay = now.getDay();
  const currentMinutes = now.getHours() * 60 + now.getMinutes();

  const isActive = Array.isArray(schedules)
    ? schedules.some((schedule) => {
        if (!schedule.enabled || schedule.dayOfWeek !== currentDay) {
          return false;
        }

        const [startHours, startMinutes] = schedule.startTime.split(":").map(Number);
        const [endHours, endMinutes] = schedule.endTime.split(":").map(Number);

        const startMinutesTotal = startHours * 60 + startMinutes;
        const endMinutesTotal = endHours * 60 + endMinutes;

        // Handle schedules that cross midnight
        if (endMinutesTotal < startMinutesTotal) {
          return currentMinutes >= startMinutesTotal || currentMinutes < endMinutesTotal;
        }

        return currentMinutes >= startMinutesTotal && currentMinutes < endMinutesTotal;
      })
    : false;

  return isActive;
}

/**
 * Get all items from enabled categories
 */
export function useEnabledCategoryItems() {
  const { data: categories } = useBlockingCategories();

  const items = Array.isArray(categories)
    ? categories
        .filter((cat) => cat.enabled)
        .flatMap((cat) => cat.items)
    : [];

  return items;
}

/**
 * Check if blocking is locked (strict mode or nuclear option)
 */
export function useIsBlockingLocked() {
  const { data: strictMode } = useStrictModeState();
  const { data: nuclearOption } = useNuclearOptionState();

  return {
    isLocked: strictMode?.enabled || nuclearOption?.active || false,
    reason: strictMode?.enabled ? "strict-mode" : nuclearOption?.active ? "nuclear-option" : null,
    strictMode,
    nuclearOption,
  };
}
