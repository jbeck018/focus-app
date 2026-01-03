/**
 * useReducedMotion Hook
 *
 * WCAG 2.1 Level AAA - 2.3.3 Animation from Interactions
 * Detects user's motion preference from system settings.
 *
 * Respects prefers-reduced-motion media query to support users with:
 * - Vestibular disorders
 * - Motion sensitivity
 * - Preference for reduced animations
 *
 * Usage:
 * ```tsx
 * const prefersReducedMotion = useReducedMotion();
 * const animationClass = prefersReducedMotion ? '' : 'animate-pulse';
 * ```
 *
 * @returns boolean - true if user prefers reduced motion
 */

import { useState, useEffect } from "react";

export function useReducedMotion(): boolean {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(() => {
    // Initialize with the current value if available
    // SSR check - TypeScript doesn't know this might run on server
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (typeof window !== "undefined" && window.matchMedia) {
      return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    }
    return false;
  });

  useEffect(() => {
    // SSR check - matchMedia might not be available
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (typeof window === "undefined" || !window.matchMedia) {
      return;
    }

    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");

    // Listen for changes
    const handleChange = (event: MediaQueryListEvent) => {
      setPrefersReducedMotion(event.matches);
    };

    // Modern browsers - addEventListener is the standard API
    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, []);

  return prefersReducedMotion;
}
