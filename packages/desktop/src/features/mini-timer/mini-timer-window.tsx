// features/mini-timer/mini-timer-window.tsx - Floating mini-timer window component
// Timer state is owned by Rust backend, with local ticking for smooth updates

import { useEffect, useState, useCallback, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { MiniTimerControls } from "./mini-timer-controls";
import { formatTime } from "@/hooks/useTimer";

// Timer tick payload from Rust backend (matches commands/timer.rs TimerTickPayload)
interface TimerTickPayload {
  sessionId: string;
  elapsedSeconds: number;
  remainingSeconds: number;
  plannedDurationMinutes: number;
  sessionType: string;
  isRunning: boolean;
  isPaused: boolean;
}

export function MiniTimerWindow() {
  // Backend state (synced from events)
  const [backendState, setBackendState] = useState<TimerTickPayload | null>(null);
  // Local display seconds (ticks down independently for smooth UI)
  const [displaySeconds, setDisplaySeconds] = useState<number | null>(null);
  const [opacity, setOpacity] = useState(0.4);
  const [isDragging, setIsDragging] = useState(false);

  // Track if we've received any backend events
  const hasReceivedEvent = useRef(false);

  const currentWindow = getCurrentWindow();

  // Derived state
  const hasActiveSession = backendState !== null;
  const isRunning = backendState?.isRunning ?? false;
  const isPaused = backendState?.isPaused ?? false;
  const remainingSeconds = displaySeconds ?? backendState?.remainingSeconds ?? 0;
  const sessionType = backendState?.sessionType ?? "focus";
  const isBreak = sessionType === "break";

  const displayTime = hasActiveSession ? formatTime(remainingSeconds) : "--:--";

  // Listen to backend timer ticks and sync state
  useEffect(() => {
    console.info("[mini-timer] Setting up event listeners...");

    const unlistenTick = listen<TimerTickPayload>("timer-tick", (event) => {
      hasReceivedEvent.current = true;
      setBackendState(event.payload);
      setDisplaySeconds(event.payload.remainingSeconds);
    });

    const unlistenComplete = listen<TimerTickPayload>("timer-completed", () => {
      // Session completed
      setBackendState(null);
      setDisplaySeconds(null);
    });

    const unlistenExtend = listen<{ plannedDurationMinutes: number }>(
      "session-extended",
      (event) => {
        // Session extended
        setBackendState((prev) =>
          prev ? { ...prev, plannedDurationMinutes: event.payload.plannedDurationMinutes } : null
        );
      }
    );

    // Request initial state from backend on mount
    invoke<TimerTickPayload | null>("get_timer_state")
      .then((state) => {
        if (state) {
          // Initial state loaded
          setBackendState(state);
          setDisplaySeconds(state.remainingSeconds);
        }
      })
      .catch(() => {});

    return () => {
      unlistenTick.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenExtend.then((fn) => fn());
    };
  }, []);

  // Local timer: tick down every second for smooth UI
  // If backend events aren't arriving, this keeps the timer moving
  useEffect(() => {
    if (!hasActiveSession || isPaused) return;

    const tickInterval = setInterval(() => {
      setDisplaySeconds((prev) => {
        if (prev === null || prev <= 0) return prev;
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(tickInterval);
  }, [hasActiveSession, isPaused]);

  // Handle hover for opacity changes
  const handleMouseEnter = useCallback(() => {
    if (!isDragging) {
      setOpacity(1);
    }
  }, [isDragging]);

  const handleMouseLeave = useCallback(() => {
    if (!isDragging) {
      setOpacity(0.4);
    }
  }, [isDragging]);

  // Handle window dragging
  const handleMouseDown = useCallback(() => {
    setIsDragging(true);
    setOpacity(0.8);
    currentWindow.startDragging().catch((error: unknown) => {
      console.error("Failed to start dragging:", error);
    });
  }, [currentWindow]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    setOpacity(0.4);

    // Save position after dragging
    currentWindow
      .outerPosition()
      .then((position) => {
        invoke("set_mini_timer_position", {
          position: {
            x: position.x,
            y: position.y,
          },
        }).catch((error: unknown) => {
          console.error("Failed to save position:", error);
        });
      })
      .catch((error: unknown) => {
        console.error("Failed to get position:", error);
      });
  }, [currentWindow]);

  // Handle double-click to open main window
  const handleDoubleClick = useCallback(() => {
    invoke("focus_main_window").catch((error: unknown) => {
      console.error("Failed to focus main window:", error);
    });
  }, []);

  // Handle right-click context menu (future: settings menu)
  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    // Future: Show context menu with settings options
  }, []);

  return (
    <div
      className="h-screen w-full flex items-center justify-center select-none"
      style={{
        opacity,
        transition: isDragging ? "none" : "opacity 0.2s ease",
      }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onContextMenu={handleContextMenu}
    >
      <div
        className={`
          relative rounded-lg backdrop-blur-md shadow-2xl overflow-hidden
          ${isBreak ? "bg-green-500/20 border border-green-500/30" : "bg-blue-500/20 border border-blue-500/30"}
        `}
      >
        {/* Draggable header area */}
        <div
          className="px-4 py-2 cursor-move"
          onMouseDown={handleMouseDown}
          onMouseUp={handleMouseUp}
          onDoubleClick={handleDoubleClick}
        >
          <div className="flex items-center justify-between gap-2">
            {/* Session indicator */}
            <div className="flex items-center gap-2">
              <div
                className={`
                  w-2 h-2 rounded-full
                  ${isRunning ? "bg-green-400 animate-pulse" : isPaused ? "bg-yellow-400" : "bg-gray-400"}
                `}
              />
              <span className="text-xs font-medium text-white/80">
                {hasActiveSession ? (isPaused ? "Paused" : isBreak ? "Break" : "Focus") : "Idle"}
              </span>
            </div>

            {/* Close button */}
            <button
              type="button"
              onClick={() => invoke("close_mini_timer")}
              className="text-white/60 hover:text-white/90 transition-colors"
              aria-label="Close mini timer"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>

        {/* Timer display */}
        <div className="px-4 pb-2">
          <div className="text-3xl font-mono font-bold text-white tracking-tight text-center">
            {displayTime}
          </div>
        </div>

        {/* Controls */}
        {hasActiveSession && (
          <div className="px-4 pb-3">
            <MiniTimerControls isRunning={isRunning} isPaused={isPaused} />
          </div>
        )}

        {/* No active session message */}
        {!hasActiveSession && (
          <div className="px-4 pb-3 text-center">
            <p className="text-xs text-white/60">No active session</p>
          </div>
        )}
      </div>
    </div>
  );
}
