/**
 * Streak Milestone Celebration Component
 *
 * Displays animated celebrations when milestones are achieved
 * - Confetti animation for major milestones
 * - Fireworks for legendary achievements
 * - Sparkle effects for smaller wins
 * - Dismissable with animation
 */

import React, { useEffect, useState } from "react";
import { StreakMilestone } from "@focusflow/types";
import { cn } from "../../lib/utils";

interface MilestoneCelebrationProps {
  milestone: StreakMilestone;
  onDismiss: () => void;
  className?: string;
}

export function MilestoneCelebration({
  milestone,
  onDismiss,
  className,
}: MilestoneCelebrationProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isExiting, setIsExiting] = useState(false);

  useEffect(() => {
    // Trigger entrance animation
    setTimeout(() => setIsVisible(true), 100);

    // Auto-dismiss after 5 seconds
    const timer = setTimeout(() => {
      handleDismiss();
    }, 5000);

    return () => clearTimeout(timer);
  }, []);

  const handleDismiss = () => {
    setIsExiting(true);
    setTimeout(() => {
      onDismiss();
    }, 500);
  };

  const animationType = getAnimationType(milestone.tier);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className={cn(
          "absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity duration-500",
          isVisible && !isExiting ? "opacity-100" : "opacity-0"
        )}
        onClick={handleDismiss}
      />

      {/* Animation Layer */}
      {animationType === "confetti" && <ConfettiAnimation />}
      {animationType === "fireworks" && <FireworksAnimation />}
      {animationType === "sparkle" && <SparkleAnimation />}

      {/* Celebration Card */}
      <div
        className={cn(
          "celebration-card relative z-10 w-full max-w-md transform rounded-lg border bg-card p-8 shadow-2xl transition-all duration-500",
          isVisible && !isExiting
            ? "scale-100 opacity-100"
            : "scale-75 opacity-0",
          className
        )}
      >
        {/* Milestone Badge */}
        <div className="mb-6 flex justify-center">
          <div
            className={cn(
              "milestone-badge flex h-24 w-24 items-center justify-center rounded-full text-5xl",
              getTierGradient(milestone.tier)
            )}
          >
            {getTierIcon(milestone.tier)}
          </div>
        </div>

        {/* Content */}
        <div className="text-center">
          <h2 className="text-2xl font-bold">Milestone Achieved!</h2>
          <p className="mt-2 text-lg font-semibold capitalize text-primary">
            {milestone.tier} Tier
          </p>
          <p className="mt-4 text-muted-foreground">
            You've maintained a {milestone.daysRequired}-day focus streak
          </p>

          {milestone.reward && (
            <div className="mt-6 rounded-lg border bg-muted/50 p-4">
              <div className="text-sm font-medium">Reward Unlocked</div>
              <div className="mt-1 flex items-center justify-center gap-2 text-lg">
                <span>❄️</span>
                <span className="capitalize">{milestone.reward.replace("_", " ")}</span>
              </div>
            </div>
          )}
        </div>

        {/* Dismiss Button */}
        <button
          onClick={handleDismiss}
          className="mt-8 w-full rounded-md bg-primary px-4 py-3 font-medium text-primary-foreground transition-colors hover:bg-primary/90"
        >
          Awesome!
        </button>
      </div>
    </div>
  );
}

function ConfettiAnimation() {
  const confettiPieces = Array.from({ length: 50 }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    delay: Math.random() * 2,
    duration: 3 + Math.random() * 2,
    color: ["#ef4444", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6"][
      Math.floor(Math.random() * 5)
    ],
  }));

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      {confettiPieces.map((piece) => (
        <div
          key={piece.id}
          className="confetti-piece absolute -top-10 h-3 w-3"
          style={
            {
              left: `${piece.x}%`,
              backgroundColor: piece.color,
              animationDelay: `${piece.delay}s`,
              animationDuration: `${piece.duration}s`,
              "--rotation": `${Math.random() * 360}deg`,
            } as React.CSSProperties
          }
        />
      ))}
    </div>
  );
}

function FireworksAnimation() {
  const fireworks = Array.from({ length: 8 }, (_, i) => ({
    id: i,
    x: 20 + Math.random() * 60,
    y: 20 + Math.random() * 40,
    delay: Math.random() * 1.5,
  }));

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      {fireworks.map((firework) => (
        <div
          key={firework.id}
          className="firework absolute"
          style={{
            left: `${firework.x}%`,
            top: `${firework.y}%`,
            animationDelay: `${firework.delay}s`,
          }}
        >
          {Array.from({ length: 12 }, (_, i) => (
            <div
              key={i}
              className="firework-particle"
              style={{
                transform: `rotate(${i * 30}deg)`,
              }}
            />
          ))}
        </div>
      ))}
    </div>
  );
}

function SparkleAnimation() {
  const sparkles = Array.from({ length: 30 }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    y: Math.random() * 100,
    delay: Math.random() * 2,
    size: 2 + Math.random() * 4,
  }));

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      {sparkles.map((sparkle) => (
        <div
          key={sparkle.id}
          className="sparkle absolute"
          style={{
            left: `${sparkle.x}%`,
            top: `${sparkle.y}%`,
            width: `${sparkle.size}px`,
            height: `${sparkle.size}px`,
            animationDelay: `${sparkle.delay}s`,
          }}
        />
      ))}
    </div>
  );
}

function getTierIcon(tier: string): string {
  switch (tier.toLowerCase()) {
    case "bronze":
      return "🥉";
    case "silver":
      return "🥈";
    case "gold":
      return "🥇";
    case "platinum":
      return "💎";
    case "diamond":
      return "👑";
    default:
      return "⭐";
  }
}

function getTierGradient(tier: string): string {
  switch (tier.toLowerCase()) {
    case "bronze":
      return "bg-gradient-to-br from-orange-400 to-orange-600";
    case "silver":
      return "bg-gradient-to-br from-gray-300 to-gray-500";
    case "gold":
      return "bg-gradient-to-br from-yellow-300 to-yellow-500";
    case "platinum":
      return "bg-gradient-to-br from-cyan-300 to-blue-500";
    case "diamond":
      return "bg-gradient-to-br from-purple-400 to-pink-500";
    default:
      return "bg-gradient-to-br from-gray-400 to-gray-600";
  }
}

function getAnimationType(
  tier: string
): "confetti" | "fireworks" | "sparkle" {
  switch (tier.toLowerCase()) {
    case "diamond":
    case "platinum":
      return "fireworks";
    case "gold":
      return "confetti";
    default:
      return "sparkle";
  }
}

// Global CSS for animations (add to your global CSS file or index.css)
export const celebrationStyles = `
/* Celebration Card Pulse */
@keyframes celebration-pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

.celebration-card {
  animation: celebration-pulse 2s ease-in-out infinite;
}

/* Milestone Badge Glow */
@keyframes badge-glow {
  0%, 100% {
    box-shadow: 0 0 20px rgba(255, 215, 0, 0.5);
  }
  50% {
    box-shadow: 0 0 40px rgba(255, 215, 0, 0.8);
  }
}

.milestone-badge {
  animation: badge-glow 2s ease-in-out infinite;
}

/* Confetti Animation */
@keyframes confetti-fall {
  0% {
    transform: translateY(0) rotate(0deg);
    opacity: 1;
  }
  100% {
    transform: translateY(100vh) rotate(var(--rotation));
    opacity: 0;
  }
}

.confetti-piece {
  animation: confetti-fall 3s ease-in infinite;
  border-radius: 2px;
}

/* Fireworks Animation */
@keyframes firework-explode {
  0% {
    opacity: 1;
    transform: scale(0);
  }
  50% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: scale(1);
  }
}

.firework {
  animation: firework-explode 1.5s ease-out infinite;
}

.firework-particle {
  position: absolute;
  width: 4px;
  height: 4px;
  background: radial-gradient(circle, #fff 0%, #f59e0b 100%);
  border-radius: 50%;
  transform-origin: center;
}

.firework-particle::before {
  content: '';
  position: absolute;
  width: 100%;
  height: 30px;
  background: linear-gradient(to bottom, #f59e0b, transparent);
  border-radius: 50%;
}

/* Sparkle Animation */
@keyframes sparkle-twinkle {
  0%, 100% {
    opacity: 0;
    transform: scale(0) rotate(0deg);
  }
  50% {
    opacity: 1;
    transform: scale(1) rotate(180deg);
  }
}

.sparkle {
  background: radial-gradient(circle, #fff 0%, #fbbf24 100%);
  border-radius: 50%;
  animation: sparkle-twinkle 2s ease-in-out infinite;
  box-shadow: 0 0 10px #fbbf24;
}

/* Responsive adjustments */
@media (max-width: 640px) {
  .celebration-card {
    margin: 1rem;
  }
}
`;
