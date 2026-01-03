/**
 * Type definitions for Focus Sessions
 *
 * Implements branded types for type safety and runtime validation
 */

// Branded types for compile-time safety
export type SessionId = string & { readonly __brand: "SessionId" };
export type Timestamp = number & { readonly __brand: "Timestamp" };
export type Minutes = number & { readonly __brand: "Minutes" };

// Type guards and constructors
export const SessionId = (id: string): SessionId => id as SessionId;
export const Timestamp = (ms: number): Timestamp => ms as Timestamp;
export const Minutes = (m: number): Minutes => {
  if (m < 0) throw new Error("Minutes cannot be negative");
  return m as Minutes;
};

// Session type enum with const assertion for literal types
export const SessionType = {
  FOCUS: "focus",
  BREAK: "break",
  LONG_BREAK: "long_break",
} as const;

export type SessionType = (typeof SessionType)[keyof typeof SessionType];

// Session status using discriminated union for type-safe state management
export type SessionStatus =
  | { type: "idle" }
  | { type: "running"; startedAt: Timestamp; pausedAt?: Timestamp }
  | { type: "paused"; startedAt: Timestamp; pausedAt: Timestamp; elapsedMs: number }
  | { type: "completed"; startedAt: Timestamp; completedAt: Timestamp; actualMinutes: Minutes }
  | { type: "interrupted"; startedAt: Timestamp; interruptedAt: Timestamp; reason?: string };

// Core session entity with readonly properties for immutability
export interface FocusSession {
  readonly id: SessionId;
  readonly startedAt: Timestamp | null;
  readonly endedAt: Timestamp | null;
  readonly plannedMinutes: Minutes;
  readonly actualMinutes: Minutes | null;
  readonly sessionType: SessionType;
  readonly completed: boolean;
  readonly interrupted: boolean;
  readonly notes: string | null;
  readonly createdAt: Timestamp;
}

// Session creation DTO with required fields only
export interface CreateSessionDTO {
  readonly plannedMinutes: Minutes;
  readonly sessionType: SessionType;
  readonly notes?: string;
}

// Session update DTO using Partial for optional updates
export type UpdateSessionDTO = Partial<{
  readonly endedAt: Timestamp;
  readonly actualMinutes: Minutes;
  readonly completed: boolean;
  readonly interrupted: boolean;
  readonly notes: string;
}>;

// Session with computed fields (never stored in DB)
export interface SessionWithStats extends FocusSession {
  readonly remainingMinutes: Minutes;
  readonly completionPercentage: number;
  readonly isOvertime: boolean;
}

// Session filters using optional discriminated union
export type SessionFilters = {
  readonly dateRange?: {
    readonly from: Timestamp;
    readonly to: Timestamp;
  };
  readonly sessionType?: SessionType;
  readonly completed?: boolean;
  readonly interrupted?: boolean;
};

// Session sort options with type-safe field names
export const SessionSortField = {
  STARTED_AT: "startedAt",
  ENDED_AT: "endedAt",
  PLANNED_MINUTES: "plannedMinutes",
  ACTUAL_MINUTES: "actualMinutes",
} as const;

export type SessionSortField = (typeof SessionSortField)[keyof typeof SessionSortField];

export interface SessionSortOptions {
  readonly field: SessionSortField;
  readonly direction: "asc" | "desc";
}

// Result type for error handling without exceptions
export type SessionResult<T> = { success: true; data: T } | { success: false; error: SessionError };

// Discriminated union for session errors
export type SessionError =
  | { type: "validation"; field: string; message: string }
  | { type: "not_found"; sessionId: SessionId }
  | { type: "already_running"; sessionId: SessionId }
  | { type: "database"; message: string }
  | { type: "unknown"; message: string };

// Type-safe session event system
export type SessionEvent =
  | { type: "session_started"; session: FocusSession }
  | { type: "session_paused"; sessionId: SessionId; pausedAt: Timestamp }
  | { type: "session_resumed"; sessionId: SessionId; resumedAt: Timestamp }
  | { type: "session_completed"; session: FocusSession }
  | { type: "session_interrupted"; session: FocusSession; reason?: string };

// Utility type for extracting event data by type
export type ExtractEventData<T extends SessionEvent["type"]> = Extract<SessionEvent, { type: T }>;
