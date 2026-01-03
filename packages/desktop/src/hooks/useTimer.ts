// hooks/useTimer.ts - Timer logic with interval management

import { useCallback, useEffect, useRef, useState } from "react";

interface UseTimerOptions {
  initialSeconds?: number;
  targetSeconds?: number;
  onComplete?: () => void;
  onTick?: (seconds: number) => void;
}

export function useTimer({
  initialSeconds = 0,
  targetSeconds,
  onComplete,
  onTick,
}: UseTimerOptions = {}) {
  const [seconds, setSeconds] = useState(initialSeconds);
  const [isRunning, setIsRunning] = useState(false);
  const intervalRef = useRef<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const pausedTimeRef = useRef<number>(0);

  const start = useCallback(() => {
    if (isRunning) return;

    startTimeRef.current = Date.now() - pausedTimeRef.current * 1000;
    setIsRunning(true);
  }, [isRunning]);

  const pause = useCallback(() => {
    if (!isRunning) return;

    pausedTimeRef.current = seconds;
    setIsRunning(false);
  }, [isRunning, seconds]);

  const reset = useCallback(() => {
    setIsRunning(false);
    setSeconds(initialSeconds);
    pausedTimeRef.current = 0;
    startTimeRef.current = null;
  }, [initialSeconds]);

  const toggle = useCallback(() => {
    if (isRunning) {
      pause();
    } else {
      start();
    }
  }, [isRunning, pause, start]);

  useEffect(() => {
    if (!isRunning) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }

    intervalRef.current = window.setInterval(() => {
      if (startTimeRef.current === null) return;

      const elapsed = Math.floor((Date.now() - startTimeRef.current) / 1000);
      setSeconds(elapsed);
      onTick?.(elapsed);

      // Check if target reached
      if (targetSeconds && elapsed >= targetSeconds) {
        onComplete?.();
        setIsRunning(false);
      }
    }, 100); // Update every 100ms for smooth display

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [isRunning, targetSeconds, onComplete, onTick]);

  return {
    seconds,
    isRunning,
    start,
    pause,
    reset,
    toggle,
    setSeconds,
  };
}

// Format seconds to MM:SS or HH:MM:SS
export function formatTime(totalSeconds: number | undefined): string {
  // Handle undefined or NaN values by defaulting to 0
  const seconds = !isNaN(Number(totalSeconds)) ? Number(totalSeconds) : 0;

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours.toString().padStart(2, "0")}:${minutes
      .toString()
      .padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }

  return `${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

// Format seconds to human readable (e.g., "25 minutes")
export function formatDuration(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  return `${minutes} minute${minutes !== 1 ? "s" : ""}`;
}
