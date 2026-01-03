// features/achievements/achievement-gallery.tsx - Achievement showcase and gallery

import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { AchievementCard } from "./achievement-card";
import { useAchievements, useAchievementStats } from "@/hooks/use-achievements";
import { Trophy, Lock, Target, TrendingUp, Award, Sparkles } from "lucide-react";

const categories = [
  { value: "all", label: "All", icon: Trophy },
  { value: "session", label: "Sessions", icon: Target },
  { value: "streak", label: "Streaks", icon: TrendingUp },
  { value: "time", label: "Time", icon: Award },
  { value: "blocking", label: "Blocking", icon: Lock },
  { value: "special", label: "Special", icon: Sparkles },
];

export function AchievementGallery() {
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [showOnlyUnlocked, setShowOnlyUnlocked] = useState(false);

  const { data: stats, isLoading: statsLoading } = useAchievementStats();
  const { data: achievements, isLoading } = useAchievements();

  // Filter achievements based on category and unlock status
  const filteredAchievements = achievements?.filter((achievement) => {
    const categoryMatch = selectedCategory === "all" || achievement.category === selectedCategory;
    const unlockedMatch = !showOnlyUnlocked || achievement.unlocked;
    return categoryMatch && unlockedMatch;
  });

  // Group achievements by rarity for stats
  const achievementsByRarity = achievements?.reduce(
    (acc, achievement) => {
      if (achievement.unlocked) {
        acc[achievement.rarity] = (acc[achievement.rarity] || 0) + 1;
      }
      return acc;
    },
    {} as Record<string, number>
  );

  if (isLoading || statsLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <Trophy className="w-16 h-16 text-muted-foreground mx-auto mb-4 animate-pulse" />
          <p className="text-muted-foreground">Loading achievements...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Total Points</CardDescription>
            <CardTitle className="text-3xl text-amber-500">{stats?.totalPoints || 0}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Trophy className="w-4 h-4" />
              <span>Achievement Points</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Unlocked</CardDescription>
            <CardTitle className="text-3xl">
              {stats?.unlockedCount || 0}
              <span className="text-lg text-muted-foreground">
                /{stats?.totalAchievements || 0}
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Progress value={stats?.completionPercentage || 0} className="h-2" />
            <p className="text-xs text-muted-foreground mt-1">
              {Math.round(stats?.completionPercentage || 0)}% complete
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Legendary</CardDescription>
            <CardTitle className="text-3xl text-amber-500">
              {achievementsByRarity?.legendary || 0}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Sparkles className="w-4 h-4" />
              <span>Rare achievements</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Epic</CardDescription>
            <CardTitle className="text-3xl text-purple-500">
              {achievementsByRarity?.epic || 0}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Award className="w-4 h-4" />
              <span>Epic achievements</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Category Tabs */}
      <Tabs value={selectedCategory} onValueChange={setSelectedCategory}>
        <div className="flex items-center justify-between mb-4">
          <TabsList className="grid w-full max-w-2xl grid-cols-6">
            {categories.map((category) => {
              const Icon = category.icon;
              const count =
                achievements?.filter(
                  (a) => (category.value === "all" || a.category === category.value) && a.unlocked
                ).length || 0;

              return (
                <TabsTrigger
                  key={category.value}
                  value={category.value}
                  className="flex items-center gap-2"
                >
                  <Icon className="w-4 h-4" />
                  <span className="hidden sm:inline">{category.label}</span>
                  <Badge variant="secondary" className="text-xs ml-1">
                    {count}
                  </Badge>
                </TabsTrigger>
              );
            })}
          </TabsList>

          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowOnlyUnlocked(!showOnlyUnlocked)}
              className={`
                px-3 py-1.5 rounded-md text-sm font-medium transition-colors
                ${
                  showOnlyUnlocked
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted text-muted-foreground hover:bg-muted/80"
                }
              `}
            >
              {showOnlyUnlocked ? "Unlocked Only" : "Show All"}
            </button>
          </div>
        </div>

        {categories.map((category) => (
          <TabsContent key={category.value} value={category.value} className="space-y-4">
            {filteredAchievements && filteredAchievements.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 auto-rows-fr">
                {filteredAchievements.map((achievement) => (
                  <AchievementCard
                    key={achievement.id}
                    achievement={achievement}
                    showProgress={!achievement.unlocked}
                    className="h-full"
                  />
                ))}
              </div>
            ) : (
              <Card>
                <CardContent className="flex flex-col items-center justify-center py-12">
                  <Lock className="w-16 h-16 text-muted-foreground mb-4" />
                  <h3 className="text-lg font-semibold mb-2">
                    {showOnlyUnlocked ? "No unlocked achievements yet" : "No achievements found"}
                  </h3>
                  <p className="text-sm text-muted-foreground text-center max-w-md">
                    {showOnlyUnlocked
                      ? "Complete focus sessions to unlock achievements in this category!"
                      : "This category is empty."}
                  </p>
                </CardContent>
              </Card>
            )}
          </TabsContent>
        ))}
      </Tabs>

      {/* Next Achievements to Unlock */}
      {filteredAchievements && filteredAchievements.some((a) => !a.unlocked) && (
        <div>
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Target className="w-5 h-5" />
            Next to Unlock
          </h3>
          <div className="grid grid-cols-1 gap-3">
            {filteredAchievements
              .filter((a) => !a.unlocked && !a.hidden)
              .sort((a, b) => b.progressPercentage - a.progressPercentage)
              .slice(0, 3)
              .map((achievement) => (
                <AchievementCard
                  key={achievement.id}
                  achievement={achievement}
                  compact
                  showProgress
                />
              ))}
          </div>
        </div>
      )}
    </div>
  );
}
