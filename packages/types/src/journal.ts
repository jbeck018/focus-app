// types/journal.ts - Trigger journaling type definitions

// Internal triggers (Indistractable framework)
export type InternalTrigger = "boredom" | "anxiety" | "stress" | "fatigue";

// External triggers
export type ExternalTrigger = "notification" | "person" | "environment" | "other";

export type TriggerType = InternalTrigger | ExternalTrigger;

export type Emotion =
  | "frustrated"
  | "anxious"
  | "tired"
  | "distracted"
  | "curious"
  | "bored"
  | "overwhelmed"
  | "neutral";

export interface JournalEntry {
  id: string;
  session_id: string | null;
  trigger_type: TriggerType;
  emotion: Emotion | null;
  notes: string | null;
  intensity: number | null; // 1-5
  created_at: string;
}

export interface CreateJournalEntryRequest {
  session_id?: string;
  trigger_type: TriggerType;
  emotion?: Emotion;
  notes?: string;
  intensity?: number;
}

export interface TriggerInsight {
  trigger_type: TriggerType;
  frequency: number;
  peak_hour: number | null;
  peak_day: number | null;
}

export interface PeakTimes {
  peak_hour: number | null;
  peak_day: number | null;
}

// Trigger display info
export const TRIGGER_INFO: Record<TriggerType, { label: string; emoji: string; category: "internal" | "external" }> = {
  boredom: { label: "Boredom", emoji: "😒", category: "internal" },
  anxiety: { label: "Anxiety", emoji: "😰", category: "internal" },
  stress: { label: "Stress", emoji: "😫", category: "internal" },
  fatigue: { label: "Fatigue", emoji: "😴", category: "internal" },
  notification: { label: "Notification", emoji: "🔔", category: "external" },
  person: { label: "Person", emoji: "👤", category: "external" },
  environment: { label: "Environment", emoji: "🏠", category: "external" },
  other: { label: "Other", emoji: "❓", category: "external" },
};

export const EMOTION_INFO: Record<Emotion, { label: string; emoji: string }> = {
  frustrated: { label: "Frustrated", emoji: "😤" },
  anxious: { label: "Anxious", emoji: "😰" },
  tired: { label: "Tired", emoji: "😴" },
  distracted: { label: "Distracted", emoji: "🤔" },
  curious: { label: "Curious", emoji: "🧐" },
  bored: { label: "Bored", emoji: "😒" },
  overwhelmed: { label: "Overwhelmed", emoji: "😵" },
  neutral: { label: "Neutral", emoji: "😐" },
};

// Day names for insights
export const DAY_NAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
