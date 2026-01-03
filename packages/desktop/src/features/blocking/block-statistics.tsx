// features/blocking/block-statistics.tsx - Block attempt statistics and analytics

import { TrendingUp, Shield, Clock, Target } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { useBlockStatistics } from "@/hooks/use-blocking-advanced";
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";

export function BlockStatistics() {
  const { data: stats, isLoading } = useBlockStatistics(7);

  if (isLoading) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="text-sm text-muted-foreground">Loading statistics...</div>
        </CardContent>
      </Card>
    );
  }

  if (!stats) {
    return null;
  }

  // Prepare data for hourly chart
  const hourlyData = Array.isArray(stats.attemptsByHour)
    ? stats.attemptsByHour.map((hour) => ({
        hour: `${hour.hour}:00`,
        attempts: hour.count,
      }))
    : [];

  // Calculate highest distraction hour
  const peakHour = Array.isArray(stats.attemptsByHour)
    ? stats.attemptsByHour.reduce(
        (max, hour) => (hour.count > max.count ? hour : max),
        { hour: 0, count: 0 }
      )
    : { hour: 0, count: 0 };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Block Statistics</h3>
        <p className="text-sm text-muted-foreground">
          Track your distraction patterns and blocking effectiveness
        </p>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Total Attempts</CardTitle>
              <Shield className="h-4 w-4 text-muted-foreground" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalAttempts.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground mt-1">
              All-time blocked attempts
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">This Week</CardTitle>
              <TrendingUp className="h-4 w-4 text-muted-foreground" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.attemptsThisWeek.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground mt-1">
              Last 7 days
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Today</CardTitle>
              <Target className="h-4 w-4 text-muted-foreground" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.attemptsToday.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground mt-1">
              Distractions blocked today
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Top Blocked Items */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Most Blocked Sites & Apps</CardTitle>
          <CardDescription>Your most common distractions this week</CardDescription>
        </CardHeader>
        <CardContent>
          {!Array.isArray(stats.topBlockedItems) || stats.topBlockedItems.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              No blocked attempts yet this week
            </div>
          ) : (
            <div className="space-y-3">
              {stats.topBlockedItems.slice(0, 10).map((item, index) => {
                const percentage = (item.count / stats.attemptsThisWeek) * 100;
                return (
                  <div key={`${item.itemType}-${item.itemValue}`} className="space-y-2">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-muted-foreground">
                          #{index + 1}
                        </span>
                        <span className="text-sm font-medium">{item.itemValue}</span>
                        <Badge variant="outline" className="text-xs">
                          {item.itemType}
                        </Badge>
                      </div>
                      <span className="text-sm font-semibold">{item.count}x</span>
                    </div>
                    <Progress value={percentage} className="h-2" />
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Hourly Distribution */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-base">Distraction Patterns</CardTitle>
              <CardDescription>When you're most likely to get distracted</CardDescription>
            </div>
            {peakHour.count > 0 && (
              <Badge variant="secondary" className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                Peak: {peakHour.hour}:00
              </Badge>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {stats.attemptsThisWeek === 0 ? (
            <div className="text-sm text-muted-foreground">
              No attempts this week to analyze
            </div>
          ) : (
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={hourlyData}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                <XAxis
                  dataKey="hour"
                  tick={{ fontSize: 12 }}
                  interval={2}
                  className="text-muted-foreground"
                />
                <YAxis tick={{ fontSize: 12 }} className="text-muted-foreground" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "hsl(var(--popover))",
                    border: "1px solid hsl(var(--border))",
                    borderRadius: "var(--radius)",
                  }}
                  labelStyle={{ color: "hsl(var(--popover-foreground))" }}
                />
                <Bar
                  dataKey="attempts"
                  fill="hsl(var(--primary))"
                  radius={[4, 4, 0, 0]}
                />
              </BarChart>
            </ResponsiveContainer>
          )}
        </CardContent>
      </Card>

      {/* Insights */}
      {stats.attemptsThisWeek > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Insights</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="text-sm">
              {stats.attemptsToday > stats.attemptsThisWeek / 7 && (
                <p className="text-amber-600 dark:text-amber-400">
                  Today has higher than average distraction attempts. Consider enabling strict
                  mode or the nuclear option for better focus.
                </p>
              )}
              {Array.isArray(stats.topBlockedItems) && stats.topBlockedItems.length > 0 && (
                <p className="text-muted-foreground">
                  Your top distraction is <span className="font-medium text-foreground">{stats.topBlockedItems[0].itemValue}</span> with{" "}
                  {stats.topBlockedItems[0].count} attempts this week.
                </p>
              )}
              {peakHour.count > 0 && (
                <p className="text-muted-foreground mt-2">
                  You're most distracted around <span className="font-medium text-foreground">{peakHour.hour}:00</span>. Consider
                  scheduling focused work during other hours.
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
