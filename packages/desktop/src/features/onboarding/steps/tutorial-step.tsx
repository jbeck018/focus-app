// features/onboarding/steps/tutorial-step.tsx - Quick tutorial of core features

import { StepWrapper } from "../step-wrapper";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Timer,
  Shield,
  BookOpen,
  BarChart,
  Calendar,
  Bot,
  CheckCircle2,
  Circle,
  Sparkles,
  Lock,
} from "lucide-react";

// Tutorial topics
interface TutorialTopic {
  id: string;
  title: string;
  description: string;
  icon: string;
  duration: string;
}

const TUTORIAL_TOPICS: TutorialTopic[] = [
  {
    id: "focus-timer",
    title: "Focus Timer",
    description: "Start focused work sessions with customizable durations and automatic blocking",
    icon: "timer",
    duration: "2 min",
  },
  {
    id: "blocking",
    title: "App & Website Blocking",
    description: "Block distracting apps and websites during focus sessions",
    icon: "shield",
    duration: "2 min",
  },
  {
    id: "journal",
    title: "Trigger Journaling",
    description: "Log your internal triggers to understand and overcome them",
    icon: "book-open",
    duration: "2 min",
  },
  {
    id: "analytics",
    title: "Analytics & Insights",
    description: "Track your productivity patterns and see your progress over time",
    icon: "bar-chart",
    duration: "2 min",
  },
  {
    id: "calendar",
    title: "Calendar Integration",
    description: "Sync your calendar and get smart focus time suggestions (Pro)",
    icon: "calendar",
    duration: "2 min",
  },
  {
    id: "ai-coach",
    title: "AI Coach",
    description: "Get personalized advice based on the Indistractable framework (Pro)",
    icon: "bot",
    duration: "2 min",
  },
];

interface TutorialStepProps {
  viewedTutorials: string[];
  onTutorialViewed: (tutorialId: string) => void;
  progress: number;
  currentStep: number;
  totalSteps: number;
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
}

const ICON_MAP = {
  timer: Timer,
  shield: Shield,
  "book-open": BookOpen,
  "bar-chart": BarChart,
  calendar: Calendar,
  bot: Bot,
};

export function TutorialStep({
  viewedTutorials,
  onTutorialViewed,
  progress,
  currentStep,
  totalSteps,
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
}: TutorialStepProps) {
  const toggleTutorial = (tutorialId: string) => {
    onTutorialViewed(tutorialId);
  };

  const completedCount = viewedTutorials.length;
  const freeTopicsCount = TUTORIAL_TOPICS.filter((t) =>
    ["focus-timer", "blocking", "journal", "analytics"].includes(t.id)
  ).length;

  return (
    <StepWrapper
      title="Explore FocusFlow"
      description="Get familiar with the key features that will help you stay focused"
      progress={progress}
      currentStep={currentStep}
      totalSteps={totalSteps}
      onNext={onNext}
      onPrevious={onPrevious}
      canGoNext={canGoNext}
      canGoPrevious={canGoPrevious}
      nextLabel="Complete Setup"
    >
      {/* Progress indicator */}
      <div className="flex items-center justify-between p-4 rounded-lg bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-950/30 dark:to-purple-950/30 border">
        <div className="space-y-1">
          <p className="text-sm font-medium">Your Progress</p>
          <p className="text-2xl font-bold">
            {completedCount} / {TUTORIAL_TOPICS.length}
          </p>
        </div>
        <Sparkles className="h-10 w-10 text-purple-600 dark:text-purple-400" />
      </div>

      {/* Tutorial cards */}
      <div className="space-y-3">
        {TUTORIAL_TOPICS.map((topic) => {
          const Icon = ICON_MAP[topic.icon as keyof typeof ICON_MAP];
          const isViewed = viewedTutorials.includes(topic.id);
          const isPro = ["calendar", "ai-coach"].includes(topic.id);

          return (
            <Card
              key={topic.id}
              className={`cursor-pointer transition-all border-2 ${
                isViewed ? "border-primary bg-primary/5" : "hover:border-primary/50"
              }`}
              onClick={() => !isViewed && toggleTutorial(topic.id)}
            >
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex items-start gap-3 flex-1">
                    <div
                      className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${
                        isViewed
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted text-muted-foreground"
                      }`}
                    >
                      <Icon className="h-5 w-5" />
                    </div>
                    <div className="flex-1 space-y-1">
                      <div className="flex items-center gap-2 flex-wrap">
                        <CardTitle className="text-base">{topic.title}</CardTitle>
                        {isPro && (
                          <Badge variant="outline" className="gap-1 text-xs">
                            <Lock className="h-3 w-3" />
                            Pro
                          </Badge>
                        )}
                        <Badge variant="secondary" className="text-xs">
                          {topic.duration}
                        </Badge>
                      </div>
                      <CardDescription className="text-sm">{topic.description}</CardDescription>
                    </div>
                  </div>
                  <div className="flex-shrink-0">
                    {isViewed ? (
                      <CheckCircle2 className="h-6 w-6 text-primary" />
                    ) : (
                      <Circle className="h-6 w-6 text-muted-foreground" />
                    )}
                  </div>
                </div>
              </CardHeader>

              {isViewed && (
                <CardContent className="pt-0">
                  <TutorialContent topicId={topic.id} />
                </CardContent>
              )}
            </Card>
          );
        })}
      </div>

      {/* Info note */}
      <div className="p-4 rounded-lg bg-muted/50 border space-y-2">
        <p className="text-sm font-medium">Click each feature to learn more</p>
        <p className="text-sm text-muted-foreground">
          Don't worry - you can explore these features at your own pace after setup. This is just a
          quick overview to get you started.
        </p>
      </div>

      {/* Completion message */}
      {completedCount >= freeTopicsCount && (
        <div className="p-4 rounded-lg bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800">
          <div className="flex items-center gap-2 text-green-900 dark:text-green-100">
            <CheckCircle2 className="h-5 w-5 text-green-600 dark:text-green-400" />
            <p className="text-sm font-medium">
              Great job! You're ready to start your FocusFlow journey.
            </p>
          </div>
        </div>
      )}
    </StepWrapper>
  );
}

interface TutorialContentProps {
  topicId: string;
}

function TutorialContent({ topicId }: TutorialContentProps) {
  const content = {
    "focus-timer": (
      <div className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          The Focus Timer helps you implement timeboxing - a core Indistractable practice:
        </p>
        <ul className="space-y-1 ml-4 text-muted-foreground">
          <li>• Set your desired focus duration</li>
          <li>• Start the session to activate blocking</li>
          <li>• Get notified when time is up</li>
          <li>• All sessions are tracked for analytics</li>
        </ul>
      </div>
    ),
    blocking: (
      <div className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          Block distracting apps and websites during focus sessions:
        </p>
        <ul className="space-y-1 ml-4 text-muted-foreground">
          <li>• Blocks activate automatically with sessions</li>
          <li>• Customize your blocklist anytime</li>
          <li>• Works system-wide for maximum effectiveness</li>
          <li>• Emergency override available if needed</li>
        </ul>
      </div>
    ),
    journal: (
      <div className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          Track your internal triggers to understand what drives distraction:
        </p>
        <ul className="space-y-1 ml-4 text-muted-foreground">
          <li>• Log triggers when you feel distracted</li>
          <li>• Record emotions and intensity</li>
          <li>• Discover patterns in your behavior</li>
          <li>• Learn to master your triggers over time</li>
        </ul>
      </div>
    ),
    analytics: (
      <div className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          Visualize your productivity and track your progress:
        </p>
        <ul className="space-y-1 ml-4 text-muted-foreground">
          <li>• See total focus time and sessions</li>
          <li>• Track weekly and monthly trends</li>
          <li>• Monitor productivity scores</li>
          <li>• Identify peak performance times</li>
        </ul>
      </div>
    ),
    calendar: (
      <div className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          Sync your calendar for smart focus scheduling (Pro feature):
        </p>
        <ul className="space-y-1 ml-4 text-muted-foreground">
          <li>• Connect Google Calendar</li>
          <li>• Get suggested focus time slots</li>
          <li>• See meeting load and free time</li>
          <li>• Schedule focus sessions automatically</li>
        </ul>
      </div>
    ),
    "ai-coach": (
      <div className="space-y-2 text-sm">
        <p className="text-muted-foreground">
          Get personalized coaching based on your data (Pro feature):
        </p>
        <ul className="space-y-1 ml-4 text-muted-foreground">
          <li>• Daily tips based on your patterns</li>
          <li>• Advice rooted in Indistractable principles</li>
          <li>• Pattern analysis and insights</li>
          <li>• Reflection prompts for growth</li>
        </ul>
      </div>
    ),
  };

  return content[topicId as keyof typeof content];
}
