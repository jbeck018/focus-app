/**
 * Type definitions for Analytics
 *
 * Uses mapped types and conditional types for flexible analytics queries
 */

import type { SessionType } from "./session";
import type { Timestamp, Minutes } from "./session";

// Branded types for analytics
export type DateString = string & { readonly __brand: "DateString" }; // YYYY-MM-DD format
export type Percentage = number & { readonly __brand: "Percentage" }; // 0-100

// Constructors
export const DateString = (date: string): DateString => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) {
    throw new Error(`Invalid date format: ${date}. Expected YYYY-MM-DD`);
  }
  return date as DateString;
};

export const Percentage = (value: number): Percentage => {
  if (value < 0 || value > 100) {
    throw new Error(`Invalid percentage: ${value}. Must be between 0 and 100`);
  }
  return value as Percentage;
};

// Daily aggregate (matches SQLite schema)
export interface DailyAggregate {
  readonly id: number;
  readonly date: DateString;
  readonly focusMinutes: Minutes;
  readonly sessionsCompleted: number;
  readonly distractionsBlocked: number;
  readonly triggersLogged: number;
  readonly internalTriggers: number;
  readonly externalTriggers: number;
  readonly syncedAt: Timestamp | null;
  readonly createdAt: Timestamp;
}

// Computed analytics with derived metrics
export interface DailyStats extends DailyAggregate {
  readonly completionRate: Percentage;
  readonly avgSessionLength: Minutes;
  readonly focusScore: number; // 0-100 composite score
  readonly productivityRating: "poor" | "fair" | "good" | "excellent";
}

// Time range for analytics queries
export interface DateRange {
  readonly from: DateString;
  readonly to: DateString;
}

// Analytics period with type-safe literals
export const AnalyticsPeriod = {
  TODAY: "today",
  WEEK: "week",
  MONTH: "month",
  QUARTER: "quarter",
  YEAR: "year",
  CUSTOM: "custom",
} as const;

export type AnalyticsPeriod = (typeof AnalyticsPeriod)[keyof typeof AnalyticsPeriod];

// Conditional type for period parameters
export type PeriodParams<T extends AnalyticsPeriod> = T extends "custom"
  ? { period: T; dateRange: DateRange }
  : { period: T };

// Analytics query with type-safe period handling
export type AnalyticsQuery =
  | PeriodParams<"today">
  | PeriodParams<"week">
  | PeriodParams<"month">
  | PeriodParams<"quarter">
  | PeriodParams<"year">
  | PeriodParams<"custom">;

// Session breakdown by type
export type SessionBreakdown = {
  readonly [K in SessionType]: {
    readonly count: number;
    readonly totalMinutes: Minutes;
    readonly avgMinutes: Minutes;
  };
};

// Hourly distribution (for heatmap visualization)
export type HourlyDistribution = {
  readonly [hour: number]: Minutes; // 0-23
};

// Day of week distribution
export type WeekdayDistribution = {
  readonly monday: Minutes;
  readonly tuesday: Minutes;
  readonly wednesday: Minutes;
  readonly thursday: Minutes;
  readonly friday: Minutes;
  readonly saturday: Minutes;
  readonly sunday: Minutes;
};

// Trend data with direction indicator
export interface TrendData<T extends number> {
  readonly current: T;
  readonly previous: T;
  readonly change: number; // percentage
  readonly direction: "up" | "down" | "stable";
}

// Comprehensive analytics response
export interface AnalyticsOverview {
  readonly period: AnalyticsQuery;
  readonly totalFocusMinutes: TrendData<Minutes>;
  readonly totalSessions: TrendData<number>;
  readonly avgSessionLength: TrendData<Minutes>;
  readonly completionRate: TrendData<Percentage>;
  readonly distractionsBlocked: TrendData<number>;
  readonly focusScore: TrendData<number>;
  readonly sessionBreakdown: SessionBreakdown;
  readonly hourlyDistribution: HourlyDistribution;
  readonly weekdayDistribution: WeekdayDistribution;
  readonly streakDays: number;
  readonly longestStreak: number;
}

// Leaderboard entry for team analytics
export interface LeaderboardEntry {
  readonly userId: string;
  readonly displayName: string;
  readonly avatar: string | null;
  readonly focusMinutes: Minutes;
  readonly sessionsCompleted: number;
  readonly focusScore: number;
  readonly rank: number;
}

// Team analytics (aggregated, privacy-preserving)
export interface TeamAnalytics {
  readonly orgId: string;
  readonly memberCount: number; // Must be >= 5
  readonly period: AnalyticsQuery;
  readonly totalFocusMinutes: Minutes;
  readonly totalSessions: number;
  readonly avgFocusMinutesPerMember: Minutes;
  readonly topPerformers: readonly LeaderboardEntry[]; // Top 10
  readonly teamFocusScore: number;
  readonly hourlyDistribution: HourlyDistribution;
}

// Analytics export format
export const ExportFormat = {
  CSV: "csv",
  JSON: "json",
  PDF: "pdf",
} as const;

export type ExportFormat = (typeof ExportFormat)[keyof typeof ExportFormat];

// Export request
export interface AnalyticsExportRequest {
  readonly format: ExportFormat;
  readonly query: AnalyticsQuery;
  readonly includeRawData: boolean;
}

// Insight generation (AI-powered suggestions)
export interface Insight {
  readonly id: string;
  readonly type: "pattern" | "achievement" | "recommendation" | "warning";
  readonly severity: "low" | "medium" | "high";
  readonly title: string;
  readonly description: string;
  readonly actionable: boolean;
  readonly suggestedAction?: string;
  readonly metadata: Record<string, unknown>;
  readonly generatedAt: Timestamp;
}

// Goal tracking
export interface FocusGoal {
  readonly id: string;
  readonly type: "daily" | "weekly" | "monthly";
  readonly targetMinutes: Minutes;
  readonly currentMinutes: Minutes;
  readonly progress: Percentage;
  readonly isAchieved: boolean;
  readonly deadline: DateString;
}

// Streak tracking
export interface FocusStreak {
  readonly currentStreak: number;
  readonly longestStreak: number;
  readonly lastSessionDate: DateString | null;
  readonly streakHistory: readonly { date: DateString; maintained: boolean }[];
}

// Utility type for aggregating metrics
export type AggregateMetrics<T extends Record<string, number>> = {
  readonly [K in keyof T]: {
    readonly sum: T[K];
    readonly avg: T[K];
    readonly min: T[K];
    readonly max: T[K];
  };
};

// Time-series data point
export interface TimeSeriesDataPoint {
  readonly timestamp: Timestamp;
  readonly value: number;
  readonly label?: string;
}

// Chart data with type-safe series
export interface ChartData<T extends string = string> {
  readonly labels: readonly string[];
  readonly datasets: readonly {
    readonly label: T;
    readonly data: readonly number[];
    readonly color?: string;
  }[];
}
