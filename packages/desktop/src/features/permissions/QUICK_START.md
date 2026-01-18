# DegradedModeBanner - Quick Start Guide

## TL;DR

Add a persistent banner that shows when blocking features are broken:

```tsx
// App.tsx
import { PermissionStatusProvider, PermissionIntegration } from "@/features/permissions";

function App() {
  return (
    <PermissionStatusProvider>
      <YourApp />
      <PermissionIntegration />  {/* ← Add this line */}
    </PermissionStatusProvider>
  );
}
```

Done! The banner will automatically show when permissions are missing.

---

## What You Get

### The Banner
- Fixed to bottom of screen
- Cannot be dismissed (encourages fixing permissions)
- Shows what's broken ("website blocking", "app blocking")
- Color-coded (amber = degraded, red = broken)
- "Fix This" button

### The Modal
- Opens when user clicks "Fix This"
- Platform-specific instructions (macOS/Windows/Linux)
- "Check Again" button after fixing
- Can be dismissed

---

## File Locations

```
/features/permissions/
├── degraded-mode-banner.tsx          ← Main banner component
├── permission-modal.tsx               ← Modal with instructions
├── permission-integration-example.tsx ← Pre-wired integration
├── use-permissions.ts                 ← Hook to access status
└── types.ts                           ← TypeScript types
```

---

## Key Components

### 1. DegradedModeBanner

```tsx
<DegradedModeBanner onFixClick={() => setShowModal(true)} />
```

**When it shows:** When `overall_status` is 'degraded' or 'non_functional'

**Props:**
- `onFixClick?: () => void` - Called when "Fix This" is clicked

### 2. PermissionModal

```tsx
<PermissionModal open={showModal} onOpenChange={setShowModal} />
```

**Props:**
- `open?: boolean` - Control modal visibility
- `onOpenChange?: (open: boolean) => void` - Handle state changes

### 3. usePermissions Hook

```tsx
const { permissionStatus, isDegraded, hasFullPermissions } = usePermissions();
```

**Returns:**
- `permissionStatus` - Detailed status object
- `isDegraded` - Boolean: true if any permissions missing
- `hasFullPermissions` - Boolean: true if all permissions granted
- `recheckPermissions` - Function to recheck permissions

---

## Common Use Cases

### 1. Default Setup (Easiest)

```tsx
import { PermissionStatusProvider, PermissionIntegration } from "@/features/permissions";

<PermissionStatusProvider>
  <App />
  <PermissionIntegration />
</PermissionStatusProvider>
```

### 2. With Custom Modal Handler

```tsx
const [showModal, setShowModal] = useState(false);

<DegradedModeBanner onFixClick={() => {
  console.log("User wants to fix permissions");
  setShowModal(true);
}} />
```

### 3. Conditional Feature Display

```tsx
const { hasFullPermissions } = usePermissions();

<Button disabled={!hasFullPermissions}>
  Start Blocking Session
</Button>
```

### 4. Show Warning in UI

```tsx
const { isDegraded, permissionStatus } = usePermissions();

{isDegraded && (
  <Alert variant="destructive">
    Missing: {!permissionStatus.hosts_file_writable && "website blocking"}
  </Alert>
)}
```

---

## Styling Overview

**Position:** Bottom of screen (fixed)
**Colors:** Amber (degraded) / Red (non-functional)
**Animation:** Slides up on mount
**Dark Mode:** Automatic
**Z-Index:** 50 (above content, below modals)

---

## Permission States

### `fully_functional`
- ✅ All permissions granted
- Banner: Hidden
- All features: Available

### `degraded`
- ⚠️ Some permissions missing
- Banner: Amber/yellow
- Some features: Limited or unavailable

### `non_functional`
- ❌ Critical permissions missing
- Banner: Red
- Blocking features: Completely unavailable

---

## Troubleshooting

**Banner doesn't show?**
1. Check `PermissionStatusProvider` wraps your app
2. Verify Tauri `check_permissions` command exists
3. Check console for errors

**"Fix This" doesn't work?**
1. Ensure `onFixClick` callback is provided
2. Check modal is rendered
3. Verify modal state is wired correctly

**Permission check fails?**
1. Check Tauri command registration
2. Verify backend permissions
3. Test with mock data

---

## What the Backend Needs

The frontend expects a Tauri command:

```rust
#[tauri::command]
fn check_permissions() -> Result<PermissionStatus, String> {
  // Check hosts file write access
  // Check process monitoring access
  // Return status
}
```

See `BACKEND_EXAMPLE.rs` for full implementation.

---

## Next Steps

1. ✅ Add `PermissionIntegration` to your App.tsx
2. ✅ Test by simulating missing permissions
3. ✅ Customize colors/text if needed (see component source)
4. ✅ Add analytics tracking (see USAGE_EXAMPLES.md)

---

## Learn More

- **README.md** - Full documentation
- **USAGE_EXAMPLES.md** - 8 detailed examples
- **IMPLEMENTATION_SUMMARY.md** - Technical details
- **degraded-mode-banner-preview.tsx** - Visual testing tool

---

## Quick Copy-Paste Templates

### Basic Integration
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

### Manual Integration
```tsx
import { useState } from "react";
import {
  PermissionStatusProvider,
  DegradedModeBanner,
  PermissionModal
} from "@/features/permissions";

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

### Using Permission Status
```tsx
import { usePermissions } from "@/features/permissions";

function MyComponent() {
  const { isDegraded, hasFullPermissions, permissionStatus } = usePermissions();

  if (isDegraded) {
    return <Warning>Some features unavailable</Warning>;
  }

  return <FullFunctionality />;
}
```

---

**Questions?** Check the full docs in README.md or USAGE_EXAMPLES.md
