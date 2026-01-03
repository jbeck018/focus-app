// App.tsx - Main application component

import { useState } from "react";
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar";
import { AppSidebar, type ViewType } from "@/components/app-sidebar";
import { Toaster } from "@/components/ui/sonner";
import { SkipLink } from "@/components/accessibility/skip-link";
import { FocusTimer } from "@/features/FocusTimer";
import { Dashboard } from "@/features/Dashboard";
import { BlockingSettings } from "@/features/BlockingSettings";
import { Journal } from "@/features/Journal";
import { Calendar } from "@/features/Calendar";
import { AICoach } from "@/features/AICoach";
import { TeamDashboard } from "@/features/TeamDashboard";
import { AchievementGallery } from "@/features/achievements";
import { StreakDashboard } from "@/features/streaks";
import { AnalyticsDashboard } from "@/features/analytics";
import { OnboardingWizard } from "@/features/onboarding/onboarding-wizard";
import { useMiniTimerShortcut } from "@/hooks/useKeyboardShortcuts";
import { useNeedsOnboarding } from "@/hooks/use-onboarding";

function App() {
  const [activeView, setActiveView] = useState<ViewType>("timer");
  const needsOnboarding = useNeedsOnboarding();
  const [showOnboarding, setShowOnboarding] = useState(needsOnboarding);

  // Register global keyboard shortcuts
  useMiniTimerShortcut();

  // Show onboarding wizard if needed
  if (showOnboarding) {
    return <OnboardingWizard onComplete={() => setShowOnboarding(false)} />;
  }

  // Render component based on active view
  const renderView = () => {
    switch (activeView) {
      case "timer":
        return <FocusTimer />;
      case "dashboard":
        return <Dashboard />;
      case "calendar":
        return <Calendar />;
      case "streaks":
        return <StreakDashboard />;
      case "analytics":
        return <AnalyticsDashboard />;
      case "achievements":
        return <AchievementGallery />;
      case "journal":
        return <Journal />;
      case "coach":
        return <AICoach />;
      case "team":
        return <TeamDashboard />;
      case "blocking":
        return <BlockingSettings />;
      default:
        return <FocusTimer />;
    }
  };

  return (
    <SidebarProvider defaultOpen={true}>
      {/* Skip to main content link - WCAG 2.1 Level A (2.4.1) */}
      <SkipLink href="#main-content">Skip to main content</SkipLink>

      {/* Sidebar Navigation */}
      <AppSidebar activeView={activeView} onViewChange={setActiveView} />

      {/* Main Content Area */}
      <SidebarInset className="flex flex-col">
        {/* Header */}
        <header
          className="sticky top-0 z-10 flex h-13 shrink-0 items-center gap-2 border-b bg-background px-4"
          role="banner"
        >
          <SidebarTrigger className="-ml-1" />
          <div className="flex-1">
            <p className="text-sm text-muted-foreground hidden md:block">
              Stay focused, block distractions, achieve more
            </p>
          </div>
        </header>

        {/* Main Content */}
        <main
          id="main-content"
          className="flex-1 overflow-auto p-4 md:p-6"
          tabIndex={-1}
          role="main"
        >
          {/*
            Timer view: Vertically centered (hero/focal element)
            All other views: Top-aligned (standard page layout)
          */}
          <div
            className={
              activeView === "timer"
                ? "min-h-full flex flex-col items-center justify-center"
                : "w-full max-w-6xl mx-auto"
            }
          >
            {activeView === "timer" ? (
              <div className="w-full max-w-md">{renderView()}</div>
            ) : (
              renderView()
            )}
          </div>
        </main>
      </SidebarInset>

      {/* Toast notifications - Accessible status region */}
      <Toaster position="bottom-right" />
    </SidebarProvider>
  );
}

export default App;
