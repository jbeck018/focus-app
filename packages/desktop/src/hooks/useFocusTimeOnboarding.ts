// hooks/useFocusTimeOnboarding.ts - Focus Time onboarding state management

import { useCallback, useSyncExternalStore } from "react";
import { useFocusTimeEvents } from "./useFocusTime";

const STORAGE_KEY = "focusTimeOnboardingComplete";

// External store for localStorage sync
function subscribe(callback: () => void) {
  window.addEventListener("storage", callback);
  return () => window.removeEventListener("storage", callback);
}

function getSnapshot(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

function getServerSnapshot(): boolean {
  return false;
}

export interface UseFocusTimeOnboardingReturn {
  hasCompletedOnboarding: boolean;
  shouldShowOnboarding: boolean;
  completeOnboarding: () => void;
  resetOnboarding: () => void;
  skipOnboarding: () => void;
}

/**
 * Hook to manage Focus Time onboarding state
 * Shows onboarding if:
 * 1. User hasn't completed onboarding (localStorage)
 * 2. AND no Focus Time events exist in calendar
 */
export function useFocusTimeOnboarding(): UseFocusTimeOnboardingReturn {
  // Use useSyncExternalStore for proper localStorage sync without setState in effect
  const hasCompletedOnboarding = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);

  const { data: events, isLoading: eventsLoading } = useFocusTimeEvents();

  // Complete onboarding and save to localStorage
  const completeOnboarding = useCallback(() => {
    try {
      localStorage.setItem(STORAGE_KEY, "true");
      // Dispatch storage event for other tabs/components
      window.dispatchEvent(new StorageEvent("storage", { key: STORAGE_KEY }));
    } catch (err) {
      console.error("Failed to save Focus Time onboarding completion:", err);
    }
  }, []);

  // Reset onboarding state (for testing or re-triggering from settings)
  const resetOnboarding = useCallback(() => {
    try {
      localStorage.removeItem(STORAGE_KEY);
      // Dispatch storage event for other tabs/components
      window.dispatchEvent(new StorageEvent("storage", { key: STORAGE_KEY }));
    } catch (err) {
      console.error("Failed to reset Focus Time onboarding:", err);
    }
  }, []);

  // Skip onboarding without showing again
  const skipOnboarding = useCallback(() => {
    completeOnboarding();
  }, [completeOnboarding]);

  // Determine if we should show onboarding
  // Show if: not completed AND (loading OR no events exist)
  const shouldShowOnboarding =
    !hasCompletedOnboarding && (eventsLoading || !events || events.length === 0);

  return {
    hasCompletedOnboarding,
    shouldShowOnboarding,
    completeOnboarding,
    resetOnboarding,
    skipOnboarding,
  };
}
