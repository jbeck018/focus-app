// types/auth.ts - Authentication type definitions

export interface UserInfo {
  id: string;
  email: string;
  created_at: string;
  subscription_tier: SubscriptionTier;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: UserInfo;
}

export interface AuthState {
  is_authenticated: boolean;
  user: UserInfo | null;
  access_token: string | null;
  refresh_token: string | null;
  trailbase_url: string;
}

export type SubscriptionTier = "free" | "pro" | "team";

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterCredentials {
  email: string;
  password: string;
  confirmPassword?: string;
}

// Session limits per tier
export const SESSION_LIMITS: Record<SubscriptionTier, number> = {
  free: 3,
  pro: Infinity,
  team: Infinity,
} as const;

// Tier features
export interface TierFeatures {
  dailySessions: number;
  cloudSync: boolean;
  aiCoach: boolean;
  triggerJournaling: boolean;
  calendarIntegration: boolean;
  teamDashboard: boolean;
  sharedBlocklists: boolean;
}

export const TIER_FEATURES: Record<SubscriptionTier, TierFeatures> = {
  free: {
    dailySessions: 3,
    cloudSync: false,
    aiCoach: false,
    triggerJournaling: true,
    calendarIntegration: false,
    teamDashboard: false,
    sharedBlocklists: false,
  },
  pro: {
    dailySessions: Infinity,
    cloudSync: true,
    aiCoach: true,
    triggerJournaling: true,
    calendarIntegration: true,
    teamDashboard: false,
    sharedBlocklists: false,
  },
  team: {
    dailySessions: Infinity,
    cloudSync: true,
    aiCoach: true,
    triggerJournaling: true,
    calendarIntegration: true,
    teamDashboard: true,
    sharedBlocklists: true,
  },
} as const;
