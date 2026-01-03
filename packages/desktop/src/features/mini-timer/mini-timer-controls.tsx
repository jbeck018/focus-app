// features/mini-timer/mini-timer-controls.tsx - Compact control buttons for mini-timer
// Uses backend toggle_timer_pause command for cross-window sync

import { useCallback, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface MiniTimerControlsProps {
  isRunning: boolean;
  isPaused: boolean;
}

export function MiniTimerControls({ isRunning, isPaused }: MiniTimerControlsProps) {
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<"toggle" | "stop" | "extend" | null>(null);

  // Auto-clear errors after 3 seconds
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  // Handle pause/resume toggle via backend command
  const handleToggle = useCallback(async () => {
    setIsLoading("toggle");
    setError(null);
    try {
      await invoke("toggle_timer_pause");
    } catch (err) {
      console.error("Failed to toggle timer:", err);
      setError("Failed to toggle");
    } finally {
      setIsLoading(null);
    }
  }, []);

  // Handle stop session
  const handleStop = useCallback(async () => {
    setIsLoading("stop");
    setError(null);
    try {
      const completed = false; // User manually stopped
      await invoke("end_focus_session", { completed });
    } catch (err) {
      console.error("Failed to stop session:", err);
      setError("Failed to stop");
    } finally {
      setIsLoading(null);
    }
  }, []);

  // Handle add 5 minutes
  const handleAddTime = useCallback(async () => {
    setIsLoading("extend");
    setError(null);
    try {
      await invoke("extend_session", { additionalMinutes: 5 });
    } catch (err) {
      console.error("Failed to extend session:", err);
      setError("Failed to extend");
    } finally {
      setIsLoading(null);
    }
  }, []);

  // Show play icon when paused, pause icon when running
  const showPlayIcon = isPaused || !isRunning;

  return (
    <div className="flex flex-col items-center gap-1">
      {/* Error indicator */}
      {error && (
        <div className="text-xs text-red-400 animate-pulse mb-1">
          {error}
        </div>
      )}

      <div className="flex items-center justify-center gap-2">
        {/* Pause/Resume button */}
        <button
          type="button"
          onClick={handleToggle}
          disabled={isLoading !== null}
          className={`
            p-2 rounded-md transition-all
            bg-white/10 hover:bg-white/20 active:bg-white/30
            text-white/90 hover:text-white
            disabled:opacity-50 disabled:cursor-not-allowed
            ${isLoading === "toggle" ? "animate-pulse" : ""}
          `}
          aria-label={showPlayIcon ? "Resume timer" : "Pause timer"}
          title={showPlayIcon ? "Resume" : "Pause"}
        >
        {showPlayIcon ? (
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polygon points="5 3 19 12 5 21 5 3" />
          </svg>
        ) : (
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <rect x="6" y="4" width="4" height="16" />
            <rect x="14" y="4" width="4" height="16" />
          </svg>
        )}
      </button>

        {/* Stop button */}
        <button
          type="button"
          onClick={handleStop}
          disabled={isLoading !== null}
          className={`
            p-2 rounded-md transition-all
            bg-red-500/20 hover:bg-red-500/30 active:bg-red-500/40
            text-red-300 hover:text-red-200
            disabled:opacity-50 disabled:cursor-not-allowed
            ${isLoading === "stop" ? "animate-pulse" : ""}
          `}
          aria-label="Stop session"
          title="Stop"
        >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        </svg>
      </button>

        {/* Add 5 minutes button */}
        <button
          type="button"
          onClick={handleAddTime}
          disabled={isLoading !== null}
          className={`
            px-2 py-2 rounded-md transition-all
            bg-white/10 hover:bg-white/20 active:bg-white/30
            text-white/90 hover:text-white
            text-xs font-medium
            disabled:opacity-50 disabled:cursor-not-allowed
            ${isLoading === "extend" ? "animate-pulse" : ""}
          `}
          aria-label="Add 5 minutes"
          title="Add 5 minutes"
        >
          +5m
        </button>
      </div>
    </div>
  );
}
