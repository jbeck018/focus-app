// features/FocusTimer.tsx - Main focus timer component
// Timer state is owned by Rust backend for perfect cross-window sync

import { useCallback, useState, useEffect } from "react";
import { Play, Pause, Square, Lock, Maximize2, Plus } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { formatTime } from "@/hooks/useTimer";
import { useSessionStore } from "@/stores/sessionStore";
import { useAuthStore } from "@/stores/authStore";
import { useStartSession, useEndSession } from "@/hooks/useTauriCommands";
import type { SessionType } from "@focusflow/types";

const PRESET_DURATIONS = [
  { label: "25 min", minutes: 25, type: "focus" as SessionType },
  { label: "50 min", minutes: 50, type: "focus" as SessionType },
  { label: "5 min", minutes: 5, type: "break" as SessionType },
  { label: "15 min", minutes: 15, type: "break" as SessionType },
];

// Timer tick payload from Rust backend
interface TimerTickPayload {
  sessionId: string;
  elapsedSeconds: number;
  remainingSeconds: number;
  plannedDurationMinutes: number;
  sessionType: string;
  isRunning: boolean;
  isPaused: boolean;
}

// Extension duration presets in minutes
const EXTENSION_PRESETS = [5, 10, 15, 25];

export function FocusTimer() {
  const [showStartDialog, setShowStartDialog] = useState(false);
  const [showUpgradeDialog, setShowUpgradeDialog] = useState(false);
  const [showExtendDialog, setShowExtendDialog] = useState(false);
  const [extensionMinutes, setExtensionMinutes] = useState(15);
  const [isExtending, setIsExtending] = useState(false);
  const [duration, setDuration] = useState(25);
  const [sessionType, setSessionType] = useState<SessionType>("focus");
  const [announcement, setAnnouncement] = useState("");

  // Timer state from backend (single source of truth)
  const [timerTick, setTimerTick] = useState<TimerTickPayload | null>(null);

  const { activeSession, startSession, endSession, updateSessionDuration } = useSessionStore();
  const canStartSession = useAuthStore((s) => s.canStartSession);
  const getRemainingDailySessions = useAuthStore((s) => s.getRemainingDailySessions);
  const syncSessionCount = useAuthStore((s) => s.syncSessionCount);
  const setSessionCount = useAuthStore((s) => s.setSessionCount);
  const isUnlimited = useAuthStore((s) => s.isUnlimited);

  const startMutation = useStartSession();
  const endMutation = useEndSession();

  const remainingDailySessions = getRemainingDailySessions();
  const hasUnlimitedSessions = remainingDailySessions === Infinity;

  // Derived state from backend timer tick
  const isRunning = timerTick?.isRunning ?? false;
  const isPaused = timerTick?.isPaused ?? false;
  const elapsedSeconds = timerTick?.elapsedSeconds ?? 0;
  const remainingSeconds = timerTick?.remainingSeconds ?? 0;
  const plannedSeconds = (timerTick?.plannedDurationMinutes ?? duration) * 60;

  // Calculate progress percentage
  const progress =
    activeSession && plannedSeconds > 0
      ? Math.min((elapsedSeconds / plannedSeconds) * 100, 100)
      : 0;

  const displayTime = activeSession ? formatTime(remainingSeconds) : "00:00";

  // Handle stopping session
  const handleStopSession = useCallback(
    async (completed: boolean) => {
      try {
        await endMutation.mutateAsync(completed);
        endSession();
        setTimerTick(null);
      } catch (error: unknown) {
        console.error("Failed to end session:", error);
      }
    },
    [endMutation, endSession]
  );

  // Listen to backend timer ticks (broadcasts to all windows)
  useEffect(() => {
    const unlistenTick = listen<TimerTickPayload>("timer-tick", (event) => {
      setTimerTick(event.payload);
    });

    const unlistenComplete = listen<TimerTickPayload>("timer-completed", () => {
      // Auto-complete the session when timer finishes
      handleStopSession(true);
    });

    // Request initial state on mount
    invoke<TimerTickPayload | null>("get_timer_state")
      .then((state) => {
        if (state) setTimerTick(state);
      })
      .catch((err: unknown) => {
        console.debug("No active timer state:", err);
      });

    return () => {
      unlistenTick.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, [handleStopSession]);

  // Listen for session extension events
  useEffect(() => {
    const extendPromise = listen<{ plannedDurationMinutes: number }>(
      "session-extended",
      (event) => {
        updateSessionDuration(event.payload.plannedDurationMinutes);
      }
    );

    return () => {
      extendPromise.then((unlisten) => unlisten());
    };
  }, [updateSessionDuration]);

  // Listen for session count changes from backend (cross-window sync)
  useEffect(() => {
    // Sync session count from backend on mount
    syncSessionCount();

    // Listen for session-count-changed events
    const unlistenCount = listen<{ sessionsToday: number; dailyLimit: number }>(
      "session-count-changed",
      (event) => {
        setSessionCount(event.payload.sessionsToday, event.payload.dailyLimit, isUnlimited);
      }
    );

    return () => {
      unlistenCount.then((fn) => fn());
    };
  }, [syncSessionCount, setSessionCount, isUnlimited]);

  // Handle toggle pause/resume via backend
  const handleTogglePause = useCallback(async () => {
    try {
      await invoke("toggle_timer_pause");
    } catch (error) {
      console.error("Failed to toggle pause:", error);
    }
  }, []);

  // Handle extending the session
  const handleExtendSession = useCallback(async () => {
    if (isExtending) return;
    setIsExtending(true);
    try {
      await invoke("extend_session", { additionalMinutes: extensionMinutes });
      setShowExtendDialog(false);
      setAnnouncement(`Session extended by ${extensionMinutes} minutes`);
    } catch (error) {
      console.error("Failed to extend session:", error);
    } finally {
      setIsExtending(false);
    }
  }, [extensionMinutes, isExtending]);

  const handleStartButtonClick = () => {
    if (!canStartSession()) {
      setShowUpgradeDialog(true);
      return;
    }
    setShowStartDialog(true);
  };

  const handleStartSession = async () => {
    // Double-check limit before starting (frontend check for quick UX)
    if (!canStartSession()) {
      setShowStartDialog(false);
      setShowUpgradeDialog(true);
      return;
    }

    try {
      const response = await startMutation.mutateAsync({
        plannedDurationMinutes: duration,
        sessionType,
        blockedApps: [],
        blockedWebsites: [],
      });

      startSession({
        id: response.id,
        startTime: response.startTime,
        plannedDurationMinutes: duration,
        sessionType,
        blockedApps: [],
        blockedWebsites: [],
      });

      // Session count is now automatically synced from backend via session-count-changed event

      setShowStartDialog(false);
    } catch (error: unknown) {
      console.error("Failed to start session:", error);

      // Handle session limit error from backend
      const errorMessage = error instanceof Error ? error.message : String(error);
      if (
        errorMessage.includes("SessionLimitReached") ||
        errorMessage.includes("Daily session limit")
      ) {
        setShowStartDialog(false);
        setShowUpgradeDialog(true);
        // Sync the latest count from backend
        syncSessionCount();
      }
    }
  };

  // Announce time updates to screen readers at intervals
  // This effect synchronizes screen reader announcements with timer state
  useEffect(() => {
    if (!activeSession || !isRunning) return;

    const minutes = Math.floor(remainingSeconds / 60);
    const secs = remainingSeconds % 60;

    // Use queueMicrotask to avoid cascading renders
    queueMicrotask(() => {
      if (minutes > 0 && secs === 0 && minutes % 5 === 0) {
        setAnnouncement(`${minutes} minutes remaining`);
      } else if (minutes === 1 && secs === 0) {
        setAnnouncement("1 minute remaining");
      } else if (minutes === 0 && secs <= 10 && secs > 0) {
        setAnnouncement(`${secs} seconds remaining`);
      } else if (remainingSeconds === 0) {
        setAnnouncement(
          `${activeSession.sessionType === "focus" ? "Focus session" : "Break"} completed`
        );
      }
    });
  }, [remainingSeconds, activeSession, isRunning]);

  // Announce session state changes
  // This effect synchronizes screen reader announcements with session state
  useEffect(() => {
    // Use queueMicrotask to avoid cascading renders
    queueMicrotask(() => {
      if (activeSession && isRunning) {
        setAnnouncement(
          `${activeSession.sessionType === "focus" ? "Focus session" : "Break"} started`
        );
      } else if (activeSession && isPaused) {
        setAnnouncement("Timer paused");
      }
    });
  }, [activeSession, isRunning, isPaused]);

  // Handle opening mini-timer
  const handleOpenMiniTimer = useCallback(() => {
    invoke("open_mini_timer").catch((error: unknown) => {
      console.error("Failed to open mini-timer:", error);
    });
  }, []);

  return (
    <Card className="w-full">
      <CardHeader className="text-center">
        <div className="flex items-center justify-between">
          <div className="flex-1" />
          <CardTitle className="text-lg font-medium flex-1" id="timer-heading">
            {activeSession
              ? activeSession.sessionType === "focus"
                ? "Focus Session"
                : "Break Time"
              : "Ready to Focus?"}
          </CardTitle>
          <div className="flex-1 flex justify-end">
            {activeSession && (
              <Button
                size="sm"
                variant="ghost"
                onClick={handleOpenMiniTimer}
                className="h-8 w-8 p-0"
                aria-label="Open mini timer (Cmd+Shift+M)"
                title="Open mini timer (Cmd+Shift+M)"
              >
                <Maximize2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Live region for screen reader announcements */}
        <div role="status" aria-live="polite" aria-atomic="true" className="sr-only">
          {announcement}
        </div>

        {/* Timer Display */}
        <div className="text-center" role="timer" aria-label="Focus timer">
          <div
            className="text-6xl font-mono font-bold tracking-tight"
            aria-label={`Time remaining: ${displayTime}`}
          >
            {displayTime}
          </div>
          {activeSession && (
            <p className="text-sm text-muted-foreground mt-2" role="status">
              {isPaused ? "Paused" : isRunning ? "Session in progress" : "Starting..."}
            </p>
          )}
        </div>

        {/* Progress Bar */}
        {activeSession && (
          <Progress
            value={progress}
            className="h-2"
            aria-label={`Session progress: ${Math.round(progress)}%`}
          />
        )}

        {/* Controls */}
        <div className="flex justify-center gap-3">
          {activeSession ? (
            <>
              <Button
                size="lg"
                variant={isRunning ? "outline" : "default"}
                onClick={handleTogglePause}
                className="w-24"
                aria-label={isRunning ? "Pause timer" : "Resume timer"}
              >
                {isRunning ? (
                  <>
                    <Pause className="mr-2 h-4 w-4" aria-hidden="true" />
                    Pause
                  </>
                ) : (
                  <>
                    <Play className="mr-2 h-4 w-4" aria-hidden="true" />
                    Resume
                  </>
                )}
              </Button>
              <Button
                size="lg"
                variant="secondary"
                onClick={() => setShowExtendDialog(true)}
                className="w-24"
                aria-label="Extend session"
              >
                <Plus className="mr-2 h-4 w-4" aria-hidden="true" />
                Extend
              </Button>
              <Button
                size="lg"
                variant="destructive"
                onClick={() => handleStopSession(false)}
                className="w-24"
                aria-label="Stop session"
              >
                <Square className="mr-2 h-4 w-4" aria-hidden="true" />
                Stop
              </Button>

              {/* Extend Session Dialog */}
              <Dialog open={showExtendDialog} onOpenChange={setShowExtendDialog}>
                <DialogContent aria-describedby="extend-dialog-description">
                  <DialogHeader>
                    <DialogTitle>Extend Session</DialogTitle>
                    <DialogDescription id="extend-dialog-description">
                      Add more time to your current focus session
                    </DialogDescription>
                  </DialogHeader>
                  <div className="space-y-4 py-4">
                    {/* Extension Presets */}
                    <div className="space-y-2" role="group" aria-labelledby="extend-duration-label">
                      <Label id="extend-duration-label">Quick Add</Label>
                      <div className="flex flex-wrap gap-2">
                        {EXTENSION_PRESETS.map((preset) => (
                          <Button
                            key={preset}
                            variant={extensionMinutes === preset ? "default" : "outline"}
                            size="sm"
                            onClick={() => setExtensionMinutes(preset)}
                            aria-label={`Extend by ${preset} minutes`}
                            aria-pressed={extensionMinutes === preset}
                          >
                            +{preset} min
                          </Button>
                        ))}
                      </div>
                    </div>

                    {/* Custom Duration */}
                    <div className="space-y-2">
                      <Label htmlFor="extend-custom-duration">Custom (minutes)</Label>
                      <Input
                        id="extend-custom-duration"
                        type="number"
                        min={1}
                        max={120}
                        value={extensionMinutes}
                        onChange={(e) =>
                          setExtensionMinutes(
                            Math.min(120, Math.max(1, parseInt(e.target.value) || 1))
                          )
                        }
                      />
                    </div>
                  </div>
                  <DialogFooter>
                    <Button variant="outline" onClick={() => setShowExtendDialog(false)}>
                      Cancel
                    </Button>
                    <Button
                      onClick={handleExtendSession}
                      disabled={isExtending}
                      aria-label={isExtending ? "Extending session" : "Extend session"}
                    >
                      {isExtending ? "Extending..." : `Add ${extensionMinutes} min`}
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </>
          ) : (
            <>
              {/* Start Session Button */}
              <Button
                size="lg"
                className="w-40"
                onClick={handleStartButtonClick}
                aria-label="Start new focus session"
              >
                <Play className="mr-2 h-4 w-4" aria-hidden="true" />
                Start Session
              </Button>

              {/* Start Session Dialog */}
              <Dialog open={showStartDialog} onOpenChange={setShowStartDialog}>
                <DialogContent aria-describedby="dialog-description">
                  <DialogHeader>
                    <DialogTitle>Start a Focus Session</DialogTitle>
                    {!hasUnlimitedSessions && (
                      <DialogDescription id="dialog-description">
                        {remainingDailySessions} session{remainingDailySessions !== 1 ? "s" : ""}{" "}
                        remaining today
                      </DialogDescription>
                    )}
                  </DialogHeader>
                  <div className="space-y-4 py-4">
                    {/* Session Type */}
                    <Tabs
                      value={sessionType}
                      onValueChange={(v) => setSessionType(v as SessionType)}
                    >
                      <TabsList className="w-full" aria-label="Session type">
                        <TabsTrigger value="focus" className="flex-1" aria-label="Focus session">
                          Focus
                        </TabsTrigger>
                        <TabsTrigger value="break" className="flex-1" aria-label="Break session">
                          Break
                        </TabsTrigger>
                      </TabsList>
                    </Tabs>

                    {/* Duration Presets */}
                    <div className="space-y-2" role="group" aria-labelledby="duration-label">
                      <Label id="duration-label">Duration</Label>
                      <div className="flex flex-wrap gap-2">
                        {PRESET_DURATIONS.filter((p) => p.type === sessionType).map((preset) => (
                          <Button
                            key={preset.label}
                            variant={duration === preset.minutes ? "default" : "outline"}
                            size="sm"
                            onClick={() => setDuration(preset.minutes)}
                            aria-label={`Set duration to ${preset.label}`}
                            aria-pressed={duration === preset.minutes}
                          >
                            {preset.label}
                          </Button>
                        ))}
                      </div>
                    </div>

                    {/* Custom Duration */}
                    <div className="space-y-2">
                      <Label htmlFor="custom-duration">Custom (minutes)</Label>
                      <Input
                        id="custom-duration"
                        type="number"
                        min={1}
                        max={180}
                        value={duration}
                        onChange={(e) => setDuration(Math.max(1, parseInt(e.target.value) || 1))}
                      />
                    </div>
                  </div>
                  <DialogFooter>
                    <Button
                      onClick={handleStartSession}
                      disabled={startMutation.isPending}
                      aria-label={startMutation.isPending ? "Starting session" : "Start session"}
                    >
                      {startMutation.isPending ? "Starting..." : "Start"}
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>

              {/* Upgrade Dialog */}
              <Dialog open={showUpgradeDialog} onOpenChange={setShowUpgradeDialog}>
                <DialogContent aria-describedby="upgrade-description">
                  <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                      <Lock className="h-5 w-5" aria-hidden="true" />
                      Daily Limit Reached
                    </DialogTitle>
                    <DialogDescription id="upgrade-description">
                      You&apos;ve used all 3 free sessions for today. Upgrade to Pro for unlimited
                      sessions.
                    </DialogDescription>
                  </DialogHeader>
                  <div className="space-y-4 py-4">
                    <div className="p-4 bg-primary/5 rounded-lg border border-primary/20">
                      <p className="font-medium">FocusFlow Pro - $8/month</p>
                      <ul className="mt-2 text-sm text-muted-foreground space-y-1">
                        <li>Unlimited focus sessions</li>
                        <li>Cloud sync across devices</li>
                        <li>AI productivity coach</li>
                        <li>Calendar integration</li>
                        <li>Advanced analytics</li>
                      </ul>
                    </div>
                  </div>
                  <DialogFooter className="flex-col sm:flex-row gap-2">
                    <Button variant="outline" onClick={() => setShowUpgradeDialog(false)}>
                      Maybe Later
                    </Button>
                    <Button onClick={() => setShowUpgradeDialog(false)}>Upgrade to Pro</Button>
                    {/* Dev mode: simulate upgrade */}
                    <Button
                      variant="secondary"
                      onClick={async () => {
                        try {
                          // Set backend subscription tier
                          await invoke("dev_set_subscription_tier", { tier: "pro" });
                          // Update frontend state
                          useAuthStore.getState().setSubscriptionTier("pro");
                          useAuthStore.setState({ isUnlimited: true });
                          // Sync session count with new tier
                          await syncSessionCount();
                          setShowUpgradeDialog(false);
                          console.log("[Dev] Upgraded to Pro tier");
                        } catch (error) {
                          console.error("[Dev] Failed to upgrade:", error);
                        }
                      }}
                      className="text-xs"
                      title="Dev only: simulate Pro upgrade"
                    >
                      [Dev] Upgrade
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </>
          )}
        </div>

        {/* Session Limit Indicator (Free Tier Only) */}
        {/* Double-click to reset daily limit (dev feature) */}
        {!activeSession && !hasUnlimitedSessions && (
          <div
            className="text-center text-sm text-muted-foreground cursor-pointer select-none"
            role="status"
            aria-live="polite"
            onDoubleClick={() => {
              useAuthStore.getState().resetDailySessionCount();
              console.log("[Dev] Daily session count reset");
            }}
            title="Double-click to reset (dev)"
          >
            {remainingDailySessions > 0 ? (
              <span>{remainingDailySessions} of 3 sessions remaining today</span>
            ) : (
              <span className="text-destructive">Daily limit reached</span>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
