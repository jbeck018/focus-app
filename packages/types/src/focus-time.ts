// types/focus-time.ts - Focus Time calendar-based blocking types

import type { CalendarProvider } from "./calendar";

/** Focus Time event parsed from calendar */
export interface FocusTimeEvent {
  id: string;
  title: string;
  start_time: string;
  end_time: string;
  provider: CalendarProvider;
  allowed_apps: string[];
  allowed_categories: string[];
  is_active: boolean;
  created_from_calendar: boolean;
}

/** Allowed application for Focus Time */
export interface AllowedApp {
  name: string;
  category?: string;
  icon?: string;
}

/** Focus Time state */
export interface FocusTimeState {
  active: boolean;
  current_event: FocusTimeEvent | null;
  allowed_apps: string[];
  allowed_categories: string[];
  remaining_seconds: number;
  can_override: boolean;
}

/** Focus Time override action */
export interface FocusTimeOverride {
  action: "add_app" | "remove_app" | "end_early";
  app_name?: string;
  reason?: string;
}

/** Focus Time calendar event format */
export interface FocusTimeCalendarFormat {
  title_prefix: string; // e.g., "🎯 Focus Time"
  allowed_apps_tag: string; // e.g., "@coding @slack"
  category_mapping: Record<string, string[]>;
}

/** Predefined app categories */
export const FOCUS_TIME_CATEGORIES = {
  "@coding": ["Visual Studio Code", "IntelliJ IDEA", "Xcode", "Sublime Text", "Vim"],
  "@communication": ["Slack", "Microsoft Teams", "Discord", "Zoom", "Telegram"],
  "@browser": ["Google Chrome", "Firefox", "Safari", "Microsoft Edge"],
  "@design": ["Figma", "Sketch", "Adobe Photoshop", "Adobe Illustrator"],
  "@productivity": ["Notion", "Obsidian", "Evernote", "Microsoft OneNote"],
  "@terminal": ["Terminal", "iTerm2", "Alacritty", "Hyper"],
  "@music": ["Spotify", "Apple Music", "YouTube Music"],
} as const;

export type FocusTimeCategory = keyof typeof FOCUS_TIME_CATEGORIES;

/** Focus Time setup instructions */
export const FOCUS_TIME_INSTRUCTIONS = {
  google: {
    title: "How to create Focus Time events in Google Calendar",
    steps: [
      "Open Google Calendar",
      'Create a new event with title starting with "🎯 Focus Time"',
      'Add allowed apps in description: "@coding @communication"',
      'Or list specific apps: "Visual Studio Code, Slack"',
      "Save the event",
    ],
    example: '🎯 Focus Time: Deep Work\nDescription: @coding @terminal @music',
  },
  microsoft: {
    title: "How to create Focus Time events in Outlook Calendar",
    steps: [
      "Open Outlook Calendar",
      'Create a new event with title starting with "🎯 Focus Time"',
      'Add allowed apps in notes: "@coding @communication"',
      'Or list specific apps: "Visual Studio Code, Slack"',
      "Save the event",
    ],
    example: '🎯 Focus Time: Deep Work\nNotes: @coding @terminal @music',
  },
} as const;
