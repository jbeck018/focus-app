/**
 * Streak Statistics Panel
 *
 * Displays weekly/monthly streak statistics with visual indicators
 * - Current vs longest streak comparison
 * - Perfect days tracking
 * - Average sessions and focus time
 * - Activity rate
 */

import { useState } from "react";
import {
  useCurrentStreak,
  useStreakStats,
  useNextMilestone,
} from "../../hooks/use-streaks";
import { cn } from "../../lib/utils";

interface StreakStatsProps {
  className?: string;
}

export function StreakStats({ className }: StreakStatsProps) {
  const [period, setPeriod] = useState<"week" | "month">("week");
  const { data: currentStreak } = useCurrentStreak();
  const { data: stats, isLoading } = useStreakStats(period);
  const nextMilestone = useNextMilestone();

  if (isLoading) {
    return (
      <div className={cn("rounded-lg border bg-card p-6", className)}>
        <div className="animate-pulse space-y-4">
          <div className="h-8 w-48 rounded bg-muted" />
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="h-20 rounded bg-muted" />
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={cn("rounded-lg border bg-card", className)}>
      {/* Header */}
      <div className="border-b p-6">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Streak Statistics</h3>
            <p className="text-sm text-muted-foreground">
              Your focus performance over time
            </p>
          </div>

          {/* Period toggle */}
          <div className="flex rounded-lg border bg-muted p-1">
            <button
              onClick={() => setPeriod("week")}
              className={cn(
                "rounded-md px-4 py-1.5 text-sm font-medium transition-colors",
                period === "week"
                  ? "bg-background shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              Week
            </button>
            <button
              onClick={() => setPeriod("month")}
              className={cn(
                "rounded-md px-4 py-1.5 text-sm font-medium transition-colors",
                period === "month"
                  ? "bg-background shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              Month
            </button>
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="p-6">
        {/* Current Streak Highlight */}
        <div className="mb-6 rounded-lg bg-gradient-to-r from-orange-500/10 to-red-500/10 p-6">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Current Streak
              </div>
              <div className="mt-1 flex items-baseline gap-2">
                <span className="text-4xl font-bold">
                  {currentStreak?.currentCount || 0}
                </span>
                <span className="text-lg text-muted-foreground">days</span>
              </div>
              {currentStreak?.isInGracePeriod && (
                <div className="mt-2 flex items-center gap-2 text-sm text-orange-500">
                  <div className="h-2 w-2 animate-pulse rounded-full bg-orange-500" />
                  Grace period active
                </div>
              )}
            </div>

            <div className="text-right">
              <div className="text-sm font-medium text-muted-foreground">
                Longest Streak
              </div>
              <div className="mt-1 flex items-baseline justify-end gap-2">
                <span className="text-3xl font-bold text-muted-foreground">
                  {currentStreak?.longestCount || 0}
                </span>
                <span className="text-muted-foreground">days</span>
              </div>
            </div>
          </div>

          {/* Next Milestone Progress */}
          {nextMilestone && (
            <div className="mt-4">
              <div className="mb-2 flex items-center justify-between text-sm">
                <span className="font-medium">
                  Next: {nextMilestone.milestone.tier} ({nextMilestone.milestone.daysRequired} days)
                </span>
                <span className="text-muted-foreground">
                  {nextMilestone.daysRemaining} days to go
                </span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full rounded-full bg-gradient-to-r from-orange-500 to-red-500 transition-all duration-500"
                  style={{ width: `${nextMilestone.progress}%` }}
                />
              </div>
            </div>
          )}
        </div>

        {/* Statistics Grid */}
        {stats && (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <StatCard
              label="Active Days"
              value={stats.activeDays}
              total={stats.totalDays}
              icon="📅"
              color="blue"
            />
            <StatCard
              label="Total Sessions"
              value={stats.totalSessions}
              subtitle={`${stats?.averageSessionsPerDay?.toFixed(1) ?? '0'}/day avg`}
              icon="🎯"
              color="green"
            />
            <StatCard
              label="Focus Time"
              value={formatMinutes(stats.totalFocusMinutes)}
              subtitle={`${formatMinutes(stats.averageFocusMinutesPerDay)}/day avg`}
              icon="⏱️"
              color="purple"
            />
            <StatCard
              label="Perfect Days"
              value={stats.perfectDays}
              subtitle="4+ sessions"
              icon="⭐"
              color="yellow"
            />
          </div>
        )}

        {/* Additional Metrics */}
        {stats && (
          <div className="mt-6 grid gap-4 sm:grid-cols-2">
            <div className="rounded-lg border p-4">
              <div className="text-sm font-medium text-muted-foreground">
                Activity Rate
              </div>
              <div className="mt-2">
                <div className="flex items-baseline gap-2">
                  <span className="text-2xl font-bold">
                    {Math.round((stats.activeDays / stats.totalDays) * 100)}%
                  </span>
                  <span className="text-sm text-muted-foreground">
                    of {period === "week" ? "week" : "month"}
                  </span>
                </div>
                <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted">
                  <div
                    className="h-full rounded-full bg-blue-500 transition-all duration-500"
                    style={{
                      width: `${(stats.activeDays / stats.totalDays) * 100}%`,
                    }}
                  />
                </div>
              </div>
            </div>

            <div className="rounded-lg border p-4">
              <div className="text-sm font-medium text-muted-foreground">
                Consistency Score
              </div>
              <div className="mt-2">
                <div className="flex items-baseline gap-2">
                  <span className="text-2xl font-bold">
                    {calculateConsistencyScore(stats)}
                  </span>
                  <span className="text-sm text-muted-foreground">/ 100</span>
                </div>
                <div className="mt-1 text-xs text-muted-foreground">
                  Based on streak and perfect days
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface StatCardProps {
  label: string;
  value: number | string;
  total?: number;
  subtitle?: string;
  icon: string;
  color: "blue" | "green" | "purple" | "yellow";
}

function StatCard({ label, value, total, subtitle, icon, color }: StatCardProps) {
  const colorClasses = {
    blue: "bg-blue-500/10 text-blue-600 dark:text-blue-400",
    green: "bg-green-500/10 text-green-600 dark:text-green-400",
    purple: "bg-purple-500/10 text-purple-600 dark:text-purple-400",
    yellow: "bg-yellow-500/10 text-yellow-600 dark:text-yellow-400",
  };

  return (
    <div className="rounded-lg border p-4">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="text-sm font-medium text-muted-foreground">{label}</div>
          <div className="mt-2 flex items-baseline gap-1">
            <span className="text-2xl font-bold">{value}</span>
            {total !== undefined && (
              <span className="text-sm text-muted-foreground">/ {total}</span>
            )}
          </div>
          {subtitle && (
            <div className="mt-1 text-xs text-muted-foreground">{subtitle}</div>
          )}
        </div>
        <div className={cn("rounded-lg p-2 text-xl", colorClasses[color])}>
          {icon}
        </div>
      </div>
    </div>
  );
}

function formatMinutes(minutes?: number): string {
  if (!minutes || minutes <= 0) {
    return '0m';
  }
  if (minutes < 60) {
    return `${Math.round(minutes)}m`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = Math.round(minutes % 60);
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}

function calculateConsistencyScore(stats: any): number {
  // Calculate score based on:
  // - Activity rate (40 points)
  // - Perfect days ratio (30 points)
  // - Current streak (30 points)

  const activityScore = (stats.activeDays / stats.totalDays) * 40;
  const perfectDaysScore = (stats.perfectDays / stats.activeDays) * 30;
  const streakScore = Math.min(stats.currentStreak / stats.totalDays, 1) * 30;

  return Math.round(activityScore + perfectDaysScore + streakScore);
}
