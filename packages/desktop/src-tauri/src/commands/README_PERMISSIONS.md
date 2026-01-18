# Permissions Module

Quick reference for the FocusFlow permission checking system.

## Files

- **`permissions.rs`** - Main permission checking implementation
- **`../bindings/permissions.ts`** - TypeScript type definitions
- **`../PERMISSIONS.md`** - Comprehensive documentation
- **`../examples/permission_check_example.ts`** - Usage examples

## Quick Start

### Rust (Backend)

```rust
use crate::commands::permissions::{check_permissions, get_permission_instructions};

// Check permissions
let status = check_permissions().await?;

// Get platform-specific instructions
let instructions = get_permission_instructions("".to_string()).await?;
```

### TypeScript (Frontend)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Check permissions
const status = await invoke('check_permissions');

// Get instructions (auto-detect platform)
const instructions = await invoke('get_permission_instructions', { platform: '' });
```

## Key Types

### PermissionStatus

```rust
pub struct PermissionStatus {
    pub hosts_file_writable: bool,
    pub hosts_file_error: Option<String>,
    pub hosts_file_path: String,
    pub process_monitoring_available: bool,
    pub process_monitoring_error: Option<String>,
    pub process_termination_available: bool,
    pub process_termination_error: Option<String>,
    pub overall_status: OverallPermissionStatus,
    pub recommendations: Vec<String>,
    pub platform: String,
}
```

### OverallPermissionStatus

```rust
pub enum OverallPermissionStatus {
    FullyFunctional,  // All features work
    Degraded,         // Some features work
    NonFunctional,    // No privileged features work
}
```

### PlatformInstructions

```rust
pub struct PlatformInstructions {
    pub platform: String,
    pub primary_method: PermissionMethod,
    pub alternative_methods: Vec<PermissionMethod>,
    pub requires_restart: bool,
    pub security_notes: Vec<String>,
}
```

## Commands

### `check_permissions()`

Returns comprehensive permission status with detailed error messages.

**No parameters required**

**Returns**: `Result<PermissionStatus>`

### `get_permission_instructions(platform: String)`

Returns platform-specific setup instructions.

**Parameters**:
- `platform` - Platform name ("macos", "windows", "linux", or "" for auto-detect)

**Returns**: `Result<PlatformInstructions>`

## Platform Requirements

### macOS
- **Permission**: Full Disk Access
- **File**: `/etc/hosts`
- **Setup**: System Settings → Privacy & Security → Full Disk Access

### Windows
- **Permission**: Administrator privileges
- **File**: `C:\Windows\System32\drivers\etc\hosts`
- **Setup**: Properties → Compatibility → Run as administrator

### Linux
- **Permission**: sudo/root access
- **File**: `/etc/hosts`
- **Setup**: Create sudoers rule or run with sudo

## Status Levels

| Level | Hosts File | Process Monitoring | Process Termination | Description |
|-------|------------|-------------------|---------------------|-------------|
| ✅ **Fully Functional** | ✅ | ✅ | ✅ | All blocking features work |
| ⚠️ **Degraded** | ❌ | ✅ | ✅ | App blocking works, website blocking limited |
| ❌ **Non-Functional** | ❌ | ❌ | ❌ | Only frontend fallbacks available |

## Common Use Cases

### 1. Startup Permission Check

```typescript
const status = await invoke('check_permissions');
if (status.overall_status !== 'fully_functional') {
  showPermissionSetupDialog(status);
}
```

### 2. Show Setup Instructions

```typescript
const instructions = await invoke('get_permission_instructions', { platform: '' });
displayInstructions(instructions.primary_method);
```

### 3. Feature Gating

```typescript
const status = await invoke('check_permissions');

if (status.hosts_file_writable) {
  enableWebsiteBlocking();
} else {
  showHostsFileSetupRequired();
}
```

### 4. Permission Monitoring

```typescript
setInterval(async () => {
  const status = await invoke('check_permissions');
  updatePermissionUI(status);
}, 30000); // Check every 30 seconds
```

## Error Handling

All permission errors are returned as descriptive strings in the error fields:

```typescript
const status = await invoke('check_permissions');

if (!status.hosts_file_writable && status.hosts_file_error) {
  console.error('Hosts file issue:', status.hosts_file_error);
  // e.g., "Permission denied. Elevated privileges required."
}
```

## Testing

```bash
# Run unit tests
cargo test commands::permissions

# Run all permission-related tests
cargo test permissions
```

## Related Commands

These existing commands complement the permission system:

- `get_blocking_capabilities()` - Legacy capability check
- `check_hosts_file_permissions()` - Simple hosts file check
- `get_elevation_instructions()` - Legacy instruction getter

The new `permissions.rs` module provides more comprehensive checking and better error diagnostics.

## See Also

- **[Full Documentation](../PERMISSIONS.md)** - Comprehensive guide
- **[Examples](../examples/permission_check_example.ts)** - Code examples
- **[TypeScript Bindings](../bindings/permissions.ts)** - Type definitions
