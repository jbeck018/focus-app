// features/analytics/charts/productivity-chart.tsx - Productivity score visualization

import { useMemo } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
  type MouseHandlerDataParam,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { ProductivityScorePoint } from "@focusflow/types";
import { Activity } from "lucide-react";

interface ProductivityChartProps {
  data: readonly ProductivityScorePoint[];
  showFocusMinutes?: boolean;
  onDataPointClick?: (dataPoint: ProductivityScorePoint) => void;
}

export function ProductivityChart({
  data,
  showFocusMinutes: _showFocusMinutes = false,
  onDataPointClick,
}: ProductivityChartProps) {
  const chartData = useMemo(
    () =>
      data.map((point) => ({
        date: formatDate(point.date),
        score: point.score,
        focusMinutes: point.focusMinutes,
        distractionsBlocked: point.distractionsBlocked,
      })),
    [data]
  );

  const averageScore = useMemo(() => {
    if (data.length === 0) return 0;
    return Math.round(data.reduce((sum, d) => sum + d.score, 0) / data.length);
  }, [data]);

  const scoreStatus = useMemo(() => {
    if (averageScore >= 80) return { label: "Excellent", color: "hsl(var(--chart-1))" };
    if (averageScore >= 60) return { label: "Good", color: "hsl(var(--chart-2))" };
    if (averageScore >= 40) return { label: "Fair", color: "hsl(var(--chart-3))" };
    return { label: "Needs Improvement", color: "hsl(var(--destructive))" };
  }, [averageScore]);

  const handleClick = (clickData: MouseHandlerDataParam) => {
    if (onDataPointClick && typeof clickData.activeTooltipIndex === "number") {
      const clickedItem = chartData[clickData.activeTooltipIndex];
      if (clickedItem.date) {
        const original = data.find((d) => formatDate(d.date) === clickedItem.date);
        if (original) {
          onDataPointClick(original);
        }
      }
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle className="text-base font-semibold">Productivity Score</CardTitle>
        <div className="flex items-center gap-2 text-sm">
          <Activity className="h-4 w-4" />
          <span style={{ color: scoreStatus.color }} className="font-medium">
            {averageScore} - {scoreStatus.label}
          </span>
        </div>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <AreaChart data={chartData} onClick={handleClick}>
            <defs>
              <linearGradient id="productivityGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="hsl(var(--primary))" stopOpacity={0.3} />
                <stop offset="95%" stopColor="hsl(var(--primary))" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis
              dataKey="date"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: "hsl(var(--muted-foreground))" }}
              className="text-muted-foreground"
            />
            <YAxis
              domain={[0, 100]}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: "hsl(var(--muted-foreground))" }}
              className="text-muted-foreground"
              label={{
                value: "Score",
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
            <ReferenceLine
              y={60}
              stroke="hsl(var(--muted-foreground))"
              strokeDasharray="3 3"
              label={{ value: "Target", position: "right", fontSize: 11 }}
            />
            <Area
              type="monotone"
              dataKey="score"
              stroke="hsl(var(--primary))"
              fill="url(#productivityGradient)"
              strokeWidth={2}
              dot={{ r: 4, fill: "hsl(var(--primary))" }}
              activeDot={{ r: 6, fill: "hsl(var(--primary))" }}
            />
          </AreaChart>
        </ResponsiveContainer>
        <div className="mt-4 grid grid-cols-4 gap-2 text-center text-xs">
          <div>
            <p className="text-muted-foreground">Average</p>
            <p className="text-lg font-semibold">{averageScore}</p>
          </div>
          <div>
            <p className="text-muted-foreground">Peak</p>
            <p className="text-lg font-semibold">
              {data.length > 0 ? Math.max(...data.map((d) => d.score)) : 0}
            </p>
          </div>
          <div>
            <p className="text-muted-foreground">Low</p>
            <p className="text-lg font-semibold">
              {data.length > 0 ? Math.min(...data.map((d) => d.score)) : 0}
            </p>
          </div>
          <div>
            <p className="text-muted-foreground">Days</p>
            <p className="text-lg font-semibold">{data.length}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// Custom tooltip component
interface TooltipPayload {
  value?: number;
  payload?: {
    focusMinutes?: number;
    distractionsBlocked?: number;
  };
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: TooltipPayload[];
  label?: string;
}

function CustomTooltip({ active, payload, label }: CustomTooltipProps) {
  if (!active || !payload?.length) {
    return null;
  }

  const score = payload[0]?.value ?? 0;
  const focusMinutes = payload[0]?.payload?.focusMinutes ?? 0;
  const distractionsBlocked = payload[0]?.payload?.distractionsBlocked ?? 0;

  const getScoreLabel = (score: number) => {
    if (score >= 80) return "Excellent";
    if (score >= 60) return "Good";
    if (score >= 40) return "Fair";
    return "Needs Improvement";
  };

  return (
    <div className="rounded-lg border bg-background p-3 shadow-lg">
      <p className="mb-2 font-semibold">{label}</p>
      <div className="space-y-1">
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">Score:</span>
          <span className="font-medium">
            {score} - {getScoreLabel(score)}
          </span>
        </div>
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">Focus Time:</span>
          <span className="font-medium">{formatDuration(focusMinutes)}</span>
        </div>
        {distractionsBlocked > 0 && (
          <div className="flex items-center justify-between gap-4">
            <span className="text-xs text-muted-foreground">Distractions:</span>
            <span className="font-medium">{distractionsBlocked}</span>
          </div>
        )}
      </div>
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
