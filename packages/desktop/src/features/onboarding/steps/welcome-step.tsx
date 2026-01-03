// features/onboarding/steps/welcome-step.tsx - Welcome step for onboarding

import { StepWrapper } from "../step-wrapper";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Sparkles, Target, Brain, Shield } from "lucide-react";

interface WelcomeStepProps {
  userName: string;
  onUserNameChange: (value: string) => void;
  progress: number;
  currentStep: number;
  totalSteps: number;
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
}

export function WelcomeStep({
  userName,
  onUserNameChange,
  progress,
  currentStep,
  totalSteps,
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
}: WelcomeStepProps) {
  const isNextDisabled = !userName.trim();

  return (
    <StepWrapper
      title="Welcome to FocusFlow"
      description="Your journey to becoming Indistractable starts here"
      progress={progress}
      currentStep={currentStep}
      totalSteps={totalSteps}
      onNext={onNext}
      onPrevious={onPrevious}
      canGoNext={canGoNext}
      canGoPrevious={canGoPrevious}
      isNextDisabled={isNextDisabled}
      nextLabel="Get Started"
    >
      {/* Hero section */}
      <div className="text-center space-y-4 py-6">
        <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-white">
          <Sparkles className="h-10 w-10" />
        </div>
        <h3 className="text-xl font-semibold">
          Master Your Focus, Reclaim Your Time
        </h3>
        <p className="text-muted-foreground max-w-lg mx-auto">
          FocusFlow is built on the proven Indistractable framework by Nir Eyal.
          In the next few minutes, we'll help you set up your productivity system.
        </p>
      </div>

      {/* What you'll learn */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 my-8">
        <FeatureCard
          icon={<Brain className="h-5 w-5" />}
          title="Master Your Triggers"
          description="Understand what drives distraction"
          color="bg-blue-50 text-blue-600 dark:bg-blue-950 dark:text-blue-400"
        />
        <FeatureCard
          icon={<Target className="h-5 w-5" />}
          title="Time for Traction"
          description="Schedule what matters most"
          color="bg-green-50 text-green-600 dark:bg-green-950 dark:text-green-400"
        />
        <FeatureCard
          icon={<Shield className="h-5 w-5" />}
          title="Block Distractions"
          description="Remove external triggers"
          color="bg-orange-50 text-orange-600 dark:bg-orange-950 dark:text-orange-400"
        />
        <FeatureCard
          icon={<Sparkles className="h-5 w-5" />}
          title="Build Better Habits"
          description="Make focus automatic"
          color="bg-purple-50 text-purple-600 dark:bg-purple-950 dark:text-purple-400"
        />
      </div>

      {/* Name input */}
      <div className="space-y-3 max-w-md mx-auto">
        <Label htmlFor="userName" className="text-base">
          What should we call you?
        </Label>
        <Input
          id="userName"
          type="text"
          placeholder="Enter your name"
          value={userName}
          onChange={(e) => onUserNameChange(e.target.value)}
          className="text-lg py-6"
          autoFocus
        />
        <p className="text-sm text-muted-foreground">
          We'll use this to personalize your experience
        </p>
      </div>

      {/* Time estimate */}
      <div className="text-center pt-4">
        <p className="text-sm text-muted-foreground">
          Setup takes about 3-5 minutes
        </p>
      </div>
    </StepWrapper>
  );
}

interface FeatureCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  color: string;
}

function FeatureCard({ icon, title, description, color }: FeatureCardProps) {
  return (
    <div className="flex gap-3 p-4 rounded-lg border bg-card">
      <div className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${color}`}>
        {icon}
      </div>
      <div className="space-y-1">
        <h4 className="font-medium">{title}</h4>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
    </div>
  );
}
