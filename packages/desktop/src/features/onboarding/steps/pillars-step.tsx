// features/onboarding/steps/pillars-step.tsx - Explains the 4 pillars of Indistractable

import { useState } from "react";
import { StepWrapper } from "../step-wrapper";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Brain, Calendar, BellOff, ShieldCheck, CheckCircle2 } from "lucide-react";

interface IndistractablePillar {
  id: string;
  title: string;
  description: string;
  icon: string;
  color: string;
  examples: string[];
  actionItems: string[];
}

const INDISTRACTABLE_PILLARS: IndistractablePillar[] = [
  {
    id: "internal-triggers",
    title: "Master Internal Triggers",
    description:
      "Understand the uncomfortable emotional states that prompt you to seek distraction. Learn to surf the urge instead of giving in.",
    icon: "brain",
    color: "from-blue-500 to-cyan-500",
    examples: [
      "Feeling bored during a task",
      "Anxiety about upcoming deadlines",
      "Restlessness when working alone",
      "Curiosity about notifications",
    ],
    actionItems: [
      "Use the journal to log your triggers",
      "Practice the 10-minute rule",
      "Reimagine the task to make it more engaging",
    ],
  },
  {
    id: "make-time-traction",
    title: "Make Time for Traction",
    description:
      "Schedule your day with intention. Turn your values into time. Without planning, you'll default to distraction.",
    icon: "calendar",
    color: "from-green-500 to-emerald-500",
    examples: [
      "Time-boxing important work",
      "Scheduling breaks and rejuvenation",
      "Blocking time for relationships",
      "Planning weekly review sessions",
    ],
    actionItems: [
      "Create a timeboxed calendar",
      "Use the FocusTimer for dedicated sessions",
      "Align your time with your values",
    ],
  },
  {
    id: "hack-external-triggers",
    title: "Hack Back External Triggers",
    description:
      "External triggers aren't always bad, but they should serve you, not control you. Reduce, remove, or restructure them.",
    icon: "bell-off",
    color: "from-orange-500 to-amber-500",
    examples: ["Social media notifications", "Email alerts", "Slack messages", "News websites"],
    actionItems: [
      "Block distracting apps and websites",
      "Turn off non-essential notifications",
      "Use focus mode during deep work",
    ],
  },
  {
    id: "prevent-with-pacts",
    title: "Prevent Distraction with Pacts",
    description:
      "Make it harder to do things you don't want to do. Use effort pacts, price pacts, and identity pacts.",
    icon: "shield-check",
    color: "from-purple-500 to-pink-500",
    examples: [
      "Effort pact: Install website blockers",
      "Price pact: Bet money on completing tasks",
      "Identity pact: Tell others your commitments",
      "Use FocusFlow's blocking during sessions",
    ],
    actionItems: [
      "Set up app and website blocking",
      "Create accountability with team features",
      "Make distractions harder to access",
    ],
  },
];

interface PillarsStepProps {
  progress: number;
  currentStep: number;
  totalSteps: number;
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
}

const ICON_MAP = {
  brain: Brain,
  calendar: Calendar,
  "bell-off": BellOff,
  "shield-check": ShieldCheck,
};

export function PillarsStep({
  progress,
  currentStep,
  totalSteps,
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
}: PillarsStepProps) {
  const [expandedPillar, setExpandedPillar] = useState<string | null>(null);

  return (
    <StepWrapper
      title="The Indistractable Framework"
      description="Learn the 4 pillars that will transform how you handle distractions"
      progress={progress}
      currentStep={currentStep}
      totalSteps={totalSteps}
      onNext={onNext}
      onPrevious={onPrevious}
      canGoNext={canGoNext}
      canGoPrevious={canGoPrevious}
    >
      {/* Introduction */}
      <div className="prose dark:prose-invert max-w-none">
        <p className="text-base text-muted-foreground">
          Based on Nir Eyal's bestselling book "Indistractable," these four pillars form a
          comprehensive system for controlling your attention and choosing your life.
        </p>
      </div>

      {/* Pillars grid */}
      <div className="grid grid-cols-1 gap-4 mt-6">
        {INDISTRACTABLE_PILLARS.map((pillar, index) => {
          const Icon = ICON_MAP[pillar.icon as keyof typeof ICON_MAP] || Brain;
          const isExpanded = expandedPillar === pillar.id;

          return (
            <Card
              key={pillar.id}
              className={`cursor-pointer transition-all border-2 ${
                isExpanded ? "ring-2 ring-primary" : ""
              }`}
              onClick={() => setExpandedPillar(isExpanded ? null : pillar.id)}
            >
              <CardHeader>
                <div className="flex items-start gap-4">
                  <div
                    className={`flex-shrink-0 w-12 h-12 rounded-xl bg-gradient-to-br ${pillar.color} flex items-center justify-center text-white`}
                  >
                    <Icon className="h-6 w-6" />
                  </div>
                  <div className="flex-1 space-y-1">
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className="font-mono">
                        {index + 1}
                      </Badge>
                      <CardTitle className="text-lg">{pillar.title}</CardTitle>
                    </div>
                    <CardDescription className="text-sm">{pillar.description}</CardDescription>
                  </div>
                </div>
              </CardHeader>

              {isExpanded && (
                <CardContent className="pt-0 space-y-4">
                  {/* Examples */}
                  <div>
                    <h4 className="font-medium text-sm mb-2">Common Examples:</h4>
                    <ul className="space-y-1">
                      {pillar.examples.map((example, idx) => (
                        <li
                          key={idx}
                          className="text-sm text-muted-foreground flex items-start gap-2"
                        >
                          <span className="text-primary mt-0.5">•</span>
                          <span>{example}</span>
                        </li>
                      ))}
                    </ul>
                  </div>

                  {/* Action items */}
                  <div>
                    <h4 className="font-medium text-sm mb-2">How FocusFlow Helps:</h4>
                    <ul className="space-y-2">
                      {pillar.actionItems.map((action, idx) => (
                        <li
                          key={idx}
                          className="text-sm text-muted-foreground flex items-start gap-2"
                        >
                          <CheckCircle2 className="h-4 w-4 text-green-600 dark:text-green-400 mt-0.5 flex-shrink-0" />
                          <span>{action}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                </CardContent>
              )}
            </Card>
          );
        })}
      </div>

      {/* Call to action */}
      <div className="mt-6 p-4 rounded-lg bg-muted/50 border">
        <p className="text-sm text-center text-muted-foreground">
          Click each pillar to learn more. FocusFlow implements all four pillars to help you become
          truly Indistractable.
        </p>
      </div>
    </StepWrapper>
  );
}
