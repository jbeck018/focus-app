// features/achievements/achievement-toast.tsx - Custom achievement unlock toast

import { toast } from "sonner";
import { Trophy, Sparkles } from "lucide-react";
import type { Achievement } from "@focusflow/types";

const rarityColors = {
  common: "text-slate-500",
  rare: "text-blue-500",
  epic: "text-purple-500",
  legendary: "text-amber-500",
} as const;

const rarityBackgrounds = {
  common: "bg-slate-50 dark:bg-slate-950 border-slate-200 dark:border-slate-800",
  rare: "bg-blue-50 dark:bg-blue-950 border-blue-200 dark:border-blue-800",
  epic: "bg-purple-50 dark:bg-purple-950 border-purple-200 dark:border-purple-800",
  legendary: "bg-amber-50 dark:bg-amber-950 border-amber-200 dark:border-amber-800",
} as const;

/**
 * Show a custom toast notification for unlocked achievement
 */
export function showAchievementToast(achievement: Achievement) {
  const rarityColor = rarityColors[achievement.rarity as keyof typeof rarityColors];
  const rarityBg = rarityBackgrounds[achievement.rarity as keyof typeof rarityBackgrounds];

  toast.custom(
    (t) => (
      <div
        className={`
          relative overflow-hidden rounded-lg border-2 p-4 shadow-lg
          animate-in slide-in-from-top-5 duration-300
          ${rarityBg}
        `}
      >
        {/* Sparkle animation for legendary/epic */}
        {(achievement.rarity === "legendary" || achievement.rarity === "epic") && (
          <div className="absolute inset-0 pointer-events-none">
            <Sparkles className={`absolute top-2 right-2 w-4 h-4 animate-pulse ${rarityColor}`} />
            <Sparkles
              className={`absolute bottom-3 left-3 w-3 h-3 animate-pulse delay-150 ${rarityColor}`}
            />
          </div>
        )}

        <div className="flex items-start gap-3 relative z-10">
          {/* Achievement Icon */}
          <div
            className={`
              flex items-center justify-center w-12 h-12 rounded-full
              bg-white dark:bg-slate-900 shadow-md text-2xl
              ring-2 ${rarityColor.replace("text-", "ring-")}
            `}
          >
            {achievement.icon}
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <Trophy className={`w-4 h-4 ${rarityColor}`} />
              <h3 className="font-bold text-sm text-foreground">Achievement Unlocked!</h3>
            </div>

            <h4 className="font-semibold text-base mb-1 text-foreground">{achievement.name}</h4>

            <p className="text-sm text-muted-foreground mb-2">{achievement.description}</p>

            <div className="flex items-center gap-3">
              <span
                className={`
                  text-xs font-semibold px-2 py-0.5 rounded-full
                  bg-white dark:bg-slate-900 ${rarityColor}
                  uppercase tracking-wide
                `}
              >
                {achievement.rarity}
              </span>
              <span className="text-xs font-bold text-amber-500 flex items-center gap-1">
                <Trophy className="w-3 h-3" />+{achievement.points} points
              </span>
            </div>
          </div>

          {/* Close button */}
          <button
            onClick={() => toast.dismiss(t)}
            className="text-muted-foreground hover:text-foreground transition-colors"
            aria-label="Dismiss"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
    ),
    {
      duration: 5000,
      position: "top-center",
    }
  );
}

/**
 * Show multiple achievement toasts with a delay between them
 */
export function showAchievementToasts(achievements: Achievement[]) {
  achievements.forEach((achievement, index) => {
    setTimeout(() => {
      showAchievementToast(achievement);
    }, index * 1000); // 1 second delay between each toast
  });
}

/**
 * Show a simple achievement notification (fallback)
 */
export function showSimpleAchievementToast(achievement: Achievement) {
  toast.success(`Achievement Unlocked: ${achievement.name}`, {
    description: `${achievement.icon} ${achievement.description} - ${achievement.points} points!`,
    duration: 5000,
  });
}
