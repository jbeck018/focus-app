// types/index.ts - Core type definitions for FocusFlow

export type SessionType = "focus" | "break" | "custom";

export interface Session {
  id: string;
  startTime: string;
  endTime?: string;
  plannedDurationMinutes: number;
  actualDurationSeconds?: number;
  sessionType: SessionType;
  completed: boolean;
  notes?: string;
}

export interface ActiveSession {
  id: string;
  startTime: string;
  plannedDurationMinutes: number;
  sessionType: SessionType;
  blockedApps: string[];
  blockedWebsites: string[];
}

export interface BlockedItem {
  id: number;
  itemType: "app" | "website";
  value: string;
  enabled: boolean;
}

// Backend response type for get_blocked_items command
export interface BlockedItemsResponse {
  apps: string[];
  websites: string[];
}

export interface DailyAnalytics {
  date: string;
  totalFocusSeconds: number;
  totalBreakSeconds: number;
  sessionsCompleted: number;
  sessionsAbandoned: number;
  productivityScore?: number;
}

export interface StartSessionRequest {
  plannedDurationMinutes: number;
  sessionType: SessionType;
  blockedApps: string[];
  blockedWebsites: string[];
}

export interface SessionResponse {
  id: string;
  startTime: string;
  plannedDurationMinutes: number;
  sessionType: string;
}

export interface ExportData {
  version: string;
  exportedAt: string;
  sessions: Session[];
  blockedItems: BlockedItem[];
}
