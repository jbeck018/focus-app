/**
 * ScreenReaderOnly Component
 *
 * WCAG 2.1 - Screen Reader Support
 * Visually hides content while keeping it accessible to screen readers.
 *
 * Use cases:
 * - Descriptive labels for icon buttons
 * - Additional context for screen reader users
 * - Status messages and live region updates
 *
 * Note: Uses sr-only utility class for proper accessibility
 * (clips content, positions off-screen, maintains readability)
 */

import * as React from "react";
import { cn } from "@/lib/utils";

interface ScreenReaderOnlyProps extends React.HTMLAttributes<HTMLSpanElement> {
  children: React.ReactNode;
  as?: "span" | "div" | "p";
}

export function ScreenReaderOnly({
  children,
  className,
  as: Component = "span",
  ...props
}: ScreenReaderOnlyProps) {
  return (
    <Component
      className={cn("sr-only", className)}
      {...props}
    >
      {children}
    </Component>
  );
}
