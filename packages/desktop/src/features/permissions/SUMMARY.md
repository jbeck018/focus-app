# FocusFlow Permission System - Complete Implementation

This directory contains a comprehensive permission management system for FocusFlow that alerts users when blocking features won't work properly due to missing system permissions.

## 📁 File Structure

```
packages/desktop/src/features/permissions/
├── types.ts                              # TypeScript type definitions
├── permission-status-context.tsx         # React context provider for permission state
├── use-permissions.ts                    # Custom hook for accessing permissions
├── permission-modal.tsx                  # Startup modal for permission warnings
├── degraded-mode-banner.tsx             # Persistent banner for degraded state
├── permission-integration-example.tsx    # Example integration component
├── index.ts                             # Main export file
├── README.md                            # Complete documentation
├── INTEGRATION_EXAMPLE.tsx              # Full App.tsx integration example
├── BACKEND_EXAMPLE.rs                   # Rust backend implementation example
├── setup-guides/
│   ├── macos-guide.tsx                  # Detailed macOS setup guide
│   └── windows-guide.tsx                # Detailed Windows setup guide
└── SUMMARY.md                           # This file
```

## 🎯 Core Components

### 1. **PermissionStatusProvider** (`permission-status-context.tsx`)
- React context provider that manages global permission state
- Automatically checks permissions on mount via `check_permissions` command
- Provides `recheckPermissions()` function for manual checks
- Exposes permission status to all child components

**Usage:**
```tsx
import { PermissionStatusProvider } from "@/features/permissions";

<PermissionStatusProvider>
  <YourApp />
</PermissionStatusProvider>
```

### 2. **usePermissions** Hook (`use-permissions.ts`)
- Custom hook for accessing permission state
- Returns: `permissionStatus`, `isLoading`, `hasFullPermissions`, `isDegraded`, `recheckPermissions`

**Usage:**
```tsx
import { usePermissions } from "@/features/permissions";

const { permissionStatus, isDegraded, recheckPermissions } = usePermissions();
```

### 3. **PermissionModal** (`permission-modal.tsx`)
- Shows on app startup if permissions are missing
- Can be controlled or uncontrolled (auto-shows by default)
- Platform-specific instructions (macOS, Windows, Linux)
- "Don't show again" preference stored in localStorage
- Actions: "Open Guide", "Check Again", "Continue Anyway"

**Features:**
- Red X icons for missing permissions
- Green check icons for granted permissions
- Platform detection and specific instructions
- Responsive design with mobile support
- WCAG 2.1 AA accessible

**Usage:**
```tsx
// Auto-show on startup (uncontrolled)
<PermissionModal />

// Controlled mode
<PermissionModal open={isOpen} onOpenChange={setIsOpen} />
```

### 4. **DegradedModeBanner** (`degraded-mode-banner.tsx`)
- Persistent banner shown at bottom of screen when permissions are degraded
- Animates in/out smoothly
- Shows missing features clearly
- "Fix This" button to open permission modal
- Color-coded: amber for degraded, red for non-functional

**Usage:**
```tsx
<DegradedModeBanner onFixClick={() => setShowModal(true)} />
```

### 5. **PermissionIntegration** (`permission-integration-example.tsx`)
- Complete integration component combining banner + modal
- Drop-in solution for adding permission UI

**Usage:**
```tsx
import { PermissionIntegration } from "@/features/permissions/permission-integration-example";

<PermissionStatusProvider>
  <YourApp />
  <PermissionIntegration />
</PermissionStatusProvider>
```

## 📊 Type Definitions

### PermissionStatus
```typescript
interface PermissionStatus {
  hosts_file_writable: boolean;              // Can write to /etc/hosts or Windows equivalent
  hosts_file_error: string | null;           // Error message if hosts file check failed
  process_monitoring_available: boolean;     // Can monitor running processes
  process_monitoring_error: string | null;   // Error message if process monitoring check failed
  overall_status: 'fully_functional' | 'degraded' | 'non_functional';
}
```

### PermissionContextValue
```typescript
interface PermissionContextValue {
  permissionStatus: PermissionStatus | null;
  isLoading: boolean;
  hasFullPermissions: boolean;              // overall_status === 'fully_functional'
  isDegraded: boolean;                      // overall_status === 'degraded' || 'non_functional'
  recheckPermissions: () => Promise<void>;
}
```

## 🔧 Backend Integration

The system requires a Tauri command named `check_permissions`:

```rust
#[tauri::command]
fn check_permissions() -> Result<PermissionStatus, String> {
    // Implementation provided in BACKEND_EXAMPLE.rs
}
```

See `BACKEND_EXAMPLE.rs` for complete Rust implementation with:
- Hosts file write access checking
- Process monitoring capability detection
- Platform-specific checks (macOS/Windows/Linux)
- Error handling and detailed error messages

## 🚀 Quick Start Integration

### Step 1: Add Provider to App.tsx
```tsx
import { PermissionStatusProvider, PermissionModal } from "@/features/permissions";

function App() {
  return (
    <PermissionStatusProvider>
      <AchievementCelebrationProvider>
        <SidebarProvider>
          {/* Your app content */}

          {/* Add permission modal - auto-shows on startup if needed */}
          <PermissionModal />

          <Toaster />
        </SidebarProvider>
      </AchievementCelebrationProvider>
    </PermissionStatusProvider>
  );
}
```

### Step 2: Use in Components (Optional)
```tsx
import { usePermissions } from "@/features/permissions";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";

function BlockingSettings() {
  const { isDegraded, permissionStatus } = usePermissions();

  if (isDegraded) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Limited Functionality</AlertTitle>
        <AlertDescription>
          Some features unavailable due to missing permissions.
        </AlertDescription>
      </Alert>
    );
  }

  return <div>{/* Full settings UI */}</div>;
}
```

### Step 3: Add Persistent Banner (Optional)
```tsx
import { PermissionIntegration } from "@/features/permissions/permission-integration-example";

function App() {
  return (
    <PermissionStatusProvider>
      <YourApp />
      <PermissionIntegration /> {/* Adds both banner and modal */}
    </PermissionStatusProvider>
  );
}
```

## 🎨 UI Components Used

- **shadcn/ui Dialog** - Modal component with focus trap
- **shadcn/ui Alert** - Status messages
- **shadcn/ui Button** - Action buttons
- **shadcn/ui Checkbox** - "Don't show again" option
- **shadcn/ui Label** - Form labels
- **shadcn/ui Separator** - Visual dividers
- **lucide-react icons** - AlertTriangle, CheckCircle2, XCircle, RefreshCw, ExternalLink

## 📱 Platform Support

### macOS
- Hosts file: `/etc/hosts`
- Process monitoring: Accessibility permissions required
- Instructions: System Preferences > Security & Privacy > Accessibility

### Windows
- Hosts file: `C:\Windows\System32\drivers\etc\hosts`
- Process monitoring: May require admin privileges
- Instructions: Run as Administrator

### Linux
- Hosts file: `/etc/hosts`
- Process monitoring: Requires `/proc` filesystem access
- Instructions: `sudo chmod 644 /etc/hosts`

## ♿ Accessibility Features

All components follow WCAG 2.1 Level AA standards:
- Keyboard navigation support
- Screen reader friendly with proper ARIA labels
- Focus management in modals
- Color contrast compliance
- Semantic HTML structure
- `role="status"` and `aria-live="polite"` for banner

## 💾 LocalStorage Keys

- `focusflow_dont_show_permission_modal` - Boolean flag for dismissing startup modal

## 🔄 Permission Check Flow

```
1. App Startup
   ↓
2. PermissionStatusProvider mounts
   ↓
3. Calls check_permissions() Tauri command
   ↓
4. Backend checks:
   - Hosts file write access
   - Process monitoring capability
   ↓
5. Returns PermissionStatus
   ↓
6. Frontend updates context
   ↓
7. If degraded && !dismissed:
   - Show PermissionModal
   - Show DegradedModeBanner
   ↓
8. User can:
   - View instructions
   - Check again after fixing
   - Dismiss temporarily
   - Don't show again (persist)
```

## 🧪 Testing Considerations

When testing the permission system:

1. **Mock Permission States**
   - Test with all permissions granted
   - Test with missing hosts file permission
   - Test with missing process monitoring
   - Test with all permissions missing

2. **Modal Behavior**
   - Verify auto-show on startup
   - Test "Don't show again" persistence
   - Test "Check Again" functionality
   - Verify controlled vs uncontrolled modes

3. **Banner Behavior**
   - Test banner visibility when degraded
   - Verify animation timing
   - Test "Fix This" button integration

4. **Platform Detection**
   - Verify correct platform detection
   - Test platform-specific instructions
   - Check instruction accuracy

## 📚 Related Documentation

- **README.md** - Complete feature documentation
- **INTEGRATION_EXAMPLE.tsx** - Full App.tsx integration
- **BACKEND_EXAMPLE.rs** - Rust backend implementation
- **setup-guides/** - Platform-specific detailed guides

## 🎯 Key Design Decisions

1. **Auto-show Modal**: Modal automatically shows on startup if permissions missing (can be suppressed)
2. **Non-blocking**: Users can continue with limited functionality
3. **Platform-aware**: Different instructions for macOS/Windows/Linux
4. **Persistent Banner**: Always visible reminder when degraded (can be hidden)
5. **LocalStorage**: Preferences stored locally, not in database
6. **Real-time Checks**: Permission status checked on startup and on-demand
7. **Accessible**: Full WCAG 2.1 AA compliance throughout

## 🚨 Important Notes

- Backend must implement `check_permissions` Tauri command
- Modal shows automatically if permissions degraded (unless dismissed)
- Banner persists at bottom of screen until permissions fixed
- Platform detection based on user agent
- All UI components are accessible and keyboard navigable
- Works with or without user authentication

## 🎉 Features Highlights

✅ Automatic permission detection on startup
✅ Platform-specific setup instructions
✅ Visual status indicators (icons + colors)
✅ "Don't show again" preference
✅ Real-time permission rechecking
✅ Persistent degraded mode banner
✅ Fully accessible (WCAG 2.1 AA)
✅ Responsive design for all screen sizes
✅ Dark mode compatible
✅ TypeScript type safety
✅ React Query for state management
✅ Controlled and uncontrolled modal modes
✅ Detailed error messages
✅ Link to external documentation
✅ Smooth animations and transitions
