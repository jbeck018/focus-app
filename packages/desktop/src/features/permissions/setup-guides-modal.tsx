// features/permissions/setup-guides-modal.tsx - Modal wrapper for setup guides

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Shield, X } from "lucide-react";
import { SetupGuides } from "./setup-guides";
import type { Platform } from "./setup-guides";

interface SetupGuidesModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: () => void;
  defaultPlatform?: Platform;
  title?: string;
  description?: string;
}

export function SetupGuidesModal({
  open,
  onOpenChange,
  onComplete,
  defaultPlatform,
  title = "Setup Blocking Permissions",
  description = "Follow these steps to enable FocusFlow's blocking features",
}: SetupGuidesModalProps) {
  const handleComplete = () => {
    if (onComplete) {
      onComplete();
    }
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Shield className="h-6 w-6 text-primary" />
              <div>
                <DialogTitle className="text-xl">{title}</DialogTitle>
                {description && (
                  <DialogDescription className="mt-1">
                    {description}
                  </DialogDescription>
                )}
              </div>
            </div>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => onOpenChange(false)}
              aria-label="Close dialog"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </DialogHeader>

        <div className="mt-4">
          <SetupGuides
            showHeader={false}
            defaultPlatform={defaultPlatform}
            onComplete={handleComplete}
          />
        </div>
      </DialogContent>
    </Dialog>
  );
}

// Standalone modal trigger component
interface SetupGuidesButtonProps {
  variant?: "default" | "outline" | "ghost";
  size?: "default" | "sm" | "lg";
  defaultPlatform?: Platform;
  onComplete?: () => void;
  children?: React.ReactNode;
}

export function SetupGuidesButton({
  variant = "outline",
  size = "default",
  defaultPlatform,
  onComplete,
  children = "Setup Permissions",
}: SetupGuidesButtonProps) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button
        variant={variant}
        size={size}
        onClick={() => setOpen(true)}
        className="gap-2"
      >
        <Shield className="h-4 w-4" />
        {children}
      </Button>

      <SetupGuidesModal
        open={open}
        onOpenChange={setOpen}
        onComplete={onComplete}
        defaultPlatform={defaultPlatform}
      />
    </>
  );
}
