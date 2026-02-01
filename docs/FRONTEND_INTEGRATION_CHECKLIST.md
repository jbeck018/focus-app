# Frontend Integration Checklist

Quick guide to integrate the permission checking system into your frontend.

## 1. Copy Type Definitions

```bash
# Copy TypeScript bindings to your frontend src directory
cp packages/desktop/src-tauri/bindings/permissions.ts packages/desktop/src/lib/permissions.ts
```

## 2. Add Permission Check on Startup

**File**: `src/App.tsx` or `src/main.tsx`

```typescript
import { invoke } from '@tauri-apps/api/core';

async function checkPermissionsOnStartup() {
  try {
    const status = await invoke('check_permissions');

    if (status.overall_status !== 'fully_functional') {
      // Show permission setup dialog
      showPermissionSetupModal(status);
    }
  } catch (error) {
    console.error('Permission check failed:', error);
  }
}

// Call on app mount
useEffect(() => {
  checkPermissionsOnStartup();
}, []);
```

## 3. Create Permission Setup Dialog Component

**File**: `src/components/PermissionSetup.tsx`

Use the example from `packages/desktop/src-tauri/examples/permission_check_example.ts`

Key features:
- [ ] Display current permission status
- [ ] Show platform-specific instructions
- [ ] "Recheck Permissions" button
- [ ] Visual status indicators (✅/⚠️/❌)
- [ ] Expandable alternative methods
- [ ] Security notes section

## 4. Add to Settings Page

**File**: `src/pages/Settings.tsx`

Add a "Permissions" section:

```typescript
function PermissionsSection() {
  const [status, setStatus] = useState(null);

  useEffect(() => {
    invoke('check_permissions').then(setStatus);
  }, []);

  return (
    <section>
      <h3>Permissions</h3>
      <StatusIndicator status={status?.overall_status} />
      <button onClick={() => showPermissionSetupDialog(status)}>
        View Setup Instructions
      </button>
    </section>
  );
}
```

## 5. Feature Gating

Gate features based on permissions:

```typescript
// Website blocking UI
const status = await invoke('check_permissions');

if (status.hosts_file_writable) {
  return <WebsiteBlockingForm />;
} else {
  return (
    <div>
      <p>Website blocking requires additional permissions.</p>
      <button onClick={showPermissionSetup}>Setup</button>
    </div>
  );
}
```

## 6. Add Permission Monitoring (Optional)

```typescript
// Monitor permissions periodically
useEffect(() => {
  const interval = setInterval(async () => {
    const status = await invoke('check_permissions');
    updatePermissionStatus(status);
  }, 30000); // Every 30 seconds

  return () => clearInterval(interval);
}, []);
```

## 7. Add to Onboarding Flow (Recommended)

Insert permission setup as a step in your onboarding:

```typescript
const onboardingSteps = [
  { id: 'welcome', component: <Welcome /> },
  { id: 'permissions', component: <PermissionSetup /> }, // ← Add this
  { id: 'configure', component: <ConfigureBlocking /> },
  { id: 'complete', component: <Complete /> },
];
```

## 8. Add Visual Indicators

Create reusable status components:

```typescript
function StatusBadge({ status }) {
  const badges = {
    fully_functional: { text: 'Fully Functional', color: 'green', icon: '✅' },
    degraded: { text: 'Degraded', color: 'yellow', icon: '⚠️' },
    non_functional: { text: 'Non-Functional', color: 'red', icon: '❌' }
  };

  const badge = badges[status];

  return (
    <span className={`badge badge-${badge.color}`}>
      {badge.icon} {badge.text}
    </span>
  );
}
```

## 9. Error Handling

Always handle permission check failures:

```typescript
try {
  const status = await invoke('check_permissions');
  // Use status
} catch (error) {
  console.error('Permission check failed:', error);
  // Show fallback UI or error message
  showErrorNotification('Failed to check permissions. Please try again.');
}
```

## 10. Testing

Test the permission flow:

```typescript
// Test 1: Permission check returns valid data
test('check_permissions returns valid status', async () => {
  const status = await invoke('check_permissions');
  expect(status).toHaveProperty('overall_status');
  expect(['fully_functional', 'degraded', 'non_functional'])
    .toContain(status.overall_status);
});

// Test 2: Instructions can be retrieved
test('get_permission_instructions returns instructions', async () => {
  const instructions = await invoke('get_permission_instructions', { platform: '' });
  expect(instructions).toHaveProperty('primary_method');
  expect(instructions.primary_method.steps.length).toBeGreaterThan(0);
});
```

## Quick Reference

### Available Commands

```typescript
// Check permissions
const status = await invoke('check_permissions');

// Get instructions (auto-detect platform)
const instructions = await invoke('get_permission_instructions', { platform: '' });

// Get instructions for specific platform
const macInstructions = await invoke('get_permission_instructions', { platform: 'macos' });
```

### Status Values

- `"fully_functional"` - All features work
- `"degraded"` - Some features work
- `"non_functional"` - No privileged features work

### Permission Fields

```typescript
status.hosts_file_writable          // boolean
status.hosts_file_error             // string | null
status.process_monitoring_available // boolean
status.process_monitoring_error     // string | null
status.process_termination_available// boolean
status.process_termination_error    // string | null
status.recommendations              // string[]
```

## Styling Recommendations

```css
/* Permission status colors */
.status-fully-functional {
  color: #22c55e; /* green */
}

.status-degraded {
  color: #eab308; /* yellow */
}

.status-non-functional {
  color: #ef4444; /* red */
}

/* Permission setup dialog */
.permission-setup {
  max-width: 600px;
  padding: 2rem;
}

.permission-method {
  border: 1px solid #e5e7eb;
  padding: 1rem;
  margin: 1rem 0;
  border-radius: 0.5rem;
}

.permission-method.primary {
  border-color: #3b82f6;
  background: #eff6ff;
}

.steps {
  padding-left: 1.5rem;
  margin: 1rem 0;
}

.steps li {
  margin: 0.5rem 0;
}

.security-notes {
  background: #fef3c7;
  padding: 1rem;
  border-radius: 0.5rem;
  margin-top: 1rem;
}

.warning-box {
  background: #fee2e2;
  border: 1px solid #f87171;
  padding: 1rem;
  border-radius: 0.5rem;
  margin: 1rem 0;
}
```

## Resources

- **Main Documentation**: `packages/desktop/src-tauri/PERMISSIONS.md`
- **Quick Reference**: `packages/desktop/src-tauri/src/commands/README_PERMISSIONS.md`
- **Examples**: `packages/desktop/src-tauri/examples/permission_check_example.ts`
- **Type Definitions**: `packages/desktop/src-tauri/bindings/permissions.ts`

## Checklist

- [ ] Copy type definitions to frontend
- [ ] Add permission check on app startup
- [ ] Create permission setup dialog component
- [ ] Add to settings page
- [ ] Implement feature gating
- [ ] Add permission monitoring (optional)
- [ ] Integrate into onboarding flow
- [ ] Create visual status indicators
- [ ] Add error handling
- [ ] Write frontend tests
- [ ] Style permission UI components
- [ ] Test on all platforms (macOS, Windows, Linux)

## Support

If you need help:
1. Check the comprehensive docs: `PERMISSIONS.md`
2. Review the examples: `examples/permission_check_example.ts`
3. Consult the quick reference: `src/commands/README_PERMISSIONS.md`

---

**Happy Integrating! 🚀**
