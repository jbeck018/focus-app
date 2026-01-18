// features/achievements/achievement-celebration.tsx - Achievement celebration orchestrator
//
// Renders the appropriate celebration component based on tier level.
// Should be mounted at the app root level to display celebrations globally.

import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { toast } from "sonner";
import { Trophy, Sparkles, Star } from "lucide-react";
import {
  useAchievementCelebration,
  playCelebrationSound,
  type AchievementUnlockPayload,
} from "@/hooks/use-achievement-celebration";
import { EpicCelebration } from "./epic-celebration";

// Generate sparkle positions outside of React render
function generateSparklePositions() {
  return Array.from({ length: 8 }).map(() => ({
    left: 10 + Math.random() * 80,
    top: 10 + Math.random() * 80,
  }));
}

// Major celebration (Tier 4) - Large animated toast with confetti
function MajorCelebration({ celebration }: { celebration: AchievementUnlockPayload }) {
  const { achievement, isFirstInCategory } = celebration;

  // Pre-compute random positions on mount (lazy initializer runs once)
  const [sparklePositions] = useState(generateSparklePositions);

  return (
    <motion.div
      className="relative overflow-hidden rounded-xl border-2 p-5 shadow-xl bg-gradient-to-br from-blue-50 to-purple-50 dark:from-blue-950 dark:to-purple-950 border-purple-300 dark:border-purple-700"
      initial={{ scale: 0.8, opacity: 0, y: -20 }}
      animate={{ scale: 1, opacity: 1, y: 0 }}
      exit={{ scale: 0.8, opacity: 0, y: -20 }}
    >
      {/* Sparkles animation */}
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        {sparklePositions.map((pos, i) => (
          <motion.div
            key={i}
            className="absolute text-purple-400"
            style={{
              left: `${pos.left}%`,
              top: `${pos.top}%`,
            }}
            initial={{ scale: 0, opacity: 0 }}
            animate={{
              scale: [0, 1, 0],
              opacity: [0, 1, 0],
            }}
            transition={{
              duration: 1.5,
              delay: i * 0.2,
              repeat: Infinity,
              repeatDelay: 1,
            }}
          >
            <Sparkles className="w-4 h-4" />
          </motion.div>
        ))}
      </div>

      <div className="flex items-start gap-4 relative z-10">
        {/* Animated icon */}
        <motion.div
          className="flex items-center justify-center w-16 h-16 rounded-full bg-white dark:bg-slate-900 shadow-lg text-3xl ring-2 ring-purple-400"
          animate={{
            scale: [1, 1.1, 1],
          }}
          transition={{
            duration: 0.5,
            repeat: 2,
          }}
        >
          {achievement.icon}
        </motion.div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Trophy className="w-5 h-5 text-purple-500" />
            <h3 className="font-bold text-purple-700 dark:text-purple-300">
              Achievement Unlocked!
            </h3>
          </div>

          <h4 className="font-bold text-lg mb-1 text-foreground">{achievement.name}</h4>
          <p className="text-sm text-muted-foreground mb-3">{achievement.description}</p>

          <div className="flex flex-wrap items-center gap-2">
            <span className="text-xs font-bold px-2 py-1 rounded-full bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300 uppercase">
              {achievement.rarity}
            </span>
            <span className="text-xs font-bold text-amber-500 flex items-center gap-1">
              <Trophy className="w-3 h-3" />+{achievement.points} points
            </span>
            {isFirstInCategory && (
              <span className="text-xs font-medium text-blue-500 flex items-center gap-1">
                <Star className="w-3 h-3 fill-current" />
                First in category!
              </span>
            )}
          </div>
        </div>
      </div>
    </motion.div>
  );
}

// Standard celebration (Tier 3) - Enhanced toast with subtle sparkle
function StandardCelebration({ celebration }: { celebration: AchievementUnlockPayload }) {
  const { achievement } = celebration;

  return (
    <motion.div
      className="relative overflow-hidden rounded-lg border p-4 shadow-lg bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700"
      initial={{ x: 50, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: 50, opacity: 0 }}
    >
      {/* Subtle glow */}
      <motion.div
        className="absolute inset-0 bg-gradient-to-r from-amber-400/10 to-purple-400/10 pointer-events-none"
        animate={{ opacity: [0.3, 0.5, 0.3] }}
        transition={{ duration: 2, repeat: Infinity }}
      />

      <div className="flex items-center gap-3 relative z-10">
        <motion.div
          className="flex items-center justify-center w-12 h-12 rounded-full bg-slate-100 dark:bg-slate-800 text-2xl ring-1 ring-amber-400"
          animate={{ rotate: [0, 5, -5, 0] }}
          transition={{ duration: 0.5 }}
        >
          {achievement.icon}
        </motion.div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 mb-0.5">
            <Trophy className="w-4 h-4 text-amber-500" />
            <span className="font-semibold text-sm text-foreground">Achievement Unlocked</span>
          </div>
          <h4 className="font-bold text-foreground">{achievement.name}</h4>
          <p className="text-xs text-muted-foreground">
            +{achievement.points} points • {achievement.rarity}
          </p>
        </div>
      </div>
    </motion.div>
  );
}

// Light celebration (Tier 2) - Simple toast with icon highlight
function LightCelebration({ celebration }: { celebration: AchievementUnlockPayload }) {
  const { achievement } = celebration;

  return (
    <motion.div
      className="flex items-center gap-3 p-3 rounded-lg border bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 shadow"
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
    >
      <div className="flex items-center justify-center w-10 h-10 rounded-full bg-slate-100 dark:bg-slate-800 text-xl">
        {achievement.icon}
      </div>
      <div>
        <span className="font-medium text-sm text-foreground">{achievement.name}</span>
        <p className="text-xs text-muted-foreground">+{achievement.points} points</p>
      </div>
    </motion.div>
  );
}

// Toast-based celebrations for tiers 2-4 (shown in sonner toast)
function showTieredToast(celebration: AchievementUnlockPayload) {
  const { celebrationTier } = celebration;

  const durations: Record<number, number> = {
    4: 5000,
    3: 4000,
    2: 3000,
  };

  // Play sound for tier 3-4
  if (celebrationTier >= 3) {
    playCelebrationSound(celebrationTier);
  }

  // Only show toast for valid celebration tiers
  if (celebrationTier < 2 || celebrationTier > 4) {
    return;
  }

  toast.custom(
    () => {
      switch (celebrationTier) {
        case 4:
          return <MajorCelebration celebration={celebration} />;
        case 3:
          return <StandardCelebration celebration={celebration} />;
        case 2:
        default:
          return <LightCelebration celebration={celebration} />;
      }
    },
    {
      duration: durations[celebrationTier] || 3000,
      position: "top-center",
    }
  );
}

// Main orchestrator component - mount at app root
export function AchievementCelebrationProvider({ children }: { children: React.ReactNode }) {
  const { currentCelebration, dismissCelebration } = useAchievementCelebration();

  // Handle non-tier-5 celebrations via toast
  useEffect(() => {
    if (currentCelebration && currentCelebration.celebrationTier < 5) {
      showTieredToast(currentCelebration);
    }
  }, [currentCelebration]);

  return (
    <>
      {children}

      {/* Tier 5 (Epic) celebrations render as full-screen modal */}
      <AnimatePresence>
        {currentCelebration?.celebrationTier === 5 && (
          <EpicCelebration celebration={currentCelebration} onDismiss={dismissCelebration} />
        )}
      </AnimatePresence>
    </>
  );
}

// Re-export for convenience
export type { AchievementUnlockPayload };
