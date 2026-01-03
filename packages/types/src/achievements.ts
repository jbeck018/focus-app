/**
 * Type definitions for Achievement/Badge System
 *
 * Implements a comprehensive gamification system with achievements,
 * progress tracking, and unlockable badges for productivity milestones.
 */

// Achievement categories
export const AchievementCategory = {
  SESSION: "session",
  STREAK: "streak",
  TIME: "time",
  BLOCKING: "blocking",
  SPECIAL: "special",
} as const;

export type AchievementCategory =
  (typeof AchievementCategory)[keyof typeof AchievementCategory];

// Achievement rarity levels
export const AchievementRarity = {
  COMMON: "common",
  RARE: "rare",
  EPIC: "epic",
  LEGENDARY: "legendary",
} as const;

export type AchievementRarity =
  (typeof AchievementRarity)[keyof typeof AchievementRarity];

// Core achievement entity
export interface Achievement {
  readonly id: number;
  readonly key: string; // Unique identifier (e.g., "first_focus", "streak_7")
  readonly name: string;
  readonly description: string;
  readonly icon: string; // Emoji or icon identifier
  readonly category: AchievementCategory;
  readonly rarity: AchievementRarity;
  readonly threshold: number; // The count/value needed to unlock
  readonly points: number; // Gamification points awarded
  readonly hidden: boolean; // Hidden until unlocked
  readonly order: number; // Display order within category
}

// User's unlocked achievement
export interface UserAchievement {
  readonly id: number;
  readonly userId: string | null;
  readonly achievementId: number;
  readonly unlockedAt: string; // ISO timestamp
  readonly progress?: number; // Current progress (for display)
  readonly notificationSent: boolean;
}

// Achievement with unlock status
export interface AchievementWithStatus extends Achievement {
  readonly unlocked: boolean;
  readonly unlockedAt: string | null;
  readonly progress: number; // Current progress toward threshold
  readonly progressPercentage: number; // 0-100
}

// Achievement progress tracking
export interface AchievementProgress {
  readonly achievementKey: string;
  readonly currentValue: number;
  readonly threshold: number;
  readonly percentage: number;
}

// Achievement statistics for user
export interface AchievementStats {
  readonly totalAchievements: number;
  readonly unlockedCount: number;
  readonly totalPoints: number;
  readonly completionPercentage: number;
  readonly recentUnlocks: UserAchievement[];
  readonly nextToUnlock: AchievementWithStatus[];
}

// Achievement unlock event
export interface AchievementUnlockEvent {
  readonly achievement: Achievement;
  readonly unlockedAt: string;
  readonly isNew: boolean; // First time unlocking
}

// Achievement check result
export interface AchievementCheckResult {
  readonly newlyUnlocked: Achievement[];
  readonly updatedProgress: AchievementProgress[];
}

// DTO for creating/updating achievement
export interface CreateAchievementDTO {
  readonly key: string;
  readonly name: string;
  readonly description: string;
  readonly icon: string;
  readonly category: AchievementCategory;
  readonly rarity: AchievementRarity;
  readonly threshold: number;
  readonly points: number;
  readonly hidden?: boolean;
  readonly order?: number;
}

// Predefined achievement keys (for type-safe references)
export const AchievementKey = {
  // Session achievements
  FIRST_FOCUS: "first_focus",
  SESSIONS_10: "sessions_10",
  SESSIONS_50: "sessions_50",
  SESSIONS_100: "sessions_100",
  SESSIONS_500: "sessions_500",
  SESSIONS_1000: "sessions_1000",

  // Streak achievements
  STREAK_3: "streak_3",
  STREAK_7: "streak_7",
  STREAK_14: "streak_14",
  STREAK_30: "streak_30",
  STREAK_100: "streak_100",
  STREAK_365: "streak_365",

  // Time achievements (in hours)
  TIME_1H: "time_1h",
  TIME_10H: "time_10h",
  TIME_50H: "time_50h",
  TIME_100H: "time_100h",
  TIME_500H: "time_500h",
  TIME_1000H: "time_1000h",

  // Blocking achievements
  FIRST_BLOCK: "first_block",
  BLOCKS_100: "blocks_100",
  BLOCKS_500: "blocks_500",
  BLOCKS_1000: "blocks_1000",

  // Special achievements
  NIGHT_OWL: "night_owl", // Session between 10PM-4AM
  EARLY_BIRD: "early_bird", // Session between 5AM-7AM
  WEEKEND_WARRIOR: "weekend_warrior", // 10 weekend sessions
  MARATHON: "marathon", // Single session over 2 hours
  PERFECTIONIST: "perfectionist", // 20 sessions with 100% completion
  CONSISTENCY_KING: "consistency_king", // Sessions every day for a week
  ZERO_DISTRACTIONS: "zero_distractions", // Complete 10 sessions with no blocks
} as const;

export type AchievementKey =
  (typeof AchievementKey)[keyof typeof AchievementKey];

// Achievement notification preferences
export interface AchievementNotificationSettings {
  readonly enabled: boolean;
  readonly sound: boolean;
  readonly showToast: boolean;
  readonly showBadge: boolean;
}

// Result type for achievement operations
export type AchievementResult<T> =
  | { success: true; data: T }
  | { success: false; error: AchievementError };

// Discriminated union for achievement errors
export type AchievementError =
  | { type: "not_found"; achievementId: number }
  | { type: "already_unlocked"; achievementKey: string }
  | { type: "validation"; field: string; message: string }
  | { type: "database"; message: string }
  | { type: "unknown"; message: string };
