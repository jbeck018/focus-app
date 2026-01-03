// features/analytics/analytics-dashboard.tsx - Main interactive analytics dashboard

import { useState, useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useAnalyticsDashboard } from "@/hooks/use-analytics";
import type {
  DateRangeFilter,
  DateRangePreset,
  ChartGranularity,
  ChartFilters,
} from "@focusflow/types";
import {
  FocusTrendChart,
  SessionCompletionChart,
  ProductivityChart,
  TimeOfDayChart,
  DistractionsChart,
  CalendarHeatmap,
} from "./charts";
import { Calendar, Download, TrendingUp, Clock, Target, Flame, Zap, Loader2 } from "lucide-react";
import { exportToCSV, exportToPNG } from "./export-utils";
import { toast } from "sonner";

export function AnalyticsDashboard() {
  const [dateRangeFilter, setDateRangeFilter] = useState<DateRangeFilter>({
    preset: "last_30_days",
  });

  const [granularity, _setGranularity] = useState<ChartGranularity>("daily");
  const [isExporting, setIsExporting] = useState(false);

  const filters: ChartFilters = useMemo(
    () => ({
      dateRange: dateRangeFilter,
      granularity,
      includeAbandoned: true,
    }),
    [dateRangeFilter, granularity]
  );

  const { data: analytics, isLoading, error } = useAnalyticsDashboard(filters);

  const handleExport = async (format: "png" | "csv") => {
    if (!analytics) {
      toast.error("No data to export");
      return;
    }

    setIsExporting(true);
    try {
      if (format === "csv") {
        await exportToCSV(analytics);
        toast.success("Analytics exported to CSV successfully");
      } else if (format === "png") {
        await exportToPNG(analytics);
        toast.success("Dashboard exported to PNG successfully");
      }
    } catch (error) {
      console.error("Export failed:", error);
      toast.error(error instanceof Error ? error.message : "Export failed");
    } finally {
      setIsExporting(false);
    }
  };

  const handleDateRangeChange = (preset: DateRangePreset) => {
    setDateRangeFilter({ preset: preset as Exclude<DateRangePreset, "custom"> });
  };

  if (error) {
    return (
      <div className="flex h-[400px] items-center justify-center">
        <div className="text-center">
          <p className="text-lg font-medium text-destructive">Error loading analytics</p>
          <p className="text-sm text-muted-foreground">
            {error instanceof Error ? error.message : "An unknown error occurred"}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6" data-export-container="analytics-dashboard">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Analytics Dashboard</h2>
          <p className="text-muted-foreground">Deep insights into your productivity patterns</p>
        </div>

        <div className="flex items-center gap-2">
          {/* Date Range Selector */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="gap-2">
                <Calendar className="h-4 w-4" />
                {getDateRangeLabel(dateRangeFilter)}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => handleDateRangeChange("today")}>
                Today
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("yesterday")}>
                Yesterday
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("last_7_days")}>
                Last 7 Days
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("last_14_days")}>
                Last 14 Days
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("last_30_days")}>
                Last 30 Days
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("this_month")}>
                This Month
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("last_month")}>
                Last Month
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleDateRangeChange("this_year")}>
                This Year
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Export Button */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="gap-2"
                disabled={isExporting || isLoading}
              >
                {isExporting ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Download className="h-4 w-4" />
                )}
                {isExporting ? "Exporting..." : "Export"}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => handleExport("csv")} disabled={isExporting}>
                Export as CSV
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleExport("png")} disabled={isExporting}>
                Export as PNG
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* Summary Stats */}
      {analytics && (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <StatCard
            title="Total Focus Time"
            value={formatDuration(analytics.summary.totalFocusMinutes)}
            icon={<Clock className="h-4 w-4 text-muted-foreground" />}
            subtitle={`${analytics.summary.totalSessions} sessions`}
          />
          <StatCard
            title="Completion Rate"
            value={`${analytics.summary.completionRate}%`}
            icon={<Target className="h-4 w-4 text-muted-foreground" />}
            subtitle={`Avg ${formatDuration(analytics.summary.averageSessionLength)}/session`}
          />
          <StatCard
            title="Productivity Score"
            value={analytics.summary.averageProductivityScore}
            icon={<TrendingUp className="h-4 w-4 text-muted-foreground" />}
            subtitle={getScoreLabel(analytics.summary.averageProductivityScore)}
            variant={getScoreVariant(analytics.summary.averageProductivityScore)}
          />
          <StatCard
            title="Current Streak"
            value={`${analytics.summary.currentStreak} days`}
            icon={<Flame className="h-4 w-4 text-muted-foreground" />}
            subtitle={`Best: ${analytics.summary.longestStreak} days`}
          />
        </div>
      )}

      {/* Loading State */}
      {isLoading && (
        <div className="flex h-[400px] items-center justify-center">
          <div className="text-center">
            <Zap className="mx-auto mb-4 h-12 w-12 animate-pulse text-primary" />
            <p className="text-lg font-medium">Loading analytics...</p>
          </div>
        </div>
      )}

      {/* Charts */}
      {analytics && !isLoading && (
        <Tabs defaultValue="overview" className="space-y-4">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="trends">Trends</TabsTrigger>
            <TabsTrigger value="patterns">Patterns</TabsTrigger>
            <TabsTrigger value="calendar">Calendar</TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <FocusTrendChart data={analytics.focusTrend} variant="area" showSessions={true} />
              <SessionCompletionChart data={analytics.sessionCompletion} />
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              <ProductivityChart data={analytics.productivityScores} />
              <TimeOfDayChart data={analytics.timeOfDay} />
            </div>
          </TabsContent>

          <TabsContent value="trends" className="space-y-4">
            <FocusTrendChart data={analytics.focusTrend} variant="line" showSessions={true} />
            <div className="grid gap-4 md:grid-cols-2">
              <SessionCompletionChart data={analytics.sessionCompletion} />
              <ProductivityChart data={analytics.productivityScores} />
            </div>
          </TabsContent>

          <TabsContent value="patterns" className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <TimeOfDayChart data={analytics.timeOfDay} />
              <DistractionsChart data={analytics.topDistractions} maxItems={10} />
            </div>
            {analytics.summary.mostProductiveHour !== undefined &&
             analytics.summary.mostProductiveHour !== null && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-base">Insights</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2 text-sm">
                    <p>
                      Your most productive time is{" "}
                      <Badge variant="outline">
                        {formatHour(analytics.summary.mostProductiveHour)}
                      </Badge>
                    </p>
                    {analytics.summary.mostProductiveDay && (
                      <p>
                        Your best day was{" "}
                        <Badge variant="outline">
                          {formatDate(analytics.summary.mostProductiveDay)}
                        </Badge>
                      </p>
                    )}
                  </div>
                </CardContent>
              </Card>
            )}
          </TabsContent>

          <TabsContent value="calendar" className="space-y-4">
            <CalendarHeatmap data={analytics.calendarHeatmap} year={new Date().getFullYear()} />
          </TabsContent>
        </Tabs>
      )}
    </div>
  );
}

// Stat Card Component
interface StatCardProps {
  title: string;
  value: string | number;
  icon: React.ReactNode;
  subtitle?: string;
  variant?: "default" | "success" | "warning" | "danger";
}

function StatCard({ title, value, icon, subtitle, variant = "default" }: StatCardProps) {
  const variantColors = {
    default: "text-foreground",
    success: "text-green-600 dark:text-green-400",
    warning: "text-yellow-600 dark:text-yellow-400",
    danger: "text-red-600 dark:text-red-400",
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        {icon}
      </CardHeader>
      <CardContent>
        <div className={`text-2xl font-bold ${variantColors[variant]}`}>{value}</div>
        {subtitle && <p className="text-xs text-muted-foreground">{subtitle}</p>}
      </CardContent>
    </Card>
  );
}

// Helper Functions
function getDateRangeLabel(filter: DateRangeFilter): string {
  if (filter.preset === "custom") {
    return "Custom Range";
  }

  const labels: Record<Exclude<DateRangePreset, "custom">, string> = {
    today: "Today",
    yesterday: "Yesterday",
    last_7_days: "Last 7 Days",
    last_14_days: "Last 14 Days",
    last_30_days: "Last 30 Days",
    last_90_days: "Last 90 Days",
    this_week: "This Week",
    last_week: "Last Week",
    this_month: "This Month",
    last_month: "Last Month",
    this_year: "This Year",
  };

  return labels[filter.preset];
}

function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}

function getScoreLabel(score: number): string {
  if (score >= 80) return "Excellent";
  if (score >= 60) return "Good";
  if (score >= 40) return "Fair";
  return "Needs Work";
}

function getScoreVariant(score: number): "default" | "success" | "warning" | "danger" {
  if (score >= 80) return "success";
  if (score >= 60) return "default";
  if (score >= 40) return "warning";
  return "danger";
}

function formatHour(hour: number): string {
  if (hour === 0) return "12 AM";
  if (hour < 12) return `${hour} AM`;
  if (hour === 12) return "12 PM";
  return `${hour - 12} PM`;
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
  });
}

// Re-export charts for convenience
export {
  FocusTrendChart,
  SessionCompletionChart,
  ProductivityChart,
  TimeOfDayChart,
  DistractionsChart,
  CalendarHeatmap,
};
