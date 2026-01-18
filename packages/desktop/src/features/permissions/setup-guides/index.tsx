// features/permissions/setup-guides/index.tsx - Main setup guide component with platform detection

import { useEffect, useState } from "react";
import { platform } from "@tauri-apps/plugin-os";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Monitor, Info } from "lucide-react";
import { MacOSGuide } from "./macos-guide";
import { WindowsGuide } from "./windows-guide";
import { LinuxGuide } from "./linux-guide";
import type { Platform } from "./use-platform";

export type { Platform };

interface PlatformInfo {
  name: string;
  icon: string;
  description: string;
}

const platformInfo: Record<Platform, PlatformInfo> = {
  macos: {
    name: "macOS",
    icon: "",
    description: "Setup instructions for macOS systems",
  },
  windows: {
    name: "Windows",
    icon: "",
    description: "Setup instructions for Windows systems",
  },
  linux: {
    name: "Linux",
    icon: "",
    description: "Setup instructions for Linux distributions",
  },
};

interface SetupGuidesProps {
  showHeader?: boolean;
  defaultPlatform?: Platform;
  onComplete?: () => void;
}

export function SetupGuides({ showHeader = true, defaultPlatform, onComplete }: SetupGuidesProps) {
  const [detectedPlatform, setDetectedPlatform] = useState<Platform | null>(null);
  const [selectedPlatform, setSelectedPlatform] = useState<Platform>(defaultPlatform ?? "macos");
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    function detectPlatform() {
      try {
        const platformName = platform();

        let detected: Platform;
        switch (platformName) {
          case "macos":
          case "ios":
            detected = "macos";
            break;
          case "windows":
            detected = "windows";
            break;
          case "linux":
          case "android":
            detected = "linux";
            break;
          default:
            detected = "linux";
        }

        setDetectedPlatform(detected);

        // Only override selected platform if no default was provided
        if (!defaultPlatform) {
          setSelectedPlatform(detected);
        }
      } catch (error) {
        console.error("Failed to detect platform:", error);
        // Fallback to provided default or macOS
        setDetectedPlatform(defaultPlatform ?? "macos");
      } finally {
        setIsLoading(false);
      }
    }

    detectPlatform();
  }, [defaultPlatform]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {showHeader && (
        <Card>
          <CardHeader>
            <div className="flex items-center gap-3">
              <Monitor className="h-6 w-6" />
              <div>
                <CardTitle className="text-2xl">Permissions Setup Guide</CardTitle>
                <CardDescription className="mt-1">
                  Configure FocusFlow to block distracting websites and applications
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert>
              <Info className="h-4 w-4" />
              <AlertTitle>Why are permissions needed?</AlertTitle>
              <AlertDescription className="space-y-2">
                <p>
                  FocusFlow uses DNS-level blocking by modifying your system's hosts file. This is
                  the most effective way to block distractions because:
                </p>
                <ul className="list-disc list-inside text-sm space-y-1 mt-2 ml-2">
                  <li>Works across all browsers and applications</li>
                  <li>Cannot be bypassed without system-level access</li>
                  <li>No browser extensions or third-party services required</li>
                  <li>Blocks at the network level before requests are made</li>
                </ul>
              </AlertDescription>
            </Alert>

            {detectedPlatform && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-lg">
                <Badge variant="secondary" className="bg-primary/10">
                  Detected Platform
                </Badge>
                <span className="text-sm font-medium">{platformInfo[detectedPlatform].name}</span>
                <span className="text-sm text-muted-foreground ml-auto">
                  Showing {platformInfo[selectedPlatform].name} instructions
                </span>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      <Tabs
        value={selectedPlatform}
        onValueChange={(value) => setSelectedPlatform(value as Platform)}
        className="w-full"
      >
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="macos" className="gap-2">
            <span className="hidden sm:inline">{platformInfo.macos.icon}</span>
            {platformInfo.macos.name}
            {detectedPlatform === "macos" && (
              <Badge variant="secondary" className="ml-1 bg-primary/10 text-xs px-1.5 py-0">
                Current
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="windows" className="gap-2">
            <span className="hidden sm:inline">{platformInfo.windows.icon}</span>
            {platformInfo.windows.name}
            {detectedPlatform === "windows" && (
              <Badge variant="secondary" className="ml-1 bg-primary/10 text-xs px-1.5 py-0">
                Current
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="linux" className="gap-2">
            <span className="hidden sm:inline">{platformInfo.linux.icon}</span>
            {platformInfo.linux.name}
            {detectedPlatform === "linux" && (
              <Badge variant="secondary" className="ml-1 bg-primary/10 text-xs px-1.5 py-0">
                Current
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="macos" className="mt-6">
          <MacOSGuide />
        </TabsContent>

        <TabsContent value="windows" className="mt-6">
          <WindowsGuide />
        </TabsContent>

        <TabsContent value="linux" className="mt-6">
          <LinuxGuide />
        </TabsContent>
      </Tabs>

      {onComplete && (
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Setup Complete?</p>
                <p className="text-sm text-muted-foreground">
                  Click continue once you've configured permissions
                </p>
              </div>
              <button
                onClick={onComplete}
                className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
              >
                Continue
              </button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

// Export individual guide components for flexible usage
export { MacOSGuide } from "./macos-guide";
export { WindowsGuide } from "./windows-guide";
export { LinuxGuide } from "./linux-guide";

// Re-export the platform hook from its own file (for fast refresh compatibility)
export { usePlatform } from "./use-platform";
