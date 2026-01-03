/**
 * Example Usage of Enhanced Streak System
 *
 * This file demonstrates how to integrate the streak system into your app.
 * Remove this file in production - it's for reference only.
 */

import { useCallback, useEffect, useState } from "react";
import { StreakDashboard } from "./streak-dashboard";
import { MilestoneCelebration } from "./streak-milestone-celebration";
import type { StreakMilestone } from "@focusflow/types";
import {
  useStreaks,
  useUpdateStreakHistory,
  useStreakNotifications,
} from "../../hooks/use-streaks";

/**
 * Example 1: Full Streak Dashboard
 *
 * Drop-in dashboard showing all streak features
 */
export function Example1_FullDashboard() {
  return (
    <div className="container mx-auto p-6">
      <StreakDashboard />
    </div>
  );
}

/**
 * Example 2: Updating Streak After Session Completion
 *
 * Call this when a user completes a focus session
 */
export function Example2_SessionCompleteHandler() {
  const updateStreak = useUpdateStreakHistory();

  const handleSessionComplete = () => {
    // Your existing session completion logic
    console.log("Session completed!");

    // Update streak history for today
    updateStreak.mutate(undefined, {
      onSuccess: (data) => {
        console.log("Streak updated:", data);
      },
      onError: (error) => {
        console.error("Failed to update streak:", error);
      },
    });
  };

  return (
    <button
      onClick={handleSessionComplete}
      className="rounded-md bg-primary px-4 py-2 text-primary-foreground"
    >
      Complete Session
    </button>
  );
}

/**
 * Example 3: Streak Status Widget
 *
 * Minimal widget showing current streak
 */
export function Example3_StreakWidget() {
  const { currentStreak, isLoading } = useStreaks();

  if (isLoading) return <div>Loading...</div>;

  return (
    <div className="rounded-lg border bg-card p-4">
      <div className="text-sm text-muted-foreground">Current Streak</div>
      <div className="mt-1 flex items-baseline gap-2">
        <span className="text-3xl font-bold">{currentStreak?.currentCount ?? 0}</span>
        <span className="text-muted-foreground">days</span>
      </div>
      {currentStreak?.isInGracePeriod && (
        <div className="mt-2 text-xs text-orange-500">Grace period active</div>
      )}
    </div>
  );
}

/**
 * Example 4: Streak Notifications
 *
 * Display notifications for streak status
 */
export function Example4_StreakNotifications() {
  const { shouldNotifyGracePeriod, shouldNotifyRiskBroken, gracePeriodEndsAt, currentCount } =
    useStreakNotifications();

  if (shouldNotifyGracePeriod) {
    return (
      <div className="rounded-lg border border-orange-500 bg-orange-500/10 p-4">
        <div className="font-medium">Grace Period Active</div>
        <p className="text-sm text-muted-foreground">
          Complete a session before{" "}
          {gracePeriodEndsAt && new Date(gracePeriodEndsAt).toLocaleTimeString()} to maintain your
          streak
        </p>
      </div>
    );
  }

  if (shouldNotifyRiskBroken) {
    return (
      <div className="rounded-lg border border-red-500 bg-red-500/10 p-4">
        <div className="font-medium">Streak at Risk!</div>
        <p className="text-sm text-muted-foreground">
          Your {currentCount}-day streak will break if you don't complete a session today
        </p>
      </div>
    );
  }

  return null;
}

/**
 * Example 5: Milestone Achievement Handler
 *
 * Automatically show celebrations when milestones are achieved
 */
export function Example5_MilestoneHandler() {
  const { milestones } = useStreaks();
  const [celebrating, setCelebrating] = useState<StreakMilestone | null>(null);

  useEffect(() => {
    if (!milestones) return;

    // Find milestone achieved in the last 5 seconds
    const justAchieved = milestones.find(
      (m) => m.isAchieved && m.achievedAt && Date.now() - new Date(m.achievedAt).getTime() < 5000
    );

    if (justAchieved && !celebrating) {
      // Schedule state update for next tick to avoid cascading renders
      const timer = setTimeout(() => {
        setCelebrating(justAchieved);
      }, 0);
      return () => clearTimeout(timer);
    }
  }, [milestones, celebrating]);

  return (
    <>
      {celebrating && (
        <MilestoneCelebration milestone={celebrating} onDismiss={() => setCelebrating(null)} />
      )}
    </>
  );
}

/**
 * Example 6: Using Individual Components
 *
 * Mix and match components for custom layouts
 */
export function Example6_CustomLayout() {
  const { currentStreak, heatmap: _heatmap, weekStats, freezes } = useStreaks();

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      {/* Current Streak Card */}
      <div className="rounded-lg border bg-card p-6">
        <h3 className="text-lg font-semibold">Your Streak</h3>
        <div className="mt-4 flex items-baseline gap-2">
          <span className="text-5xl font-bold">{currentStreak?.currentCount ?? 0}</span>
          <span className="text-xl text-muted-foreground">days</span>
        </div>
        {freezes && freezes.totalAvailable > 0 && (
          <div className="mt-4 text-sm text-muted-foreground">
            {freezes.totalAvailable} freeze{freezes.totalAvailable !== 1 ? "s" : ""} available
          </div>
        )}
      </div>

      {/* Weekly Stats Card */}
      <div className="rounded-lg border bg-card p-6">
        <h3 className="text-lg font-semibold">This Week</h3>
        {weekStats && (
          <div className="mt-4 grid grid-cols-2 gap-4">
            <div>
              <div className="text-sm text-muted-foreground">Sessions</div>
              <div className="text-2xl font-bold">{weekStats.totalSessions}</div>
            </div>
            <div>
              <div className="text-sm text-muted-foreground">Focus Time</div>
              <div className="text-2xl font-bold">
                {Math.round(weekStats.totalFocusMinutes / 60)}h
              </div>
            </div>
            <div>
              <div className="text-sm text-muted-foreground">Active Days</div>
              <div className="text-2xl font-bold">{weekStats.activeDays}</div>
            </div>
            <div>
              <div className="text-sm text-muted-foreground">Perfect Days</div>
              <div className="text-2xl font-bold">{weekStats.perfectDays}</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Example 7: Integration with Focus Timer
 *
 * Update streak when timer completes
 */
export function Example7_FocusTimerIntegration() {
  const [timeLeft, setTimeLeft] = useState(1500); // 25 minutes
  const [isRunning, setIsRunning] = useState(false);
  const updateStreak = useUpdateStreakHistory();

  const handleTimerComplete = useCallback(() => {
    console.log("Timer completed!");
    // Update streak history
    updateStreak.mutate();
  }, [updateStreak]);

  useEffect(() => {
    if (!isRunning || timeLeft <= 0) return;

    const timer = setInterval(() => {
      setTimeLeft((prev) => {
        if (prev <= 1) {
          setIsRunning(false);
          handleTimerComplete();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [isRunning, timeLeft, handleTimerComplete]);

  return (
    <div className="rounded-lg border bg-card p-6">
      <div className="text-center">
        <div className="text-4xl font-bold">
          {Math.floor(timeLeft / 60)}:{(timeLeft % 60).toString().padStart(2, "0")}
        </div>
        <button
          onClick={() => setIsRunning(!isRunning)}
          className="mt-4 rounded-md bg-primary px-6 py-2 text-primary-foreground"
        >
          {isRunning ? "Pause" : "Start"}
        </button>
      </div>
    </div>
  );
}

/**
 * Example 8: Refetching Streak Data
 *
 * Manually refresh streak data when needed
 */
export function Example8_ManualRefresh() {
  const { currentStreak, refetch, isLoading } = useStreaks();

  return (
    <div className="rounded-lg border bg-card p-6">
      <div className="flex items-center justify-between">
        <div>
          <div className="text-sm text-muted-foreground">Current Streak</div>
          <div className="text-2xl font-bold">{currentStreak?.currentCount ?? 0} days</div>
        </div>
        <button
          onClick={() => refetch()}
          disabled={isLoading}
          className="rounded-md border px-4 py-2 text-sm hover:bg-muted disabled:opacity-50"
        >
          {isLoading ? "Refreshing..." : "Refresh"}
        </button>
      </div>
    </div>
  );
}
