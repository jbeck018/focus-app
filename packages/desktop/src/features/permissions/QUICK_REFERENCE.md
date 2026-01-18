# Permission System - Quick Reference

One-page reference for the FocusFlow permission system.

## 🚀 Quick Start (Copy & Paste)

### 1. Add to App.tsx

```tsx
import { PermissionStatusProvider, PermissionIntegration } from "@/features/permissions";

function App() {
  return (
    <PermissionStatusProvider>
      <YourApp />
      <PermissionIntegration />
    </PermissionStatusProvider>
  );
}
```

**Done!** This adds both the banner and modal with automatic behavior.

---

## 📚 Common Use Cases

### Check Permission Status in Any Component

```tsx
import { usePermissions } from "@/features/permissions";

function MyComponent() {
  const { isDegraded, hasFullPermissions, permissionStatus } = usePermissions();

  if (isDegraded) {
    return <Alert>Some features unavailable</Alert>;
  }

  return <FullFeatureUI />;
}
```

### Manually Trigger Permission Modal

```tsx
import { useState } from "react";
import { PermissionModal, usePermissions } from "@/features/permissions";

function SettingsPage() {
  const [showModal, setShowModal] = useState(false);
  const { isDegraded } = usePermissions();

  return (
    <>
      {isDegraded && (
        <Button onClick={() => setShowModal(true)}>
          Fix Permissions
        </Button>
      )}
      <PermissionModal open={showModal} onOpenChange={setShowModal} />
    </>
  );
}
```

### Manual Banner + Modal Integration

```tsx
import { useState } from "react";
import { DegradedModeBanner, PermissionModal } from "@/features/permissions";

function App() {
  const [showModal, setShowModal] = useState(false);

  return (
    <PermissionStatusProvider>
      <YourApp />
      <DegradedModeBanner onFixClick={() => setShowModal(true)} />
      <PermissionModal open={showModal} onOpenChange={setShowModal} />
    </PermissionStatusProvider>
  );
}
```

### Trigger Recheck After Action

```tsx
import { usePermissions } from "@/features/permissions";

function PermissionButton() {
  const { recheckPermissions } = usePermissions();

  const handleFixAttempt = async () => {
    // User followed instructions
    await openSystemPreferences();
    // Now recheck
    await recheckPermissions();
  };

  return <Button onClick={handleFixAttempt}>Open System Prefs</Button>;
}
```

---

## 🎯 API Reference

### `usePermissions()` Hook

```typescript
const {
  permissionStatus,     // Full status object | null
  isLoading,           // Boolean - true during check
  hasFullPermissions,  // Boolean - all permissions granted
  isDegraded,          // Boolean - some/all permissions missing
  recheckPermissions,  // Function - trigger new check
} = usePermissions();
```

### `PermissionStatus` Type

```typescript
interface PermissionStatus {
  hosts_file_writable: boolean;
  hosts_file_error: string | null;
  process_monitoring_available: boolean;
  process_monitoring_error: string | null;
  overall_status: 'fully_functional' | 'degraded' | 'non_functional';
}
```

### `<PermissionModal>` Props

```typescript
interface PermissionModalProps {
  open?: boolean;         // Controlled mode: is modal open?
  onOpenChange?: (open: boolean) => void;  // Controlled mode: callback
}

// Uncontrolled (auto-shows):
<PermissionModal />

// Controlled:
<PermissionModal open={isOpen} onOpenChange={setIsOpen} />
```

### `<DegradedModeBanner>` Props

```typescript
interface DegradedModeBannerProps {
  onFixClick?: () => void;  // Callback when "Fix This" clicked
}

<DegradedModeBanner onFixClick={() => setShowModal(true)} />
```

---

## 🔧 Backend Requirements

### Rust Command (Required)

```rust
#[tauri::command]
fn check_permissions() -> Result<PermissionStatus, String> {
    // Check hosts file write access
    let hosts_writable = check_hosts_file();
    let hosts_error = if !hosts_writable {
        Some("Permission denied".to_string())
    } else {
        None
    };

    // Check process monitoring
    let process_available = check_process_monitoring();
    let process_error = if !process_available {
        Some("Cannot access processes".to_string())
    } else {
        None
    };

    // Calculate overall status
    let overall_status = match (hosts_writable, process_available) {
        (true, true) => "fully_functional",
        (true, false) | (false, true) => "degraded",
        (false, false) => "non_functional",
    };

    Ok(PermissionStatus {
        hosts_file_writable: hosts_writable,
        hosts_file_error: hosts_error,
        process_monitoring_available: process_available,
        process_monitoring_error: process_error,
        overall_status: overall_status.to_string(),
    })
}
```

### Register Command

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        check_permissions,  // Add this
        // ... other commands
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

---

## 🎨 Styling Customization

### Change Banner Position

```tsx
// Default: bottom-4 (bottom of screen)
// Change in degraded-mode-banner.tsx line ~55:

"fixed bottom-4 left-4 right-4 z-50"  // Bottom
"fixed top-4 left-4 right-4 z-50"     // Top
"fixed top-16 left-4 right-4 z-50"    // Below header
```

### Change Banner Colors

```tsx
// degraded-mode-banner.tsx
// Amber (warning) for degraded:
"bg-amber-500/10 border-amber-500/50"

// Red (destructive) for non-functional:
"bg-destructive/10 border-destructive/50"

// Customize as needed
```

### Change Modal Size

```tsx
// permission-modal.tsx line ~191:
<DialogContent className="max-w-2xl ...">  // Default
<DialogContent className="max-w-3xl ...">  // Larger
<DialogContent className="max-w-lg ...">   // Smaller
```

---

## 🐛 Troubleshooting

### Modal Not Showing

1. Check provider wraps app: `<PermissionStatusProvider>`
2. Check permissions are degraded: `isDegraded === true`
3. Check localStorage: `localStorage.getItem('focusflow_dont_show_permission_modal')`
4. Clear localStorage: `localStorage.removeItem('focusflow_dont_show_permission_modal')`
5. Use controlled mode: `<PermissionModal open={true} />`

### Banner Not Showing

1. Check permissions are degraded: `isDegraded === true`
2. Check z-index conflicts (banner is z-50)
3. Check if element is actually in DOM (React DevTools)
4. Check CSS: `opacity-0 pointer-events-none` when hidden

### Hook Error: "must be used within a PermissionStatusProvider"

Wrap your component tree with the provider:
```tsx
<PermissionStatusProvider>
  <ComponentUsingHook />
</PermissionStatusProvider>
```

### Backend Not Called

1. Check Tauri command registered
2. Check command name matches: `"check_permissions"`
3. Check Tauri dev tools for errors
4. Test command directly: `invoke('check_permissions')`

### LocalStorage Not Persisting

1. Check browser/app allows localStorage
2. Check incognito/private mode (doesn't persist)
3. Check storage quota
4. Manual test: `localStorage.setItem('test', 'value')`

---

## 📊 Status Overview

| Status | Hosts File | Process Monitoring | UI Shown | Color |
|--------|------------|-------------------|----------|-------|
| **Fully Functional** | ✅ | ✅ | None | Green |
| **Degraded** | ✅ | ❌ | Banner + Modal | Amber |
| **Degraded** | ❌ | ✅ | Banner + Modal | Amber |
| **Non-Functional** | ❌ | ❌ | Banner + Modal | Red |

---

## 🔑 Key Files

```
permissions/
├── index.ts                              # Main exports
├── types.ts                              # TypeScript types
├── use-permissions.ts                    # Hook
├── permission-status-context.tsx         # Provider
├── permission-modal.tsx                  # Modal UI
├── degraded-mode-banner.tsx             # Banner UI
└── permission-integration-example.tsx    # Combined component
```

---

## 💡 Tips

1. **Use `PermissionIntegration`** for quickest setup
2. **Controlled modal** for manual triggers (like from banner)
3. **Uncontrolled modal** for auto-show on startup
4. **Check `isDegraded`** before showing permission-dependent features
5. **Call `recheckPermissions()`** after user follows instructions
6. **Clear localStorage** during development/testing
7. **Test on all platforms** - instructions differ per OS

---

## 📱 Platform-Specific

### macOS
- Hosts: `/etc/hosts`
- Command: `sudo chmod 644 /etc/hosts`
- Process: System Preferences > Accessibility

### Windows
- Hosts: `C:\Windows\System32\drivers\etc\hosts`
- Command: Run as Administrator
- Process: Add to antivirus exceptions

### Linux
- Hosts: `/etc/hosts`
- Command: `sudo chmod 644 /etc/hosts`
- Process: Check `/proc` access

---

## ✅ Checklist

### Initial Setup
- [ ] Backend `check_permissions` command implemented
- [ ] Command registered in Tauri
- [ ] Provider wraps app
- [ ] PermissionIntegration or manual components added
- [ ] Tested on target platform

### Before Release
- [ ] Test all permission states
- [ ] Test "don't show again" persistence
- [ ] Test recheck functionality
- [ ] Test on all platforms
- [ ] Verify accessibility (keyboard, screen reader)
- [ ] Check responsive design

---

## 🔗 More Resources

- **README.md** - Full documentation
- **ARCHITECTURE.md** - System design details
- **TESTING.md** - Testing guide
- **INTEGRATION_EXAMPLE.tsx** - Complete integration example
- **BACKEND_EXAMPLE.rs** - Rust implementation example

---

## 🆘 Need Help?

Common questions:
- **How do I test?** - See TESTING.md
- **How does it work?** - See ARCHITECTURE.md
- **How do I integrate?** - See INTEGRATION_EXAMPLE.tsx
- **What about backend?** - See BACKEND_EXAMPLE.rs

Found a bug? Check the troubleshooting section above first!
