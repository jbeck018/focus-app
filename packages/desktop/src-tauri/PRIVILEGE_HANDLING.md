# Privilege Handling Strategy for Website Blocking

This document describes the robust privilege handling system implemented for FocusFlow's website blocking feature.

## Overview

Website blocking through hosts file modification requires elevated privileges on all platforms. This implementation provides:

1. **Early Detection** - Check permissions before attempting modification
2. **Clear User Guidance** - Platform-specific instructions for granting permissions
3. **Graceful Fallbacks** - Multiple blocking strategies when elevated privileges aren't available
4. **Transparent Communication** - Users understand what's working and what's not

## Architecture

### Modules

#### `blocking/capabilities.rs`
Central module for capability detection and permission checking.

**Key Types:**
- `BlockingCapabilities` - Complete capability report
- `BlockingMethod` - Available blocking methods (HostsFile, ProcessTermination, FrontendOnly)
- `ElevationInstructions` - Platform-specific permission guidance

**Key Functions:**
- `check_capabilities()` - Returns complete capability assessment
- `check_hosts_file_writable()` - Tests if hosts file can be written
- `get_elevation_instructions()` - Returns platform-specific instructions

#### `blocking/hosts.rs`
Enhanced with permission checking functions.

**New Functions:**
- `check_hosts_permissions()` - Async check for hosts file write permissions
- `get_hosts_path()` - Now public for capability reporting

### Blocking Methods (Priority Order)

1. **Hosts File** (Most Effective)
   - Requires: Elevated privileges
   - Pros: System-wide blocking, cannot be bypassed by apps
   - Cons: Requires user to grant permissions
   - Used by: Cold Turkey, SelfControl, Freedom

2. **Process Termination** (Effective for Apps)
   - Requires: Standard user privileges (may need elevation for system processes)
   - Pros: Blocks applications effectively
   - Cons: Doesn't block websites in all browsers

3. **Frontend-Only** (Fallback)
   - Requires: No special privileges
   - Pros: Always available
   - Cons: Can be bypassed, relies on frontend cooperation

## Tauri Commands

### Capability Detection

#### `get_blocking_capabilities()`
Returns complete capability assessment.

**Frontend Usage:**
```typescript
import { invoke } from '@tauri-apps/api/core';

interface BlockingCapabilities {
  hosts_file_writable: boolean;
  hosts_file_path: string;
  process_termination_available: boolean;
  recommended_method: 'hosts_file' | 'process_termination' | 'frontend_only';
  available_methods: string[];
  limitations: string[];
  platform: string;
}

const capabilities = await invoke<BlockingCapabilities>('get_blocking_capabilities');

if (!capabilities.hosts_file_writable) {
  console.warn('Hosts file blocking not available:', capabilities.limitations);
}
```

**Response Example (macOS without permissions):**
```json
{
  "hosts_file_writable": false,
  "hosts_file_path": "/etc/hosts",
  "process_termination_available": true,
  "recommended_method": "process_termination",
  "available_methods": ["process_termination", "frontend_only"],
  "limitations": [
    "Hosts file at /etc/hosts is not writable. Website blocking requires elevated privileges."
  ],
  "platform": "macOS"
}
```

#### `get_elevation_instructions()`
Returns platform-specific instructions for granting permissions.

**Frontend Usage:**
```typescript
interface ElevationInstructions {
  platform: string;
  primary_method: string;
  alternative_methods: string[];
  steps: string[];
  security_notes: string[];
  requires_restart: boolean;
}

const instructions = await invoke<ElevationInstructions>('get_elevation_instructions');

// Display instructions to user
console.log('To enable full blocking:');
instructions.steps.forEach((step, i) => {
  console.log(`${i + 1}. ${step}`);
});
```

**Response Example (macOS):**
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
    "FocusFlow only modifies the hosts file and does not access other system files",
    "Similar permissions are required by apps like Cold Turkey and SelfControl"
  ],
  "requires_restart": true
}
```

**Response Example (Windows):**
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
    "FocusFlow only modifies the hosts file for website blocking",
    "This is the same permission required by Freedom, Cold Turkey, and similar apps"
  ],
  "requires_restart": true
}
```

#### `check_hosts_file_permissions()`
Simple boolean check for hosts file writability.

**Frontend Usage:**
```typescript
const hasPermission = await invoke<boolean>('check_hosts_file_permissions');

if (hasPermission) {
  console.log('✓ Full website blocking available');
} else {
  console.log('⚠ Limited blocking - elevated privileges needed');
}
```

## Frontend Integration Guide

### 1. Check Capabilities on App Startup

```typescript
// During app initialization
async function initializeBlockingFeature() {
  const capabilities = await invoke<BlockingCapabilities>('get_blocking_capabilities');

  // Store in app state
  setBlockingCapabilities(capabilities);

  // Show notification if permissions needed
  if (!capabilities.hosts_file_writable) {
    showPermissionNotification(capabilities);
  }
}
```

### 2. Display Permission Banner

When hosts file isn't writable, show a banner with instructions:

```typescript
function PermissionBanner({ capabilities }: { capabilities: BlockingCapabilities }) {
  const [instructions, setInstructions] = useState<ElevationInstructions | null>(null);

  const showInstructions = async () => {
    const inst = await invoke<ElevationInstructions>('get_elevation_instructions');
    setInstructions(inst);
  };

  if (capabilities.hosts_file_writable) {
    return null; // Don't show banner if permissions are granted
  }

  return (
    <Banner variant="warning">
      <AlertTriangle />
      <div>
        <h3>Website blocking requires elevated permissions</h3>
        <p>{capabilities.limitations.join(' ')}</p>
        <Button onClick={showInstructions}>
          Show Setup Instructions
        </Button>
      </div>
    </Banner>
  );
}
```

### 3. Display Setup Instructions Modal

```typescript
function SetupInstructionsModal({ instructions }: { instructions: ElevationInstructions }) {
  return (
    <Modal>
      <h2>Setup Website Blocking for {instructions.platform}</h2>

      <Section>
        <h3>Method: {instructions.primary_method}</h3>
        <ol>
          {instructions.steps.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
      </Section>

      {instructions.alternative_methods.length > 0 && (
        <Section>
          <h3>Alternative Methods</h3>
          <ul>
            {instructions.alternative_methods.map((method, i) => (
              <li key={i}>{method}</li>
            ))}
          </ul>
        </Section>
      )}

      <Section>
        <h3>Security Notes</h3>
        <ul>
          {instructions.security_notes.map((note, i) => (
            <li key={i}>{note}</li>
          ))}
        </ul>
      </Section>

      {instructions.requires_restart && (
        <Alert variant="info">
          You'll need to restart FocusFlow after granting permissions
        </Alert>
      )}
    </Modal>
  );
}
```

### 4. Periodic Permission Checks

Check if permissions were granted after user follows instructions:

```typescript
// Check every 5 seconds when waiting for permissions
useEffect(() => {
  if (!capabilities.hosts_file_writable) {
    const interval = setInterval(async () => {
      const hasPermission = await invoke<boolean>('check_hosts_file_permissions');

      if (hasPermission) {
        // Permissions granted! Update capabilities
        const newCapabilities = await invoke<BlockingCapabilities>('get_blocking_capabilities');
        setBlockingCapabilities(newCapabilities);

        showSuccessNotification('Website blocking enabled!');
        clearInterval(interval);
      }
    }, 5000);

    return () => clearInterval(interval);
  }
}, [capabilities]);
```

### 5. Graceful Degradation

Show different UI based on available capabilities:

```typescript
function BlockingSettings({ capabilities }: { capabilities: BlockingCapabilities }) {
  return (
    <Settings>
      <h2>Website Blocking</h2>

      {capabilities.hosts_file_writable ? (
        <StatusBadge variant="success">
          ✓ Full Blocking Enabled
        </StatusBadge>
      ) : (
        <StatusBadge variant="warning">
          ⚠ Limited Blocking Active
        </StatusBadge>
      )}

      <MethodSelector>
        <h3>Active Method: {capabilities.recommended_method}</h3>
        <p>
          {capabilities.recommended_method === 'hosts_file'
            ? 'System-wide blocking via hosts file (most secure)'
            : capabilities.recommended_method === 'process_termination'
            ? 'App blocking via process termination'
            : 'Frontend-based blocking (can be bypassed)'}
        </p>
      </MethodSelector>

      {capabilities.limitations.length > 0 && (
        <Limitations>
          <h3>Current Limitations</h3>
          <ul>
            {capabilities.limitations.map((limitation, i) => (
              <li key={i}>{limitation}</li>
            ))}
          </ul>
          <Button onClick={showSetupInstructions}>
            Fix This
          </Button>
        </Limitations>
      )}
    </Settings>
  );
}
```

## Implementation Flow

### At App Startup

1. Call `get_blocking_capabilities()` to assess current state
2. Store capabilities in app state
3. If `hosts_file_writable` is false, show notification banner
4. Log capabilities for debugging

### When User Starts Focus Session

1. Check `capabilities.hosts_file_writable`
2. If true: Use hosts file blocking (existing behavior)
3. If false:
   - Show warning that blocking is limited
   - Offer to show setup instructions
   - Use fallback blocking methods (frontend-based)

### When User Views Settings

1. Display current blocking capabilities
2. Show setup instructions button if permissions needed
3. Display which method is currently active
4. Show any limitations

### After User Grants Permissions

1. User restarts app (if required by platform)
2. App calls `get_blocking_capabilities()` again
3. Detects `hosts_file_writable` is now true
4. Show success notification
5. Enable full blocking features

## Testing

### Manual Testing Checklist

**macOS:**
- [ ] App detects lack of Full Disk Access
- [ ] Instructions are accurate for current macOS version
- [ ] After granting Full Disk Access and restarting, app detects it
- [ ] Hosts file modification works after permission granted

**Windows:**
- [ ] App detects when not running as Administrator
- [ ] Instructions show how to run as admin
- [ ] After running as admin, app detects elevation
- [ ] Hosts file modification works with elevation

**Linux:**
- [ ] App detects lack of write permission to /etc/hosts
- [ ] Instructions show multiple options (sudo, permissions, sudoers)
- [ ] After granting permission, app detects it
- [ ] Hosts file modification works

### Unit Tests

Run tests:
```bash
cargo test --package focus-app blocking::capabilities
```

Tests cover:
- Platform detection
- Capability reporting
- Elevation instruction generation

## Security Considerations

### Why These Permissions Are Needed

**Hosts File Modification:**
- System-level DNS resolution
- Cannot be bypassed by applications
- Industry standard for blocking apps (Cold Turkey, Freedom, SelfControl)

### What We DON'T Access

- No other system files
- No user documents
- No network traffic monitoring
- No process memory reading

### Transparency

- Clear documentation of what permissions do
- Comparison to well-known apps (Cold Turkey, SelfControl)
- Open source code for audit
- Minimal permission scope

## Comparison to Similar Apps

| App | macOS Permission | Windows Permission | Blocking Method |
|-----|-----------------|-------------------|-----------------|
| Cold Turkey | Full Disk Access | Run as Administrator | Hosts file |
| Freedom | Full Disk Access | Run as Administrator | Hosts file + VPN |
| SelfControl | Full Disk Access | N/A | Hosts file + Firewall |
| FocusFlow | Full Disk Access | Run as Administrator | Hosts file + Process termination |

## Troubleshooting

### "Hosts file is not writable" on macOS

**Solution:**
1. Open System Settings > Privacy & Security > Full Disk Access
2. Add FocusFlow to the list
3. Restart FocusFlow

### "Hosts file is not writable" on Windows

**Solution:**
1. Right-click FocusFlow icon
2. Select "Run as administrator"
3. Or set to always run as admin in Properties > Compatibility

### "Hosts file is not writable" on Linux

**Solution:**
1. Run with sudo: `sudo focusflow`
2. Or grant write permission: `sudo chmod 666 /etc/hosts` (less secure)
3. Or create sudoers rule (recommended, see instructions)

### Permissions granted but still not working

**Solution:**
1. Ensure app was fully restarted
2. Check file path is correct for platform
3. Check system logs for permission errors
4. Try running test: `cargo test check_hosts_permissions`

## Future Enhancements

1. **Auto-elevation** - Request elevation programmatically on app start
2. **Browser Extensions** - Supplement hosts file with extension-based blocking
3. **VPN-based Blocking** - Alternative to hosts file (like Freedom)
4. **Firewall Integration** - Block at network layer (like SelfControl)
5. **Permission Wizard** - Step-by-step GUI wizard for granting permissions

## References

- [Cold Turkey Permission Documentation](https://getcoldturkey.com/)
- [SelfControl macOS Permissions](https://selfcontrolapp.com/)
- [Freedom Multi-Platform Blocking](https://freedom.to/)
- [macOS Full Disk Access Guide](https://support.apple.com/guide/mac-help/mh40583/mac)
- [Windows UAC Documentation](https://docs.microsoft.com/en-us/windows/security/identity-protection/user-account-control/)
