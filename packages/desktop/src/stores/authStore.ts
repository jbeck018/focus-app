// stores/authStore.ts - Zustand store for authentication state
// Session counts are now synced from the Rust backend (single source of truth)

import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";
import type { UserInfo, SubscriptionTier, TierFeatures } from "@focusflow/types";

// Backend response type for session count
interface SessionCountResponse {
  sessionsToday: number;
  dailyLimit: number;
  isUnlimited: boolean;
}

interface AuthStoreState {
  // Auth state
  isAuthenticated: boolean;
  isLoading: boolean;
  user: UserInfo | null;
  error: string | null;

  // Subscription state (synced from backend)
  subscriptionTier: SubscriptionTier;
  sessionsUsedToday: number;
  dailyLimit: number;
  isUnlimited: boolean;

  // Actions
  setUser: (user: UserInfo | null) => void;
  setAuthenticated: (isAuthenticated: boolean) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  setSubscriptionTier: (tier: SubscriptionTier) => void;
  syncSessionCount: () => Promise<void>;
  setSessionCount: (count: number, limit: number, isUnlimited: boolean) => void;
  resetDailySessionCount: () => void;
  logout: () => void;

  // Computed getters
  canStartSession: () => boolean;
  getRemainingDailySessions: () => number;
  getTierFeatures: () => TierFeatures;
}

const TIER_FEATURES_MAP: Record<SubscriptionTier, TierFeatures> = {
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
};

export const useAuthStore = create<AuthStoreState>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        isAuthenticated: false,
        isLoading: false,
        user: null,
        error: null,
        subscriptionTier: "free",
        sessionsUsedToday: 0,
        dailyLimit: 3,
        isUnlimited: false,

        // Actions
        setUser: (user) =>
          set({
            user,
            isAuthenticated: !!user,
            subscriptionTier: user?.subscription_tier ?? "free",
          }),

        setAuthenticated: (isAuthenticated) => set({ isAuthenticated }),

        setLoading: (isLoading) => set({ isLoading }),

        setError: (error) => set({ error }),

        setSubscriptionTier: (subscriptionTier) => set({ subscriptionTier }),

        // Sync session count from backend (single source of truth)
        syncSessionCount: async () => {
          try {
            const response = await invoke<SessionCountResponse>("get_todays_session_count");
            set({
              sessionsUsedToday: response.sessionsToday,
              dailyLimit: response.dailyLimit,
              isUnlimited: response.isUnlimited,
            });
          } catch (error) {
            console.error("Failed to sync session count:", error);
          }
        },

        // Direct setter for session count (used by event listener)
        setSessionCount: (count, limit, isUnlimited) =>
          set({
            sessionsUsedToday: count,
            dailyLimit: limit,
            isUnlimited,
          }),

        // Dev feature: reset daily count (for testing)
        resetDailySessionCount: () => set({ sessionsUsedToday: 0 }),

        logout: () =>
          set({
            isAuthenticated: false,
            user: null,
            error: null,
          }),

        // Computed getters
        canStartSession: () => {
          const state = get();
          if (state.isUnlimited) return true;
          return state.sessionsUsedToday < state.dailyLimit;
        },

        getRemainingDailySessions: () => {
          const state = get();
          if (state.isUnlimited) return Infinity;
          return Math.max(0, state.dailyLimit - state.sessionsUsedToday);
        },

        getTierFeatures: () => {
          const state = get();
          return TIER_FEATURES_MAP[state.subscriptionTier];
        },
      }),
      {
        name: "focusflow-auth",
        partialize: (state) => ({
          subscriptionTier: state.subscriptionTier,
          // Note: sessionsUsedToday is now synced from backend, but we keep it
          // in localStorage as a cache for offline/quick display
          sessionsUsedToday: state.sessionsUsedToday,
          dailyLimit: state.dailyLimit,
          isUnlimited: state.isUnlimited,
        }),
      }
    ),
    { name: "AuthStore" }
  )
);
