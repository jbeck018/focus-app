// components/FocusTimeOverlay.tsx - Floating overlay for active Focus Time

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useFocusTimeState, useFocusTimeActions } from "@/hooks/useFocusTime";
import { AppSelector } from "./AppSelector";
import { formatTime } from "@/hooks/useTimer";
import { Target, Settings, Clock, HelpCircle } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";

export function FocusTimeOverlay() {
  const { data: state, isLoading } = useFocusTimeState();
  const { addApp, removeApp, endEarly } = useFocusTimeActions();
  const [showAppSelector, setShowAppSelector] = useState(false);
  const [showEndConfirm, setShowEndConfirm] = useState(false);

  if (isLoading || !state?.active || !state.current_event) {
    return null;
  }

  const { allowed_apps, remaining_seconds } = state;

  const handleToggleApp = (appName: string) => {
    if (allowed_apps.includes(appName)) {
      removeApp.mutate(appName);
    } else {
      addApp.mutate(appName);
    }
  };

  // Handle category toggle using backend expansion
  const handleToggleCategory = async (categoryId: string) => {
    try {
      // Expand category to get all apps using backend command
      const expandedApps = await invoke<string[]>("expand_focus_time_categories", {
        items: [categoryId],
      });

      // Check if all apps in the category are already selected
      const allSelected = expandedApps.every((app) => allowed_apps.includes(app));

      if (allSelected) {
        // Remove all apps in category
        for (const app of expandedApps) {
          removeApp.mutate(app);
        }
      } else {
        // Add all apps in category that aren't already selected
        for (const app of expandedApps) {
          if (!allowed_apps.includes(app)) {
            addApp.mutate(app);
          }
        }
      }
    } catch (error) {
      console.error("Failed to expand category:", error);
    }
  };

  const handleEndEarly = () => {
    endEarly.mutate("Ended by user");
    setShowEndConfirm(false);
  };

  return (
    <>
      {/* Floating overlay */}
      <div className="fixed top-4 right-4 z-50">
        <Card className="border-green-500/50 bg-green-500/10 backdrop-blur-sm shadow-lg">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <Target className="h-5 w-5 text-green-500" />
                <div>
                  <p className="text-sm font-medium">Focus Time Active</p>
                  <div className="flex items-center gap-2 mt-1">
                    <Clock className="h-3 w-3 text-muted-foreground" />
                    <p className="text-xs text-muted-foreground">
                      {formatTime(remaining_seconds)} remaining
                    </p>
                  </div>
                </div>
              </div>
              <div className="flex gap-2">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="outline" size="sm" onClick={() => setShowAppSelector(true)}>
                      <Settings className="h-4 w-4 mr-1" />
                      Modify
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent className="max-w-xs">
                    Add or remove apps from your allowed list without ending Focus Time
                  </TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="destructive" size="sm" onClick={() => setShowEndConfirm(true)}>
                      End
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent className="max-w-xs">
                    End Focus Time immediately. You can always restart from a scheduled event.
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>

            {/* Allowed apps preview */}
            <div className="mt-3 pt-3 border-t border-border/50">
              <p className="text-xs text-muted-foreground mb-1 flex items-center gap-1">
                Allowed apps:
                <Tooltip>
                  <TooltipTrigger asChild>
                    <HelpCircle className="h-3 w-3 cursor-help" />
                  </TooltipTrigger>
                  <TooltipContent className="max-w-xs">
                    These apps are currently accessible. All other apps are blocked during this
                    Focus Time session.
                  </TooltipContent>
                </Tooltip>
              </p>
              <div className="flex flex-wrap gap-1">
                {allowed_apps.slice(0, 3).map((app) => (
                  <Badge key={app} variant="secondary" className="text-xs">
                    {app}
                  </Badge>
                ))}
                {allowed_apps.length > 3 && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Badge variant="secondary" className="text-xs cursor-help">
                        +{allowed_apps.length - 3} more
                      </Badge>
                    </TooltipTrigger>
                    <TooltipContent className="max-w-xs">
                      <div className="space-y-1">
                        <p className="font-medium">All allowed apps:</p>
                        <p className="text-xs">{allowed_apps.join(", ")}</p>
                      </div>
                    </TooltipContent>
                  </Tooltip>
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* App Selector Dialog */}
      <Dialog open={showAppSelector} onOpenChange={setShowAppSelector}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              Modify Allowed Apps
              <Tooltip>
                <TooltipTrigger asChild>
                  <HelpCircle className="h-4 w-4 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent className="max-w-xs">
                  Keyboard shortcut: Press Esc to close this dialog
                </TooltipContent>
              </Tooltip>
            </DialogTitle>
            <DialogDescription>
              Add or remove apps for this Focus Time session. Changes apply immediately.
            </DialogDescription>
          </DialogHeader>
          <AppSelector
            selectedApps={allowed_apps}
            onToggleApp={handleToggleApp}
            onToggleCategory={handleToggleCategory}
            onClose={() => setShowAppSelector(false)}
          />
        </DialogContent>
      </Dialog>

      {/* End Early Confirmation */}
      <AlertDialog open={showEndConfirm} onOpenChange={setShowEndConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>End Focus Time Early?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to end this Focus Time session? You still have{" "}
              {formatTime(remaining_seconds)} remaining.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleEndEarly}>End Session</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
