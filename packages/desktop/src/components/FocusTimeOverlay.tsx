// components/FocusTimeOverlay.tsx - Floating overlay for active Focus Time

import { useState } from "react";
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
import { FOCUS_TIME_CATEGORIES, type FocusTimeCategory } from "@focusflow/types";
import { Target, Settings, Clock } from "lucide-react";
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

  const handleToggleCategory = (category: FocusTimeCategory) => {
    const categoryApps = FOCUS_TIME_CATEGORIES[category];
    const allSelected = categoryApps.every((app) => allowed_apps.includes(app));

    if (allSelected) {
      // Remove all apps in category
      categoryApps.forEach((app) => removeApp.mutate(app));
    } else {
      // Add all apps in category
      categoryApps.forEach((app) => {
        if (!allowed_apps.includes(app)) {
          addApp.mutate(app);
        }
      });
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
                <Button variant="outline" size="sm" onClick={() => setShowAppSelector(true)}>
                  <Settings className="h-4 w-4 mr-1" />
                  Modify
                </Button>
                <Button variant="destructive" size="sm" onClick={() => setShowEndConfirm(true)}>
                  End
                </Button>
              </div>
            </div>

            {/* Allowed apps preview */}
            <div className="mt-3 pt-3 border-t border-border/50">
              <p className="text-xs text-muted-foreground mb-1">Allowed apps:</p>
              <div className="flex flex-wrap gap-1">
                {allowed_apps.slice(0, 3).map((app) => (
                  <Badge key={app} variant="secondary" className="text-xs">
                    {app}
                  </Badge>
                ))}
                {allowed_apps.length > 3 && (
                  <Badge variant="secondary" className="text-xs">
                    +{allowed_apps.length - 3} more
                  </Badge>
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
            <DialogTitle>Modify Allowed Apps</DialogTitle>
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
