/**
 * useKeyboardNavigation Hook
 *
 * WCAG 2.1 Level A - 2.1.1 Keyboard Accessible
 * Provides keyboard navigation utilities for complex UI patterns.
 *
 * Supports:
 * - Arrow key navigation for lists/grids
 * - Home/End navigation
 * - Enter/Space activation
 * - Escape to close/cancel
 *
 * Usage:
 * ```tsx
 * const { currentIndex, handlers } = useKeyboardNavigation({
 *   itemCount: items.length,
 *   onSelect: (index) => handleSelect(items[index]),
 *   orientation: 'vertical'
 * });
 * ```
 */

import { useState, useCallback, useEffect } from 'react';

export type Orientation = 'horizontal' | 'vertical' | 'both';

interface UseKeyboardNavigationOptions {
  itemCount: number;
  initialIndex?: number;
  onSelect?: (index: number) => void;
  onEscape?: () => void;
  orientation?: Orientation;
  loop?: boolean;
  enabled?: boolean;
}

interface KeyboardNavigationHandlers {
  onKeyDown: (event: React.KeyboardEvent) => void;
  currentIndex: number;
  setCurrentIndex: (index: number) => void;
}

export function useKeyboardNavigation({
  itemCount,
  initialIndex = 0,
  onSelect,
  onEscape,
  orientation = 'vertical',
  loop = true,
  enabled = true,
}: UseKeyboardNavigationOptions): KeyboardNavigationHandlers {
  const [currentIndex, setCurrentIndex] = useState(initialIndex);

  // Reset index if itemCount changes
  useEffect(() => {
    if (currentIndex >= itemCount) {
      setCurrentIndex(Math.max(0, itemCount - 1));
    }
  }, [itemCount, currentIndex]);

  const handleNext = useCallback(() => {
    setCurrentIndex((prev) => {
      if (prev >= itemCount - 1) {
        return loop ? 0 : prev;
      }
      return prev + 1;
    });
  }, [itemCount, loop]);

  const handlePrevious = useCallback(() => {
    setCurrentIndex((prev) => {
      if (prev <= 0) {
        return loop ? itemCount - 1 : prev;
      }
      return prev - 1;
    });
  }, [itemCount, loop]);

  const handleFirst = useCallback(() => {
    setCurrentIndex(0);
  }, []);

  const handleLast = useCallback(() => {
    setCurrentIndex(itemCount - 1);
  }, [itemCount]);

  const handleSelect = useCallback(() => {
    if (onSelect) {
      onSelect(currentIndex);
    }
  }, [currentIndex, onSelect]);

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (!enabled) return;

      const { key } = event;

      // Determine navigation keys based on orientation
      const nextKeys = orientation === 'horizontal' ? ['ArrowRight'] : ['ArrowDown'];
      const prevKeys = orientation === 'horizontal' ? ['ArrowLeft'] : ['ArrowUp'];

      if (orientation === 'both') {
        nextKeys.push('ArrowRight', 'ArrowDown');
        prevKeys.push('ArrowLeft', 'ArrowUp');
      }

      // Handle navigation
      if (nextKeys.includes(key)) {
        event.preventDefault();
        handleNext();
      } else if (prevKeys.includes(key)) {
        event.preventDefault();
        handlePrevious();
      } else if (key === 'Home') {
        event.preventDefault();
        handleFirst();
      } else if (key === 'End') {
        event.preventDefault();
        handleLast();
      } else if (key === 'Enter' || key === ' ') {
        event.preventDefault();
        handleSelect();
      } else if (key === 'Escape' && onEscape) {
        event.preventDefault();
        onEscape();
      }
    },
    [
      enabled,
      orientation,
      handleNext,
      handlePrevious,
      handleFirst,
      handleLast,
      handleSelect,
      onEscape,
    ]
  );

  return {
    onKeyDown,
    currentIndex,
    setCurrentIndex,
  };
}

/**
 * useRovingTabIndex Hook
 *
 * Implements roving tabindex pattern for composite widgets
 * ARIA Authoring Practices - Keyboard Navigation
 *
 * Usage:
 * ```tsx
 * const tabIndex = useRovingTabIndex(index, currentIndex);
 * <button tabIndex={tabIndex}>Item {index}</button>
 * ```
 */
export function useRovingTabIndex(
  itemIndex: number,
  currentIndex: number
): number {
  return itemIndex === currentIndex ? 0 : -1;
}
