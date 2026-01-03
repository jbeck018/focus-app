// hooks/use-analytics.ts - Analytics data fetching and transformation

import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type {
  AnalyticsDashboardData,
  ChartFilters,
  DateRangeFilter,
  DateRangePreset,
  ChartGranularity,
  FocusTrendDataPoint,
  SessionCompletionData,
  ProductivityScorePoint,
  TimeOfDayDistribution,
  DistractionData,
  CalendarHeatmapDay,
  DateString,
  Minutes,
  Percentage,
} from "@focusflow/types";
import type { DailyAnalytics, Session } from "@focusflow/types";

// Tauri command response types (matches Rust structs from commands/analytics.rs)
interface DailyStatsResponse {
  date: string;
  total_focus_minutes: number;
  total_break_minutes: number;
  sessions_completed: number;
  sessions_abandoned: number;
  productivity_score: number;
}

// Tauri command response types (matches Rust structs from commands/blocking-advanced.rs)
interface BlockedItemStats {
  item_type: string;
  item_value: string;
  count: number;
}

interface BlockStatistics {
  total_attempts: number;
  attempts_today: number;
  attempts_this_week: number;
  attempts_this_month: number;
  top_blocked_items: BlockedItemStats[];
  // Note: We only use the fields above in this hook
}

// Query keys
export const analyticsKeys = {
  dashboard: (filters: ChartFilters) => ["analytics", "dashboard", filters] as const,
  focusTrend: (dateRange: DateRangeFilter, granularity: ChartGranularity) =>
    ["analytics", "focus-trend", dateRange, granularity] as const,
  sessionCompletion: (dateRange: DateRangeFilter) =>
    ["analytics", "session-completion", dateRange] as const,
  productivity: (dateRange: DateRangeFilter) => ["analytics", "productivity", dateRange] as const,
  timeOfDay: (dateRange: DateRangeFilter) => ["analytics", "time-of-day", dateRange] as const,
  distractions: (dateRange: DateRangeFilter, limit: number) =>
    ["analytics", "distractions", dateRange, limit] as const,
  calendarHeatmap: (year: number) => ["analytics", "calendar-heatmap", year] as const,
};

// Helper: Convert preset to date range
export function getDateRangeFromPreset(preset: DateRangePreset): {
  startDate: string;
  endDate: string;
} {
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const endDate = today.toISOString().split("T")[0];
  let startDate = endDate;

  switch (preset) {
    case "today":
      startDate = endDate;
      break;
    case "yesterday": {
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);
      startDate = yesterday.toISOString().split("T")[0];
      break;
    }
    case "last_7_days": {
      const start = new Date(today);
      start.setDate(start.getDate() - 6);
      startDate = start.toISOString().split("T")[0];
      break;
    }
    case "last_14_days": {
      const start = new Date(today);
      start.setDate(start.getDate() - 13);
      startDate = start.toISOString().split("T")[0];
      break;
    }
    case "last_30_days": {
      const start = new Date(today);
      start.setDate(start.getDate() - 29);
      startDate = start.toISOString().split("T")[0];
      break;
    }
    case "last_90_days": {
      const start = new Date(today);
      start.setDate(start.getDate() - 89);
      startDate = start.toISOString().split("T")[0];
      break;
    }
    case "this_week": {
      const start = new Date(today);
      const day = start.getDay();
      const diff = start.getDate() - day + (day === 0 ? -6 : 1);
      start.setDate(diff);
      startDate = start.toISOString().split("T")[0];
      break;
    }
    case "last_week": {
      const start = new Date(today);
      const day = start.getDay();
      const diff = start.getDate() - day + (day === 0 ? -6 : 1) - 7;
      start.setDate(diff);
      startDate = start.toISOString().split("T")[0];
      const end = new Date(start);
      end.setDate(end.getDate() + 6);
      return { startDate, endDate: end.toISOString().split("T")[0] };
    }
    case "this_month": {
      const start = new Date(today.getFullYear(), today.getMonth(), 1);
      startDate = start.toISOString().split("T")[0];
      break;
    }
    case "last_month": {
      const start = new Date(today.getFullYear(), today.getMonth() - 1, 1);
      startDate = start.toISOString().split("T")[0];
      const end = new Date(today.getFullYear(), today.getMonth(), 0);
      return { startDate, endDate: end.toISOString().split("T")[0] };
    }
    case "this_year": {
      const start = new Date(today.getFullYear(), 0, 1);
      startDate = start.toISOString().split("T")[0];
      break;
    }
  }

  return { startDate, endDate };
}

// Helper: Get day of week name
function getDayOfWeek(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", { weekday: "short" });
}

// Helper: Calculate intensity for heatmap
function calculateIntensity(focusMinutes: number): 0 | 1 | 2 | 3 | 4 {
  if (focusMinutes === 0) return 0;
  if (focusMinutes < 30) return 1;
  if (focusMinutes < 60) return 2;
  if (focusMinutes < 120) return 3;
  return 4;
}

// Hook: Get complete dashboard data
export function useAnalyticsDashboard(filters: ChartFilters) {
  return useQuery({
    queryKey: analyticsKeys.dashboard(filters),
    queryFn: async (): Promise<AnalyticsDashboardData> => {
      const { startDate, endDate } =
        filters.dateRange.preset === "custom"
          ? filters.dateRange.range
          : getDateRangeFromPreset(filters.dateRange.preset);

      // Calculate days between start and end
      const start = new Date(startDate);
      const end = new Date(endDate);
      const days = Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)) + 1;

      // Fetch raw data from Tauri
      const [dailyStatsRaw, sessionsRaw, blockStatsRaw] = await Promise.all([
        invoke<DailyStatsResponse[]>("get_date_range_stats", { startDate, endDate }),
        invoke<Session[]>("get_session_history", { days }),
        invoke<BlockStatistics>("get_block_statistics", { days }).catch(() => null),
      ]);

      // Transform Rust response to expected format
      const dailyStats: DailyAnalytics[] = dailyStatsRaw.map((stat) => ({
        date: stat.date,
        totalFocusSeconds: stat.total_focus_minutes * 60,
        totalBreakSeconds: stat.total_break_minutes * 60,
        sessionsCompleted: stat.sessions_completed,
        sessionsAbandoned: stat.sessions_abandoned,
        productivityScore: stat.productivity_score,
      }));

      // Transform data for charts
      const focusTrend: FocusTrendDataPoint[] = dailyStats.map((stat) => ({
        date: stat.date as DateString,
        focusMinutes: Math.round(stat.totalFocusSeconds / 60) as Minutes,
        sessions: stat.sessionsCompleted,
        completionRate: (stat.sessionsCompleted + stat.sessionsAbandoned > 0
          ? Math.round(
              (stat.sessionsCompleted / (stat.sessionsCompleted + stat.sessionsAbandoned)) * 100
            )
          : 0) as Percentage,
        dayOfWeek: getDayOfWeek(stat.date),
      }));

      const sessionCompletion: SessionCompletionData[] = dailyStats.map((stat) => ({
        date: stat.date as DateString,
        completed: stat.sessionsCompleted,
        abandoned: stat.sessionsAbandoned,
        total: stat.sessionsCompleted + stat.sessionsAbandoned,
        rate: (stat.sessionsCompleted + stat.sessionsAbandoned > 0
          ? Math.round(
              (stat.sessionsCompleted / (stat.sessionsCompleted + stat.sessionsAbandoned)) * 100
            )
          : 0) as Percentage,
      }));

      const productivityScores: ProductivityScorePoint[] = dailyStats.map((stat) => ({
        date: stat.date as DateString,
        score: stat.productivityScore ?? 0,
        focusMinutes: Math.round(stat.totalFocusSeconds / 60) as Minutes,
        distractionsBlocked: blockStatsRaw?.attempts_today ?? 0,
      }));

      // Calculate time of day distribution
      const hourlyData: Record<number, { minutes: number; count: number }> = {};
      for (let i = 0; i < 24; i++) {
        hourlyData[i] = { minutes: 0, count: 0 };
      }

      sessionsRaw.forEach((session) => {
        if (session.completed && session.startTime) {
          const hour = new Date(session.startTime).getHours();
          const minutes = Math.round((session.actualDurationSeconds ?? 0) / 60);
          hourlyData[hour].minutes += minutes;
          hourlyData[hour].count += 1;
        }
      });

      const totalMinutes = Object.values(hourlyData).reduce((sum, h) => sum + h.minutes, 0);
      const timeOfDay: TimeOfDayDistribution[] = Object.entries(hourlyData).map(([hour, data]) => ({
        hour: parseInt(hour),
        label: formatHourLabel(parseInt(hour)),
        focusMinutes: data.minutes as Minutes,
        sessions: data.count,
        percentage: (totalMinutes > 0
          ? Math.round((data.minutes / totalMinutes) * 100)
          : 0) as Percentage,
      }));

      // Transform block statistics into distraction data
      const topDistractions: DistractionData[] = (blockStatsRaw?.top_blocked_items ?? []).map(
        (item) => ({
          name: item.item_value,
          type: item.item_type === "app" ? ("app" as const) : ("website" as const),
          blockedCount: item.count,
          lastBlocked: null, // TODO: Add lastBlocked timestamp from Rust
          trend: "stable" as const, // TODO: Calculate trend from historical data
        })
      );

      // Generate calendar heatmap for the year
      const currentYear = new Date().getFullYear();
      const calendarHeatmap: CalendarHeatmapDay[] = generateYearCalendar(currentYear, dailyStats);

      // Calculate summary stats
      const totalFocusMinutes = focusTrend.reduce((sum, d) => sum + d.focusMinutes, 0);
      const totalSessions = sessionCompletion.reduce((sum, d) => sum + d.completed, 0);
      const averageSessionLength =
        totalSessions > 0 ? Math.round(totalFocusMinutes / totalSessions) : 0;
      const completedSessions = sessionCompletion.reduce((sum, d) => sum + d.completed, 0);
      const abandonedSessions = sessionCompletion.reduce((sum, d) => sum + d.abandoned, 0);
      const completionRate =
        completedSessions + abandonedSessions > 0
          ? Math.round((completedSessions / (completedSessions + abandonedSessions)) * 100)
          : 0;
      const averageProductivityScore =
        productivityScores.length > 0
          ? Math.round(
              productivityScores.reduce((sum, p) => sum + p.score, 0) / productivityScores.length
            )
          : 0;

      const mostProductiveDay =
        focusTrend.length > 0
          ? focusTrend.reduce((max, d) => (d.focusMinutes > max.focusMinutes ? d : max)).date
          : null;

      const mostProductiveHour =
        timeOfDay.length > 0
          ? timeOfDay.reduce((max, h) => (h.focusMinutes > max.focusMinutes ? h : max)).hour
          : null;

      return {
        dateRange: { startDate: startDate as DateString, endDate: endDate as DateString },
        granularity: filters.granularity,
        focusTrend,
        sessionCompletion,
        productivityScores,
        timeOfDay,
        topDistractions,
        calendarHeatmap,
        summary: {
          totalFocusMinutes: totalFocusMinutes as Minutes,
          totalSessions,
          averageSessionLength: averageSessionLength as Minutes,
          completionRate: completionRate as Percentage,
          totalDistractionsBlocked: blockStatsRaw?.total_attempts ?? 0,
          averageProductivityScore,
          currentStreak: calculateCurrentStreak(dailyStats),
          longestStreak: calculateLongestStreak(dailyStats),
          mostProductiveDay,
          mostProductiveHour,
        },
      };
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Helper: Format hour label
function formatHourLabel(hour: number): string {
  if (hour === 0) return "12 AM";
  if (hour < 12) return `${hour} AM`;
  if (hour === 12) return "12 PM";
  return `${hour - 12} PM`;
}

// Helper: Generate year calendar
function generateYearCalendar(year: number, stats: DailyAnalytics[]): CalendarHeatmapDay[] {
  const calendar: CalendarHeatmapDay[] = [];
  const statsMap = new Map(stats.map((s) => [s.date, s]));

  const startDate = new Date(year, 0, 1);
  const endDate = new Date(year, 11, 31);

  for (let d = new Date(startDate); d <= endDate; d.setDate(d.getDate() + 1)) {
    const dateStr = d.toISOString().split("T")[0];
    const stat = statsMap.get(dateStr);
    const focusMinutes = stat ? Math.round(stat.totalFocusSeconds / 60) : 0;

    calendar.push({
      date: dateStr as DateString,
      focusMinutes: focusMinutes as Minutes,
      sessions: stat?.sessionsCompleted ?? 0,
      intensity: calculateIntensity(focusMinutes),
      hasData: !!stat,
    });
  }

  return calendar;
}

// Helper: Calculate current streak
function calculateCurrentStreak(stats: DailyAnalytics[]): number {
  if (stats.length === 0) return 0;

  const sortedStats = [...stats]
    .filter((s) => s.sessionsCompleted > 0)
    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());

  if (sortedStats.length === 0) return 0;

  let streak = 0;
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  for (let i = 0; i < sortedStats.length; i++) {
    const statDate = new Date(sortedStats[i].date);
    statDate.setHours(0, 0, 0, 0);

    const expectedDate = new Date(today);
    expectedDate.setDate(expectedDate.getDate() - i);

    if (statDate.getTime() === expectedDate.getTime()) {
      streak++;
    } else {
      break;
    }
  }

  return streak;
}

// Helper: Calculate longest streak
function calculateLongestStreak(stats: DailyAnalytics[]): number {
  if (stats.length === 0) return 0;

  const sortedStats = [...stats].sort(
    (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()
  );

  let maxStreak = 0;
  let currentStreak = 0;
  let lastDate: Date | null = null;

  sortedStats.forEach((stat) => {
    if (stat.sessionsCompleted === 0) {
      currentStreak = 0;
      lastDate = null;
      return;
    }

    const statDate = new Date(stat.date);
    statDate.setHours(0, 0, 0, 0);

    if (lastDate) {
      const expectedDate = new Date(lastDate);
      expectedDate.setDate(expectedDate.getDate() + 1);

      if (statDate.getTime() === expectedDate.getTime()) {
        currentStreak++;
      } else {
        currentStreak = 1;
      }
    } else {
      currentStreak = 1;
    }

    maxStreak = Math.max(maxStreak, currentStreak);
    lastDate = statDate;
  });

  return maxStreak;
}

// Export utility functions
export { formatHourLabel, calculateIntensity };
