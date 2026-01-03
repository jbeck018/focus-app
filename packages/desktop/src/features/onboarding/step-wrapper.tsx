// features/onboarding/step-wrapper.tsx - Reusable wrapper for onboarding steps

import { ReactNode } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { ChevronLeft, ChevronRight } from "lucide-react";

interface StepWrapperProps {
  title: string;
  description: string;
  children: ReactNode;
  progress: number;
  currentStep: number;
  totalSteps: number;
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
  nextLabel?: string;
  previousLabel?: string;
  isNextDisabled?: boolean;
  className?: string;
}

export function StepWrapper({
  title,
  description,
  children,
  progress,
  currentStep,
  totalSteps,
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
  nextLabel = "Next",
  previousLabel = "Previous",
  isNextDisabled = false,
  className = "",
}: StepWrapperProps) {
  return (
    <div className={`w-full max-w-3xl mx-auto space-y-6 ${className}`}>
      {/* Progress indicator */}
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span>
            Step {currentStep + 1} of {totalSteps}
          </span>
          <span>{progress}% Complete</span>
        </div>
        <Progress value={progress} className="h-2" />
      </div>

      {/* Main content card */}
      <Card className="border-2">
        <CardHeader>
          <CardTitle className="text-2xl">{title}</CardTitle>
          <CardDescription className="text-base">{description}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {children}

          {/* Navigation buttons */}
          <div className="flex items-center justify-between pt-6 border-t">
            <Button
              variant="outline"
              onClick={onPrevious}
              disabled={!canGoPrevious}
              className="gap-2"
            >
              <ChevronLeft className="h-4 w-4" />
              {previousLabel}
            </Button>

            <Button
              onClick={onNext}
              disabled={!canGoNext || isNextDisabled}
              className="gap-2"
            >
              {nextLabel}
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
