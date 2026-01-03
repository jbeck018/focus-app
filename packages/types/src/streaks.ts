/**
 * Type definitions for Enhanced Streak System
 *
 * Features:
 * - GitHub-style contribution heatmap
 * - Streak freezes (weekly + earned)
 * - Milestone celebrations
 * - Grace period recovery
 * - Detailed statistics
 */

// Branded types for type safety
export type StreakId = string & { readonly __brand: "StreakId" };
export type DateString = string & { readonly __brand: "DateString" }; // YYYY-MM-DD format

// Type constructors
export const StreakId = (id: string): StreakId => id as StreakId;
export const DateString = (date: string): DateString => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) {
    throw new Error(`Invalid date format: ${date}. Expected YYYY-MM-DD`);
  }
  return date as DateString;
};

// Streak freeze source
export const FreezeSource = {
  WEEKLY: "weekly",
  ACHIEVEMENT: "achievement",
  PURCHASE: "purchase",
} as const;

export type FreezeSource = (typeof FreezeSource)[keyof typeof FreezeSource];

// Streak milestone tiers
export const MilestoneTier = {
  BRONZE: "bronze",   // 7 days
  SILVER: "silver",   // 30 days
  GOLD: "gold",       // 90 days
  PLATINUM: "platinum", // 180 days
  DIAMOND: "diamond",   // 365 days
} as const;

export type MilestoneTier = (typeof MilestoneTier)[keyof typeof MilestoneTier];

// Streak freeze entity
export interface StreakFreeze {
  readonly id: StreakId;
  readonly userId: string | null;
  readonly usedAt: string | null; // ISO timestamp
  readonly source: FreezeSource;
  readonly createdAt: string; // ISO timestamp
  readonly expiresAt: string | null; // ISO timestamp (for weekly freezes)
}

// Streak history entry (daily record)
export interface StreakHistoryEntry {
  readonly id: StreakId;
  readonly userId: string | null;
  readonly date: DateString;
  readonly sessionsCount: number;
  readonly focusMinutes: number;
  readonly wasFrozen: boolean; // Was this day maintained via freeze?
  readonly createdAt: string;
}

// Current streak status
export interface CurrentStreak {
  readonly currentCount: number;
  readonly longestCount: number;
  readonly lastActivityDate: DateString | null;
  readonly isInGracePeriod: boolean; // Within 2 hours after midnight
  readonly gracePeriodEndsAt: string | null; // ISO timestamp
}

// Streak milestone achievement
export interface StreakMilestone {
  readonly tier: MilestoneTier;
  readonly daysRequired: number;
  readonly achievedAt: string | null; // ISO timestamp
  readonly isAchieved: boolean;
  readonly reward?: string; // e.g., "streak_freeze" or "badge"
}

// Heatmap cell data for calendar visualization
export interface HeatmapCell {
  readonly date: DateString;
  readonly sessionsCount: number;
  readonly focusMinutes: number;
  readonly intensity: number; // 0-4 for color intensity
  readonly wasFrozen: boolean;
}

// Streak statistics (weekly/monthly aggregates)
export interface StreakStats {
  readonly period: "week" | "month";
  readonly startDate: DateString;
  readonly endDate: DateString;
  readonly totalDays: number;
  readonly activeDays: number;
  readonly totalSessions: number;
  readonly totalFocusMinutes: number;
  readonly averageSessionsPerDay: number;
  readonly averageFocusMinutesPerDay: number;
  readonly perfectDays: number; // Days with 4+ sessions
  readonly currentStreak: number;
  readonly longestStreakInPeriod: number;
}

// Available freezes
export interface AvailableFreezes {
  readonly weeklyFreeze: StreakFreeze | null; // Refreshes every Monday
  readonly earnedFreezes: StreakFreeze[];
  readonly totalAvailable: number;
}

// Request to use a streak freeze
export interface UseStreakFreezeRequest {
  readonly freezeId: StreakId;
  readonly date: DateString; // The date to apply the freeze to
}

// Milestone celebration data
export interface MilestoneCelebration {
  readonly milestone: StreakMilestone;
  readonly message: string;
  readonly animationType: "confetti" | "fireworks" | "sparkle";
  readonly dismissedAt?: string;
}

// Streak recovery options
export interface StreakRecoveryOptions {
  readonly canRecover: boolean;
  readonly reason: string;
  readonly availableFreezes: AvailableFreezes;
  readonly missedDate: DateString;
  readonly gracePeriodActive: boolean;
}

// Heatmap data for visualization
export interface StreakHeatmapData {
  readonly startDate: DateString;
  readonly endDate: DateString;
  readonly cells: HeatmapCell[];
  readonly maxIntensity: number; // For scaling colors
}

// Create streak history entry DTO
export interface CreateStreakHistoryDTO {
  readonly date: DateString;
  readonly sessionsCount: number;
  readonly focusMinutes: number;
  readonly wasFrozen?: boolean;
}

// Streak analytics filters
export interface StreakFilters {
  readonly dateRange?: {
    readonly from: DateString;
    readonly to: DateString;
  };
  readonly minSessions?: number;
  readonly includeFrozen?: boolean;
}

// Result type for streak operations
export type StreakResult<T> =
  | { success: true; data: T }
  | { success: false; error: StreakError };

// Discriminated union for streak errors
export type StreakError =
  | { type: "no_freezes_available"; message: string }
  | { type: "freeze_already_used"; freezeId: StreakId }
  | { type: "invalid_date"; date: string; message: string }
  | { type: "grace_period_expired"; message: string }
  | { type: "database"; message: string }
  | { type: "unknown"; message: string };

// Event types for real-time updates
export type StreakEvent =
  | { type: "streak_extended"; currentCount: number; date: DateString }
  | { type: "streak_broken"; previousCount: number }
  | { type: "milestone_achieved"; milestone: StreakMilestone }
  | { type: "freeze_used"; freeze: StreakFreeze; date: DateString }
  | { type: "freeze_earned"; freeze: StreakFreeze }
  | { type: "weekly_freeze_available"; freeze: StreakFreeze };

// Helper constants
export const MILESTONE_DAYS: Record<MilestoneTier, number> = {
  bronze: 7,
  silver: 30,
  gold: 90,
  platinum: 180,
  diamond: 365,
};

export const GRACE_PERIOD_HOURS = 2;
export const MIN_SESSIONS_FOR_STREAK = 1; // Minimum sessions to count as active day
export const PERFECT_DAY_SESSIONS = 4; // Sessions needed for "perfect day"

// Utility type for extracting event data by type
export type ExtractStreakEventData<T extends StreakEvent["type"]> = Extract<
  StreakEvent,
  { type: T }
>;
