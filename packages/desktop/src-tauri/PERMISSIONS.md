# FocusFlow Permission System

## Overview

FocusFlow's blocking features require elevated privileges to function effectively. This document explains the permission system, what's required, and how to implement permission checking in the frontend.

## Architecture

### Permission Checking Module

Located at `src/commands/permissions.rs`, this module provides:

1. **Comprehensive Status Checks** - Detailed permission analysis
2. **Platform-Specific Instructions** - Step-by-step guides for macOS, Windows, and Linux
3. **Error Diagnostics** - Specific error messages for troubleshooting

### Capabilities Required

#### 1. Hosts File Access
- **File**: `/etc/hosts` (macOS/Linux) or `C:\Windows\System32\drivers\etc\hosts` (Windows)
- **Permission**: Write access
- **Purpose**: Block websites by redirecting domains to 127.0.0.1
- **Requirement**:
  - macOS: Full Disk Access permission
  - Windows: Run as Administrator
  - Linux: sudo or sudoers rule

#### 2. Process Monitoring
- **Purpose**: Enumerate running processes to detect blocked applications
- **Requirement**: Generally available, may require elevation for system processes
- **Uses**: `sysinfo` crate for cross-platform process enumeration

#### 3. Process Termination
- **Purpose**: Close blocked applications during focus sessions
- **Requirement**:
  - User processes: Generally available
  - System processes: May require elevation
  - Graceful termination followed by force-kill if needed

## Permission Status Levels

### Fully Functional
All blocking features work:
- ✅ Hosts file writable
- ✅ Process monitoring available
- ✅ Process termination available

**User Experience**: Complete website and app blocking without workarounds.

### Degraded
Some features work:
- ❌ Hosts file not writable
- ✅ Process monitoring available
- ✅ Process termination available

**User Experience**: App blocking works, but website blocking requires frontend fallback (browser extension or manual enforcement).

### Non-Functional
No privileged features work:
- ❌ Hosts file not writable
- ❌ Process monitoring unavailable
- ❌ Process termination unavailable

**User Experience**: Only frontend-based blocking available (easily bypassed).

## Commands

### `check_permissions()`

Returns comprehensive permission status.

**Rust Signature:**
```rust
#[tauri::command]
pub async fn check_permissions() -> Result<PermissionStatus>
```

**TypeScript Usage:**
```typescript
import { Commands } from './bindings/permissions';

const status = await Commands.checkPermissions();

console.log('Overall status:', status.overall_status);
console.log('Hosts file:', status.hosts_file_writable);
console.log('Process monitoring:', status.process_monitoring_available);

if (status.overall_status !== 'fully_functional') {
  console.log('Recommendations:', status.recommendations);
}
```

**Response Structure:**
```typescript
interface PermissionStatus {
  hosts_file_writable: boolean;
  hosts_file_error: string | null;
  hosts_file_path: string;
  process_monitoring_available: boolean;
  process_monitoring_error: string | null;
  process_termination_available: boolean;
  process_termination_error: string | null;
  overall_status: "fully_functional" | "degraded" | "non_functional";
  recommendations: string[];
  platform: string;
}
```

### `get_permission_instructions(platform: string)`

Returns platform-specific setup instructions.

**Rust Signature:**
```rust
#[tauri::command]
pub async fn get_permission_instructions(platform: String) -> Result<PlatformInstructions>
```

**TypeScript Usage:**
```typescript
// Auto-detect platform (recommended)
const instructions = await Commands.getPermissionInstructions('');

// Or specify platform explicitly
const macInstructions = await Commands.getPermissionInstructions('macos');
const winInstructions = await Commands.getPermissionInstructions('windows');
const linuxInstructions = await Commands.getPermissionInstructions('linux');

console.log('Primary method:', instructions.primary_method.name);
console.log('Steps:', instructions.primary_method.steps);
console.log('Alternatives:', instructions.alternative_methods);
```

**Response Structure:**
```typescript
interface PlatformInstructions {
  platform: string;
  primary_method: PermissionMethod;
  alternative_methods: PermissionMethod[];
  requires_restart: boolean;
  security_notes: string[];
}

interface PermissionMethod {
  name: string;
  steps: string[];
  is_permanent: boolean;
  is_recommended: boolean;
  grants: string[];
}
```

## Frontend Implementation Guide

### 1. Permission Check on Startup

```typescript
async function checkPermissionsOnStartup() {
  try {
    const status = await Commands.checkPermissions();

    if (status.overall_status === 'fully_functional') {
      // All good - proceed normally
      return;
    }

    // Show permission setup UI
    showPermissionSetupDialog(status);
  } catch (error) {
    console.error('Permission check failed:', error);
  }
}
```

### 2. Permission Setup Dialog Component

```typescript
interface PermissionSetupDialogProps {
  status: PermissionStatus;
}

function PermissionSetupDialog({ status }: PermissionSetupDialogProps) {
  const [instructions, setInstructions] = useState<PlatformInstructions | null>(null);

  useEffect(() => {
    Commands.getPermissionInstructions('').then(setInstructions);
  }, []);

  if (!instructions) return <LoadingSpinner />;

  return (
    <Dialog>
      <h2>Setup Required</h2>
      <p>{PermissionHelpers.getStatusMessage(status)}</p>

      <div className="recommendations">
        <h3>What's Missing:</h3>
        <ul>
          {status.recommendations.map((rec, i) => (
            <li key={i}>{rec}</li>
          ))}
        </ul>
      </div>

      <div className="instructions">
        <h3>{instructions.primary_method.name}</h3>
        <p className="badge">
          {instructions.primary_method.is_permanent ? '✓ Permanent' : '⚠ Temporary'}
        </p>
        <ol>
          {instructions.primary_method.steps.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
      </div>

      {instructions.alternative_methods.length > 0 && (
        <details>
          <summary>Alternative Methods</summary>
          {instructions.alternative_methods.map((method, i) => (
            <div key={i}>
              <h4>{method.name}</h4>
              <ol>
                {method.steps.map((step, j) => (
                  <li key={j}>{step}</li>
                ))}
              </ol>
            </div>
          ))}
        </details>
      )}

      <div className="security-notes">
        <h4>Security Notes:</h4>
        <ul>
          {instructions.security_notes.map((note, i) => (
            <li key={i}>{note}</li>
          ))}
        </ul>
      </div>

      {instructions.requires_restart && (
        <div className="warning">
          ⚠️ You'll need to restart FocusFlow after granting permissions
        </div>
      )}

      <button onClick={recheckPermissions}>
        Recheck Permissions
      </button>
    </Dialog>
  );
}
```

### 3. Periodic Permission Monitoring

```typescript
function usePermissionMonitoring() {
  const [status, setStatus] = useState<PermissionStatus | null>(null);

  useEffect(() => {
    // Check immediately
    Commands.checkPermissions().then(setStatus);

    // Recheck every 30 seconds
    const interval = setInterval(() => {
      Commands.checkPermissions().then(setStatus);
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  return status;
}
```

### 4. Conditional Feature Display

```typescript
function BlockingSettings() {
  const status = usePermissionMonitoring();

  if (!status) return <LoadingSpinner />;

  return (
    <div>
      <h2>Blocking Settings</h2>

      {/* Website Blocking Section */}
      <section>
        <h3>Website Blocking</h3>
        {status.hosts_file_writable ? (
          <WebsiteBlockingForm />
        ) : (
          <PermissionRequired
            feature="website blocking"
            error={status.hosts_file_error}
          />
        )}
      </section>

      {/* App Blocking Section */}
      <section>
        <h3>App Blocking</h3>
        {status.process_monitoring_available ? (
          <AppBlockingForm />
        ) : (
          <PermissionRequired
            feature="app blocking"
            error={status.process_monitoring_error}
          />
        )}
      </section>

      {/* Status Indicator */}
      <StatusBadge status={status.overall_status} />
    </div>
  );
}
```

## Platform-Specific Notes

### macOS

**Required Permission**: Full Disk Access

**Steps**:
1. System Settings → Privacy & Security → Full Disk Access
2. Authenticate and add FocusFlow
3. Restart the app

**Why**: macOS System Integrity Protection (SIP) prevents modifying `/etc/hosts` without explicit user permission.

**Alternatives**:
- Run with `sudo` (temporary)
- Use automation permissions (less effective)

### Windows

**Required Permission**: Administrator privileges

**Steps**:
1. Right-click FocusFlow → Properties
2. Compatibility tab → "Run as administrator"
3. Restart the app

**Why**: Windows User Account Control (UAC) protects system files in `C:\Windows\System32\`.

**Alternatives**:
- Right-click → "Run as administrator" each time (temporary)

### Linux

**Required Permission**: sudo/root access

**Steps** (Recommended):
1. Create sudoers rule: `sudo visudo`
2. Add: `username ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts`
3. No restart needed

**Why**: Linux permissions protect `/etc/hosts` from non-root modifications.

**Alternatives**:
- Run with `sudo focusflow` each time
- `chmod 666 /etc/hosts` (⚠️ security risk, not recommended)

## Testing

### Unit Tests

Run the permission module tests:

```bash
cd packages/desktop/src-tauri
cargo test commands::permissions
```

### Integration Testing

Test permission checking:

```typescript
describe('Permission System', () => {
  it('should check permissions', async () => {
    const status = await Commands.checkPermissions();
    expect(status).toBeDefined();
    expect(status.platform).toBeTruthy();
    expect(['fully_functional', 'degraded', 'non_functional'])
      .toContain(status.overall_status);
  });

  it('should get platform instructions', async () => {
    const instructions = await Commands.getPermissionInstructions('');
    expect(instructions).toBeDefined();
    expect(instructions.primary_method.steps.length).toBeGreaterThan(0);
  });
});
```

## Troubleshooting

### "Hosts file not writable" on macOS
→ Grant Full Disk Access in System Settings

### "Permission denied" on Windows
→ Run as Administrator

### "Cannot enumerate processes" on Linux
→ Check if running with sufficient permissions

### Permission granted but still not working
→ Restart the application (especially on macOS/Windows)

## Security Considerations

1. **Minimal Permissions**: FocusFlow only modifies the hosts file and monitors user processes
2. **Transparent**: All permission requirements are clearly documented
3. **User Control**: Permissions can be revoked at any time through system settings
4. **Industry Standard**: Same permissions required by Cold Turkey, Freedom, SelfControl, etc.

## Future Enhancements

- [ ] Automatic permission detection and prompt on first run
- [ ] "Test Permissions" button to verify setup
- [ ] Visual permission checklist in settings
- [ ] Platform-specific helper scripts for permission setup
- [ ] Fallback to DNS-based blocking when hosts file unavailable
- [ ] Browser extension integration for enhanced website blocking
