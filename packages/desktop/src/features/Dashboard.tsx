// features/Dashboard.tsx - Main dashboard with stats

import { Target, Clock, Flame, TrendingUp } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useDailyStats, useWeeklyStats } from "@/hooks/useTauriCommands";
import { formatDuration } from "@/hooks/useTimer";

export function Dashboard() {
  const { data: dailyStats, isLoading: dailyLoading } = useDailyStats();
  const { data: weeklyStats, isLoading: weeklyLoading } = useWeeklyStats();

  // Calculate weekly totals
  const weeklyTotals = (Array.isArray(weeklyStats) ? weeklyStats : []).reduce(
    (acc, day) => ({
      focusSeconds: acc.focusSeconds + day.totalFocusSeconds,
      sessions: acc.sessions + day.sessionsCompleted,
      abandoned: acc.abandoned + day.sessionsAbandoned,
    }),
    { focusSeconds: 0, sessions: 0, abandoned: 0 }
  );

  const completionRate =
    weeklyTotals.sessions + weeklyTotals.abandoned > 0
      ? Math.round((weeklyTotals.sessions / (weeklyTotals.sessions + weeklyTotals.abandoned)) * 100)
      : 0;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Dashboard</h2>
          <p className="text-muted-foreground">Your productivity at a glance</p>
        </div>
        <Badge variant="outline" className="text-xs">
          {new Date().toLocaleDateString("en-US", {
            weekday: "long",
            month: "short",
            day: "numeric",
          })}
        </Badge>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {/* Today's Focus Time */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Today's Focus</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {dailyLoading ? "..." : formatDuration(dailyStats?.totalFocusSeconds ?? 0)}
            </div>
            <p className="text-xs text-muted-foreground">
              {dailyStats?.sessionsCompleted ?? 0} sessions completed
            </p>
          </CardContent>
        </Card>

        {/* Weekly Total */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">This Week</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {weeklyLoading ? "..." : formatDuration(weeklyTotals.focusSeconds)}
            </div>
            <p className="text-xs text-muted-foreground">{weeklyTotals.sessions} sessions total</p>
          </CardContent>
        </Card>

        {/* Completion Rate */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Completion Rate</CardTitle>
            <Target className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{completionRate}%</div>
            <p className="text-xs text-muted-foreground">
              {weeklyTotals.abandoned} sessions abandoned
            </p>
          </CardContent>
        </Card>

        {/* Streak */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Streak</CardTitle>
            <Flame className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {calculateStreak(Array.isArray(weeklyStats) ? weeklyStats : [])} days
            </div>
            <p className="text-xs text-muted-foreground">Keep it going!</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

// Calculate consecutive days with at least one completed session
function calculateStreak(stats: Array<{ date: string; sessionsCompleted: number }>): number {
  if (!stats.length) return 0;

  const sortedDays = [...stats]
    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
    .filter((day) => day.sessionsCompleted > 0);

  if (!sortedDays.length) return 0;

  let streak = 0;
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  for (let i = 0; i < sortedDays.length; i++) {
    const dayDate = new Date(sortedDays[i].date);
    dayDate.setHours(0, 0, 0, 0);

    const expectedDate = new Date(today);
    expectedDate.setDate(expectedDate.getDate() - i);

    if (dayDate.getTime() === expectedDate.getTime()) {
      streak++;
    } else {
      break;
    }
  }

  return streak;
}
