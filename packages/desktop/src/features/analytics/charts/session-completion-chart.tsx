// features/analytics/charts/session-completion-chart.tsx - Session completion rate visualization

import { useMemo } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { SessionCompletionData } from "@focusflow/types";
import { Target } from "lucide-react";

interface SessionCompletionChartProps {
  data: readonly SessionCompletionData[];
  onDataPointClick?: (dataPoint: SessionCompletionData) => void;
}

export function SessionCompletionChart({ data, onDataPointClick }: SessionCompletionChartProps) {
  const chartData = useMemo(
    () =>
      data.map((point) => ({
        date: formatDate(point.date),
        completed: point.completed,
        abandoned: point.abandoned,
        rate: point.rate,
      })),
    [data]
  );

  const overallRate = useMemo(() => {
    const totalCompleted = data.reduce((sum, d) => sum + d.completed, 0);
    const totalAbandoned = data.reduce((sum, d) => sum + d.abandoned, 0);
    const total = totalCompleted + totalAbandoned;
    return total > 0 ? Math.round((totalCompleted / total) * 100) : 0;
  }, [data]);

  const handleClick = (dataPoint: any) => {
    if (onDataPointClick && dataPoint) {
      const original = data.find((d) => formatDate(d.date) === dataPoint.date);
      if (original) {
        onDataPointClick(original);
      }
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle className="text-base font-semibold">Session Completion Rate</CardTitle>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Target className="h-4 w-4" />
          <span>{overallRate}% completion</span>
        </div>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={chartData} onClick={handleClick}>
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
                value: "Sessions",
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
            <Legend wrapperStyle={{ fontSize: "12px" }} iconType="rect" />
            <Bar
              dataKey="completed"
              stackId="a"
              fill="hsl(var(--chart-1))"
              name="Completed"
              radius={[0, 0, 4, 4]}
            />
            <Bar
              dataKey="abandoned"
              stackId="a"
              fill="hsl(var(--destructive))"
              name="Abandoned"
              radius={[4, 4, 0, 0]}
            />
          </BarChart>
        </ResponsiveContainer>
        <div className="mt-4 grid grid-cols-3 gap-4 text-center text-xs">
          <div>
            <p className="text-muted-foreground">Total Sessions</p>
            <p className="text-lg font-semibold">{data.reduce((sum, d) => sum + d.total, 0)}</p>
          </div>
          <div>
            <p className="text-muted-foreground">Completed</p>
            <p className="text-lg font-semibold text-green-600 dark:text-green-400">
              {data.reduce((sum, d) => sum + d.completed, 0)}
            </p>
          </div>
          <div>
            <p className="text-muted-foreground">Abandoned</p>
            <p className="text-lg font-semibold text-red-600 dark:text-red-400">
              {data.reduce((sum, d) => sum + d.abandoned, 0)}
            </p>
          </div>
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

  const completed = payload.find((p: any) => p.dataKey === "completed")?.value ?? 0;
  const abandoned = payload.find((p: any) => p.dataKey === "abandoned")?.value ?? 0;
  const total = completed + abandoned;
  const rate = total > 0 ? Math.round((completed / total) * 100) : 0;

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
      <div className="mt-2 border-t pt-2 text-xs text-muted-foreground">
        Completion Rate: <span className="font-medium">{rate}%</span>
      </div>
    </div>
  );
}

// Helper: Format date for display
function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}
