# FocusFlow Permission System

A comprehensive permission management system for FocusFlow that alerts users when blocking features won't work properly due to missing system permissions.

## Features

- Persistent banner that shows when blocking features are degraded
- Startup modal that checks permissions on app launch
- Platform-specific setup instructions (macOS, Windows, Linux)
- Real-time permission status checking
- "Don't show again" preference (stored in localStorage)
- Clear visual indicators for permission status
- Accessible design with WCAG 2.1 AA compliance

## Architecture

### Components

1. **DegradedModeBanner** - Persistent bottom banner for degraded permissions (NEW)
2. **PermissionModal** - Detailed modal with setup instructions
3. **PermissionStatusProvider** - React context provider that manages permission state
4. **usePermissions** - Custom hook for accessing permission state
5. **types.ts** - TypeScript type definitions

### Permission Types

```typescript
interface PermissionStatus {
  hosts_file_writable: boolean;
  hosts_file_error: string | null;
  process_monitoring_available: boolean;
  process_monitoring_error: string | null;
  overall_status: 'fully_functional' | 'degraded' | 'non_functional';
}
```

## Integration

### Quick Start (Recommended)

Use the pre-built integration component that wires everything together:

```tsx
import { PermissionStatusProvider } from "@/features/permissions";
import { PermissionIntegration } from "@/features/permissions/permission-integration-example";

function App() {
  return (
    <PermissionStatusProvider>
      <YourMainApp />
      {/* Adds both banner and modal with automatic wiring */}
      <PermissionIntegration />
    </PermissionStatusProvider>
  );
}
```

### Manual Integration

For more control, integrate the components manually:

#### Step 1: Wrap App with Provider

In your `App.tsx`:

```tsx
import { useState } from "react";
import {
  PermissionStatusProvider,
  PermissionModal,
  DegradedModeBanner
} from "@/features/permissions";

function App() {
  const [showPermissionModal, setShowPermissionModal] = useState(false);

  return (
    <PermissionStatusProvider>
      <AchievementCelebrationProvider>
        <SidebarProvider defaultOpen={true}>
          {/* Your app content */}

          {/* Persistent banner at bottom */}
          <DegradedModeBanner onFixClick={() => setShowPermissionModal(true)} />

          {/* Detailed permission modal */}
          <PermissionModal
            open={showPermissionModal}
            onOpenChange={setShowPermissionModal}
          />

          <Toaster position="bottom-right" />
        </SidebarProvider>
      </AchievementCelebrationProvider>
    </PermissionStatusProvider>
  );
}
```

#### Step 2: Use Permission Status in Components

```tsx
import { usePermissions } from "@/features/permissions";

function BlockingSettings() {
  const { permissionStatus, hasFullPermissions, isDegraded, recheckPermissions } = usePermissions();

  if (isDegraded) {
    return (
      <Alert variant="destructive">
        <AlertTriangle className="h-4 w-4" />
        <AlertTitle>Limited Functionality</AlertTitle>
        <AlertDescription>
          Some blocking features are unavailable due to missing permissions.
          <Button onClick={recheckPermissions} size="sm" className="mt-2">
            Check Permissions
          </Button>
        </AlertDescription>
      </Alert>
    );
  }

  // Render full settings...
}
```

## Backend Requirements

The system expects a Tauri command named `check_permissions` that returns:

```rust
#[tauri::command]
fn check_permissions() -> Result<PermissionStatus, String> {
    // Check hosts file write access
    let hosts_file_writable = can_write_to_hosts_file();
    let hosts_file_error = if !hosts_file_writable {
        Some("Permission denied writing to /etc/hosts".to_string())
    } else {
        None
    };

    // Check process monitoring
    let process_monitoring_available = can_monitor_processes();
    let process_monitoring_error = if !process_monitoring_available {
        Some("Unable to access process list".to_string())
    } else {
        None
    };

    // Determine overall status
    let overall_status = if hosts_file_writable && process_monitoring_available {
        "fully_functional"
    } else if hosts_file_writable || process_monitoring_available {
        "degraded"
    } else {
        "non_functional"
    };

    Ok(PermissionStatus {
        hosts_file_writable,
        hosts_file_error,
        process_monitoring_available,
        process_monitoring_error,
        overall_status: overall_status.to_string(),
    })
}
```

## User Flow

### New User Flow (with DegradedModeBanner)

1. **App Startup**: Provider calls `check_permissions` command
2. **Permission Check**: Backend validates system permissions
3. **Banner Display**: If permissions are degraded:
   - Persistent banner appears at bottom of screen
   - Shows what's not working (e.g., "website blocking")
   - Color-coded by severity (amber for degraded, red for non-functional)
   - "Fix This" button prominently displayed
4. **User Clicks "Fix This"**:
   - Opens PermissionModal with detailed instructions
   - Platform-specific setup guide shown
   - User can check permissions again after fixing
5. **Permission Fixed**:
   - Banner automatically disappears
   - Full functionality restored

### Legacy Flow (PermissionModal only)

1. **App Startup**: Provider calls `check_permissions` command
2. **Permission Check**: Backend validates system permissions
3. **Modal Display**: If permissions are missing and user hasn't dismissed:
   - Shows permission status with visual indicators
   - Displays platform-specific instructions
   - Offers actions: "Open Guide", "Check Again", "Continue Anyway"
4. **User Actions**:
   - Can check "Don't show again" to suppress modal
   - Can recheck permissions after granting access
   - Can continue with limited functionality

## Styling

### DegradedModeBanner

The banner uses modern design patterns:

- **Position**: Fixed to bottom of viewport (bottom-4, left-4, right-4)
- **Background**: Semi-transparent with backdrop blur effect
- **Colors**:
  - Degraded: Amber/yellow tones (`amber-500/10` background, `amber-500/50` border)
  - Non-functional: Red/destructive tones (`destructive/10` background, `destructive/50` border)
- **Icons**: `AlertTriangle` for degraded, `XCircle` for non-functional
- **Animation**: Smooth slide-up on mount (translate-y with opacity fade)
- **Max Width**: 2xl (prevents banner from being too wide on large screens)
- **Dark Mode**: Automatically adjusts with Tailwind dark mode
- **Accessibility**: ARIA live region for screen reader announcements

### PermissionModal

The modal uses shadcn/ui components and Tailwind CSS:

- Red X icons for missing permissions
- Green check icons for granted permissions
- Amber warning for degraded state
- Destructive variant for non-functional state
- Responsive design with mobile support
- Dark mode compatible

## Accessibility

- WCAG 2.1 Level AA compliant
- Keyboard navigation support
- Screen reader friendly
- Focus trap in modal
- Clear ARIA labels
- Semantic HTML structure

## LocalStorage Keys

- `focusflow_dont_show_permission_modal` - Boolean flag for "Don't show again"

## Future Enhancements

- [ ] Add permission request flows (where supported)
- [ ] Implement progressive permission requests
- [ ] Add telemetry for permission denial rates
- [ ] Support for additional platform-specific permissions
- [ ] In-app permission testing tools
- [ ] Automatic retry with exponential backoff
- [ ] Permission status indicator in sidebar
