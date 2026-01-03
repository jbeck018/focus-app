// features/mini-timer/types.ts - Type definitions for mini-timer

import type { ActiveSession } from "@focusflow/types";

/**
 * Timer state synchronized between main window and mini-timer
 */
export interface TimerState {
  activeSession: ActiveSession | null;
  elapsedSeconds: number;
  isRunning: boolean;
}

/**
 * Payload for timer state update events
 */
export interface TimerEventPayload {
  activeSession: ActiveSession | null;
  elapsedSeconds: number;
  isRunning: boolean;
}

/**
 * Window position coordinates
 */
export interface WindowPosition {
  x: number;
  y: number;
}

/**
 * Mini-timer window configuration
 */
export interface MiniTimerConfig {
  width: number;
  height: number;
  opacity: {
    idle: number;
    hover: number;
    dragging: number;
  };
  position?: WindowPosition;
}

/**
 * Mini-timer control actions
 */
export type MiniTimerAction =
  | { type: "toggle" }
  | { type: "stop" }
  | { type: "extend"; minutes: number }
  | { type: "close" };

/**
 * Default mini-timer configuration
 */
export const DEFAULT_MINI_TIMER_CONFIG: MiniTimerConfig = {
  width: 200,
  height: 80,
  opacity: {
    idle: 0.4,
    hover: 1.0,
    dragging: 0.8,
  },
};
