// features/analytics/charts/focus-trend-chart.tsx - Focus time trend visualization

import { useMemo } from "react";
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { FocusTrendDataPoint } from "@focusflow/types";
import { TrendingUp } from "lucide-react";

interface FocusTrendChartProps {
  data: readonly FocusTrendDataPoint[];
  variant?: "line" | "area";
  showSessions?: boolean;
  onDataPointClick?: (dataPoint: FocusTrendDataPoint) => void;
}

export function FocusTrendChart({
  data,
  variant = "line",
  showSessions = true,
  onDataPointClick,
}: FocusTrendChartProps) {
  const chartData = useMemo(
    () =>
      data.map((point) => ({
        date: formatDate(point.date),
        focusMinutes: point.focusMinutes,
        sessions: point.sessions,
        dayOfWeek: point.dayOfWeek,
      })),
    [data]
  );

  const totalMinutes = useMemo(() => data.reduce((sum, d) => sum + d.focusMinutes, 0), [data]);
  const averageMinutes = useMemo(
    () => (data.length > 0 ? Math.round(totalMinutes / data.length) : 0),
    [data, totalMinutes]
  );

  const handleClick = (dataPoint: any) => {
    if (onDataPointClick && dataPoint) {
      const original = data.find((d) => formatDate(d.date) === dataPoint.date);
      if (original) {
        onDataPointClick(original);
      }
    }
  };

  const Chart = variant === "area" ? AreaChart : LineChart;
  const DataComponent = variant === "area" ? Area : Line;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle className="text-base font-semibold">Focus Time Trend</CardTitle>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <TrendingUp className="h-4 w-4" />
          <span>{formatDuration(averageMinutes)} avg/day</span>
        </div>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <Chart data={chartData} onClick={handleClick}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis
              dataKey="date"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: "hsl(var(--muted-foreground))" }}
              className="text-muted-foreground"
            />
            <YAxis
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: "hsl(var(--muted-foreground))" }}
              className="text-muted-foreground"
              label={{
                value: "Minutes",
                angle: -90,
                position: "insideLeft",
                style: { fontSize: 12 },
              }}
            />
            <Tooltip
              content={<CustomTooltip />}
              contentStyle={{
                backgroundColor: "hsl(var(--background))",
                border: "1px solid hsl(var(--border))",
                borderRadius: "6px",
                fontSize: "12px",
              }}
            />
            <Legend wrapperStyle={{ fontSize: "12px" }} iconType="line" />
            <DataComponent
              type="monotone"
              dataKey="focusMinutes"
              stroke="hsl(var(--primary))"
              fill="hsl(var(--primary))"
              fillOpacity={variant === "area" ? 0.2 : 1}
              strokeWidth={2}
              name="Focus Time (min)"
              dot={{ r: 4, fill: "hsl(var(--primary))" }}
              activeDot={{ r: 6, fill: "hsl(var(--primary))" }}
            />
            {showSessions && (
              <DataComponent
                type="monotone"
                dataKey="sessions"
                stroke="hsl(var(--chart-2))"
                fill="hsl(var(--chart-2))"
                fillOpacity={variant === "area" ? 0.1 : 1}
                strokeWidth={2}
                name="Sessions"
                dot={{ r: 3, fill: "hsl(var(--chart-2))" }}
                activeDot={{ r: 5, fill: "hsl(var(--chart-2))" }}
                yAxisId="right"
              />
            )}
            {showSessions && <YAxis yAxisId="right" orientation="right" tick={{ fontSize: 12 }} />}
          </Chart>
        </ResponsiveContainer>
        <div className="mt-4 flex items-center justify-between text-xs text-muted-foreground">
          <span>Total: {formatDuration(totalMinutes)}</span>
          <span>{data.length} days</span>
        </div>
      </CardContent>
    </Card>
  );
}

// Custom tooltip component
function CustomTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) {
    return null;
  }

  return (
    <div className="rounded-lg border bg-background p-3 shadow-lg">
      <p className="mb-2 font-semibold">{label}</p>
      {payload.map((entry: any, index: number) => (
        <div key={index} className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full" style={{ backgroundColor: entry.color }} />
          <span className="text-xs">
            {entry.name}: <span className="font-medium">{entry.value}</span>
          </span>
        </div>
      ))}
      {payload[0]?.payload?.dayOfWeek && (
        <p className="mt-1 text-xs text-muted-foreground">{payload[0].payload.dayOfWeek}</p>
      )}
    </div>
  );
}

// Helper: Format date for display
function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
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
