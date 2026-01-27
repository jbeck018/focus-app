// types/calendar.ts - Calendar integration types

/** Supported calendar providers */
export type CalendarProvider = "google" | "microsoft";

/** Calendar connection status */
export interface CalendarConnection {
  provider: CalendarProvider;
  connected: boolean;
  email: string | null;
  last_sync: string | null;
}

/** Calendar event from external calendar */
export interface CalendarEvent {
  id: string;
  title: string;
  start_time: string;
  end_time: string;
  is_all_day: boolean;
  is_busy: boolean;
  location: string | null;
  provider: CalendarProvider;
}

/** Focus block suggestion based on calendar gaps */
export interface FocusBlockSuggestion {
  start_time: string;
  end_time: string;
  duration_minutes: number;
  reason: string;
}

/** Meeting load statistics */
export interface MeetingLoad {
  total_meeting_hours_this_week: number;
  average_daily_meetings: number;
  busiest_day: string | null;
  longest_free_block_minutes: number;
}

/** OAuth authorization URL response */
export interface AuthorizationUrl {
  url: string;
  state: string;
}

/** OAuth configuration status */
export interface OAuthConfigStatus {
  google_configured: boolean;
  microsoft_configured: boolean;
  google_setup_url: string;
  microsoft_setup_url: string;
}

/** Provider info for UI display */
export const PROVIDER_INFO: Record<CalendarProvider, { label: string; icon: string; color: string }> = {
  google: {
    label: "Google Calendar",
    icon: "📅",
    color: "#4285F4",
  },
  microsoft: {
    label: "Microsoft Outlook",
    icon: "📆",
    color: "#0078D4",
  },
};
