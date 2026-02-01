// components/FocusTimeOnboarding.tsx - Interactive onboarding flow for Focus Time

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Calendar,
  Shield,
  CheckCircle,
  ArrowRight,
  ArrowLeft,
  X,
  Sparkles,
  Clock,
  Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { useFocusTimeOnboarding } from "@/hooks/useFocusTimeOnboarding";
import { cn } from "@/lib/utils";

interface FocusTimeOnboardingProps {
  onComplete?: () => void;
  onSkip?: () => void;
}

type OnboardingStep = "welcome" | "how-it-works" | "setup" | "ready";

const STEPS: OnboardingStep[] = ["welcome", "how-it-works", "setup", "ready"];

export function FocusTimeOnboarding({ onComplete, onSkip }: FocusTimeOnboardingProps) {
  const [currentStep, setCurrentStep] = useState<OnboardingStep>("welcome");
  const { completeOnboarding, skipOnboarding } = useFocusTimeOnboarding();

  const currentStepIndex = STEPS.indexOf(currentStep);
  const isFirstStep = currentStepIndex === 0;
  const isLastStep = currentStepIndex === STEPS.length - 1;
  const progress = ((currentStepIndex + 1) / STEPS.length) * 100;

  const handleNext = useCallback(() => {
    if (!isLastStep) {
      const nextStep = STEPS[currentStepIndex + 1];
      setCurrentStep(nextStep);
    } else {
      completeOnboarding();
      onComplete?.();
    }
  }, [currentStepIndex, isLastStep, completeOnboarding, onComplete]);

  const handlePrevious = useCallback(() => {
    if (!isFirstStep) {
      const prevStep = STEPS[currentStepIndex - 1];
      setCurrentStep(prevStep);
    }
  }, [currentStepIndex, isFirstStep]);

  const handleSkip = useCallback(() => {
    skipOnboarding();
    onSkip?.();
  }, [skipOnboarding, onSkip]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        transition={{ duration: 0.2 }}
        className="relative w-full max-w-2xl mx-4"
      >
        <Card className="relative overflow-hidden">
          {/* Progress bar */}
          <div className="absolute top-0 left-0 right-0 h-1 bg-muted">
            <motion.div
              className="h-full bg-primary"
              initial={{ width: "0%" }}
              animate={{ width: `${progress}%` }}
              transition={{ duration: 0.3 }}
            />
          </div>

          {/* Skip button */}
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={handleSkip}
            className="absolute top-4 right-4 z-10"
            aria-label="Skip onboarding"
          >
            <X className="size-4" />
          </Button>

          <CardContent className="pt-8">
            <AnimatePresence mode="wait">
              {currentStep === "welcome" && <WelcomeStep key="welcome" onNext={handleNext} />}
              {currentStep === "how-it-works" && (
                <HowItWorksStep key="how-it-works" onNext={handleNext} onBack={handlePrevious} />
              )}
              {currentStep === "setup" && (
                <SetupStep key="setup" onNext={handleNext} onBack={handlePrevious} />
              )}
              {currentStep === "ready" && (
                <ReadyStep key="ready" onComplete={handleNext} onBack={handlePrevious} />
              )}
            </AnimatePresence>

            {/* Step indicators */}
            <div className="flex justify-center gap-2 mt-8">
              {STEPS.map((step, index) => (
                <div
                  key={step}
                  className={cn(
                    "h-2 rounded-full transition-all duration-300",
                    index === currentStepIndex
                      ? "w-8 bg-primary"
                      : index < currentStepIndex
                        ? "w-2 bg-primary/50"
                        : "w-2 bg-muted"
                  )}
                  aria-label={`Step ${index + 1} of ${STEPS.length}`}
                />
              ))}
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </div>
  );
}

// Step 1: Welcome
function WelcomeStep({ onNext }: { onNext: () => void }) {
  return (
    <StepContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        className="text-center space-y-6"
      >
        <div className="flex justify-center">
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ delay: 0.2, type: "spring", stiffness: 200 }}
            className="p-4 rounded-full bg-primary/10"
          >
            <Sparkles className="size-12 text-primary" />
          </motion.div>
        </div>

        <div className="space-y-2">
          <h2 className="text-3xl font-bold">Welcome to Focus Time!</h2>
          <p className="text-lg text-muted-foreground">Your calendar-powered distraction blocker</p>
        </div>

        <p className="text-muted-foreground max-w-md mx-auto">
          Focus Time automatically blocks distracting apps and websites based on your calendar
          events, helping you stay focused during important work sessions.
        </p>

        <Button onClick={onNext} size="lg" className="gap-2">
          Get Started
          <ArrowRight className="size-4" />
        </Button>
      </motion.div>
    </StepContainer>
  );
}

// Step 2: How It Works
function HowItWorksStep({ onNext, onBack }: { onNext: () => void; onBack: () => void }) {
  return (
    <StepContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        className="space-y-8"
      >
        <div className="text-center space-y-2">
          <h2 className="text-2xl font-bold">How Focus Time Works</h2>
          <p className="text-muted-foreground">Three simple steps to automated focus</p>
        </div>

        <div className="space-y-4">
          {/* Step 1 */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
            className="flex gap-4 items-start"
          >
            <div className="flex-shrink-0 p-3 rounded-lg bg-primary/10">
              <Calendar className="size-6 text-primary" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold mb-1">1. Create a Calendar Event</h3>
              <p className="text-sm text-muted-foreground">
                Add a calendar event with{" "}
                <code className="px-1.5 py-0.5 bg-muted rounded text-xs font-mono">
                  [Focus Time]
                </code>{" "}
                in the title
              </p>
              <div className="mt-2 p-2 bg-muted rounded text-xs font-mono">
                Example: "[Focus Time] Deep Work Session"
              </div>
            </div>
          </motion.div>

          {/* Step 2 */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.2 }}
            className="flex gap-4 items-start"
          >
            <div className="flex-shrink-0 p-3 rounded-lg bg-primary/10">
              <Shield className="size-6 text-primary" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold mb-1">2. Automatic Blocking Starts</h3>
              <p className="text-sm text-muted-foreground">
                When the event begins, distracting apps and websites are automatically blocked
              </p>
            </div>
          </motion.div>

          {/* Step 3 */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.3 }}
            className="flex gap-4 items-start"
          >
            <div className="flex-shrink-0 p-3 rounded-lg bg-primary/10">
              <Zap className="size-6 text-primary" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold mb-1">3. Stay Focused</h3>
              <p className="text-sm text-muted-foreground">
                Focus on your work without distractions until the event ends
              </p>
            </div>
          </motion.div>
        </div>

        <StepNavigation onNext={onNext} onBack={onBack} />
      </motion.div>
    </StepContainer>
  );
}

// Step 3: Setup
function SetupStep({ onNext, onBack }: { onNext: () => void; onBack: () => void }) {
  return (
    <StepContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        className="space-y-8"
      >
        <div className="text-center space-y-2">
          <h2 className="text-2xl font-bold">Set Up Your First Focus Time</h2>
          <p className="text-muted-foreground">Choose how you'd like to get started</p>
        </div>

        <div className="grid gap-4">
          {/* Option A: Create in Calendar */}
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
          >
            <Card className="border-2 hover:border-primary/50 transition-colors cursor-pointer group">
              <CardHeader>
                <div className="flex items-start gap-4">
                  <div className="p-2 rounded-lg bg-primary/10 group-hover:bg-primary/20 transition-colors">
                    <Calendar className="size-5 text-primary" />
                  </div>
                  <div className="flex-1">
                    <CardTitle className="text-lg">Create in Google Calendar</CardTitle>
                    <CardDescription className="mt-1">
                      Add a "[Focus Time]" event to your calendar and it will sync automatically
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
            </Card>
          </motion.div>

          {/* Option B: Quick Start */}
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
          >
            <Card className="border-2 hover:border-primary/50 transition-colors cursor-pointer group">
              <CardHeader>
                <div className="flex items-start gap-4">
                  <div className="p-2 rounded-lg bg-primary/10 group-hover:bg-primary/20 transition-colors">
                    <Clock className="size-5 text-primary" />
                  </div>
                  <div className="flex-1">
                    <CardTitle className="text-lg">Start Quick Focus Session</CardTitle>
                    <CardDescription className="mt-1">
                      Start a manual Focus Time session right now (25 minutes)
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
            </Card>
          </motion.div>
        </div>

        <div className="text-center text-sm text-muted-foreground">
          You can customize blocked apps and websites in Settings anytime
        </div>

        <StepNavigation onNext={onNext} onBack={onBack} nextLabel="Continue" />
      </motion.div>
    </StepContainer>
  );
}

// Step 4: Ready
function ReadyStep({ onComplete, onBack }: { onComplete: () => void; onBack: () => void }) {
  return (
    <StepContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        className="text-center space-y-6"
      >
        <div className="flex justify-center">
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ delay: 0.2, type: "spring", stiffness: 200 }}
            className="p-4 rounded-full bg-primary/10"
          >
            <CheckCircle className="size-12 text-primary" />
          </motion.div>
        </div>

        <div className="space-y-2">
          <h2 className="text-3xl font-bold">You're All Set!</h2>
          <p className="text-lg text-muted-foreground">Ready to boost your productivity</p>
        </div>

        <div className="space-y-3 max-w-md mx-auto text-sm text-muted-foreground">
          <div className="flex items-center gap-2 justify-center">
            <CheckCircle className="size-4 text-primary flex-shrink-0" />
            <span>Calendar events with "[Focus Time]" will automatically block distractions</span>
          </div>
          <div className="flex items-center gap-2 justify-center">
            <CheckCircle className="size-4 text-primary flex-shrink-0" />
            <span>You can start manual sessions anytime from the timer</span>
          </div>
          <div className="flex items-center gap-2 justify-center">
            <CheckCircle className="size-4 text-primary flex-shrink-0" />
            <span>Customize settings and blocked apps in the Settings page</span>
          </div>
        </div>

        <div className="flex gap-3 justify-center pt-4">
          <Button variant="outline" onClick={onBack}>
            <ArrowLeft className="size-4" />
            Back
          </Button>
          <Button onClick={onComplete} size="lg" className="gap-2">
            Start Using Focus Time
            <CheckCircle className="size-4" />
          </Button>
        </div>
      </motion.div>
    </StepContainer>
  );
}

// Helper Components
function StepContainer({ children }: { children: React.ReactNode }) {
  return <div className="min-h-[400px] flex flex-col justify-center py-8">{children}</div>;
}

function StepNavigation({
  onNext,
  onBack,
  nextLabel = "Next",
}: {
  onNext: () => void;
  onBack: () => void;
  nextLabel?: string;
}) {
  return (
    <div className="flex gap-3 justify-center">
      <Button variant="outline" onClick={onBack}>
        <ArrowLeft className="size-4" />
        Back
      </Button>
      <Button onClick={onNext} className="gap-2">
        {nextLabel}
        <ArrowRight className="size-4" />
      </Button>
    </div>
  );
}
