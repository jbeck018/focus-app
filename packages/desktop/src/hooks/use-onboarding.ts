// hooks/use-onboarding.ts - Custom hook for managing onboarding state and flow

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  OnboardingStep,
  OnboardingState,
  OnboardingData,
  OnboardingSaveRequest,
  OnboardingCompleteResponse,
} from "@focusflow/types";

const STORAGE_KEY = "focusflow-onboarding";

const STEP_ORDER: OnboardingStep[] = [
  "welcome",
  "pillars",
  "blocklist",
  "preferences",
  "tutorial",
  "complete",
];

interface UseOnboardingReturn {
  // State
  currentStep: OnboardingStep;
  onboardingData: OnboardingData;
  isComplete: boolean;
  isLoading: boolean;
  error: string | null;

  // Navigation
  nextStep: () => void;
  previousStep: () => void;
  goToStep: (step: OnboardingStep) => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
  currentStepIndex: number;
  totalSteps: number;
  progress: number;

  // Data management
  updateData: <K extends keyof OnboardingData>(
    key: K,
    value: OnboardingData[K]
  ) => void;
  completeOnboarding: () => Promise<void>;
  skipOnboarding: () => void;
  resetOnboarding: () => void;
}

const getInitialData = (): OnboardingData => ({
  userName: "",
  selectedApps: [],
  selectedWebsites: [],
  defaultFocusDuration: 25,
  defaultBreakDuration: 5,
  enableNotifications: true,
  autoStartBreaks: false,
  viewedTutorials: [],
});

const getInitialState = (): OnboardingState => ({
  currentStep: "welcome",
  completedSteps: [],
  isComplete: false,
  startedAt: new Date().toISOString(),
});

export function useOnboarding(): UseOnboardingReturn {
  const [state, setState] = useState<OnboardingState>(getInitialState);
  const [data, setData] = useState<OnboardingData>(getInitialData);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load saved state from localStorage on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        setState(parsed.state || getInitialState());
        setData(parsed.data || getInitialData());
      }
    } catch (err) {
      console.error("Failed to load onboarding state:", err);
    }
  }, []);

  // Save state to localStorage whenever it changes
  useEffect(() => {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({ state, data })
      );
    } catch (err) {
      console.error("Failed to save onboarding state:", err);
    }
  }, [state, data]);

  // Calculate derived values
  const currentStepIndex = STEP_ORDER.indexOf(state.currentStep);
  const totalSteps = STEP_ORDER.length - 1; // Excluding "complete"
  const canGoNext = currentStepIndex < STEP_ORDER.length - 1;
  const canGoPrevious = currentStepIndex > 0;
  const progress = Math.round((currentStepIndex / totalSteps) * 100);

  // Navigation functions
  const nextStep = useCallback(() => {
    if (!canGoNext) return;

    setState((prev) => {
      const nextIndex = STEP_ORDER.indexOf(prev.currentStep) + 1;
      const nextStep = STEP_ORDER[nextIndex];

      return {
        ...prev,
        currentStep: nextStep,
        completedSteps: [...new Set([...prev.completedSteps, prev.currentStep])],
      };
    });
  }, [canGoNext]);

  const previousStep = useCallback(() => {
    if (!canGoPrevious) return;

    setState((prev) => {
      const prevIndex = STEP_ORDER.indexOf(prev.currentStep) - 1;
      const prevStep = STEP_ORDER[prevIndex];

      return {
        ...prev,
        currentStep: prevStep,
      };
    });
  }, [canGoPrevious]);

  const goToStep = useCallback((step: OnboardingStep) => {
    setState((prev) => ({
      ...prev,
      currentStep: step,
    }));
  }, []);

  // Data management functions
  const updateData = useCallback(
    <K extends keyof OnboardingData>(key: K, value: OnboardingData[K]) => {
      setData((prev) => ({
        ...prev,
        [key]: value,
      }));
    },
    []
  );

  const completeOnboarding = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Save to Rust backend
      const request: OnboardingSaveRequest = {
        userName: data.userName,
        selectedApps: data.selectedApps,
        selectedWebsites: data.selectedWebsites,
        defaultFocusDuration: data.defaultFocusDuration,
        defaultBreakDuration: data.defaultBreakDuration,
        enableNotifications: data.enableNotifications,
        autoStartBreaks: data.autoStartBreaks,
      };

      const response = await invoke<OnboardingCompleteResponse>(
        "complete_onboarding",
        { data: request }
      );

      if (!response.success) {
        throw new Error(response.error || "Failed to complete onboarding");
      }

      // Mark as complete
      setState((prev) => ({
        ...prev,
        isComplete: true,
        currentStep: "complete",
        completedAt: new Date().toISOString(),
      }));

      // Clear localStorage after successful completion
      setTimeout(() => {
        localStorage.removeItem(STORAGE_KEY);
      }, 1000);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      setError(message);
      console.error("Failed to complete onboarding:", err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [data]);

  const skipOnboarding = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isComplete: true,
      currentStep: "complete",
      completedAt: new Date().toISOString(),
    }));

    // Clear localStorage
    localStorage.removeItem(STORAGE_KEY);
  }, []);

  const resetOnboarding = useCallback(() => {
    setState(getInitialState());
    setData(getInitialData());
    localStorage.removeItem(STORAGE_KEY);
  }, []);

  return {
    // State
    currentStep: state.currentStep,
    onboardingData: data,
    isComplete: state.isComplete,
    isLoading,
    error,

    // Navigation
    nextStep,
    previousStep,
    goToStep,
    canGoNext,
    canGoPrevious,
    currentStepIndex,
    totalSteps,
    progress,

    // Data management
    updateData,
    completeOnboarding,
    skipOnboarding,
    resetOnboarding,
  };
}

// Hook to check if user needs onboarding
export function useNeedsOnboarding(): boolean {
  const [needsOnboarding, setNeedsOnboarding] = useState(true);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    const checkOnboarding = async () => {
      try {
        // Check localStorage first
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
          const parsed = JSON.parse(saved);
          if (parsed.state?.isComplete) {
            setNeedsOnboarding(false);
            setIsChecking(false);
            return;
          }
        }

        // Check backend
        const result = await invoke<boolean>("is_onboarding_complete");
        setNeedsOnboarding(!result);
      } catch (err) {
        console.error("Failed to check onboarding status:", err);
        // Default to showing onboarding if check fails
        setNeedsOnboarding(true);
      } finally {
        setIsChecking(false);
      }
    };

    checkOnboarding();
  }, []);

  return isChecking ? false : needsOnboarding;
}
