# Capability Detection API Reference

## Tauri Commands

### `get_blocking_capabilities`

Returns complete assessment of available blocking methods based on system permissions.

**Parameters:** None

**Returns:** `BlockingCapabilities`

**Example:**
```typescript
const capabilities = await invoke<BlockingCapabilities>('get_blocking_capabilities');
```

**Response:**
```typescript
interface BlockingCapabilities {
  // Can we write to hosts file?
  hosts_file_writable: boolean;

  // Path to hosts file on this platform
  hosts_file_path: string;

  // Can we terminate processes?
  process_termination_available: boolean;

  // Best method to use given current permissions
  recommended_method: 'hosts_file' | 'process_termination' | 'frontend_only';

  // All available methods (in priority order)
  available_methods: Array<'hosts_file' | 'process_termination' | 'frontend_only'>;

  // Why certain methods aren't available
  limitations: string[];

  // Operating system
  platform: 'macOS' | 'Windows' | 'Linux';
}
```

**When to Call:**
- App startup
- After user grants permissions
- Before starting blocking session
- In settings page

**Error Handling:**
```typescript
try {
  const capabilities = await invoke('get_blocking_capabilities');
} catch (error) {
  console.error('Failed to check capabilities:', error);
  // Assume no permissions available
}
```

---

### `get_elevation_instructions`

Returns platform-specific instructions for granting required permissions.

**Parameters:** None

**Returns:** `ElevationInstructions`

**Example:**
```typescript
const instructions = await invoke<ElevationInstructions>('get_elevation_instructions');
```

**Response:**
```typescript
interface ElevationInstructions {
  // Operating system
  platform: string;

  // Primary recommended method
  primary_method: string;

  // Other ways to grant permissions
  alternative_methods: string[];

  // Step-by-step instructions
  steps: string[];

  // Why these permissions are needed
  security_notes: string[];

  // Whether app restart is required
  requires_restart: boolean;
}
```

**Response Examples:**

**macOS:**
```json
{
  "platform": "macOS",
  "primary_method": "Grant Full Disk Access",
  "alternative_methods": [
    "Run with sudo (temporary)",
    "Grant automation permissions"
  ],
  "steps": [
    "Open System Settings > Privacy & Security > Full Disk Access",
    "Click the lock icon and enter your password",
    "Click the '+' button and add FocusFlow from Applications",
    "Restart FocusFlow for changes to take effect"
  ],
  "security_notes": [
    "Full Disk Access allows FocusFlow to modify system files like /etc/hosts",
    "This is required for effective website blocking",
    "Similar permissions are required by apps like Cold Turkey and SelfControl"
  ],
  "requires_restart": true
}
```

**Windows:**
```json
{
  "platform": "Windows",
  "primary_method": "Run as Administrator",
  "alternative_methods": [
    "Set to always run as administrator"
  ],
  "steps": [
    "Right-click the FocusFlow icon",
    "Select 'Run as administrator'",
    "Or: Right-click > Properties > Compatibility",
    "Check 'Run this program as an administrator'",
    "Click OK and restart FocusFlow"
  ],
  "security_notes": [
    "Administrator access is required to modify the Windows hosts file",
    "Located at C:\\Windows\\System32\\drivers\\etc\\hosts",
    "This is the same permission required by Freedom, Cold Turkey, and similar apps"
  ],
  "requires_restart": true
}
```

**When to Call:**
- When user clicks "Setup Instructions" button
- When displaying "How to Fix" help dialog
- On-demand only (not on startup)

**Error Handling:**
```typescript
try {
  const instructions = await invoke('get_elevation_instructions');
} catch (error) {
  console.error('Failed to get instructions:', error);
  // Show generic message
}
```

---

### `check_hosts_file_permissions`

Quick boolean check for hosts file write permissions.

**Parameters:** None

**Returns:** `boolean` - `true` if hosts file is writable, `false` otherwise

**Example:**
```typescript
const canWrite = await invoke<boolean>('check_hosts_file_permissions');

if (canWrite) {
  console.log('Full blocking available');
} else {
  console.log('Permissions needed');
}
```

**When to Call:**
- Periodic polling (every 5 seconds) when waiting for permission grant
- Quick check before starting blocking
- In settings to show current status

**Error Handling:**
```typescript
try {
  const canWrite = await invoke('check_hosts_file_permissions');
} catch (error) {
  console.error('Permission check failed:', error);
  // Assume no permissions
  return false;
}
```

**Performance:**
- Fast: ~1-5ms
- No side effects (read-only check)
- Safe to call frequently

---

## Type Definitions

### BlockingMethod

```typescript
type BlockingMethod =
  | 'hosts_file'           // System-level hosts file (requires elevation)
  | 'process_termination'  // App blocking (may require elevation)
  | 'frontend_only';       // No system permissions needed
```

**Priority Order:**
1. `hosts_file` - Most secure, requires permissions
2. `process_termination` - App blocking, generally available
3. `frontend_only` - Always available, less secure

### Platform

```typescript
type Platform = 'macOS' | 'Windows' | 'Linux';
```

---

## Common Use Cases

### Use Case 1: Startup Check

```typescript
async function checkCapabilitiesOnStartup() {
  const capabilities = await invoke('get_blocking_capabilities');

  // Store in global state
  store.setCapabilities(capabilities);

  // Show notification if needed
  if (!capabilities.hosts_file_writable) {
    showPermissionBanner({
      message: 'Website blocking needs setup',
      action: showSetupInstructions
    });
  }

  return capabilities;
}
```

### Use Case 2: Setup Instructions Dialog

```typescript
async function showSetupDialog() {
  const instructions = await invoke('get_elevation_instructions');

  showModal({
    title: `Setup for ${instructions.platform}`,
    content: (
      <div>
        <h3>{instructions.primary_method}</h3>
        <ol>
          {instructions.steps.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
        <h4>Why this is needed</h4>
        <ul>
          {instructions.security_notes.map((note, i) => (
            <li key={i}>{note}</li>
          ))}
        </ul>
        {instructions.requires_restart && (
          <Alert>Restart required after granting permissions</Alert>
        )}
      </div>
    )
  });
}
```

### Use Case 3: Permission Polling

```typescript
function pollForPermissions(onGranted: () => void) {
  const interval = setInterval(async () => {
    const hasPermission = await invoke('check_hosts_file_permissions');

    if (hasPermission) {
      onGranted();
      clearInterval(interval);
    }
  }, 5000);

  return () => clearInterval(interval);
}

// Usage
const stopPolling = pollForPermissions(() => {
  showSuccess('Permissions granted!');
  refreshApp();
});
```

### Use Case 4: Settings Page

```typescript
function SettingsPage() {
  const [capabilities, setCapabilities] = useState(null);

  useEffect(() => {
    invoke('get_blocking_capabilities').then(setCapabilities);
  }, []);

  if (!capabilities) return <Loading />;

  return (
    <div>
      <h2>Blocking Settings</h2>

      <StatusIndicator
        status={capabilities.hosts_file_writable ? 'success' : 'warning'}
        label={capabilities.recommended_method}
      />

      <div>
        <strong>Platform:</strong> {capabilities.platform}
      </div>

      <div>
        <strong>Hosts File:</strong> {capabilities.hosts_file_path}
      </div>

      {capabilities.limitations.length > 0 && (
        <Alert variant="warning">
          <h3>Setup Required</h3>
          <ul>
            {capabilities.limitations.map((limitation, i) => (
              <li key={i}>{limitation}</li>
            ))}
          </ul>
          <Button onClick={showSetupInstructions}>
            Fix This
          </Button>
        </Alert>
      )}
    </div>
  );
}
```

---

## Best Practices

### DO

✅ Call `get_blocking_capabilities()` on app startup
✅ Store capabilities in global state
✅ Show clear UI when permissions are missing
✅ Use `check_hosts_file_permissions()` for polling
✅ Provide easy access to setup instructions
✅ Handle errors gracefully

### DON'T

❌ Call `get_elevation_instructions()` on every render
❌ Poll more frequently than every 5 seconds
❌ Show technical error messages to users
❌ Assume permissions without checking
❌ Block app usage when permissions missing (use fallbacks)

---

## Error Codes

All commands return `Result<T>` in Rust, which translates to Promise rejection in TypeScript.

**Common Errors:**
- `PermissionDenied` - Hosts file not writable
- `Io` - File system error
- `System` - Platform detection failed

**Handling:**
```typescript
try {
  const caps = await invoke('get_blocking_capabilities');
} catch (error) {
  if (error.type === 'PermissionDenied') {
    // Show setup instructions
  } else if (error.type === 'Io') {
    // File system issue
  } else {
    // Unknown error
    console.error(error);
  }
}
```

---

## Testing

### Test Without Permissions

Run app without elevation to verify:
- Commands return correct `hosts_file_writable: false`
- Limitations array is populated
- Instructions are platform-specific
- Recommended method falls back appropriately

### Test With Permissions

Run app with elevation to verify:
- Commands return correct `hosts_file_writable: true`
- Limitations array is empty
- Recommended method is `hosts_file`
- All methods available

### Test Platform Detection

Verify on each platform:
- Correct platform name returned
- Instructions match OS version
- Hosts file path is correct
- Restart requirement is accurate

---

## Performance

| Command | Typical Time | Max Time | Caching |
|---------|-------------|----------|---------|
| `get_blocking_capabilities` | 1-10ms | 50ms | No |
| `get_elevation_instructions` | <1ms | 5ms | No |
| `check_hosts_file_permissions` | 1-5ms | 20ms | No |

**Notes:**
- All commands are fast enough to call on-demand
- No need to cache results (but you can for convenience)
- File system checks are synchronous and quick

---

## Related Commands

These existing commands work alongside capability detection:

- `toggle_blocking(enable: boolean)` - Start/stop blocking (now handles permission failures gracefully)
- `add_blocked_website(domain: string)` - Add to block list
- `remove_blocked_website(domain: string)` - Remove from block list
- `get_blocked_items()` - Get current block lists

---

## Migration Guide

### Upgrading from Previous Version

**Before:**
```typescript
// Silent failure if no permissions
await invoke('toggle_blocking', { enable: true });
```

**After:**
```typescript
// Check first
const capabilities = await invoke('get_blocking_capabilities');

if (!capabilities.hosts_file_writable) {
  const instructions = await invoke('get_elevation_instructions');
  showSetupDialog(instructions);
} else {
  await invoke('toggle_blocking', { enable: true });
}
```

**Note:** The old code still works (fails gracefully), but new code provides better UX.
