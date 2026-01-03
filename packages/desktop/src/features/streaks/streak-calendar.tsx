/**
 * Streak Calendar Component
 *
 * GitHub-style contribution heatmap showing daily focus activity
 * - Color-coded cells based on activity intensity
 * - Tooltips with detailed stats
 * - Frozen days indicator
 * - Responsive grid layout
 */

import React, { useMemo } from "react";
import { useStreakHeatmap } from "../../hooks/use-streaks";
import { HeatmapCell } from "@focusflow/types";
import { cn } from "../../lib/utils";

interface StreakCalendarProps {
  months?: number;
  className?: string;
}

export function StreakCalendar({ months = 12, className }: StreakCalendarProps) {
  const { data: heatmapData, isLoading, error } = useStreakHeatmap(months);

  const calendarGrid = useMemo(() => {
    if (!heatmapData || !Array.isArray(heatmapData.cells)) return [];

    const weeks: HeatmapCell[][] = [];
    let currentWeek: HeatmapCell[] = [];

    heatmapData.cells.forEach((cell, index) => {
      const date = new Date(cell.date);
      const dayOfWeek = date.getDay();

      // Start new week on Sunday
      if (dayOfWeek === 0 && currentWeek.length > 0) {
        weeks.push(currentWeek);
        currentWeek = [];
      }

      // Fill empty slots at the beginning of the first week
      if (weeks.length === 0 && currentWeek.length === 0 && dayOfWeek > 0) {
        for (let i = 0; i < dayOfWeek; i++) {
          currentWeek.push({
            date: "", // Empty placeholder for calendar layout
            sessionsCount: 0,
            focusMinutes: 0,
            intensity: 0,
            wasFrozen: false,
          });
        }
      }

      currentWeek.push(cell);

      // Push the last week
      if (index === heatmapData.cells.length - 1) {
        weeks.push(currentWeek);
      }
    });

    return weeks;
  }, [heatmapData]);

  const months_labels = useMemo(() => {
    if (!heatmapData || calendarGrid.length === 0) return [];

    const labels: { month: string; startIndex: number }[] = [];
    let currentMonth = "";

    calendarGrid.forEach((week, weekIndex) => {
      const firstValidCell = week.find((cell) => cell.date !== "");
      if (!firstValidCell) return;

      const date = new Date(firstValidCell.date);
      const monthName = date.toLocaleDateString("en-US", { month: "short" });

      if (monthName !== currentMonth) {
        currentMonth = monthName;
        labels.push({ month: monthName, startIndex: weekIndex });
      }
    });

    return labels;
  }, [calendarGrid, heatmapData]);

  if (isLoading) {
    return (
      <div className={cn("flex items-center justify-center p-8", className)}>
        <div className="text-sm text-muted-foreground">Loading streak calendar...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={cn("flex items-center justify-center p-8", className)}>
        <div className="text-sm text-destructive">Failed to load streak calendar</div>
      </div>
    );
  }

  if (calendarGrid.length === 0) {
    return (
      <div className={cn("flex items-center justify-center p-8", className)}>
        <div className="text-sm text-muted-foreground">No streak data yet</div>
      </div>
    );
  }

  return (
    <div className={cn("streak-calendar", className)}>
      {/* Month labels */}
      <div className="mb-2 flex gap-[3px]">
        {months_labels.map((label, index) => (
          <div
            key={index}
            className="text-xs text-muted-foreground"
            style={{
              marginLeft: index === 0 ? `${label.startIndex * 15}px` : "0",
              width: "60px",
            }}
          >
            {label.month}
          </div>
        ))}
      </div>

      {/* Calendar grid */}
      <div className="flex gap-1">
        {/* Day labels */}
        <div className="flex flex-col gap-1 pr-2">
          <div className="h-3 text-xs text-muted-foreground">Sun</div>
          <div className="h-3 text-xs text-muted-foreground">Mon</div>
          <div className="h-3 text-xs text-muted-foreground">Tue</div>
          <div className="h-3 text-xs text-muted-foreground">Wed</div>
          <div className="h-3 text-xs text-muted-foreground">Thu</div>
          <div className="h-3 text-xs text-muted-foreground">Fri</div>
          <div className="h-3 text-xs text-muted-foreground">Sat</div>
        </div>

        {/* Weeks */}
        <div className="flex gap-[3px]">
          {calendarGrid.map((week, weekIndex) => (
            <div key={weekIndex} className="flex flex-col gap-[3px]">
              {week.map((cell, dayIndex) => (
                <CalendarCell key={`${weekIndex}-${dayIndex}`} cell={cell} />
              ))}
            </div>
          ))}
        </div>
      </div>

      {/* Legend */}
      <div className="mt-4 flex items-center justify-end gap-2">
        <span className="text-xs text-muted-foreground">Less</span>
        <div className="flex gap-1">
          {[0, 1, 2, 3, 4].map((intensity) => (
            <div
              key={intensity}
              className={cn("h-3 w-3 rounded-sm border", getIntensityColor(intensity))}
            />
          ))}
        </div>
        <span className="text-xs text-muted-foreground">More</span>
      </div>
    </div>
  );
}

interface CalendarCellProps {
  cell: HeatmapCell;
}

function CalendarCell({ cell }: CalendarCellProps) {
  const [showTooltip, setShowTooltip] = React.useState(false);

  if (!cell.date) {
    return <div className="h-3 w-3" />;
  }

  const date = new Date(cell.date);
  const formattedDate = date.toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  return (
    <div className="relative">
      <div
        className={cn(
          "h-3 w-3 cursor-pointer rounded-sm border transition-all hover:ring-2 hover:ring-primary",
          getIntensityColor(cell.intensity),
          cell.wasFrozen && "ring-2 ring-blue-400"
        )}
        onMouseEnter={() => setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
      />

      {showTooltip && (
        <div className="absolute left-1/2 top-full z-50 mt-2 -translate-x-1/2 transform whitespace-nowrap rounded-md bg-popover px-3 py-2 text-xs shadow-lg">
          <div className="font-semibold">{formattedDate}</div>
          <div className="mt-1 space-y-0.5 text-muted-foreground">
            <div>{cell.sessionsCount} sessions</div>
            <div>{cell.focusMinutes} minutes</div>
            {cell.wasFrozen && <div className="text-blue-400">Streak Freeze Used</div>}
          </div>
        </div>
      )}
    </div>
  );
}

function getIntensityColor(intensity: number): string {
  switch (intensity) {
    case 0:
      return "bg-muted border-muted-foreground/20";
    case 1:
      return "bg-green-200 dark:bg-green-900/30 border-green-400 dark:border-green-700";
    case 2:
      return "bg-green-400 dark:bg-green-800/50 border-green-500 dark:border-green-600";
    case 3:
      return "bg-green-600 dark:bg-green-700/70 border-green-700 dark:border-green-500";
    case 4:
      return "bg-green-700 dark:bg-green-600 border-green-800 dark:border-green-400";
    default:
      return "bg-muted border-muted-foreground/20";
  }
}

// CSS for animations (add to your global CSS or component)
// Note: These styles should be added to your global CSS file
// const _calendarStyles = `
// @keyframes pulse-ring {
//   0% {
//     transform: scale(1);
//     opacity: 1;
//   }
//   100% {
//     transform: scale(1.5);
//     opacity: 0;
//   }
// }
//
// .streak-calendar .frozen-cell::after {
//   content: '';
//   position: absolute;
//   inset: -2px;
//   border-radius: 0.125rem;
//   border: 2px solid rgb(96 165 250);
//   animation: pulse-ring 2s infinite;
// }
// `;
