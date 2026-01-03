/**
 * SkipLink Component
 *
 * WCAG 2.1 Level A - 2.4.1 Bypass Blocks
 * Provides a keyboard-accessible link to skip repetitive navigation
 * and jump directly to main content.
 *
 * ARIA Pattern: Skip Link
 * - Visually hidden until focused
 * - First focusable element in tab order
 * - Smooth scroll to target when activated
 */

import * as React from "react";
import { cn } from "@/lib/utils";

interface SkipLinkProps {
  href: string;
  children: React.ReactNode;
  className?: string;
}

export function SkipLink({ href, children, className }: SkipLinkProps) {
  const handleClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault();

    const target = document.querySelector(href);
    if (target) {
      // Set focus to target element
      (target as HTMLElement).focus();

      // Smooth scroll to target
      target.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }
  };

  return (
    <a
      href={href}
      onClick={handleClick}
      className={cn(
        // Visually hidden by default
        "sr-only",
        // Visible when focused
        "focus:not-sr-only",
        "focus:fixed focus:top-4 focus:left-4 focus:z-[9999]",
        "focus:inline-block focus:px-4 focus:py-2",
        "focus:bg-primary focus:text-primary-foreground",
        "focus:rounded-md focus:shadow-lg",
        "focus:ring-2 focus:ring-ring focus:ring-offset-2",
        "focus:outline-none",
        "font-medium text-sm",
        "transition-all",
        className
      )}
    >
      {children}
    </a>
  );
}
