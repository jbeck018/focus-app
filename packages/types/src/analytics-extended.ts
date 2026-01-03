/**
 * Extended Analytics Types for Interactive Charts
 *
 * Additional type definitions for the enhanced analytics dashboard
 */

import type { DateString, Percentage } from "./analytics";
import type { SessionType, Minutes } from "./session";

// Chart-specific data structures

export interface FocusTrendDataPoint {
  readonly date: DateString;
  readonly focusMinutes: Minutes;
  readonly sessions: number;
  readonly completionRate: Percentage;
  readonly dayOfWeek: string;
}

export interface SessionCompletionData {
  readonly date: DateString;
  readonly completed: number;
  readonly abandoned: number;
  readonly total: number;
  readonly rate: Percentage;
}

export interface ProductivityScorePoint {
  readonly date: DateString;
  readonly score: number; // 0-100
  readonly focusMinutes: Minutes;
  readonly distractionsBlocked: number;
}

export interface TimeOfDayDistribution {
  readonly hour: number; // 0-23
  readonly label: string; // "12 AM", "1 AM", etc.
  readonly focusMinutes: Minutes;
  readonly sessions: number;
  readonly percentage: Percentage;
}

export interface DistractionData {
  readonly name: string;
  readonly type: "app" | "website";
  readonly blockedCount: number;
  readonly lastBlocked: string | null; // ISO timestamp
  readonly trend: "up" | "down" | "stable";
}

export interface CalendarHeatmapDay {
  readonly date: DateString;
  readonly focusMinutes: Minutes;
  readonly sessions: number;
  readonly intensity: 0 | 1 | 2 | 3 | 4; // 0 = none, 4 = highest
  readonly hasData: boolean;
}

// Date range filters

export const DateRangePreset = {
  TODAY: "today",
  YESTERDAY: "yesterday",
  LAST_7_DAYS: "last_7_days",
  LAST_14_DAYS: "last_14_days",
  LAST_30_DAYS: "last_30_days",
  LAST_90_DAYS: "last_90_days",
  THIS_WEEK: "this_week",
  LAST_WEEK: "last_week",
  THIS_MONTH: "this_month",
  LAST_MONTH: "last_month",
  THIS_YEAR: "this_year",
  CUSTOM: "custom",
} as const;

export type DateRangePreset = (typeof DateRangePreset)[keyof typeof DateRangePreset];

export interface CustomDateRange {
  readonly startDate: DateString;
  readonly endDate: DateString;
}

export type DateRangeFilter =
  | { preset: Exclude<DateRangePreset, "custom"> }
  | { preset: "custom"; range: CustomDateRange };

// Granularity for time-based charts

export const ChartGranularity = {
  HOURLY: "hourly",
  DAILY: "daily",
  WEEKLY: "weekly",
  MONTHLY: "monthly",
} as const;

export type ChartGranularity = (typeof ChartGranularity)[keyof typeof ChartGranularity];

// Export formats

export const ChartExportFormat = {
  PNG: "png",
  SVG: "svg",
  CSV: "csv",
  JSON: "json",
} as const;

export type ChartExportFormat = (typeof ChartExportFormat)[keyof typeof ChartExportFormat];

export interface ChartExportOptions {
  readonly format: ChartExportFormat;
  readonly title: string;
  readonly includeMetadata: boolean;
  readonly width?: number;
  readonly height?: number;
}

// Aggregated analytics response

export interface AnalyticsDashboardData {
  readonly dateRange: CustomDateRange;
  readonly granularity: ChartGranularity;

  // Chart data
  readonly focusTrend: readonly FocusTrendDataPoint[];
  readonly sessionCompletion: readonly SessionCompletionData[];
  readonly productivityScores: readonly ProductivityScorePoint[];
  readonly timeOfDay: readonly TimeOfDayDistribution[];
  readonly topDistractions: readonly DistractionData[];
  readonly calendarHeatmap: readonly CalendarHeatmapDay[];

  // Summary stats
  readonly summary: {
    readonly totalFocusMinutes: Minutes;
    readonly totalSessions: number;
    readonly averageSessionLength: Minutes;
    readonly completionRate: Percentage;
    readonly totalDistractionsBlocked: number;
    readonly averageProductivityScore: number;
    readonly currentStreak: number;
    readonly longestStreak: number;
    readonly mostProductiveDay: DateString | null;
    readonly mostProductiveHour: number | null; // 0-23
  };
}

// Interactive chart configuration

export interface ChartConfig {
  readonly showGrid: boolean;
  readonly showLegend: boolean;
  readonly showTooltip: boolean;
  readonly animationDuration: number;
  readonly colorScheme: "default" | "vibrant" | "muted" | "pastel";
  readonly responsive: boolean;
}

// Drill-down data

export interface DrillDownData {
  readonly type: "day" | "hour" | "session" | "distraction";
  readonly identifier: string; // date, hour, session ID, etc.
  readonly label: string;
  readonly details: Record<string, unknown>;
}

// Chart interaction events

export interface ChartClickEvent {
  readonly chartType: "focus-trend" | "session-completion" | "productivity" | "time-of-day" | "distractions" | "calendar-heatmap";
  readonly dataPoint: unknown;
  readonly drillDown?: DrillDownData;
}

// Session type breakdown for charts

export interface SessionTypeBreakdown {
  readonly type: SessionType;
  readonly count: number;
  readonly totalMinutes: Minutes;
  readonly percentage: Percentage;
  readonly color: string;
}

// Comparison data (for period-over-period analysis)

export interface ComparisonData {
  readonly current: AnalyticsDashboardData;
  readonly previous: AnalyticsDashboardData;
  readonly changes: {
    readonly focusMinutes: { value: number; percentage: number };
    readonly sessions: { value: number; percentage: number };
    readonly completionRate: { value: number; percentage: number };
    readonly productivityScore: { value: number; percentage: number };
  };
}

// Chart theme colors (dark mode compatible)

export interface ChartThemeColors {
  readonly primary: string;
  readonly secondary: string;
  readonly success: string;
  readonly warning: string;
  readonly danger: string;
  readonly info: string;
  readonly neutral: string;
  readonly background: string;
  readonly foreground: string;
  readonly muted: string;
  readonly grid: string;
  readonly tooltip: {
    readonly background: string;
    readonly foreground: string;
    readonly border: string;
  };
}

// Loading and error states

export interface ChartLoadingState {
  readonly isLoading: boolean;
  readonly progress?: number; // 0-100
}

export interface ChartErrorState {
  readonly hasError: boolean;
  readonly errorMessage?: string;
  readonly errorCode?: string;
}

// Chart filters

export interface ChartFilters {
  readonly dateRange: DateRangeFilter;
  readonly granularity: ChartGranularity;
  readonly sessionTypes?: SessionType[];
  readonly minDuration?: Minutes;
  readonly maxDuration?: Minutes;
  readonly includeAbandoned: boolean;
}

// Analytics preferences (user settings)

export interface AnalyticsPreferences {
  readonly defaultDateRange: DateRangePreset;
  readonly defaultGranularity: ChartGranularity;
  readonly chartConfig: ChartConfig;
  readonly favoriteCharts: readonly string[];
  readonly exportDefaults: ChartExportOptions;
}
