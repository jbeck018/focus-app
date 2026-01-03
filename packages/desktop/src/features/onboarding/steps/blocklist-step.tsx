// features/onboarding/steps/blocklist-step.tsx - Setup initial blocked apps and websites

import { useState } from "react";
import { StepWrapper } from "../step-wrapper";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Smartphone, Globe, Plus, X, AlertCircle } from "lucide-react";

// Common app suggestions for blocklisting
const SUGGESTED_APPS = [
  "Chrome",
  "Safari",
  "Firefox",
  "Slack",
  "Discord",
  "Messages",
  "WhatsApp",
  "Telegram",
  "Twitter",
  "Facebook",
  "Instagram",
  "TikTok",
  "YouTube",
  "Netflix",
  "Spotify",
  "Steam",
  "Epic Games"
];

// Common website suggestions for blocklisting
const SUGGESTED_WEBSITES = [
  "twitter.com",
  "facebook.com",
  "instagram.com",
  "reddit.com",
  "youtube.com",
  "netflix.com",
  "tiktok.com",
  "twitch.tv",
  "linkedin.com",
  "news.ycombinator.com",
  "medium.com",
  "buzzfeed.com",
  "dailymail.co.uk",
  "cnn.com",
  "bbc.com"
];

interface BlocklistStepProps {
  selectedApps: string[];
  selectedWebsites: string[];
  onAppsChange: (apps: string[]) => void;
  onWebsitesChange: (websites: string[]) => void;
  progress: number;
  currentStep: number;
  totalSteps: number;
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
}

export function BlocklistStep({
  selectedApps,
  selectedWebsites,
  onAppsChange,
  onWebsitesChange,
  progress,
  currentStep,
  totalSteps,
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
}: BlocklistStepProps) {
  const [customApp, setCustomApp] = useState("");
  const [customWebsite, setCustomWebsite] = useState("");
  const [activeTab, setActiveTab] = useState<"apps" | "websites">("apps");

  const toggleApp = (app: string) => {
    if (selectedApps.includes(app)) {
      onAppsChange(selectedApps.filter((a) => a !== app));
    } else {
      onAppsChange([...selectedApps, app]);
    }
  };

  const toggleWebsite = (website: string) => {
    if (selectedWebsites.includes(website)) {
      onWebsitesChange(selectedWebsites.filter((w) => w !== website));
    } else {
      onWebsitesChange([...selectedWebsites, website]);
    }
  };

  const addCustomApp = () => {
    const trimmed = customApp.trim();
    if (trimmed && !selectedApps.includes(trimmed)) {
      onAppsChange([...selectedApps, trimmed]);
      setCustomApp("");
    }
  };

  const addCustomWebsite = () => {
    const trimmed = customWebsite.trim().toLowerCase();
    if (trimmed && !selectedWebsites.includes(trimmed)) {
      onWebsitesChange([...selectedWebsites, trimmed]);
      setCustomWebsite("");
    }
  };

  const removeApp = (app: string) => {
    onAppsChange(selectedApps.filter((a) => a !== app));
  };

  const removeWebsite = (website: string) => {
    onWebsitesChange(selectedWebsites.filter((w) => w !== website));
  };

  return (
    <StepWrapper
      title="Block Your Distractions"
      description="Choose apps and websites to block during focus sessions"
      progress={progress}
      currentStep={currentStep}
      totalSteps={totalSteps}
      onNext={onNext}
      onPrevious={onPrevious}
      canGoNext={canGoNext}
      canGoPrevious={canGoPrevious}
      nextLabel="Continue"
    >
      {/* Info banner */}
      <div className="flex gap-3 p-4 rounded-lg bg-blue-50 dark:bg-blue-950/30 border border-blue-200 dark:border-blue-800">
        <AlertCircle className="h-5 w-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" />
        <div className="text-sm text-blue-900 dark:text-blue-100 space-y-1">
          <p className="font-medium">Pillar 3: Hack Back External Triggers</p>
          <p className="text-blue-700 dark:text-blue-300">
            These blocks will only activate during your focus sessions. You can modify this list anytime.
          </p>
        </div>
      </div>

      {/* Tabs for Apps and Websites */}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as "apps" | "websites")}>
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="apps" className="gap-2">
            <Smartphone className="h-4 w-4" />
            Apps ({selectedApps.length})
          </TabsTrigger>
          <TabsTrigger value="websites" className="gap-2">
            <Globe className="h-4 w-4" />
            Websites ({selectedWebsites.length})
          </TabsTrigger>
        </TabsList>

        {/* Apps tab */}
        <TabsContent value="apps" className="space-y-4 mt-4">
          {/* Selected apps */}
          {selectedApps.length > 0 && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm font-medium">Selected Apps</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {selectedApps.map((app) => (
                    <Badge
                      key={app}
                      variant="secondary"
                      className="gap-1 px-3 py-1.5 text-sm"
                    >
                      {app}
                      <button
                        onClick={() => removeApp(app)}
                        className="ml-1 hover:text-destructive"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Suggested apps */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium">Common Distracting Apps</h4>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
              {SUGGESTED_APPS.map((app) => (
                <Button
                  key={app}
                  variant={selectedApps.includes(app) ? "default" : "outline"}
                  size="sm"
                  onClick={() => toggleApp(app)}
                  className="justify-start"
                >
                  {app}
                </Button>
              ))}
            </div>
          </div>

          {/* Add custom app */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Add Custom App</h4>
            <div className="flex gap-2">
              <Input
                placeholder="Enter app name"
                value={customApp}
                onChange={(e) => setCustomApp(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addCustomApp()}
              />
              <Button onClick={addCustomApp} size="icon" variant="outline">
                <Plus className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </TabsContent>

        {/* Websites tab */}
        <TabsContent value="websites" className="space-y-4 mt-4">
          {/* Selected websites */}
          {selectedWebsites.length > 0 && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm font-medium">Selected Websites</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {selectedWebsites.map((website) => (
                    <Badge
                      key={website}
                      variant="secondary"
                      className="gap-1 px-3 py-1.5 text-sm"
                    >
                      {website}
                      <button
                        onClick={() => removeWebsite(website)}
                        className="ml-1 hover:text-destructive"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Suggested websites */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium">Common Distracting Websites</h4>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {SUGGESTED_WEBSITES.map((website) => (
                <Button
                  key={website}
                  variant={selectedWebsites.includes(website) ? "default" : "outline"}
                  size="sm"
                  onClick={() => toggleWebsite(website)}
                  className="justify-start text-xs"
                >
                  {website}
                </Button>
              ))}
            </div>
          </div>

          {/* Add custom website */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Add Custom Website</h4>
            <div className="flex gap-2">
              <Input
                placeholder="example.com"
                value={customWebsite}
                onChange={(e) => setCustomWebsite(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addCustomWebsite()}
              />
              <Button onClick={addCustomWebsite} size="icon" variant="outline">
                <Plus className="h-4 w-4" />
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              Enter domain only (e.g., twitter.com, not https://twitter.com)
            </p>
          </div>
        </TabsContent>
      </Tabs>

      {/* Skip option */}
      <div className="text-center pt-2">
        <p className="text-sm text-muted-foreground">
          You can skip this step and add blocks later from Settings
        </p>
      </div>
    </StepWrapper>
  );
}
