// features/analytics/charts/distractions-chart.tsx - Top distractions blocked

import { useMemo } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Cell,
  type MouseHandlerDataParam,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { DistractionData } from "@focusflow/types";
import { Shield, TrendingUp, TrendingDown, Minus } from "lucide-react";

interface DistractionsChartProps {
  data: readonly DistractionData[];
  maxItems?: number;
  onDataPointClick?: (dataPoint: DistractionData) => void;
}

export function DistractionsChart({
  data,
  maxItems = 10,
  onDataPointClick,
}: DistractionsChartProps) {
  const chartData = useMemo(() => {
    const sorted = [...data].sort((a, b) => b.blockedCount - a.blockedCount);
    return sorted.slice(0, maxItems).map((item) => ({
      name: truncateName(item.name, 20),
      fullName: item.name,
      value: item.blockedCount,
      type: item.type,
      trend: item.trend,
    }));
  }, [data, maxItems]);

  const totalBlocked = useMemo(() => data.reduce((sum, d) => sum + d.blockedCount, 0), [data]);

  const getBarColor = (type: "app" | "website") => {
    return type === "app" ? "hsl(var(--chart-1))" : "hsl(var(--chart-2))";
  };

  const handleClick = (clickData: MouseHandlerDataParam) => {
    if (onDataPointClick && typeof clickData?.activeTooltipIndex === "number") {
      const clickedItem = chartData[clickData.activeTooltipIndex];
      if (clickedItem?.fullName) {
        const original = data.find((d) => d.name === clickedItem.fullName);
        if (original) {
          onDataPointClick(original);
        }
      }
    }
  };

  if (data.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-base font-semibold">Top Distractions Blocked</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-[300px] items-center justify-center text-muted-foreground">
            <p className="text-sm">No distractions blocked yet</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <CardTitle className="text-base font-semibold">Top Distractions Blocked</CardTitle>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Shield className="h-4 w-4" />
          <span>{totalBlocked} total blocks</span>
        </div>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={chartData} layout="vertical" onClick={handleClick} margin={{ left: 100 }}>
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis
              type="number"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: "hsl(var(--muted-foreground))" }}
              className="text-muted-foreground"
            />
            <YAxis
              type="category"
              dataKey="name"
              tick={{ fontSize: 11 }}
              tickLine={{ stroke: "hsl(var(--muted-foreground))" }}
              className="text-muted-foreground"
              width={100}
            />
            <Tooltip content={<CustomTooltip />} />
            <Bar dataKey="value" radius={[0, 4, 4, 0]}>
              {chartData.map((entry, index) => (
                <Cell key={index} fill={getBarColor(entry.type)} />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>

        <div className="mt-4 space-y-2">
          <div className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2">
              <div className="h-2 w-2 rounded-full bg-[hsl(var(--chart-1))]" />
              <span className="text-muted-foreground">Apps</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-2 w-2 rounded-full bg-[hsl(var(--chart-2))]" />
              <span className="text-muted-foreground">Websites</span>
            </div>
          </div>

          {chartData.length > 0 && (
            <div className="rounded-lg border bg-muted/50 p-3">
              <p className="mb-2 text-xs font-medium">Top Distraction</p>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Badge variant="outline" className="text-xs">
                    {chartData[0].type}
                  </Badge>
                  <span className="text-sm font-medium">{chartData[0].fullName}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-lg font-bold">{chartData[0].value}</span>
                  <TrendIndicator trend={chartData[0].trend} />
                </div>
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

// Trend indicator component
function TrendIndicator({ trend }: { trend: "up" | "down" | "stable" }) {
  if (trend === "up") {
    return <TrendingUp className="h-4 w-4 text-red-500" />;
  }
  if (trend === "down") {
    return <TrendingDown className="h-4 w-4 text-green-500" />;
  }
  return <Minus className="h-4 w-4 text-muted-foreground" />;
}

// Custom tooltip component
interface TooltipPayloadItem {
  payload: {
    fullName: string;
    type: string;
    value: number;
    trend: "up" | "down" | "stable";
  };
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: TooltipPayloadItem[];
}

function CustomTooltip({ active, payload }: CustomTooltipProps) {
  if (!active || !payload?.length) {
    return null;
  }

  const data = payload[0].payload;

  return (
    <div className="rounded-lg border bg-background p-3 shadow-lg">
      <div className="space-y-2">
        <div>
          <p className="font-semibold">{data.fullName}</p>
          <Badge variant="outline" className="mt-1 text-xs">
            {data.type}
          </Badge>
        </div>
        <div className="flex items-center justify-between gap-4 border-t pt-2">
          <span className="text-xs text-muted-foreground">Blocked:</span>
          <span className="text-lg font-bold">{data.value}</span>
        </div>
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">Trend:</span>
          <TrendIndicator trend={data.trend} />
        </div>
      </div>
    </div>
  );
}

// Helper: Truncate name
function truncateName(name: string, maxLength: number): string {
  if (name.length <= maxLength) return name;
  return name.substring(0, maxLength - 3) + "...";
}
