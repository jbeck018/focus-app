// features/analytics/charts/time-of-day-chart.tsx - Time of day analysis

import { useMemo } from "react";
import {
  RadialBarChart,
  RadialBar,
  PolarAngleAxis,
  PolarRadiusAxis,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { TimeOfDayDistribution } from "@focusflow/types";
import { Clock } from "lucide-react";

interface TimeOfDayChartProps {
  data: readonly TimeOfDayDistribution[];
  variant?: "radial" | "bar";
  onDataPointClick?: (dataPoint: TimeOfDayDistribution) => void;
}

export function TimeOfDayChart({
  data,
  variant: _variant = "radial",
  onDataPointClick: _onDataPointClick,
}: TimeOfDayChartProps) {
  // Group hours into time periods
  const periodData = useMemo(() => {
    const periods = {
      morning: { hours: [6, 7, 8, 9, 10, 11], minutes: 0, sessions: 0 },
      afternoon: { hours: [12, 13, 14, 15, 16, 17], minutes: 0, sessions: 0 },
      evening: { hours: [18, 19, 20, 21, 22, 23], minutes: 0, sessions: 0 },
      night: { hours: [0, 1, 2, 3, 4, 5], minutes: 0, sessions: 0 },
    };

    data.forEach((hourData) => {
      if (periods.morning.hours.includes(hourData.hour)) {
        periods.morning.minutes += hourData.focusMinutes;
        periods.morning.sessions += hourData.sessions;
      } else if (periods.afternoon.hours.includes(hourData.hour)) {
        periods.afternoon.minutes += hourData.focusMinutes;
        periods.afternoon.sessions += hourData.sessions;
      } else if (periods.evening.hours.includes(hourData.hour)) {
        periods.evening.minutes += hourData.focusMinutes;
        periods.evening.sessions += hourData.sessions;
      } else if (periods.night.hours.includes(hourData.hour)) {
        periods.night.minutes += hourData.focusMinutes;
        periods.night.sessions += hourData.sessions;
      }
    });

    const total = Object.values(periods).reduce((sum, p) => sum + p.minutes, 0);

    return [
      {
        name: "Morning (6AM-12PM)",
        value: periods.morning.minutes,
        sessions: periods.morning.sessions,
        fill: "hsl(var(--chart-1))",
        percentage: total > 0 ? Math.round((periods.morning.minutes / total) * 100) : 0,
      },
      {
        name: "Afternoon (12PM-6PM)",
        value: periods.afternoon.minutes,
        sessions: periods.afternoon.sessions,
        fill: "hsl(var(--chart-2))",
        percentage: total > 0 ? Math.round((periods.afternoon.minutes / total) * 100) : 0,
      },
      {
        name: "Evening (6PM-12AM)",
        value: periods.evening.minutes,
        sessions: periods.evening.sessions,
        fill: "hsl(var(--chart-3))",
        percentage: total > 0 ? Math.round((periods.evening.minutes / total) * 100) : 0,
      },
      {
        name: "Night (12AM-6AM)",
        value: periods.night.minutes,
        sessions: periods.night.sessions,
        fill: "hsl(var(--chart-4))",
        percentage: total > 0 ? Math.round((periods.night.minutes / total) * 100) : 0,
      },
    ].filter((p) => p.value > 0);
  }, [data]);

  const mostProductiveTime = useMemo(() => {
    const sorted = [...data].sort((a, b) => b.focusMinutes - a.focusMinutes);
    return sorted[0]?.label ?? "N/A";
  }, [data]);

  const totalSessions = useMemo(() => data.reduce((sum, d) => sum + d.sessions, 0), [data]);

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle className="text-base font-semibold">Time of Day Analysis</CardTitle>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Clock className="h-4 w-4" />
          <span>Peak: {mostProductiveTime}</span>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col items-center">
          <ResponsiveContainer width="100%" height={300}>
            <RadialBarChart
              cx="50%"
              cy="50%"
              innerRadius="20%"
              outerRadius="90%"
              data={periodData}
              startAngle={90}
              endAngle={-270}
            >
              <PolarAngleAxis type="number" domain={[0, 100]} tick={false} />
              <PolarRadiusAxis tick={false} />
              <Tooltip content={<CustomTooltip />} />
              <RadialBar background dataKey="percentage" cornerRadius={10} />
            </RadialBarChart>
          </ResponsiveContainer>

          <div className="mt-4 grid w-full grid-cols-2 gap-3 text-sm">
            {periodData.map((period, index) => (
              <div key={index} className="flex items-center gap-2 rounded-lg border p-2">
                <div className="h-3 w-3 rounded-full" style={{ backgroundColor: period.fill }} />
                <div className="flex-1">
                  <p className="text-xs font-medium">{period.name.split(" ")[0]}</p>
                  <p className="text-xs text-muted-foreground">
                    {formatDuration(period.value)} ({period.percentage}%)
                  </p>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-4 grid w-full grid-cols-2 gap-4 border-t pt-4 text-center text-xs">
            <div>
              <p className="text-muted-foreground">Total Sessions</p>
              <p className="text-lg font-semibold">{totalSessions}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Most Productive</p>
              <p className="text-lg font-semibold">{mostProductiveTime}</p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// Custom tooltip component
function CustomTooltip({ active, payload }: any) {
  if (!active || !payload?.length) {
    return null;
  }

  const data = payload[0].payload;

  return (
    <div className="rounded-lg border bg-background p-3 shadow-lg">
      <p className="mb-2 font-semibold">{data.name}</p>
      <div className="space-y-1">
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">Focus Time:</span>
          <span className="font-medium">{formatDuration(data.value)}</span>
        </div>
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">Sessions:</span>
          <span className="font-medium">{data.sessions}</span>
        </div>
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">Percentage:</span>
          <span className="font-medium">{data.percentage}%</span>
        </div>
      </div>
    </div>
  );
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
