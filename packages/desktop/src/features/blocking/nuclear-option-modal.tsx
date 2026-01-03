// features/blocking/nuclear-option-modal.tsx - Nuclear option activation modal

import { useState } from "react";
import { Bomb, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { toast } from "sonner";
import { useNuclearOptionState, useActivateNuclearOption } from "@/hooks/use-blocking-advanced";

const DURATION_OPTIONS = [
  { value: 5, label: "5 minutes", description: "Quick focus sprint" },
  { value: 10, label: "10 minutes", description: "Short deep work" },
  { value: 15, label: "15 minutes", description: "Standard pomodoro" },
  { value: 30, label: "30 minutes", description: "Extended focus" },
  { value: 60, label: "60 minutes", description: "Maximum lockdown" },
];

export function NuclearOptionModal() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedDuration, setSelectedDuration] = useState(15);
  const { data: nuclearOption } = useNuclearOptionState();
  const activateNuclear = useActivateNuclearOption();

  const handleActivate = async () => {
    try {
      await activateNuclear.mutateAsync({
        durationMinutes: selectedDuration,
      });
      toast.success(`Nuclear option activated for ${selectedDuration} minutes!`, {
        description: "Blocking is now irreversibly locked",
      });
      setIsDialogOpen(false);
    } catch (error) {
      toast.error("Failed to activate nuclear option");
      console.error(error);
    }
  };

  // Calculate progress percentage
  const progressPercentage =
    nuclearOption?.active &&
    typeof nuclearOption?.remainingSeconds === "number" &&
    typeof nuclearOption?.durationMinutes === "number"
      ? ((nuclearOption.durationMinutes * 60 - nuclearOption.remainingSeconds) /
          (nuclearOption.durationMinutes * 60)) *
        100
      : 0;

  // Format remaining time
  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <Bomb
              className={`h-5 w-5 ${nuclearOption?.active ? "text-red-500" : "text-muted-foreground"}`}
            />
            <div>
              <CardTitle className="text-base">Nuclear Option</CardTitle>
              <CardDescription>
                Irreversible time-locked blocking - use with caution
              </CardDescription>
            </div>
          </div>
          {nuclearOption?.active && (
            <Badge variant="destructive" className="animate-pulse">
              ACTIVE
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {nuclearOption?.active ? (
          <>
            {/* Active Nuclear Option Display */}
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>Nuclear Option Active</AlertTitle>
              <AlertDescription>
                Blocking is irreversibly locked for the next{" "}
                {formatTime(nuclearOption.remainingSeconds ?? 0)}
              </AlertDescription>
            </Alert>

            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Time Remaining</span>
                <span className="font-mono font-semibold">
                  {formatTime(nuclearOption.remainingSeconds ?? 0)}
                </span>
              </div>
              <Progress value={progressPercentage} className="h-2" />
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="space-y-1">
                <p className="text-muted-foreground">Started</p>
                <p className="font-medium">
                  {nuclearOption.startedAt
                    ? new Date(nuclearOption.startedAt).toLocaleTimeString()
                    : "-"}
                </p>
              </div>
              <div className="space-y-1">
                <p className="text-muted-foreground">Ends At</p>
                <p className="font-medium">
                  {nuclearOption.endsAt ? new Date(nuclearOption.endsAt).toLocaleTimeString() : "-"}
                </p>
              </div>
            </div>
          </>
        ) : (
          <>
            {/* Inactive - Show Activation Button */}
            <div className="text-sm text-muted-foreground space-y-2">
              <p>The nuclear option provides an irreversible lockdown:</p>
              <ul className="list-disc list-inside space-y-1">
                <li>Cannot be disabled once activated</li>
                <li>All blocked sites/apps remain blocked</li>
                <li>No escape hatch or override</li>
                <li>Automatically expires after duration</li>
              </ul>
            </div>

            <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
              <DialogTrigger asChild>
                <Button variant="destructive" className="w-full">
                  <Bomb className="h-4 w-4 mr-2" />
                  Activate Nuclear Option
                </Button>
              </DialogTrigger>
              <DialogContent className="max-w-md">
                <DialogHeader>
                  <DialogTitle className="flex items-center gap-2">
                    <AlertTriangle className="h-5 w-5 text-destructive" />
                    Activate Nuclear Option
                  </DialogTitle>
                  <DialogDescription>
                    This will lock all blocking irreversibly for the selected duration. There is no
                    way to undo this action once activated.
                  </DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Select Duration</label>
                    <div className="grid grid-cols-1 gap-2">
                      {DURATION_OPTIONS.map((option) => (
                        <button
                          key={option.value}
                          onClick={() => setSelectedDuration(option.value)}
                          className={`flex items-center justify-between p-3 rounded-lg border-2 transition-colors ${
                            selectedDuration === option.value
                              ? "border-destructive bg-destructive/10"
                              : "border-border hover:border-destructive/50"
                          }`}
                        >
                          <div className="text-left">
                            <p className="font-medium">{option.label}</p>
                            <p className="text-sm text-muted-foreground">{option.description}</p>
                          </div>
                          {selectedDuration === option.value && (
                            <div className="h-4 w-4 rounded-full bg-destructive" />
                          )}
                        </button>
                      ))}
                    </div>
                  </div>

                  <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertTitle>Warning</AlertTitle>
                    <AlertDescription>
                      Once activated, this cannot be cancelled. Your device will remain locked for
                      the full {selectedDuration} minute{selectedDuration !== 1 ? "s" : ""}.
                    </AlertDescription>
                  </Alert>
                </div>

                <DialogFooter>
                  <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
                    Cancel
                  </Button>
                  <Button
                    variant="destructive"
                    onClick={handleActivate}
                    disabled={activateNuclear.isPending}
                  >
                    <Bomb className="h-4 w-4 mr-2" />
                    Activate Now
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </>
        )}
      </CardContent>
    </Card>
  );
}
