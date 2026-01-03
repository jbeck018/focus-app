/**
 * Streak Freeze Modal
 *
 * Allows users to use streak freezes to maintain their streak
 * - Shows available freezes (weekly + earned)
 * - Date picker for applying freeze
 * - Confirmation dialog
 * - Freeze expiration warnings
 */

import { useState } from "react";
import {
  useAvailableFreezes,
  useUseStreakFreeze,
  useCreateWeeklyFreeze,
} from "../../hooks/use-streaks";
import type { StreakFreeze, DateString, StreakId } from "@focusflow/types";
import { cn } from "../../lib/utils";

interface StreakFreezeModalProps {
  isOpen: boolean;
  onClose: () => void;
  suggestedDate?: string; // YYYY-MM-DD format
  className?: string;
}

export function StreakFreezeModal({
  isOpen,
  onClose,
  suggestedDate,
  className,
}: StreakFreezeModalProps) {
  const { data: freezes, isLoading } = useAvailableFreezes();
  const createWeeklyFreeze = useCreateWeeklyFreeze();
  const useFreeze = useUseStreakFreeze();

  const [selectedFreezeId, setSelectedFreezeId] = useState<string | null>(null);
  const [selectedDate, setSelectedDate] = useState<string>(
    suggestedDate || new Date().toISOString().split("T")[0]
  );
  const [showConfirm, setShowConfirm] = useState(false);

  if (!isOpen) return null;

  const handleUseFreeze = async () => {
    if (!selectedFreezeId) return;

    try {
      await useFreeze.mutateAsync({
        freezeId: selectedFreezeId as StreakId,
        date: selectedDate as DateString,
      });
      onClose();
    } catch (error) {
      console.error("Failed to use freeze:", error);
    }
  };

  const handleCreateWeeklyFreeze = async () => {
    try {
      await createWeeklyFreeze.mutateAsync();
    } catch (error) {
      console.error("Failed to create weekly freeze:", error);
    }
  };

  const selectedFreeze =
    freezes?.weeklyFreeze?.id === selectedFreezeId
      ? freezes.weeklyFreeze
      : (freezes?.earnedFreezes || []).find((f) => f.id === selectedFreezeId);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />

      {/* Modal */}
      <div
        className={cn(
          "relative z-10 w-full max-w-md rounded-lg border bg-card shadow-lg",
          className
        )}
      >
        {/* Header */}
        <div className="border-b p-6">
          <h2 className="text-xl font-semibold">Use Streak Freeze</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Protect your streak by using a freeze for a missed day
          </p>
        </div>

        {/* Content */}
        <div className="p-6">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="text-sm text-muted-foreground">Loading freezes...</div>
            </div>
          ) : freezes?.totalAvailable === 0 ? (
            <div className="rounded-lg border border-dashed p-8 text-center">
              <div className="text-4xl">❄️</div>
              <div className="mt-4 text-sm font-medium">No freezes available</div>
              <p className="mt-2 text-xs text-muted-foreground">
                Freezes refresh weekly on Monday or can be earned through achievements
              </p>
              <button
                onClick={handleCreateWeeklyFreeze}
                disabled={createWeeklyFreeze.isPending}
                className="mt-4 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                {createWeeklyFreeze.isPending ? "Checking..." : "Check for Weekly Freeze"}
              </button>
            </div>
          ) : (
            <>
              {/* Date Selection */}
              <div className="mb-6">
                <label className="block text-sm font-medium">Select Date to Apply Freeze</label>
                <input
                  type="date"
                  value={selectedDate}
                  onChange={(e) => setSelectedDate(e.target.value)}
                  max={new Date().toISOString().split("T")[0]}
                  className="mt-2 w-full rounded-md border bg-background px-3 py-2 text-sm"
                />
                <p className="mt-1 text-xs text-muted-foreground">
                  You can only freeze past or current dates
                </p>
              </div>

              {/* Available Freezes */}
              <div>
                <label className="block text-sm font-medium">Choose Freeze to Use</label>
                <div className="mt-2 space-y-2">
                  {/* Weekly Freeze */}
                  {freezes?.weeklyFreeze && (
                    <FreezeOption
                      freeze={freezes.weeklyFreeze}
                      isSelected={selectedFreezeId === freezes.weeklyFreeze.id}
                      onSelect={() => setSelectedFreezeId(freezes.weeklyFreeze!.id)}
                    />
                  )}

                  {/* Earned Freezes */}
                  {(freezes?.earnedFreezes || []).map((freeze) => (
                    <FreezeOption
                      key={freeze.id}
                      freeze={freeze}
                      isSelected={selectedFreezeId === freeze.id}
                      onSelect={() => setSelectedFreezeId(freeze.id)}
                    />
                  ))}
                </div>
              </div>

              {/* Confirmation Dialog */}
              {showConfirm && selectedFreeze && (
                <div className="mt-4 rounded-lg border border-orange-500/50 bg-orange-500/10 p-4">
                  <div className="text-sm font-medium">Confirm Freeze Usage</div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    Are you sure you want to use this {selectedFreeze.source} freeze for{" "}
                    {new Date(selectedDate).toLocaleDateString()}? This action cannot be undone.
                  </p>
                </div>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 border-t p-6">
          <button
            onClick={onClose}
            className="rounded-md border px-4 py-2 text-sm font-medium hover:bg-muted"
          >
            Cancel
          </button>
          {freezes && freezes.totalAvailable > 0 && (
            <button
              onClick={() => {
                if (showConfirm) {
                  handleUseFreeze();
                } else {
                  setShowConfirm(true);
                }
              }}
              disabled={!selectedFreezeId || useFreeze.isPending}
              className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {useFreeze.isPending ? "Using Freeze..." : showConfirm ? "Confirm" : "Use Freeze"}
            </button>
          )}
        </div>

        {/* Error Display */}
        {useFreeze.error && (
          <div className="border-t bg-destructive/10 p-4 text-sm text-destructive">
            Failed to use freeze. Please try again.
          </div>
        )}
      </div>
    </div>
  );
}

interface FreezeOptionProps {
  freeze: StreakFreeze;
  isSelected: boolean;
  onSelect: () => void;
}

function FreezeOption({ freeze, isSelected, onSelect }: FreezeOptionProps) {
  const isExpiring =
    freeze.expiresAt && new Date(freeze.expiresAt).getTime() - Date.now() < 24 * 60 * 60 * 1000;

  return (
    <button
      onClick={onSelect}
      className={cn(
        "w-full rounded-lg border p-4 text-left transition-all",
        isSelected
          ? "border-primary bg-primary/5 ring-2 ring-primary"
          : "hover:border-muted-foreground/50"
      )}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <span className="text-2xl">❄️</span>
            <div>
              <div className="font-medium capitalize">
                {freeze.source === "weekly" ? "Weekly Freeze" : "Earned Freeze"}
              </div>
              {freeze.source === "achievement" && (
                <div className="text-xs text-muted-foreground">From milestone achievement</div>
              )}
            </div>
          </div>

          {freeze.expiresAt && (
            <div
              className={cn(
                "mt-2 text-xs",
                isExpiring ? "text-orange-500" : "text-muted-foreground"
              )}
            >
              {isExpiring && "⚠️ "}Expires: {new Date(freeze.expiresAt).toLocaleDateString()}
            </div>
          )}
        </div>

        {isSelected && (
          <div className="flex h-5 w-5 items-center justify-center rounded-full bg-primary">
            <div className="h-2 w-2 rounded-full bg-primary-foreground" />
          </div>
        )}
      </div>
    </button>
  );
}
