// features/blocking/strict-mode-toggle.tsx - Strict mode toggle component

import { Lock, LockOpen, AlertTriangle } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";
import {
  useStrictModeState,
  useEnableStrictMode,
  useDisableStrictMode,
} from "@/hooks/use-blocking-advanced";
import { useActiveSession } from "@/hooks/useTauriCommands";

export function StrictModeToggle() {
  const { data: strictMode } = useStrictModeState();
  const { data: activeSession } = useActiveSession();
  const enableStrictMode = useEnableStrictMode();
  const disableStrictMode = useDisableStrictMode();

  const handleToggle = async (checked: boolean) => {
    if (checked) {
      // Enable strict mode
      if (!activeSession) {
        toast.error("Start a focus session first to enable strict mode");
        return;
      }

      try {
        await enableStrictMode.mutateAsync(activeSession.id);
        toast.success("Strict mode enabled - blocking cannot be disabled until session ends");
      } catch (error) {
        toast.error("Failed to enable strict mode");
        console.error(error);
      }
    } else {
      // Disable strict mode
      if (!strictMode?.canDisable) {
        toast.error("Cannot disable strict mode while session is active");
        return;
      }

      try {
        await disableStrictMode.mutateAsync();
        toast.success("Strict mode disabled");
      } catch (error) {
        toast.error("Failed to disable strict mode");
        console.error(error);
      }
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            {strictMode?.enabled ? (
              <Lock className="h-5 w-5 text-amber-500" />
            ) : (
              <LockOpen className="h-5 w-5 text-muted-foreground" />
            )}
            <div>
              <CardTitle className="text-base">Strict Mode</CardTitle>
              <CardDescription>
                Lock blocking until session ends - no way to disable
              </CardDescription>
            </div>
          </div>
          <Switch
            checked={strictMode?.enabled ?? false}
            onCheckedChange={handleToggle}
            disabled={strictMode?.enabled && !strictMode.canDisable}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {strictMode?.enabled ? (
          <Alert variant="default" className="border-amber-500/50 bg-amber-500/10">
            <AlertTriangle className="h-4 w-4 text-amber-500" />
            <AlertTitle>Strict Mode Active</AlertTitle>
            <AlertDescription className="space-y-2">
              <p className="text-sm">
                Blocking is locked and cannot be disabled until your current session ends.
              </p>
              {activeSession && (
                <div className="flex items-center gap-2">
                  <Badge variant="secondary" className="text-xs">
                    Session: {activeSession.id.slice(0, 8)}...
                  </Badge>
                  <span className="text-xs text-muted-foreground">
                    {activeSession.plannedDurationMinutes} minutes
                  </span>
                </div>
              )}
            </AlertDescription>
          </Alert>
        ) : (
          <div className="text-sm text-muted-foreground">
            <p>When enabled during a focus session:</p>
            <ul className="list-disc list-inside mt-2 space-y-1">
              <li>Blocking cannot be disabled manually</li>
              <li>All blocked sites and apps remain blocked</li>
              <li>Only unlocks when the session ends</li>
              <li>Perfect for maintaining deep focus</li>
            </ul>
          </div>
        )}

        {!activeSession && !strictMode?.enabled && (
          <Alert>
            <AlertDescription className="text-sm">
              Start a focus session to enable strict mode
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
}
