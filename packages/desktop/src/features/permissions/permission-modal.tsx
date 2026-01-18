// features/permissions/permission-modal.tsx - Startup permission warning modal

import { useState, useEffect } from "react";
import { AlertTriangle, CheckCircle2, XCircle, ExternalLink, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { usePermissions } from "./use-permissions";
import { open } from "@tauri-apps/plugin-shell";

const DONT_SHOW_KEY = "focusflow_dont_show_permission_modal";

interface PlatformInstructionsProps {
  platform: "macos" | "windows" | "linux";
  missingHostsFile: boolean;
  missingProcessMonitoring: boolean;
}

function PlatformInstructions({
  platform,
  missingHostsFile,
  missingProcessMonitoring,
}: PlatformInstructionsProps) {
  if (platform === "macos") {
    return (
      <div className="space-y-3">
        <h4 className="font-medium text-sm">macOS Setup Instructions:</h4>
        {missingHostsFile && (
          <div className="space-y-2">
            <p className="text-sm font-medium text-muted-foreground">Hosts File Access:</p>
            <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground pl-2">
              <li>Open Terminal</li>
              <li>
                Run:{" "}
                <code className="bg-muted px-1.5 py-0.5 rounded">sudo chmod 644 /etc/hosts</code>
              </li>
              <li>Grant permission when prompted</li>
              <li>Restart FocusFlow</li>
            </ol>
          </div>
        )}
        {missingProcessMonitoring && (
          <div className="space-y-2">
            <p className="text-sm font-medium text-muted-foreground">Process Monitoring:</p>
            <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground pl-2">
              <li>Open System Preferences</li>
              <li>Go to Security & Privacy</li>
              <li>Select the Privacy tab</li>
              <li>Click Accessibility</li>
              <li>Add FocusFlow and enable it</li>
            </ol>
          </div>
        )}
      </div>
    );
  }

  if (platform === "windows") {
    return (
      <div className="space-y-3">
        <h4 className="font-medium text-sm">Windows Setup Instructions:</h4>
        {missingHostsFile && (
          <div className="space-y-2">
            <p className="text-sm font-medium text-muted-foreground">Hosts File Access:</p>
            <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground pl-2">
              <li>Right-click FocusFlow</li>
              <li>Select "Run as Administrator"</li>
              <li>Grant UAC permission when prompted</li>
            </ol>
          </div>
        )}
        {missingProcessMonitoring && (
          <div className="space-y-2">
            <p className="text-sm font-medium text-muted-foreground">Process Monitoring:</p>
            <p className="text-sm text-muted-foreground pl-2">
              Windows Defender may block process monitoring. Add FocusFlow to your antivirus
              exceptions.
            </p>
          </div>
        )}
      </div>
    );
  }

  // Linux
  return (
    <div className="space-y-3">
      <h4 className="font-medium text-sm">Linux Setup Instructions:</h4>
      {missingHostsFile && (
        <div className="space-y-2">
          <p className="text-sm font-medium text-muted-foreground">Hosts File Access:</p>
          <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground pl-2">
            <li>Open Terminal</li>
            <li>
              Run: <code className="bg-muted px-1.5 py-0.5 rounded">sudo chmod 644 /etc/hosts</code>
            </li>
            <li>Enter your password when prompted</li>
            <li>Restart FocusFlow</li>
          </ol>
        </div>
      )}
      {missingProcessMonitoring && (
        <div className="space-y-2">
          <p className="text-sm font-medium text-muted-foreground">Process Monitoring:</p>
          <p className="text-sm text-muted-foreground pl-2">
            Ensure FocusFlow has access to read /proc filesystem
          </p>
        </div>
      )}
    </div>
  );
}

interface PermissionModalProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function PermissionModal({ open: controlledOpen, onOpenChange }: PermissionModalProps = {}) {
  const { permissionStatus, isDegraded, recheckPermissions } = usePermissions();
  const [internalOpen, setInternalOpen] = useState(false);
  const [dontShowAgain, setDontShowAgain] = useState(false);
  const [isRechecking, setIsRechecking] = useState(false);

  // Use controlled or internal state
  const isOpen = controlledOpen ?? internalOpen;
  const setIsOpen = onOpenChange ?? setInternalOpen;

  // Detect platform
  const [platform, setPlatform] = useState<"macos" | "windows" | "linux">("macos");

  useEffect(() => {
    const userAgent = navigator.userAgent.toLowerCase();
    if (userAgent.includes("win")) {
      setPlatform("windows");
    } else if (userAgent.includes("linux")) {
      setPlatform("linux");
    } else {
      setPlatform("macos");
    }
  }, []);

  // Show modal on startup if permissions are missing and user hasn't dismissed (only in uncontrolled mode)
  useEffect(() => {
    if (controlledOpen !== undefined) return; // Skip auto-show in controlled mode
    if (!permissionStatus) return;

    const shouldShow = localStorage.getItem(DONT_SHOW_KEY) !== "true";
    if (isDegraded && shouldShow) {
      setInternalOpen(true);
    }
  }, [permissionStatus, isDegraded, controlledOpen]);

  const handleClose = () => {
    if (dontShowAgain) {
      localStorage.setItem(DONT_SHOW_KEY, "true");
    }
    setIsOpen(false);
  };

  const handleCheckAgain = async () => {
    setIsRechecking(true);
    try {
      await recheckPermissions();
    } finally {
      setIsRechecking(false);
    }
  };

  const handleOpenGuide = () => {
    // Open documentation URL
    open("https://focusflow.app/docs/permissions");
  };

  if (!permissionStatus) return null;

  const missingHostsFile = !permissionStatus.hosts_file_writable;
  const missingProcessMonitoring = !permissionStatus.process_monitoring_available;

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-amber-500" />
            Permission Required for Full Functionality
          </DialogTitle>
          <DialogDescription>
            Some FocusFlow features require additional system permissions to work properly.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Status Overview */}
          <Alert
            variant={
              permissionStatus.overall_status === "non_functional" ? "destructive" : "default"
            }
          >
            <AlertTriangle className="h-4 w-4" />
            <AlertTitle>
              Current Status: {permissionStatus.overall_status.replace("_", " ").toUpperCase()}
            </AlertTitle>
            <AlertDescription>
              {permissionStatus.overall_status === "degraded" &&
                "Some blocking features are limited or unavailable."}
              {permissionStatus.overall_status === "non_functional" &&
                "Blocking features are completely unavailable."}
            </AlertDescription>
          </Alert>

          {/* Permission Status List */}
          <div className="space-y-3">
            <h3 className="font-medium text-sm">Feature Status:</h3>

            {/* Hosts File Permission */}
            <div className="flex items-start gap-3 p-3 rounded-lg border">
              {missingHostsFile ? (
                <XCircle className="h-5 w-5 text-destructive shrink-0 mt-0.5" />
              ) : (
                <CheckCircle2 className="h-5 w-5 text-green-600 shrink-0 mt-0.5" />
              )}
              <div className="flex-1 space-y-1">
                <p className="font-medium text-sm">Website Blocking (DNS Level)</p>
                <p className="text-sm text-muted-foreground">
                  {missingHostsFile
                    ? "Unavailable - Cannot modify hosts file"
                    : "Available - Full DNS blocking enabled"}
                </p>
                {permissionStatus.hosts_file_error && (
                  <p className="text-xs text-destructive mt-1">
                    Error: {permissionStatus.hosts_file_error}
                  </p>
                )}
              </div>
            </div>

            {/* Process Monitoring Permission */}
            <div className="flex items-start gap-3 p-3 rounded-lg border">
              {missingProcessMonitoring ? (
                <XCircle className="h-5 w-5 text-destructive shrink-0 mt-0.5" />
              ) : (
                <CheckCircle2 className="h-5 w-5 text-green-600 shrink-0 mt-0.5" />
              )}
              <div className="flex-1 space-y-1">
                <p className="font-medium text-sm">Application Blocking</p>
                <p className="text-sm text-muted-foreground">
                  {missingProcessMonitoring
                    ? "Unavailable - Cannot monitor running applications"
                    : "Available - Full app blocking enabled"}
                </p>
                {permissionStatus.process_monitoring_error && (
                  <p className="text-xs text-destructive mt-1">
                    Error: {permissionStatus.process_monitoring_error}
                  </p>
                )}
              </div>
            </div>
          </div>

          <Separator />

          {/* Platform-specific instructions */}
          {(missingHostsFile || missingProcessMonitoring) && (
            <PlatformInstructions
              platform={platform}
              missingHostsFile={missingHostsFile}
              missingProcessMonitoring={missingProcessMonitoring}
            />
          )}

          <Separator />

          {/* Don't show again checkbox */}
          <div className="flex items-center space-x-2">
            <Checkbox
              id="dont-show"
              checked={dontShowAgain}
              onCheckedChange={(checked: boolean | "indeterminate") =>
                setDontShowAgain(checked === true)
              }
            />
            <Label htmlFor="dont-show" className="text-sm text-muted-foreground cursor-pointer">
              Don't show this warning again
            </Label>
          </div>
        </div>

        <DialogFooter className="flex-col sm:flex-row gap-2">
          <Button variant="outline" onClick={handleOpenGuide} className="w-full sm:w-auto">
            <ExternalLink className="h-4 w-4 mr-2" />
            Open Setup Guide
          </Button>
          <div className="flex gap-2 flex-1 sm:flex-initial">
            <Button
              variant="outline"
              onClick={handleCheckAgain}
              disabled={isRechecking}
              className="flex-1 sm:flex-initial"
            >
              <RefreshCw className={`h-4 w-4 mr-2 ${isRechecking ? "animate-spin" : ""}`} />
              Check Again
            </Button>
            <Button onClick={handleClose} className="flex-1 sm:flex-initial">
              Continue Anyway
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
