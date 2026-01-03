// features/onboarding/steps/preferences-step.tsx - Configure default preferences

import { StepWrapper } from "../step-wrapper";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Timer, Coffee, Bell, PlayCircle, AlertCircle } from "lucide-react";

interface PreferencesStepProps {
  defaultFocusDuration: number;
  defaultBreakDuration: number;
  enableNotifications: boolean;
  autoStartBreaks: boolean;
  onFocusDurationChange: (value: number) => void;
  onBreakDurationChange: (value: number) => void;
  onNotificationsChange: (value: boolean) => void;
  onAutoStartBreaksChange: (value: boolean) => void;
  progress: number;
  currentStep: number;
  totalSteps: number;
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
}

const FOCUS_DURATIONS = [15, 25, 30, 45, 50, 60, 90];
const BREAK_DURATIONS = [5, 10, 15, 20];

export function PreferencesStep({
  defaultFocusDuration,
  defaultBreakDuration,
  enableNotifications,
  autoStartBreaks,
  onFocusDurationChange,
  onBreakDurationChange,
  onNotificationsChange,
  onAutoStartBreaksChange,
  progress,
  currentStep,
  totalSteps,
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
}: PreferencesStepProps) {
  return (
    <StepWrapper
      title="Set Your Defaults"
      description="Configure your ideal focus session and notification preferences"
      progress={progress}
      currentStep={currentStep}
      totalSteps={totalSteps}
      onNext={onNext}
      onPrevious={onPrevious}
      canGoNext={canGoNext}
      canGoPrevious={canGoPrevious}
    >
      {/* Info banner */}
      <div className="flex gap-3 p-4 rounded-lg bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800">
        <AlertCircle className="h-5 w-5 text-green-600 dark:text-green-400 mt-0.5 flex-shrink-0" />
        <div className="text-sm text-green-900 dark:text-green-100 space-y-1">
          <p className="font-medium">Pillar 2: Make Time for Traction</p>
          <p className="text-green-700 dark:text-green-300">
            These defaults help you quickly start focus sessions. You can always adjust them per
            session.
          </p>
        </div>
      </div>

      {/* Focus duration */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
              <Timer className="h-5 w-5 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <CardTitle className="text-base">Default Focus Duration</CardTitle>
              <CardDescription>How long do you typically want to focus?</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-4 sm:grid-cols-7 gap-2">
            {FOCUS_DURATIONS.map((duration) => (
              <button
                key={duration}
                onClick={() => onFocusDurationChange(duration)}
                className={`
                  px-4 py-3 rounded-lg border-2 font-medium transition-all
                  ${
                    defaultFocusDuration === duration
                      ? "border-primary bg-primary text-primary-foreground"
                      : "border-border bg-background hover:border-primary/50"
                  }
                `}
              >
                {duration}m
              </button>
            ))}
          </div>
          <p className="text-sm text-muted-foreground">
            Recommended: 25 minutes (Pomodoro technique)
          </p>
        </CardContent>
      </Card>

      {/* Break duration */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-amber-100 dark:bg-amber-900 flex items-center justify-center">
              <Coffee className="h-5 w-5 text-amber-600 dark:text-amber-400" />
            </div>
            <div>
              <CardTitle className="text-base">Default Break Duration</CardTitle>
              <CardDescription>How long should your breaks be?</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-4 gap-2">
            {BREAK_DURATIONS.map((duration) => (
              <button
                key={duration}
                onClick={() => onBreakDurationChange(duration)}
                className={`
                  px-4 py-3 rounded-lg border-2 font-medium transition-all
                  ${
                    defaultBreakDuration === duration
                      ? "border-primary bg-primary text-primary-foreground"
                      : "border-border bg-background hover:border-primary/50"
                  }
                `}
              >
                {duration}m
              </button>
            ))}
          </div>
          <p className="text-sm text-muted-foreground">Recommended: 5 minutes for short breaks</p>
        </CardContent>
      </Card>

      {/* Notification settings */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900 flex items-center justify-center">
              <Bell className="h-5 w-5 text-purple-600 dark:text-purple-400" />
            </div>
            <div>
              <CardTitle className="text-base">Notifications</CardTitle>
              <CardDescription>Manage how FocusFlow alerts you</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between py-2">
            <div className="space-y-0.5">
              <Label htmlFor="notifications" className="text-base font-medium">
                Enable notifications
              </Label>
              <p className="text-sm text-muted-foreground">
                Get notified when sessions start and end
              </p>
            </div>
            <Switch
              id="notifications"
              checked={enableNotifications}
              onCheckedChange={onNotificationsChange}
            />
          </div>
        </CardContent>
      </Card>

      {/* Auto-start breaks */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-green-100 dark:bg-green-900 flex items-center justify-center">
              <PlayCircle className="h-5 w-5 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <CardTitle className="text-base">Break Management</CardTitle>
              <CardDescription>Control how breaks are handled</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between py-2">
            <div className="space-y-0.5">
              <Label htmlFor="autoBreaks" className="text-base font-medium">
                Auto-start breaks
              </Label>
              <p className="text-sm text-muted-foreground">
                Automatically start break timer after focus sessions
              </p>
            </div>
            <Switch
              id="autoBreaks"
              checked={autoStartBreaks}
              onCheckedChange={onAutoStartBreaksChange}
            />
          </div>
        </CardContent>
      </Card>

      {/* Summary */}
      <div className="p-4 rounded-lg bg-muted/50 border">
        <h4 className="font-medium mb-2">Your Setup Summary</h4>
        <ul className="space-y-1 text-sm text-muted-foreground">
          <li>• Focus sessions: {defaultFocusDuration} minutes</li>
          <li>• Break duration: {defaultBreakDuration} minutes</li>
          <li>• Notifications: {enableNotifications ? "Enabled" : "Disabled"}</li>
          <li>• Auto-start breaks: {autoStartBreaks ? "Yes" : "No"}</li>
        </ul>
      </div>
    </StepWrapper>
  );
}
