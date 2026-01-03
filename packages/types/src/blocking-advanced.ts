/**
 * Type definitions for Advanced Blocking Features
 *
 * Includes schedules, categories, strict mode, nuclear option, and statistics
 */

// ============================================================================
// Blocking Schedules
// ============================================================================

export interface BlockingSchedule {
  readonly id: number;
  readonly userId: string | null;
  readonly dayOfWeek: number; // 0 = Sunday, 6 = Saturday
  readonly startTime: string; // HH:MM format (24-hour)
  readonly endTime: string; // HH:MM format (24-hour)
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface CreateScheduleRequest {
  readonly dayOfWeek: number;
  readonly startTime: string;
  readonly endTime: string;
}

export interface UpdateScheduleRequest {
  readonly id: number;
  readonly enabled?: boolean;
  readonly startTime?: string;
  readonly endTime?: string;
}

// ============================================================================
// Blocking Categories
// ============================================================================

export interface BlockingCategory {
  readonly id: number;
  readonly name: string;
  readonly description: string | null;
  readonly items: string[]; // Array of sites/apps
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface CreateCategoryRequest {
  readonly name: string;
  readonly description?: string;
  readonly items: string[];
}

export interface UpdateCategoryRequest {
  readonly id: number;
  readonly name?: string;
  readonly description?: string;
  readonly items?: string[];
  readonly enabled?: boolean;
}

// Predefined category names as const
export const PredefinedCategory = {
  SOCIAL_MEDIA: "Social Media",
  NEWS: "News",
  GAMING: "Gaming",
  VIDEO: "Video",
  SHOPPING: "Shopping",
} as const;

export type PredefinedCategory = (typeof PredefinedCategory)[keyof typeof PredefinedCategory];

// ============================================================================
// Strict Mode
// ============================================================================

export interface StrictModeState {
  readonly enabled: boolean;
  readonly sessionId: string | null; // Locked until session ends
  readonly startedAt: string | null;
  readonly canDisable: boolean;
}

export interface EnableStrictModeRequest {
  readonly sessionId: string;
}

// ============================================================================
// Nuclear Option
// ============================================================================

export interface NuclearOption {
  readonly active: boolean;
  readonly durationMinutes: number;
  readonly startedAt: string | null;
  readonly endsAt: string | null;
  readonly remainingSeconds: number | null;
}

export interface ActivateNuclearOptionRequest {
  readonly durationMinutes: number; // 5, 10, 15, 30, 60 minutes
}

export interface NuclearOptionResponse {
  readonly success: boolean;
  readonly endsAt: string;
  readonly message: string;
}

// ============================================================================
// Block Statistics
// ============================================================================

export interface BlockAttempt {
  readonly id: number;
  readonly userId: string | null;
  readonly itemType: "app" | "website";
  readonly itemValue: string;
  readonly blockedAt: string;
  readonly sessionId: string | null;
}

export interface BlockStatistics {
  readonly totalAttempts: number;
  readonly attemptsToday: number;
  readonly attemptsThisWeek: number;
  readonly topBlockedItems: BlockedItemStats[];
  readonly attemptsByHour: HourlyStats[];
  readonly attemptsByDay: DailyStats[];
}

export interface BlockedItemStats {
  readonly itemType: "app" | "website";
  readonly itemValue: string;
  readonly count: number;
  readonly lastAttempt: string;
}

export interface HourlyStats {
  readonly hour: number; // 0-23
  readonly count: number;
}

export interface DailyStats {
  readonly date: string; // YYYY-MM-DD
  readonly count: number;
}

export interface GetStatisticsRequest {
  readonly days?: number; // Default 7 days
}

// ============================================================================
// Utility Types
// ============================================================================

export const DayOfWeek = {
  SUNDAY: 0,
  MONDAY: 1,
  TUESDAY: 2,
  WEDNESDAY: 3,
  THURSDAY: 4,
  FRIDAY: 5,
  SATURDAY: 6,
} as const;

export type DayOfWeek = (typeof DayOfWeek)[keyof typeof DayOfWeek];

export const DAY_NAMES = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
] as const;

// Helper to get day name from number
export const getDayName = (dayOfWeek: number): string => {
  return DAY_NAMES[dayOfWeek] ?? "Unknown";
};

// Time validation helpers
export const isValidTime = (time: string): boolean => {
  const timeRegex = /^([01]\d|2[0-3]):([0-5]\d)$/;
  return timeRegex.test(time);
};

export const parseTime = (time: string): { hours: number; minutes: number } | null => {
  if (!isValidTime(time)) return null;
  const [hours, minutes] = time.split(":").map(Number);
  return { hours, minutes };
};

// Check if current time falls within a schedule
export const isTimeInSchedule = (
  schedule: BlockingSchedule,
  currentTime: Date = new Date()
): boolean => {
  if (!schedule.enabled) return false;

  const currentDay = currentTime.getDay();
  if (currentDay !== schedule.dayOfWeek) return false;

  const start = parseTime(schedule.startTime);
  const end = parseTime(schedule.endTime);
  if (!start || !end) return false;

  const currentMinutes = currentTime.getHours() * 60 + currentTime.getMinutes();
  const startMinutes = start.hours * 60 + start.minutes;
  const endMinutes = end.hours * 60 + end.minutes;

  // Handle schedules that cross midnight
  if (endMinutes < startMinutes) {
    return currentMinutes >= startMinutes || currentMinutes < endMinutes;
  }

  return currentMinutes >= startMinutes && currentMinutes < endMinutes;
};

// ============================================================================
// Response Types
// ============================================================================

export interface ScheduleResponse {
  readonly schedules: BlockingSchedule[];
}

export interface CategoryResponse {
  readonly categories: BlockingCategory[];
}

export interface StatisticsResponse {
  readonly statistics: BlockStatistics;
}

export interface RecordAttemptRequest {
  readonly itemType: "app" | "website";
  readonly itemValue: string;
  readonly sessionId?: string;
}
