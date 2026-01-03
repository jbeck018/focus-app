/**
 * FocusTrap Component
 *
 * WCAG 2.1 Level AA - 2.4.3 Focus Order
 * Traps keyboard focus within a container (e.g., modals, dialogs).
 * Prevents focus from leaving the container until explicitly closed.
 *
 * ARIA Pattern: Modal Dialog
 * - Cycles focus from last to first focusable element (Tab)
 * - Cycles focus from first to last focusable element (Shift+Tab)
 * - Returns focus to trigger element on close
 *
 * Features:
 * - Auto-focus first focusable element on mount
 * - Restore focus to previous element on unmount
 * - Handle dynamic content changes
 */

import * as React from "react";

interface FocusTrapProps {
  children: React.ReactNode;
  enabled?: boolean;
  restoreFocus?: boolean;
  initialFocus?: React.RefObject<HTMLElement>;
  className?: string;
}

const FOCUSABLE_ELEMENTS = [
  "a[href]",
  "button:not([disabled])",
  "textarea:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(", ");

export function FocusTrap({
  children,
  enabled = true,
  restoreFocus = true,
  initialFocus,
  className,
}: FocusTrapProps) {
  const containerRef = React.useRef<HTMLDivElement>(null);
  const previousActiveElement = React.useRef<HTMLElement | null>(null);

  // Store the previously focused element
  React.useEffect(() => {
    if (enabled) {
      previousActiveElement.current = document.activeElement as HTMLElement;
    }
  }, [enabled]);

  // Set initial focus
  React.useEffect(() => {
    if (!enabled || !containerRef.current) return;

    const focusableElements = getFocusableElements(containerRef.current);

    if (initialFocus?.current) {
      initialFocus.current.focus();
    } else if (focusableElements.length > 0) {
      focusableElements[0].focus();
    }
  }, [enabled, initialFocus]);

  // Restore focus on unmount
  React.useEffect(() => {
    return () => {
      if (restoreFocus && previousActiveElement.current) {
        previousActiveElement.current.focus();
      }
    };
  }, [restoreFocus]);

  // Handle Tab and Shift+Tab
  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      if (!enabled || event.key !== "Tab" || !containerRef.current) return;

      const focusableElements = getFocusableElements(containerRef.current);
      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      if (event.shiftKey) {
        // Shift + Tab: moving backwards
        if (document.activeElement === firstElement) {
          event.preventDefault();
          lastElement?.focus();
        }
      } else {
        // Tab: moving forwards
        if (document.activeElement === lastElement) {
          event.preventDefault();
          firstElement?.focus();
        }
      }
    },
    [enabled]
  );

  if (!enabled) {
    return <>{children}</>;
  }

  return (
    <div ref={containerRef} onKeyDown={handleKeyDown} className={className}>
      {children}
    </div>
  );
}

// Helper function to get all focusable elements
function getFocusableElements(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_ELEMENTS)).filter(
    (element) =>
      !element.hasAttribute("disabled") &&
      !element.hasAttribute("aria-hidden") &&
      element.tabIndex !== -1
  );
}
