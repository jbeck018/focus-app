// features/onboarding/onboarding-wizard.tsx - Main onboarding wizard component

import { useEffect } from "react";
import { useOnboarding } from "@/hooks/use-onboarding";
import { WelcomeStep } from "./steps/welcome-step";
import { PillarsStep } from "./steps/pillars-step";
import { BlocklistStep } from "./steps/blocklist-step";
import { PreferencesStep } from "./steps/preferences-step";
import { TutorialStep } from "./steps/tutorial-step";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { CheckCircle2, Sparkles, ArrowRight } from "lucide-react";
import { toast } from "sonner";

interface OnboardingWizardProps {
  onComplete: () => void;
}

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const {
    currentStep,
    onboardingData,
    isLoading,
    error,
    nextStep,
    previousStep,
    canGoNext,
    canGoPrevious,
    currentStepIndex,
    totalSteps,
    progress,
    updateData,
    completeOnboarding,
  } = useOnboarding();

  // Show error toast if there's an error
  useEffect(() => {
    if (error) {
      toast.error("Onboarding Error", {
        description: error,
      });
    }
  }, [error]);

  // Handle completion step
  const handleComplete = async () => {
    try {
      await completeOnboarding();
      toast.success("Welcome to FocusFlow!", {
        description: "Your account is set up and ready to go.",
      });
      onComplete();
    } catch (err) {
      console.error("Failed to complete onboarding:", err);
    }
  };

  // Render complete step
  if (currentStep === "complete") {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-4">
        <div className="w-full max-w-2xl space-y-6">
          <Card className="border-2">
            <CardHeader className="text-center space-y-4 pb-4">
              <div className="mx-auto w-20 h-20 rounded-full bg-gradient-to-br from-green-500 to-emerald-600 flex items-center justify-center">
                <CheckCircle2 className="h-12 w-12 text-white" />
              </div>
              <div className="space-y-2">
                <CardTitle className="text-3xl">
                  You're All Set, {onboardingData.userName}!
                </CardTitle>
                <CardDescription className="text-base">
                  Welcome to FocusFlow. Your journey to becoming Indistractable starts now.
                </CardDescription>
              </div>
            </CardHeader>

            <CardContent className="space-y-6">
              {/* Setup summary */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <SummaryCard
                  title="Blocked Items"
                  value={`${onboardingData.selectedApps.length + onboardingData.selectedWebsites.length}`}
                  subtitle="apps and websites"
                />
                <SummaryCard
                  title="Focus Duration"
                  value={`${onboardingData.defaultFocusDuration} min`}
                  subtitle="default session"
                />
                <SummaryCard
                  title="Break Duration"
                  value={`${onboardingData.defaultBreakDuration} min`}
                  subtitle="between sessions"
                />
                <SummaryCard
                  title="Notifications"
                  value={onboardingData.enableNotifications ? "Enabled" : "Disabled"}
                  subtitle="session alerts"
                />
              </div>

              {/* Next steps */}
              <div className="space-y-3">
                <h3 className="font-semibold text-lg flex items-center gap-2">
                  <Sparkles className="h-5 w-5 text-primary" />
                  What's Next?
                </h3>
                <ul className="space-y-2">
                  <NextStepItem
                    number={1}
                    text="Start your first focus session from the Timer tab"
                  />
                  <NextStepItem
                    number={2}
                    text="Log your internal triggers in the Journal when you feel distracted"
                  />
                  <NextStepItem number={3} text="Review your progress in the Stats dashboard" />
                  <NextStepItem number={4} text="Customize your blocklist in the Block settings" />
                </ul>
              </div>

              {/* Action button */}
              <Button
                onClick={handleComplete}
                size="lg"
                className="w-full gap-2 text-lg py-6"
                disabled={isLoading}
              >
                {isLoading ? (
                  "Setting up..."
                ) : (
                  <>
                    Start Using FocusFlow
                    <ArrowRight className="h-5 w-5" />
                  </>
                )}
              </Button>

              {/* Indistractable reminder */}
              <div className="text-center p-4 rounded-lg bg-muted/50 border">
                <p className="text-sm text-muted-foreground italic">
                  "You can have anything you want in life if you just help enough other people get
                  what they want."
                  <br />
                  <span className="font-medium">- Zig Ziglar</span>
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  // Render current step
  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full">
        {currentStep === "welcome" && (
          <WelcomeStep
            userName={onboardingData.userName}
            onUserNameChange={(value) => updateData("userName", value)}
            progress={progress}
            currentStep={currentStepIndex}
            totalSteps={totalSteps}
            onNext={nextStep}
            onPrevious={previousStep}
            canGoNext={canGoNext}
            canGoPrevious={canGoPrevious}
          />
        )}

        {currentStep === "pillars" && (
          <PillarsStep
            progress={progress}
            currentStep={currentStepIndex}
            totalSteps={totalSteps}
            onNext={nextStep}
            onPrevious={previousStep}
            canGoNext={canGoNext}
            canGoPrevious={canGoPrevious}
          />
        )}

        {currentStep === "blocklist" && (
          <BlocklistStep
            selectedApps={onboardingData.selectedApps}
            selectedWebsites={onboardingData.selectedWebsites}
            onAppsChange={(apps) => updateData("selectedApps", apps)}
            onWebsitesChange={(websites) => updateData("selectedWebsites", websites)}
            progress={progress}
            currentStep={currentStepIndex}
            totalSteps={totalSteps}
            onNext={nextStep}
            onPrevious={previousStep}
            canGoNext={canGoNext}
            canGoPrevious={canGoPrevious}
          />
        )}

        {currentStep === "preferences" && (
          <PreferencesStep
            defaultFocusDuration={onboardingData.defaultFocusDuration}
            defaultBreakDuration={onboardingData.defaultBreakDuration}
            enableNotifications={onboardingData.enableNotifications}
            autoStartBreaks={onboardingData.autoStartBreaks}
            onFocusDurationChange={(value) => updateData("defaultFocusDuration", value)}
            onBreakDurationChange={(value) => updateData("defaultBreakDuration", value)}
            onNotificationsChange={(value) => updateData("enableNotifications", value)}
            onAutoStartBreaksChange={(value) => updateData("autoStartBreaks", value)}
            progress={progress}
            currentStep={currentStepIndex}
            totalSteps={totalSteps}
            onNext={nextStep}
            onPrevious={previousStep}
            canGoNext={canGoNext}
            canGoPrevious={canGoPrevious}
          />
        )}

        {currentStep === "tutorial" && (
          <TutorialStep
            viewedTutorials={onboardingData.viewedTutorials}
            onTutorialViewed={(tutorialId: string) => {
              const current = onboardingData.viewedTutorials;
              if (current.includes(tutorialId)) {
                updateData(
                  "viewedTutorials",
                  current.filter((id: string) => id !== tutorialId)
                );
              } else {
                updateData("viewedTutorials", [...current, tutorialId]);
              }
            }}
            progress={progress}
            currentStep={currentStepIndex}
            totalSteps={totalSteps}
            onNext={nextStep}
            onPrevious={previousStep}
            canGoNext={canGoNext}
            canGoPrevious={canGoPrevious}
          />
        )}
      </div>
    </div>
  );
}

interface SummaryCardProps {
  title: string;
  value: string;
  subtitle: string;
}

function SummaryCard({ title, value, subtitle }: SummaryCardProps) {
  return (
    <div className="p-4 rounded-lg border bg-card text-center space-y-1">
      <p className="text-sm text-muted-foreground">{title}</p>
      <p className="text-2xl font-bold">{value}</p>
      <p className="text-xs text-muted-foreground">{subtitle}</p>
    </div>
  );
}

interface NextStepItemProps {
  number: number;
  text: string;
}

function NextStepItem({ number, text }: NextStepItemProps) {
  return (
    <li className="flex items-start gap-3">
      <span className="flex-shrink-0 w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-medium">
        {number}
      </span>
      <span className="text-sm text-muted-foreground pt-0.5">{text}</span>
    </li>
  );
}
