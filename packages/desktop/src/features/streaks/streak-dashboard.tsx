/**
 * Streak Dashboard Component
 *
 * Main dashboard combining all streak features:
 * - Current streak overview
 * - Heatmap calendar
 * - Statistics panel
 * - Freeze management
 * - Milestone celebrations
 */

import { useState, useEffect } from "react";
import { StreakCalendar } from "./streak-calendar";
import { StreakStats } from "./streak-stats";
import { StreakFreezeModal } from "./streak-freeze-modal";
import { MilestoneCelebration } from "./streak-milestone-celebration";
import type { StreakMilestone } from "@focusflow/types";
import {
  useCurrentStreak,
  useStreakMilestones,
  useStreakNotifications,
  useAvailableFreezes,
} from "../../hooks/use-streaks";
import { cn } from "../../lib/utils";

interface StreakDashboardProps {
  className?: string;
}

export function StreakDashboard({ className }: StreakDashboardProps) {
  const { data: currentStreak } = useCurrentStreak();
  const { data: milestones } = useStreakMilestones();
  const { data: freezes } = useAvailableFreezes();
  const notifications = useStreakNotifications();

  const [showFreezeModal, setShowFreezeModal] = useState(false);
  const [celebratingMilestone, setCelebratingMilestone] = useState<StreakMilestone | null>(null);

  // Check for newly achieved milestones
  useEffect(() => {
    if (!milestones || !Array.isArray(milestones)) return;

    const justAchieved = milestones.find(
      (m) => m.isAchieved && m.achievedAt && Date.now() - new Date(m.achievedAt).getTime() < 5000 // Within last 5 seconds
    );

    if (justAchieved && !celebratingMilestone) {
      // Schedule state update for next tick to avoid cascading renders
      const timer = setTimeout(() => {
        setCelebratingMilestone(justAchieved);
      }, 0);
      return () => clearTimeout(timer);
    }
  }, [milestones, celebratingMilestone]);

  return (
    <div className={cn("space-y-6", className)}>
      {/* Header with Streak Overview */}
      <div className="rounded-lg border bg-card p-6">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-2xl font-bold">Your Focus Streak</h2>
            <p className="mt-1 text-muted-foreground">Keep the momentum going every day</p>
          </div>

          {/* Current Streak Badge */}
          <div className="text-center">
            <div className="flex items-baseline gap-2">
              <span className="text-5xl font-bold">{currentStreak?.currentCount ?? 0}</span>
              <span className="text-xl text-muted-foreground">days</span>
            </div>
            <div className="mt-1 text-sm text-muted-foreground">Current Streak</div>
          </div>
        </div>

        {/* Grace Period Warning */}
        {notifications.shouldNotifyGracePeriod && (
          <div className="mt-4 flex items-center gap-3 rounded-lg bg-orange-500/10 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-orange-500/20">
              <span className="text-xl">⚠️</span>
            </div>
            <div className="flex-1">
              <div className="font-medium">Grace Period Active</div>
              <div className="text-sm text-muted-foreground">
                Complete a session soon to maintain your streak
              </div>
            </div>
            <button
              onClick={() => setShowFreezeModal(true)}
              className="rounded-md border px-4 py-2 text-sm font-medium hover:bg-muted"
            >
              Use Freeze
            </button>
          </div>
        )}

        {/* Streak at Risk Warning */}
        {notifications.shouldNotifyRiskBroken && (
          <div className="mt-4 flex items-center gap-3 rounded-lg bg-red-500/10 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-red-500/20">
              <span className="text-xl">🔥</span>
            </div>
            <div className="flex-1">
              <div className="font-medium">Streak at Risk!</div>
              <div className="text-sm text-muted-foreground">
                You haven't completed a session today
              </div>
            </div>
            {freezes && freezes.totalAvailable > 0 && (
              <button
                onClick={() => setShowFreezeModal(true)}
                className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Use Freeze ({freezes.totalAvailable})
              </button>
            )}
          </div>
        )}

        {/* Freeze Status */}
        {freezes && freezes.totalAvailable > 0 && (
          <div className="mt-4 flex items-center gap-2">
            <span className="text-2xl">❄️</span>
            <div className="text-sm">
              <span className="font-medium">{freezes.totalAvailable}</span>
              <span className="text-muted-foreground">
                {" "}
                freeze{freezes.totalAvailable !== 1 ? "s" : ""} available
              </span>
            </div>
            <button
              onClick={() => setShowFreezeModal(true)}
              className="ml-auto text-sm text-primary hover:underline"
            >
              Manage Freezes
            </button>
          </div>
        )}
      </div>

      {/* Milestones Progress */}
      {Array.isArray(milestones) && milestones.length > 0 && (
        <div className="rounded-lg border bg-card p-6">
          <h3 className="mb-4 text-lg font-semibold">Milestones</h3>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {milestones.map((milestone) => (
              <MilestoneCard
                key={milestone.tier}
                milestone={milestone}
                currentCount={currentStreak?.currentCount ?? 0}
              />
            ))}
          </div>
        </div>
      )}

      {/* Calendar Heatmap */}
      <div className="rounded-lg border bg-card p-6">
        <h3 className="mb-4 text-lg font-semibold">Activity Calendar</h3>
        <StreakCalendar months={12} />
      </div>

      {/* Statistics */}
      <StreakStats />

      {/* Modals */}
      {showFreezeModal && (
        <StreakFreezeModal isOpen={showFreezeModal} onClose={() => setShowFreezeModal(false)} />
      )}

      {celebratingMilestone && (
        <MilestoneCelebration
          milestone={celebratingMilestone}
          onDismiss={() => setCelebratingMilestone(null)}
        />
      )}
    </div>
  );
}

interface MilestoneCardProps {
  milestone: StreakMilestone;
  currentCount: number;
}

function MilestoneCard({ milestone, currentCount }: MilestoneCardProps) {
  const progress = (currentCount / milestone.daysRequired) * 100;
  const isAchieved = milestone.isAchieved;

  return (
    <div
      className={cn(
        "rounded-lg border p-4 transition-all",
        isAchieved ? "border-primary bg-primary/5" : "border-muted hover:border-muted-foreground/50"
      )}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <span className="text-2xl">{getTierIcon(milestone.tier)}</span>
            <div>
              <div className="font-medium capitalize">{milestone.tier}</div>
              <div className="text-xs text-muted-foreground">{milestone.daysRequired} days</div>
            </div>
          </div>

          {/* Progress Bar */}
          <div className="mt-3">
            <div className="mb-1 flex items-center justify-between text-xs">
              <span className="text-muted-foreground">
                {isAchieved ? "Completed" : `${Math.round(progress)}%`}
              </span>
              {!isAchieved && (
                <span className="text-muted-foreground">
                  {milestone.daysRequired - currentCount} days to go
                </span>
              )}
            </div>
            <div className="h-1.5 overflow-hidden rounded-full bg-muted">
              <div
                className={cn(
                  "h-full rounded-full transition-all duration-500",
                  isAchieved ? "bg-primary" : "bg-gradient-to-r from-orange-500 to-red-500"
                )}
                style={{ width: `${Math.min(progress, 100)}%` }}
              />
            </div>
          </div>

          {/* Reward */}
          {milestone.reward && (
            <div className="mt-2 text-xs text-muted-foreground">
              Reward: {milestone.reward === "streak_freeze" ? "❄️ Freeze" : "🏆 Badge"}
            </div>
          )}
        </div>

        {isAchieved && (
          <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground">
            ✓
          </div>
        )}
      </div>
    </div>
  );
}

function getTierIcon(tier: string): string {
  switch (tier.toLowerCase()) {
    case "bronze":
      return "🥉";
    case "silver":
      return "🥈";
    case "gold":
      return "🥇";
    case "platinum":
      return "💎";
    case "diamond":
      return "👑";
    default:
      return "⭐";
  }
}
