# Permission System Architecture

## 🏗️ Component Hierarchy

```
App
└── PermissionStatusProvider
    ├── [Your App Components]
    ├── DegradedModeBanner
    └── PermissionModal
```

## 🔄 Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        App Startup                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│          PermissionStatusProvider.mount()                    │
│  - Creates React Context                                    │
│  - Sets initial loading state                               │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│         invoke("check_permissions")                          │
│  - Tauri IPC call to Rust backend                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│              Rust Backend Checks                             │
│  ┌──────────────────────────────────────────────┐           │
│  │ 1. Hosts File Write Access                   │           │
│  │    - Try to open /etc/hosts (Unix)           │           │
│  │    - Try to open C:\...\hosts (Windows)      │           │
│  │    - Return: bool + error message            │           │
│  └──────────────────────────────────────────────┘           │
│  ┌──────────────────────────────────────────────┐           │
│  │ 2. Process Monitoring Capability             │           │
│  │    - macOS: Check accessibility permissions  │           │
│  │    - Windows: Check process enumeration      │           │
│  │    - Linux: Check /proc access               │           │
│  │    - Return: bool + error message            │           │
│  └──────────────────────────────────────────────┘           │
│  ┌──────────────────────────────────────────────┐           │
│  │ 3. Calculate Overall Status                  │           │
│  │    - Both OK: "fully_functional"             │           │
│  │    - One OK: "degraded"                      │           │
│  │    - None OK: "non_functional"               │           │
│  └──────────────────────────────────────────────┘           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│         Return PermissionStatus                              │
│  {                                                           │
│    hosts_file_writable: bool,                               │
│    hosts_file_error: string | null,                         │
│    process_monitoring_available: bool,                      │
│    process_monitoring_error: string | null,                 │
│    overall_status: "fully_functional" | "degraded" |        │
│                   "non_functional"                           │
│  }                                                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│    Context State Updated                                     │
│  - permissionStatus: PermissionStatus                        │
│  - isLoading: false                                          │
│  - hasFullPermissions: computed                              │
│  - isDegraded: computed                                      │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│         All Consumers Re-render                              │
│  ┌──────────────────────────────────────────────┐           │
│  │ DegradedModeBanner                           │           │
│  │  - usePermissions()                          │           │
│  │  - Shows if isDegraded                       │           │
│  │  - Animates in from bottom                   │           │
│  └──────────────────────────────────────────────┘           │
│  ┌──────────────────────────────────────────────┐           │
│  │ PermissionModal                              │           │
│  │  - usePermissions()                          │           │
│  │  - Auto-shows if isDegraded && !dismissed    │           │
│  │  - Checks localStorage for "don't show"     │           │
│  └──────────────────────────────────────────────┘           │
│  ┌──────────────────────────────────────────────┐           │
│  │ Your Components                              │           │
│  │  - usePermissions()                          │           │
│  │  - Access permission state                   │           │
│  │  - Conditionally render features             │           │
│  └──────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

## 🎯 State Management

### Context State
```typescript
{
  permissionStatus: PermissionStatus | null,  // null during initial load
  isLoading: boolean,                          // true during check
  hasFullPermissions: boolean,                 // computed from status
  isDegraded: boolean,                         // computed from status
  recheckPermissions: () => Promise<void>      // trigger new check
}
```

### Local State (PermissionModal)
```typescript
{
  internalOpen: boolean,                       // modal visibility (uncontrolled)
  dontShowAgain: boolean,                      // checkbox state
  isRechecking: boolean,                       // loading state for recheck
  platform: "macos" | "windows" | "linux"     // detected platform
}
```

### LocalStorage
```typescript
{
  "focusflow_dont_show_permission_modal": "true" | "false"
}
```

## 🔌 Integration Points

### 1. Context Provider (Required)
```tsx
// Must wrap entire app
<PermissionStatusProvider>
  <App />
</PermissionStatusProvider>
```

### 2. Hook Usage (Optional)
```tsx
// Any component can use
const { isDegraded, recheckPermissions } = usePermissions();
```

### 3. UI Components (Optional)
```tsx
// Add as needed
<DegradedModeBanner onFixClick={...} />
<PermissionModal open={...} onOpenChange={...} />
```

## 🚦 Permission States

```
┌─────────────────────────────────────────────────────────┐
│                   Permission States                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  FULLY_FUNCTIONAL                                       │
│  ✓ hosts_file_writable: true                           │
│  ✓ process_monitoring_available: true                  │
│  → All blocking features available                     │
│  → No warnings shown                                   │
│  → Full app functionality                              │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  DEGRADED                                               │
│  ✓ hosts_file_writable: true                           │
│  ✗ process_monitoring_available: false                 │
│    OR                                                   │
│  ✗ hosts_file_writable: false                          │
│  ✓ process_monitoring_available: true                  │
│  → Some blocking features work                         │
│  → Amber warning banner shown                          │
│  → Modal auto-shows on startup (if not dismissed)      │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  NON_FUNCTIONAL                                         │
│  ✗ hosts_file_writable: false                          │
│  ✗ process_monitoring_available: false                 │
│  → No blocking features work                           │
│  → Red warning banner shown                            │
│  → Modal auto-shows on startup (if not dismissed)      │
│  → App still usable for timer/tracking                 │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## 🎨 UI Component States

### DegradedModeBanner

```
NOT DEGRADED          DEGRADED (Amber)       NON-FUNCTIONAL (Red)
┌───────────┐        ┌─────────────────┐    ┌─────────────────┐
│           │        │ ⚠ Limited       │    │ ✗ Unavailable   │
│  Hidden   │   →    │ Missing: X      │    │ Missing: X & Y  │
│           │        │ [Fix This]      │    │ [Fix This]      │
└───────────┘        └─────────────────┘    └─────────────────┘
```

### PermissionModal

```
DISMISSED              AUTO-SHOW (Degraded)    CONTROLLED
┌───────────┐         ┌──────────────────┐    ┌──────────────────┐
│           │         │ ⚠ Permissions    │    │ ⚠ Permissions    │
│  Hidden   │    →    │ ✓ Feature A      │    │ ✗ Feature B      │
│           │         │ ✗ Feature B      │    │ Instructions...  │
│           │         │ [Instructions]   │    │ [Check] [Close]  │
└───────────┘         └──────────────────┘    └──────────────────┘
     ↑                         ↓                        ↑
     │                         │                        │
     └─────────────────────────┴────────────────────────┘
           [Don't show again] checkbox persists
```

## 🔄 Recheck Flow

```
User clicks "Check Again"
         │
         ▼
Button shows loading spinner
         │
         ▼
recheckPermissions() called
         │
         ▼
invoke("check_permissions") again
         │
         ▼
Context state updated
         │
         ▼
All consumers re-render
         │
         ├─► If still degraded: Banner/Modal stay
         │
         └─► If fixed: Banner/Modal auto-hide
```

## 📱 Platform Detection

```javascript
const userAgent = navigator.userAgent.toLowerCase();

if (userAgent.includes("win")) {
  platform = "windows"
  hostsPath = "C:\\Windows\\System32\\drivers\\etc\\hosts"
  instructions = "Run as Administrator"
}
else if (userAgent.includes("linux")) {
  platform = "linux"
  hostsPath = "/etc/hosts"
  instructions = "sudo chmod 644 /etc/hosts"
}
else {
  platform = "macos"  // default
  hostsPath = "/etc/hosts"
  instructions = "System Preferences > Accessibility"
}
```

## 🎭 Modal Behavior Modes

### Uncontrolled Mode (Default)
```tsx
<PermissionModal />

// Behavior:
// - Auto-shows on startup if degraded
// - Checks localStorage for "don't show again"
// - Manages own open/close state
// - Perfect for "set it and forget it"
```

### Controlled Mode
```tsx
const [open, setOpen] = useState(false);
<PermissionModal open={open} onOpenChange={setOpen} />

// Behavior:
// - NEVER auto-shows
// - Parent controls visibility
// - Does NOT check localStorage
// - Perfect for manual triggers (like from banner)
```

## 🧩 Component Dependencies

```
PermissionStatusProvider
├── @tauri-apps/api/core (invoke)
└── React Context

usePermissions
└── PermissionStatusContext

PermissionModal
├── usePermissions hook
├── @tauri-apps/plugin-shell (open)
├── shadcn/ui Dialog
├── shadcn/ui Alert
├── shadcn/ui Button
├── shadcn/ui Checkbox
├── shadcn/ui Label
├── shadcn/ui Separator
└── lucide-react icons

DegradedModeBanner
├── usePermissions hook
├── shadcn/ui Button
└── lucide-react icons
```

## 🔐 Security Considerations

1. **No Sensitive Data**: No passwords or tokens stored
2. **LocalStorage Only**: Only stores UI preference (dismissed state)
3. **Read-Only Checks**: Backend checks are non-destructive
4. **User Consent**: Users can dismiss and continue with limited features
5. **Transparent**: Clear messaging about what's not working

## ⚡ Performance Characteristics

- **Initial Load**: ~50-100ms (single Tauri IPC call)
- **Recheck**: ~50-100ms (on-demand)
- **Memory**: Minimal (single context state)
- **Re-renders**: Only when permission state changes
- **Polling**: None (manual recheck only)

## 🎯 Design Principles

1. **Non-Blocking**: App works even without permissions
2. **Transparent**: Clear about what's not working
3. **Actionable**: Provides clear steps to fix
4. **Respectful**: Can be dismissed
5. **Platform-Aware**: Different instructions per OS
6. **Accessible**: Full keyboard and screen reader support
7. **Progressive**: Graceful degradation of features
