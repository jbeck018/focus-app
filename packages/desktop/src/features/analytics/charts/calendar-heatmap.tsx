// features/analytics/charts/calendar-heatmap.tsx - 365-day calendar heatmap

import { useMemo, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { CalendarHeatmapDay } from "@focusflow/types";
import { Calendar as CalendarIcon } from "lucide-react";

interface CalendarHeatmapProps {
  data: readonly CalendarHeatmapDay[];
  year?: number;
  onDayClick?: (day: CalendarHeatmapDay) => void;
}

export function CalendarHeatmap({
  data,
  year = new Date().getFullYear(),
  onDayClick,
}: CalendarHeatmapProps) {
  const [hoveredDay, setHoveredDay] = useState<CalendarHeatmapDay | null>(null);

  // Organize data into weeks
  const weeks = useMemo(() => {
    const weeksArray: CalendarHeatmapDay[][] = [];
    let currentWeek: CalendarHeatmapDay[] = [];

    // Get first day of year
    const firstDay = new Date(year, 0, 1);
    const firstDayOfWeek = firstDay.getDay();

    // Add empty days for the first week
    for (let i = 0; i < firstDayOfWeek; i++) {
      currentWeek.push({
        date: "" as any,
        focusMinutes: 0 as any,
        sessions: 0,
        intensity: 0,
        hasData: false,
      });
    }

    data.forEach((day) => {
      currentWeek.push(day);

      // If Sunday or last day, start new week
      if (currentWeek.length === 7) {
        weeksArray.push(currentWeek);
        currentWeek = [];
      }
    });

    // Add remaining days to last week
    if (currentWeek.length > 0) {
      while (currentWeek.length < 7) {
        currentWeek.push({
          date: "" as any,
          focusMinutes: 0 as any,
          sessions: 0,
          intensity: 0,
          hasData: false,
        });
      }
      weeksArray.push(currentWeek);
    }

    return weeksArray;
  }, [data, year]);

  const stats = useMemo(() => {
    const activeDays = data.filter((d) => d.hasData).length;
    const totalMinutes = data.reduce((sum, d) => sum + d.focusMinutes, 0);
    const maxDay = data.reduce(
      (max, d) => (d.focusMinutes > max.focusMinutes ? d : max),
      data[0]
    );

    return {
      activeDays,
      totalMinutes,
      maxDay: maxDay?.date ?? null,
      maxMinutes: maxDay?.focusMinutes ?? 0,
    };
  }, [data]);

  const getIntensityColor = (intensity: 0 | 1 | 2 | 3 | 4): string => {
    const colors = {
      0: "bg-muted/30",
      1: "bg-green-200 dark:bg-green-900/40",
      2: "bg-green-400 dark:bg-green-700/60",
      3: "bg-green-600 dark:bg-green-500/80",
      4: "bg-green-800 dark:bg-green-400",
    };
    return colors[intensity];
  };

  const handleDayClick = (day: CalendarHeatmapDay) => {
    if (day.hasData && onDayClick) {
      onDayClick(day);
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle className="text-base font-semibold">
          Year Activity - {year}
        </CardTitle>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <CalendarIcon className="h-4 w-4" />
          <span>{stats.activeDays} active days</span>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {/* Month labels */}
          <div className="flex gap-[2px] text-xs text-muted-foreground">
            {getMonthLabels(year).map((month, index) => (
              <div
                key={index}
                className="flex-1 text-center"
                style={{ minWidth: `${100 / 12}%` }}
              >
                {month}
              </div>
            ))}
          </div>

          {/* Calendar grid */}
          <div className="overflow-x-auto">
            <div className="inline-flex gap-[2px]">
              {/* Day labels */}
              <div className="flex flex-col gap-[2px] pr-2 text-xs text-muted-foreground">
                <div className="h-3">Mon</div>
                <div className="h-3"></div>
                <div className="h-3">Wed</div>
                <div className="h-3"></div>
                <div className="h-3">Fri</div>
                <div className="h-3"></div>
                <div className="h-3">Sun</div>
              </div>

              {/* Weeks */}
              <div className="flex gap-[2px]">
                {weeks.map((week, weekIndex) => (
                  <div key={weekIndex} className="flex flex-col gap-[2px]">
                    {week.map((day, dayIndex) => (
                      <div
                        key={dayIndex}
                        className={`h-3 w-3 rounded-sm transition-all ${
                          day.hasData
                            ? `${getIntensityColor(day.intensity)} cursor-pointer hover:ring-2 hover:ring-primary hover:ring-offset-1`
                            : "bg-transparent"
                        }`}
                        onClick={() => handleDayClick(day)}
                        onMouseEnter={() => day.hasData && setHoveredDay(day)}
                        onMouseLeave={() => setHoveredDay(null)}
                        title={
                          day.hasData
                            ? `${formatDate(day.date)}: ${formatDuration(day.focusMinutes)}`
                            : undefined
                        }
                      />
                    ))}
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Legend */}
          <div className="flex items-center justify-between border-t pt-4">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <span>Less</span>
              {[0, 1, 2, 3, 4].map((intensity) => (
                <div
                  key={intensity}
                  className={`h-3 w-3 rounded-sm ${getIntensityColor(intensity as 0 | 1 | 2 | 3 | 4)}`}
                />
              ))}
              <span>More</span>
            </div>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-3 gap-4 border-t pt-4 text-center text-xs">
            <div>
              <p className="text-muted-foreground">Active Days</p>
              <p className="text-lg font-semibold">{stats.activeDays}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Total Focus</p>
              <p className="text-lg font-semibold">
                {formatDuration(stats.totalMinutes)}
              </p>
            </div>
            <div>
              <p className="text-muted-foreground">Best Day</p>
              <p className="text-lg font-semibold">
                {formatDuration(stats.maxMinutes)}
              </p>
            </div>
          </div>

          {/* Hovered day tooltip */}
          {hoveredDay && (
            <div className="rounded-lg border bg-muted p-3">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium">{formatFullDate(hoveredDay.date)}</p>
                  <p className="text-xs text-muted-foreground">
                    {hoveredDay.sessions} session{hoveredDay.sessions !== 1 ? "s" : ""}
                  </p>
                </div>
                <Badge variant="outline" className="text-sm font-semibold">
                  {formatDuration(hoveredDay.focusMinutes)}
                </Badge>
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

// Helper: Get month labels for the year
function getMonthLabels(_year: number): string[] {
  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  return months;
}

// Helper: Format date
function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

// Helper: Format full date
function formatFullDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  });
}

// Helper: Format duration
function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}
