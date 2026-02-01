// hooks/useNavigation.ts - Simple navigation helper using custom events

import { useEffect } from "react";
import type { ViewType } from "@/components/app-sidebar";

// Custom event for navigation
const NAVIGATION_EVENT = "app-navigate";

export interface NavigationEvent extends CustomEvent {
  detail: {
    view: ViewType;
  };
}

/**
 * Navigate to a specific view
 */
export function navigateTo(view: ViewType) {
  const event = new CustomEvent(NAVIGATION_EVENT, {
    detail: { view },
  });
  window.dispatchEvent(event);
}

/**
 * Hook to listen for navigation events
 */
export function useNavigationListener(onNavigate: (view: ViewType) => void) {
  useEffect(() => {
    const handleNavigation = (event: Event) => {
      const navEvent = event as NavigationEvent;
      onNavigate(navEvent.detail.view);
    };

    window.addEventListener(NAVIGATION_EVENT, handleNavigation);

    return () => {
      window.removeEventListener(NAVIGATION_EVENT, handleNavigation);
    };
  }, [onNavigate]);
}
