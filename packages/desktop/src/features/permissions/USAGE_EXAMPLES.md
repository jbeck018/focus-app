# DegradedModeBanner Usage Examples

Complete examples showing how to use the DegradedModeBanner in different scenarios.

## Example 1: Basic Integration (Recommended)

The simplest way to add the banner to your app:

```tsx
// App.tsx
import { PermissionStatusProvider, PermissionIntegration } from "@/features/permissions";

function App() {
  return (
    <PermissionStatusProvider>
      <YourMainApp />

      {/* This adds both the banner and modal, pre-wired together */}
      <PermissionIntegration />
    </PermissionStatusProvider>
  );
}

export default App;
```

**What you get:**
- Persistent banner at bottom when permissions are degraded
- Clicking "Fix This" opens the permission modal
- Modal closes when user is done
- Banner disappears when permissions are fixed

---

## Example 2: Manual Integration with Custom Handler

If you need more control over the modal state:

```tsx
// App.tsx
import { useState } from "react";
import {
  PermissionStatusProvider,
  DegradedModeBanner,
  PermissionModal
} from "@/features/permissions";

function App() {
  const [showModal, setShowModal] = useState(false);

  const handleFixClick = () => {
    console.log("User clicked Fix This");
    // You could trigger analytics here
    setShowModal(true);
  };

  return (
    <PermissionStatusProvider>
      <YourMainApp />

      <DegradedModeBanner onFixClick={handleFixClick} />

      <PermissionModal
        open={showModal}
        onOpenChange={setShowModal}
      />
    </PermissionStatusProvider>
  );
}

export default App;
```

---

## Example 3: Banner Only (No Modal)

If you want to handle the fix action yourself:

```tsx
// App.tsx
import { PermissionStatusProvider, DegradedModeBanner } from "@/features/permissions";
import { useNavigate } from "react-router-dom";

function App() {
  const navigate = useNavigate();

  const handleFixClick = () => {
    // Navigate to a custom settings page
    navigate("/settings/permissions");
  };

  return (
    <PermissionStatusProvider>
      <YourMainApp />

      <DegradedModeBanner onFixClick={handleFixClick} />
    </PermissionStatusProvider>
  );
}

export default App;
```

---

## Example 4: Using Permission Status in Components

Show different UI based on permission status:

```tsx
// BlockingSettings.tsx
import { usePermissions } from "@/features/permissions";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AlertTriangle, CheckCircle } from "lucide-react";

export function BlockingSettings() {
  const { permissionStatus, hasFullPermissions, isDegraded } = usePermissions();

  if (!permissionStatus) {
    return <div>Loading permissions...</div>;
  }

  return (
    <div className="space-y-4">
      <h1>Blocking Settings</h1>

      {/* Show status alert */}
      {hasFullPermissions && (
        <Alert>
          <CheckCircle className="h-4 w-4" />
          <AlertTitle>All Systems Operational</AlertTitle>
          <AlertDescription>
            All blocking features are available and functioning properly.
          </AlertDescription>
        </Alert>
      )}

      {isDegraded && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle>Limited Functionality</AlertTitle>
          <AlertDescription>
            Some blocking features are unavailable. Check the banner at the bottom for details.
          </AlertDescription>
        </Alert>
      )}

      {/* Conditionally show/disable features */}
      <div>
        <h2>Website Blocking</h2>
        {permissionStatus.hosts_file_writable ? (
          <WebsiteBlockingControls />
        ) : (
          <p className="text-muted-foreground">
            Website blocking is unavailable. Missing hosts file permissions.
          </p>
        )}
      </div>

      <div>
        <h2>App Blocking</h2>
        {permissionStatus.process_monitoring_available ? (
          <AppBlockingControls />
        ) : (
          <p className="text-muted-foreground">
            App blocking is unavailable. Missing process monitoring permissions.
          </p>
        )}
      </div>
    </div>
  );
}
```

---

## Example 5: Conditional Rendering Based on Status

Only show certain features when permissions are available:

```tsx
// Dashboard.tsx
import { usePermissions } from "@/features/permissions";
import { Button } from "@/components/ui/button";

export function Dashboard() {
  const { permissionStatus, hasFullPermissions } = usePermissions();

  const handleStartBlockingSession = () => {
    if (!hasFullPermissions) {
      alert("Cannot start blocking session without full permissions");
      return;
    }

    // Start blocking session
    startSession();
  };

  return (
    <div>
      <h1>Focus Dashboard</h1>

      <Button
        onClick={handleStartBlockingSession}
        disabled={!hasFullPermissions}
      >
        Start Blocking Session
      </Button>

      {!hasFullPermissions && (
        <p className="text-sm text-muted-foreground mt-2">
          Full permissions required. See banner below for details.
        </p>
      )}
    </div>
  );
}
```

---

## Example 6: Analytics Integration

Track when users see or interact with the banner:

```tsx
// App.tsx
import { useState, useEffect } from "react";
import {
  PermissionStatusProvider,
  DegradedModeBanner,
  PermissionModal,
  usePermissions
} from "@/features/permissions";
import { analytics } from "@/lib/analytics";

function AppContent() {
  const { isDegraded, permissionStatus } = usePermissions();
  const [showModal, setShowModal] = useState(false);

  // Track when banner is shown
  useEffect(() => {
    if (isDegraded) {
      analytics.track("degraded_mode_banner_shown", {
        overall_status: permissionStatus?.overall_status,
        missing_hosts_file: !permissionStatus?.hosts_file_writable,
        missing_process_monitoring: !permissionStatus?.process_monitoring_available,
      });
    }
  }, [isDegraded, permissionStatus]);

  const handleFixClick = () => {
    analytics.track("degraded_mode_banner_fix_clicked", {
      overall_status: permissionStatus?.overall_status,
    });
    setShowModal(true);
  };

  return (
    <>
      <YourMainApp />
      <DegradedModeBanner onFixClick={handleFixClick} />
      <PermissionModal open={showModal} onOpenChange={setShowModal} />
    </>
  );
}

function App() {
  return (
    <PermissionStatusProvider>
      <AppContent />
    </PermissionStatusProvider>
  );
}

export default App;
```

---

## Example 7: Custom Styling

Customize the banner appearance (advanced):

```tsx
// CustomDegradedBanner.tsx
import { useState, useEffect } from "react";
import { AlertTriangle, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import { usePermissions } from "@/features/permissions";
import { cn } from "@/lib/utils";

export function CustomDegradedBanner({ onFixClick }: { onFixClick?: () => void }) {
  const { permissionStatus, isDegraded } = usePermissions();
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    if (isDegraded) {
      const timer = setTimeout(() => setIsVisible(true), 300);
      return () => clearTimeout(timer);
    } else {
      setIsVisible(false);
    }
  }, [isDegraded]);

  if (!permissionStatus || !isDegraded) {
    return null;
  }

  return (
    <div
      className={cn(
        "fixed top-4 right-4 z-50 transition-all duration-500",
        isVisible ? "translate-x-0 opacity-100" : "translate-x-full opacity-0"
      )}
    >
      <div className="bg-card border rounded-lg shadow-lg p-4 max-w-sm">
        <div className="flex items-start gap-3">
          <AlertTriangle className="h-5 w-5 text-amber-500 shrink-0 mt-0.5" />
          <div className="flex-1 space-y-2">
            <p className="font-semibold text-sm">Limited Functionality</p>
            <p className="text-sm text-muted-foreground">
              Some features require additional permissions.
            </p>
            <Button size="sm" onClick={onFixClick} className="w-full">
              <Settings className="h-4 w-4 mr-2" />
              Configure Permissions
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
```

---

## Example 8: Testing Different States

Useful for development and testing:

```tsx
// PermissionTestPage.tsx
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { DegradedModeBanner } from "@/features/permissions";

// Mock permission context for testing
const mockPermissionContext = {
  permissionStatus: {
    hosts_file_writable: false,
    hosts_file_error: "Permission denied",
    process_monitoring_available: true,
    process_monitoring_error: null,
    overall_status: "degraded" as const,
  },
  isLoading: false,
  hasFullPermissions: false,
  isDegraded: true,
  recheckPermissions: async () => {},
};

export function PermissionTestPage() {
  const [status, setStatus] = useState<"fully_functional" | "degraded" | "non_functional">("degraded");

  return (
    <div className="p-8 space-y-4">
      <h1>Permission Banner Testing</h1>

      <div className="flex gap-2">
        <Button onClick={() => setStatus("fully_functional")}>
          Fully Functional
        </Button>
        <Button onClick={() => setStatus("degraded")}>
          Degraded
        </Button>
        <Button onClick={() => setStatus("non_functional")}>
          Non-Functional
        </Button>
      </div>

      <div className="mt-8">
        <p>Current Status: {status}</p>
        <p className="text-sm text-muted-foreground">
          Banner should {status === "fully_functional" ? "not be visible" : "be visible"}
        </p>
      </div>

      {/* Would need to wrap with mock provider in real implementation */}
      <DegradedModeBanner onFixClick={() => console.log("Fix clicked")} />
    </div>
  );
}
```

---

## Best Practices

### DO:
- Always wrap your app with `PermissionStatusProvider`
- Use `PermissionIntegration` for quick setup
- Check `hasFullPermissions` before enabling critical features
- Provide clear feedback when features are unavailable
- Track banner interactions for product insights

### DON'T:
- Don't allow critical blocking operations without permissions
- Don't hide the banner (it's meant to be persistent)
- Don't forget to handle the `onFixClick` callback
- Don't assume permissions are available without checking

---

## Troubleshooting

### Banner doesn't appear

1. Check that `PermissionStatusProvider` wraps your app
2. Verify the backend `check_permissions` command is working
3. Check console for errors
4. Ensure `overall_status` is 'degraded' or 'non_functional'

### Banner appears but "Fix This" doesn't work

1. Verify `onFixClick` callback is provided
2. Check that `PermissionModal` is rendered
3. Ensure modal's `open` state is wired correctly

### Permissions check fails

1. Check Tauri command is registered in backend
2. Verify backend has necessary system permissions
3. Check network/IPC logs for errors
4. Test with mock data first
