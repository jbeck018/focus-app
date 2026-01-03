# Quick Start: Privilege Handling

## TL;DR

The app now detects permission issues early and guides users through setup. Three new Tauri commands available:

```typescript
// Check what's available
const capabilities = await invoke('get_blocking_capabilities');

// Get setup instructions
const instructions = await invoke('get_elevation_instructions');

// Quick permission check
const hasPermission = await invoke('check_hosts_file_permissions');
```

## Minimal Frontend Integration (5 minutes)

### 1. Check on Startup

```typescript
// In your app initialization
async function initApp() {
  const capabilities = await invoke('get_blocking_capabilities');

  if (!capabilities.hosts_file_writable) {
    alert('Website blocking needs setup. Click here for instructions.');
  }
}
```

### 2. Show Instructions

```typescript
async function showSetup() {
  const instructions = await invoke('get_elevation_instructions');

  alert(`Setup for ${instructions.platform}:\n\n` +
        instructions.steps.join('\n'));
}
```

### 3. Done!

That's it. The app will now:
- ✅ Detect permission issues
- ✅ Show clear error messages
- ✅ Provide platform-specific instructions
- ✅ Fallback to alternative blocking methods

## What Each Command Returns

### `get_blocking_capabilities()`

```json
{
  "hosts_file_writable": false,
  "hosts_file_path": "/etc/hosts",
  "recommended_method": "frontend_only",
  "limitations": ["Hosts file not writable..."],
  "platform": "macOS"
}
```

### `get_elevation_instructions()`

```json
{
  "platform": "macOS",
  "primary_method": "Grant Full Disk Access",
  "steps": [
    "Open System Settings > Privacy & Security",
    "Add FocusFlow to Full Disk Access",
    "Restart FocusFlow"
  ]
}
```

### `check_hosts_file_permissions()`

```json
true  // or false
```

## When to Call Each Command

| Command | When | Frequency |
|---------|------|-----------|
| `get_blocking_capabilities()` | App startup, after permission changes | Once or on-demand |
| `get_elevation_instructions()` | User clicks "How to fix" | On-demand |
| `check_hosts_file_permissions()` | Polling while waiting for permissions | Every 5 seconds |

## Common Patterns

### Show Permission Banner

```typescript
function App() {
  const [needsSetup, setNeedsSetup] = useState(false);

  useEffect(() => {
    invoke('get_blocking_capabilities').then(caps => {
      setNeedsSetup(!caps.hosts_file_writable);
    });
  }, []);

  if (needsSetup) {
    return <PermissionBanner />;
  }

  return <MainApp />;
}
```

### Poll for Permission Changes

```typescript
// After user clicks "I've done this"
const interval = setInterval(async () => {
  const hasPermission = await invoke('check_hosts_file_permissions');

  if (hasPermission) {
    showSuccess('All set!');
    clearInterval(interval);
  }
}, 5000);
```

### Graceful Degradation

```typescript
async function startBlocking() {
  const caps = await invoke('get_blocking_capabilities');

  if (caps.hosts_file_writable) {
    // Full blocking
    showStatus('System-wide blocking active');
  } else {
    // Limited blocking
    showWarning('Limited blocking - setup recommended');
  }

  await invoke('toggle_blocking', { enable: true });
}
```

## Platform Differences

| Platform | Permission Required | Restart Needed | Setup Time |
|----------|-------------------|----------------|------------|
| macOS | Full Disk Access | Yes | 1-2 minutes |
| Windows | Run as Administrator | Yes | 30 seconds |
| Linux | sudo or file permissions | No | Variable |

## Testing

### Test Without Permissions

Run app normally (should detect missing permissions).

### Test With Permissions

**macOS:**
```bash
# Grant Full Disk Access via System Settings, then restart
```

**Windows:**
```bash
# Right-click > Run as administrator
```

**Linux:**
```bash
sudo ./focusflow
# Or: sudo chmod 666 /etc/hosts (not recommended for production)
```

## Troubleshooting

### "Capabilities returns empty limitations"

The app has all needed permissions. No action needed.

### "Instructions don't show for my platform"

Check that platform detection is working:
```typescript
const caps = await invoke('get_blocking_capabilities');
console.log(caps.platform); // Should be: macOS, Windows, or Linux
```

### "Permission check still returns false after granting"

Restart the app. Most platforms require restart after granting permissions.

## Next Steps

For more detailed integration examples, see:
- `PRIVILEGE_HANDLING.md` - Full documentation
- `examples/frontend-integration.ts` - Complete TypeScript examples
- `README_PRIVILEGE_HANDLING.md` - Architecture overview
