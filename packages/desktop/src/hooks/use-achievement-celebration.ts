// hooks/use-achievement-celebration.ts - Achievement celebration orchestration
//
// Manages a queue of achievement celebrations with tier-based rendering.
// Listens to 'achievement-unlocked' events from the backend and displays
// appropriate celebrations based on the tier level.

import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import type { Achievement } from "@focusflow/types";

// Celebration tier determines the intensity of the unlock animation
// Tier 5 (Epic): Full-screen modal, fireworks, confetti 4s, fanfare, 6s duration
// Tier 4 (Major): Large animated toast, confetti 2s, chime, sparkles, 5s duration
// Tier 3 (Standard): Enhanced toast, subtle sparkle, soft ding, 4s duration
// Tier 2 (Light): Simple toast, icon highlight, no sound, 3s duration
// Tier 1 (Minimal): Badge indicator update only, no interruption

export interface AchievementUnlockPayload {
  achievement: Achievement;
  celebrationTier: number;
  isFirstInCategory: boolean;
  totalUnlocked: number;
  userLevel: "new" | "beginner" | "intermediate" | "advanced" | "master";
}

interface QueuedCelebration extends AchievementUnlockPayload {
  id: string;
  timestamp: number;
}

const TIER_DURATIONS: Record<number, number> = {
  5: 6000, // Epic - 6s
  4: 5000, // Major - 5s
  3: 4000, // Standard - 4s
  2: 3000, // Light - 3s
  1: 0, // Minimal - no visual celebration
};

export function useAchievementCelebration() {
  const [celebrationQueue, setCelebrationQueue] = useState<QueuedCelebration[]>([]);
  const [currentCelebration, setCurrentCelebration] = useState<QueuedCelebration | null>(null);
  const [isAnimating, setIsAnimating] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Process the next celebration in the queue
  const processNextCelebration = useCallback(() => {
    setCelebrationQueue((queue) => {
      if (queue.length === 0) {
        setCurrentCelebration(null);
        setIsAnimating(false);
        return queue;
      }

      // Get next celebration
      const [next, ...rest] = queue;

      // Skip tier 1 (minimal) - no visual celebration needed
      if (next.celebrationTier === 1) {
        // Still emit for badge updates, but don't show visual celebration
        console.log("[celebration] Tier 1 achievement - badge update only:", next.achievement.name);
        return rest;
      }

      setCurrentCelebration(next);
      setIsAnimating(true);

      // Schedule completion based on tier duration
      const duration = TIER_DURATIONS[next.celebrationTier] || 3000;
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      timeoutRef.current = setTimeout(() => {
        setCurrentCelebration(null);
        setIsAnimating(false);
        // Small delay before next celebration
        setTimeout(() => {
          processNextCelebration();
        }, 300);
      }, duration);

      return rest;
    });
  }, []);

  // Add celebration to queue
  const addCelebration = useCallback((payload: AchievementUnlockPayload) => {
    const queued: QueuedCelebration = {
      ...payload,
      id: `${payload.achievement.id}-${Date.now()}`,
      timestamp: Date.now(),
    };

    setCelebrationQueue((queue) => {
      const newQueue = [...queue, queued];

      // Sort by tier (higher tiers first for better UX)
      newQueue.sort((a, b) => {
        // If one is tier 5, it should go first
        if (a.celebrationTier === 5 && b.celebrationTier !== 5) return -1;
        if (b.celebrationTier === 5 && a.celebrationTier !== 5) return 1;
        // Otherwise preserve order
        return a.timestamp - b.timestamp;
      });

      return newQueue;
    });
  }, []);

  // Dismiss current celebration early
  const dismissCelebration = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setCurrentCelebration(null);
    setIsAnimating(false);
    setTimeout(() => {
      processNextCelebration();
    }, 100);
  }, [processNextCelebration]);

  // Start processing when queue has items and nothing is currently animating
  useEffect(() => {
    if (celebrationQueue.length > 0 && !isAnimating && !currentCelebration) {
      processNextCelebration();
    }
  }, [celebrationQueue.length, isAnimating, currentCelebration, processNextCelebration]);

  // Listen for achievement unlock events
  useEffect(() => {
    const unlisten = listen<AchievementUnlockPayload>("achievement-unlocked", (event) => {
      console.log("[celebration] Achievement unlocked:", event.payload);
      addCelebration(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [addCelebration]);

  return {
    currentCelebration,
    queueLength: celebrationQueue.length,
    isAnimating,
    dismissCelebration,
    // For testing/manual triggering
    addCelebration,
  };
}

// Sound effect helpers
export function playCelebrationSound(tier: number) {
  // Sound files would be in public/sounds/
  const soundMap: Record<number, string> = {
    5: "/sounds/fanfare.mp3",
    4: "/sounds/chime.mp3",
    3: "/sounds/ding.mp3",
  };

  const soundFile = soundMap[tier];
  if (soundFile) {
    try {
      const audio = new Audio(soundFile);
      audio.volume = tier === 5 ? 0.7 : tier === 4 ? 0.5 : 0.3;
      audio.play().catch(() => {
        // Silently fail if audio playback is blocked
      });
    } catch (e) {
      // Audio not available
    }
  }
}
