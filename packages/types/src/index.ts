// types/index.ts - Core type definitions for FocusFlow

// Re-export from domain modules
export * from "./session";

// Blocking - use explicit re-exports to avoid conflicts
export {
  type RuleId,
  type Domain,
  type AppName,
  type CronExpression,
  RuleType,
  ScheduleType,
  type StrictnessLevel,
  type RuleTarget,
  type BlockRule,
  isWebsiteRule,
  isAppRule,
  type CreateWebsiteRuleDTO,
  type CreateAppRuleDTO,
  type CreateCategoryRuleDTO,
  type CreateBlockRuleDTO,
  type BlockEvent,
  type BlockAttempt as BlockRuleAttempt,
  type BypassRequest,
  type BlockingResult,
  type BlockingError,
  BlockCategory,
  type CategoryDomains,
  CATEGORY_DOMAINS,
  type RuleStats,
} from "./blocking";

export {
  type BlockingSchedule,
  type CreateScheduleRequest,
  type UpdateScheduleRequest,
  type BlockingCategory,
  type CreateCategoryRequest,
  type UpdateCategoryRequest,
  PredefinedCategory,
  type StrictModeState,
  type EnableStrictModeRequest,
  type NuclearOption,
  type ActivateNuclearOptionRequest,
  type NuclearOptionResponse,
  type BlockAttempt,
  type BlockStatistics,
  type BlockedItemStats,
  type HourlyStats,
  type DailyStats as BlockingDailyStats,
  type GetStatisticsRequest,
  DayOfWeek,
  DAY_NAMES,
  getDayName,
  isValidTime,
  parseTime,
  isTimeInSchedule,
  type ScheduleResponse,
  type CategoryResponse,
  type StatisticsResponse,
  type RecordAttemptRequest,
} from "./blocking-advanced";

// Analytics - use explicit re-exports to avoid conflicts
export {
  type DateString,
  type Percentage,
  DateString as createDateString,
  Percentage as createPercentage,
  type DailyAggregate,
  type DailyStats,
  type DateRange,
  AnalyticsPeriod,
  type PeriodParams,
  type AnalyticsQuery,
  type SessionBreakdown,
  type HourlyDistribution,
  type WeekdayDistribution,
  type TrendData,
  type AnalyticsOverview,
  type LeaderboardEntry,
  type TeamAnalytics,
  ExportFormat,
  type AnalyticsExportRequest,
  type Insight,
  type FocusGoal,
  type FocusStreak,
  type AggregateMetrics,
  type TimeSeriesDataPoint,
  type ChartData,
} from "./analytics";

export * from "./analytics-extended";
export * from "./auth";
export * from "./journal";
export * from "./calendar";

// Coach - explicit exports to avoid conflicts
export {
  type ChatMessage,
  type CoachResponse,
  type UserContext,
  type Conversation,
  type ConversationDetail,
  type ConversationListResponse,
  type ListConversationsRequest,
  type CreateConversationRequest,
} from "./coach";

export * from "./team";
export * from "./streaks";
export * from "./achievements";
export * from "./onboarding";

// Legacy exports (keeping for backwards compatibility)
export type SessionType = "focus" | "break" | "custom";

export interface Session {
  id: string;
  startTime: string;
  endTime?: string;
  plannedDurationMinutes: number;
  actualDurationSeconds?: number;
  sessionType: SessionType;
  completed: boolean;
  notes?: string;
}

export interface ActiveSession {
  id: string;
  startTime: string;
  plannedDurationMinutes: number;
  sessionType: SessionType;
  blockedApps: string[];
  blockedWebsites: string[];
}

export interface BlockedItem {
  id: number;
  itemType: "app" | "website";
  value: string;
  enabled: boolean;
}

// Backend response type for get_blocked_items command
export interface BlockedItemsResponse {
  apps: string[];
  websites: string[];
}

export interface DailyAnalytics {
  date: string;
  totalFocusSeconds: number;
  totalBreakSeconds: number;
  sessionsCompleted: number;
  sessionsAbandoned: number;
  productivityScore?: number;
}

export interface StartSessionRequest {
  plannedDurationMinutes: number;
  sessionType: SessionType;
  blockedApps: string[];
  blockedWebsites: string[];
}

export interface SessionResponse {
  id: string;
  startTime: string;
  plannedDurationMinutes: number;
  sessionType: string;
}

export interface ExportData {
  version: string;
  exportedAt: string;
  sessions: Session[];
  blockedItems: BlockedItem[];
}
