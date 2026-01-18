// features/achievements/epic-celebration.tsx - Tier 5 epic celebration modal
//
// Full-screen celebration modal with fireworks, confetti, and fanfare.
// Used for rare/legendary achievements for new users.

import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Trophy, Sparkles, Star, Crown } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { AchievementUnlockPayload } from "@/hooks/use-achievement-celebration";
import { playCelebrationSound } from "@/hooks/use-achievement-celebration";

const rarityGradients = {
  common: "from-slate-400 to-slate-600",
  rare: "from-blue-400 to-blue-600",
  epic: "from-purple-400 to-purple-600",
  legendary: "from-amber-400 via-yellow-500 to-orange-500",
} as const;

const rarityGlow = {
  common: "shadow-slate-500/50",
  rare: "shadow-blue-500/50",
  epic: "shadow-purple-500/50",
  legendary: "shadow-amber-500/50",
} as const;

interface EpicCelebrationProps {
  celebration: AchievementUnlockPayload;
  onDismiss: () => void;
}

const CONFETTI_COLORS = ["#ffd700", "#ff6b6b", "#4ecdc4", "#45b7d1", "#a855f7", "#ec4899"];

// Generate confetti pieces outside of React render
function generateConfettiPieces() {
  return Array.from({ length: 50 }).map((_, i) => ({
    color: CONFETTI_COLORS[i % CONFETTI_COLORS.length],
    delay: Math.random() * 2,
    duration: 3 + Math.random() * 2,
    left: Math.random() * 100,
    rotation: Math.random() * 360,
    isCircle: Math.random() > 0.5,
    rotateDirection: Math.random() > 0.5 ? 1 : -1,
  }));
}

function Confetti() {
  // Pre-compute random values on mount (lazy initializer runs once)
  const [confettiPieces] = useState(generateConfettiPieces);

  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden z-40">
      {confettiPieces.map((piece, i) => (
        <motion.div
          key={i}
          className="absolute w-3 h-3"
          style={{
            left: `${piece.left}%`,
            top: -20,
            backgroundColor: piece.color,
            borderRadius: piece.isCircle ? "50%" : "2px",
            transform: `rotate(${piece.rotation}deg)`,
          }}
          initial={{ y: -20, opacity: 1 }}
          animate={{
            y: "100vh",
            opacity: [1, 1, 0.8, 0],
            rotate: piece.rotation + 360 * piece.rotateDirection,
            x: Math.sin(i) * 100,
          }}
          transition={{
            duration: piece.duration,
            delay: piece.delay,
            ease: "easeOut",
          }}
        />
      ))}
    </div>
  );
}

function Fireworks() {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden z-40">
      {Array.from({ length: 6 }).map((_, i) => {
        const positions = [
          { x: "20%", y: "30%" },
          { x: "80%", y: "25%" },
          { x: "50%", y: "20%" },
          { x: "30%", y: "40%" },
          { x: "70%", y: "35%" },
          { x: "40%", y: "28%" },
        ];
        const pos = positions[i];
        const delay = 0.5 + i * 0.3;

        return (
          <motion.div
            key={i}
            className="absolute"
            style={{ left: pos.x, top: pos.y }}
            initial={{ scale: 0, opacity: 0 }}
            animate={{
              scale: [0, 1.5, 0],
              opacity: [0, 1, 0],
            }}
            transition={{
              duration: 1.2,
              delay,
              ease: "easeOut",
            }}
          >
            <Sparkles className="w-12 h-12 text-amber-400" />
          </motion.div>
        );
      })}
    </div>
  );
}

// Generate star positions outside of React render
function generateStarPositions() {
  return Array.from({ length: 20 }).map(() => ({
    left: Math.random() * 100,
    top: Math.random() * 100,
    delay: Math.random() * 1.5,
    size: 12 + Math.random() * 16,
  }));
}

function AnimatedStars() {
  // Pre-compute random values on mount (lazy initializer runs once)
  const [starPositions] = useState(generateStarPositions);

  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden z-40">
      {starPositions.map((star, i) => (
        <motion.div
          key={i}
          className="absolute text-amber-300"
          style={{ left: `${star.left}%`, top: `${star.top}%` }}
          initial={{ scale: 0, opacity: 0, rotate: 0 }}
          animate={{
            scale: [0, 1, 0.8, 1, 0],
            opacity: [0, 1, 0.8, 1, 0],
            rotate: 180,
          }}
          transition={{
            duration: 3,
            delay: star.delay,
            ease: "easeInOut",
          }}
        >
          <Star className="fill-current" style={{ width: star.size, height: star.size }} />
        </motion.div>
      ))}
    </div>
  );
}

export function EpicCelebration({ celebration, onDismiss }: EpicCelebrationProps) {
  const { achievement, isFirstInCategory, totalUnlocked } = celebration;
  const rarity = achievement.rarity as keyof typeof rarityGradients;
  const gradient = rarityGradients[rarity];
  const glow = rarityGlow[rarity];
  const hasPlayedSound = useRef(false);
  const [pointsAnimated, setPointsAnimated] = useState(0);

  // Play sound on mount
  useEffect(() => {
    if (!hasPlayedSound.current) {
      hasPlayedSound.current = true;
      playCelebrationSound(5);
    }
  }, []);

  // Animate points counter
  useEffect(() => {
    const target = achievement.points;
    const duration = 1000;
    const start = Date.now();

    const animate = () => {
      const elapsed = Date.now() - start;
      const progress = Math.min(elapsed / duration, 1);
      // Ease out quad
      const eased = 1 - (1 - progress) * (1 - progress);
      setPointsAnimated(Math.floor(target * eased));

      if (progress < 1) {
        requestAnimationFrame(animate);
      }
    };

    requestAnimationFrame(animate);
  }, [achievement.points]);

  return (
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-50 flex items-center justify-center"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
      >
        {/* Background overlay */}
        <motion.div
          className="absolute inset-0 bg-black/80 backdrop-blur-sm"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          onClick={onDismiss}
        />

        {/* Effects layers */}
        <Confetti />
        <Fireworks />
        <AnimatedStars />

        {/* Main content */}
        <motion.div
          className="relative z-50 max-w-lg w-full mx-4"
          initial={{ scale: 0.5, opacity: 0, y: 50 }}
          animate={{ scale: 1, opacity: 1, y: 0 }}
          exit={{ scale: 0.5, opacity: 0, y: 50 }}
          transition={{ type: "spring", damping: 15, stiffness: 100 }}
        >
          <div
            className={`
              relative rounded-2xl border-2 overflow-hidden
              bg-gradient-to-br ${gradient}
              shadow-2xl ${glow}
            `}
          >
            {/* Animated border glow */}
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />

            {/* Content container */}
            <div className="relative p-8 text-center text-white">
              {/* Crown for legendary */}
              {rarity === "legendary" && (
                <motion.div
                  className="absolute -top-4 left-1/2 -translate-x-1/2"
                  initial={{ scale: 0, rotate: -20 }}
                  animate={{ scale: 1, rotate: 0 }}
                  transition={{ delay: 0.3, type: "spring" }}
                >
                  <Crown className="w-16 h-16 text-yellow-300 filter drop-shadow-lg" />
                </motion.div>
              )}

              {/* Badge "EPIC ACHIEVEMENT" */}
              <motion.div
                className="mb-4"
                initial={{ y: -20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.2 }}
              >
                <span className="inline-flex items-center gap-2 px-4 py-1 rounded-full bg-white/20 backdrop-blur text-sm font-bold uppercase tracking-wider">
                  <Sparkles className="w-4 h-4" />
                  {rarity === "legendary" ? "LEGENDARY" : "EPIC"} ACHIEVEMENT
                  <Sparkles className="w-4 h-4" />
                </span>
              </motion.div>

              {/* Achievement Icon */}
              <motion.div
                className="mb-6"
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ delay: 0.3, type: "spring", damping: 10 }}
              >
                <div
                  className={`
                    inline-flex items-center justify-center w-28 h-28 rounded-full
                    bg-white/20 backdrop-blur text-6xl
                    ring-4 ring-white/50 shadow-lg
                  `}
                >
                  {achievement.icon}
                </div>
              </motion.div>

              {/* Achievement Name */}
              <motion.h2
                className="text-3xl font-bold mb-2"
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.4 }}
              >
                {achievement.name}
              </motion.h2>

              {/* Description */}
              <motion.p
                className="text-lg opacity-90 mb-6"
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.5 }}
              >
                {achievement.description}
              </motion.p>

              {/* Points and stats */}
              <motion.div
                className="flex items-center justify-center gap-6 mb-6"
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.6 }}
              >
                <div className="flex items-center gap-2 px-4 py-2 rounded-full bg-white/20 backdrop-blur">
                  <Trophy className="w-6 h-6 text-yellow-300" />
                  <span className="text-2xl font-bold">+{pointsAnimated}</span>
                  <span className="text-sm opacity-75">points</span>
                </div>
              </motion.div>

              {/* Special badges */}
              <motion.div
                className="flex flex-wrap items-center justify-center gap-2 mb-6"
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.7 }}
              >
                {isFirstInCategory && (
                  <span className="inline-flex items-center gap-1 px-3 py-1 rounded-full bg-white/20 text-sm">
                    <Star className="w-4 h-4 fill-current" />
                    First in {achievement.category}!
                  </span>
                )}
                <span className="inline-flex items-center gap-1 px-3 py-1 rounded-full bg-white/20 text-sm">
                  <Trophy className="w-4 h-4" />
                  Achievement #{totalUnlocked}
                </span>
              </motion.div>

              {/* Dismiss button */}
              <motion.div
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.8 }}
              >
                <Button
                  onClick={onDismiss}
                  size="lg"
                  className="bg-white/20 hover:bg-white/30 text-white border-2 border-white/50"
                >
                  Continue Your Journey
                </Button>
              </motion.div>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

// Add shimmer animation to tailwind or use inline styles
const styles = `
@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}
.animate-shimmer {
  animation: shimmer 2s infinite;
}
`;

// Inject styles
if (typeof document !== "undefined") {
  const styleSheet = document.createElement("style");
  styleSheet.textContent = styles;
  document.head.appendChild(styleSheet);
}
