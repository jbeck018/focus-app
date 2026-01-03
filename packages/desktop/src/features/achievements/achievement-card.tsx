// features/achievements/achievement-card.tsx - Individual achievement display card

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import type { AchievementWithStatus } from "@focusflow/types";
import { Lock, Trophy } from "lucide-react";

interface AchievementCardProps {
  achievement: AchievementWithStatus;
  compact?: boolean;
  showProgress?: boolean;
  className?: string;
}

const rarityColors = {
  common: "bg-slate-500 text-white",
  rare: "bg-blue-500 text-white",
  epic: "bg-purple-500 text-white",
  legendary: "bg-amber-500 text-white",
} as const;

const categoryLabels = {
  session: "Sessions",
  streak: "Streaks",
  time: "Time",
  blocking: "Blocking",
  special: "Special",
} as const;

export function AchievementCard({
  achievement,
  compact = false,
  showProgress = true,
  className,
}: AchievementCardProps) {
  const rarityColor = rarityColors[achievement.rarity as keyof typeof rarityColors] || rarityColors.common;
  const categoryLabel = categoryLabels[achievement.category as keyof typeof categoryLabels] || achievement.category;

  if (compact) {
    return (
      <div
        className={cn(
          "flex items-center gap-3 p-3 rounded-lg border transition-all",
          achievement.unlocked
            ? "bg-card hover:bg-accent/50"
            : "bg-muted/30 opacity-75",
          className
        )}
      >
        <div
          className={cn(
            "flex items-center justify-center w-12 h-12 rounded-full text-2xl transition-transform",
            achievement.unlocked ? "scale-100" : "scale-90 grayscale"
          )}
        >
          {achievement.unlocked ? (
            achievement.icon
          ) : (
            <Lock className="w-6 h-6 text-muted-foreground" />
          )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h4 className={cn(
              "font-semibold text-sm truncate",
              !achievement.unlocked && "text-muted-foreground"
            )}>
              {achievement.name}
            </h4>
            <Badge variant="outline" className={cn("text-xs px-1.5 py-0", rarityColor)}>
              {achievement.rarity}
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground truncate">
            {achievement.description}
          </p>
          {showProgress && !achievement.unlocked && (
            <div className="mt-1.5 flex items-center gap-2">
              <Progress value={achievement.progressPercentage} className="h-1.5 flex-1" />
              <span className="text-xs text-muted-foreground whitespace-nowrap">
                {achievement.progress}/{achievement.threshold}
              </span>
            </div>
          )}
        </div>

        <div className="flex flex-col items-end gap-1">
          <div className="flex items-center gap-1 text-amber-500">
            <Trophy className="w-3.5 h-3.5" />
            <span className="text-xs font-semibold">{achievement.points}</span>
          </div>
          {achievement.unlocked && achievement.unlockedAt && (
            <span className="text-xs text-muted-foreground">
              {new Date(achievement.unlockedAt).toLocaleDateString()}
            </span>
          )}
        </div>
      </div>
    );
  }

  return (
    <Card
      className={cn(
        "transition-all duration-300 flex flex-col",
        achievement.unlocked
          ? "border-primary/20 hover:shadow-lg hover:scale-[1.02]"
          : "opacity-75 hover:opacity-90",
        className
      )}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start gap-3">
          <div className="flex-shrink-0">
            <div
              className={cn(
                "flex items-center justify-center w-16 h-16 rounded-full bg-accent text-4xl transition-all",
                achievement.unlocked
                  ? "ring-2 ring-primary/50 shadow-md"
                  : "grayscale opacity-60"
              )}
            >
              {achievement.unlocked ? (
                achievement.icon
              ) : (
                <Lock className="w-8 h-8 text-muted-foreground" />
              )}
            </div>
          </div>

          <div className="flex-1 min-w-0">
            <div className="flex items-start justify-between gap-3 mb-1">
              <CardTitle className={cn(
                "text-lg leading-tight",
                !achievement.unlocked && "text-muted-foreground"
              )}>
                {achievement.name}
              </CardTitle>
              <div className="flex items-center gap-1.5 text-amber-500 flex-shrink-0">
                <Trophy className="w-4 h-4" />
                <span className="text-sm font-bold whitespace-nowrap">{achievement.points} pts</span>
              </div>
            </div>
            <div className="flex items-center gap-2 flex-wrap">
              <Badge variant="outline" className={cn("text-xs", rarityColor)}>
                {achievement.rarity}
              </Badge>
              <Badge variant="secondary" className="text-xs">
                {categoryLabel}
              </Badge>
            </div>
          </div>
        </div>

        <CardDescription className="mt-2">
          {achievement.description}
        </CardDescription>
      </CardHeader>

      <CardContent className="pt-0 flex-1 flex flex-col justify-end">
        {showProgress && !achievement.unlocked && (
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Progress</span>
              <span className="font-medium">
                {achievement.progress} / {achievement.threshold}
              </span>
            </div>
            <Progress value={achievement.progressPercentage} className="h-2" />
            <p className="text-xs text-muted-foreground text-right">
              {Math.round(achievement.progressPercentage || 0)}% complete
            </p>
          </div>
        )}

        {achievement.unlocked && achievement.unlockedAt && (
          <div className="flex items-center justify-between text-sm pt-2 border-t">
            <span className="text-muted-foreground">Unlocked</span>
            <span className="font-medium">
              {new Date(achievement.unlockedAt).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
