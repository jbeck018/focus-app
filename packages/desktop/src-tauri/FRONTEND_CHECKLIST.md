# Frontend Integration Checklist

## Quick Start (15 minutes)

### Step 1: Add Type Definitions (2 minutes)

Create `src/types/capabilities.ts`:

```typescript
export interface BlockingCapabilities {
  hosts_file_writable: boolean;
  hosts_file_path: string;
  process_termination_available: boolean;
  recommended_method: 'hosts_file' | 'process_termination' | 'frontend_only';
  available_methods: Array<'hosts_file' | 'process_termination' | 'frontend_only'>;
  limitations: string[];
  platform: string;
}

export interface ElevationInstructions {
  platform: string;
  primary_method: string;
  alternative_methods: string[];
  steps: string[];
  security_notes: string[];
  requires_restart: boolean;
}
```

### Step 2: Add Store/State (3 minutes)

**For React Context:**
```typescript
// src/contexts/BlockingContext.tsx
import { createContext, useContext, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { BlockingCapabilities } from '../types/capabilities';

const BlockingContext = createContext<{
  capabilities: BlockingCapabilities | null;
  refreshCapabilities: () => Promise<void>;
}>({
  capabilities: null,
  refreshCapabilities: async () => {},
});

export function BlockingProvider({ children }) {
  const [capabilities, setCapabilities] = useState<BlockingCapabilities | null>(null);

  const refreshCapabilities = async () => {
    const caps = await invoke<BlockingCapabilities>('get_blocking_capabilities');
    setCapabilities(caps);
  };

  useEffect(() => {
    refreshCapabilities();
  }, []);

  return (
    <BlockingContext.Provider value={{ capabilities, refreshCapabilities }}>
      {children}
    </BlockingContext.Provider>
  );
}

export const useBlocking = () => useContext(BlockingContext);
```

**For Zustand:**
```typescript
// src/stores/blockingStore.ts
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { BlockingCapabilities } from '../types/capabilities';

interface BlockingStore {
  capabilities: BlockingCapabilities | null;
  loadCapabilities: () => Promise<void>;
}

export const useBlockingStore = create<BlockingStore>((set) => ({
  capabilities: null,
  loadCapabilities: async () => {
    const caps = await invoke<BlockingCapabilities>('get_blocking_capabilities');
    set({ capabilities: caps });
  },
}));
```

### Step 3: Create Permission Banner Component (5 minutes)

```typescript
// src/components/PermissionBanner.tsx
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useBlocking } from '../contexts/BlockingContext';
import type { ElevationInstructions } from '../types/capabilities';

export function PermissionBanner() {
  const { capabilities } = useBlocking();
  const [showInstructions, setShowInstructions] = useState(false);
  const [instructions, setInstructions] = useState<ElevationInstructions | null>(null);

  if (!capabilities || capabilities.hosts_file_writable) {
    return null;
  }

  const handleShowInstructions = async () => {
    const inst = await invoke<ElevationInstructions>('get_elevation_instructions');
    setInstructions(inst);
    setShowInstructions(true);
  };

  return (
    <div className="banner warning">
      <div className="banner-content">
        <h3>⚠ Website Blocking Needs Setup</h3>
        <p>{capabilities.limitations[0]}</p>
        <button onClick={handleShowInstructions}>
          Show Setup Instructions
        </button>
      </div>

      {showInstructions && instructions && (
        <SetupModal
          instructions={instructions}
          onClose={() => setShowInstructions(false)}
        />
      )}
    </div>
  );
}
```

### Step 4: Create Setup Instructions Modal (5 minutes)

```typescript
// src/components/SetupModal.tsx
import type { ElevationInstructions } from '../types/capabilities';

interface SetupModalProps {
  instructions: ElevationInstructions;
  onClose: () => void;
}

export function SetupModal({ instructions, onClose }: SetupModalProps) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Setup Website Blocking for {instructions.platform}</h2>

        <section>
          <h3>Method: {instructions.primary_method}</h3>
          <ol>
            {instructions.steps.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>
        </section>

        {instructions.alternative_methods.length > 0 && (
          <section>
            <h3>Alternative Methods</h3>
            <ul>
              {instructions.alternative_methods.map((method, i) => (
                <li key={i}>{method}</li>
              ))}
            </ul>
          </section>
        )}

        <section>
          <h3>Why This Is Needed</h3>
          <ul>
            {instructions.security_notes.map((note, i) => (
              <li key={i}>{note}</li>
            ))}
          </ul>
        </section>

        {instructions.requires_restart && (
          <div className="alert info">
            ℹ️ You'll need to restart the app after granting permissions
          </div>
        )}

        <button onClick={onClose}>Close</button>
      </div>
    </div>
  );
}
```

## Complete Integration Checklist

### Phase 1: Basic Integration (Day 1)

- [ ] Add type definitions (`capabilities.ts`)
- [ ] Create store/context for capabilities
- [ ] Load capabilities on app startup
- [ ] Create `PermissionBanner` component
- [ ] Create `SetupModal` component
- [ ] Add banner to app layout
- [ ] Test on platform without permissions

### Phase 2: Enhanced UX (Day 2)

- [ ] Add status indicator in settings
- [ ] Show blocking method in UI
- [ ] Add "Check Again" button for permissions
- [ ] Implement permission polling after setup
- [ ] Add success notification when permissions granted
- [ ] Style components to match app design
- [ ] Add loading states

### Phase 3: Polish (Day 3)

- [ ] Add animations/transitions
- [ ] Improve error handling
- [ ] Add telemetry/analytics for setup flow
- [ ] Create help documentation page
- [ ] Add keyboard shortcuts for modal
- [ ] Test on all platforms
- [ ] Accessibility audit

## Component Integration Points

### 1. App Root

```typescript
// src/App.tsx
import { BlockingProvider } from './contexts/BlockingContext';
import { PermissionBanner } from './components/PermissionBanner';

function App() {
  return (
    <BlockingProvider>
      <PermissionBanner />
      {/* Rest of your app */}
    </BlockingProvider>
  );
}
```

### 2. Settings Page

```typescript
// src/pages/Settings.tsx
import { useBlocking } from '../contexts/BlockingContext';

function SettingsPage() {
  const { capabilities, refreshCapabilities } = useBlocking();

  return (
    <div>
      <h2>Blocking Settings</h2>

      <div className="status">
        <strong>Status:</strong>
        {capabilities?.hosts_file_writable ? (
          <span className="success">✓ Full Blocking Active</span>
        ) : (
          <span className="warning">⚠ Limited Blocking</span>
        )}
      </div>

      <div className="method">
        <strong>Method:</strong> {capabilities?.recommended_method}
      </div>

      <button onClick={refreshCapabilities}>
        Check Permissions
      </button>
    </div>
  );
}
```

### 3. Focus Session Start

```typescript
// src/components/StartSessionButton.tsx
import { invoke } from '@tauri-apps/api/core';
import { useBlocking } from '../contexts/BlockingContext';

function StartSessionButton() {
  const { capabilities } = useBlocking();

  const handleStart = async () => {
    // Always allow starting session, even with limited blocking
    await invoke('start_focus_session', {
      duration: 25 * 60,
      // ... other params
    });

    // Show warning if permissions needed
    if (!capabilities?.hosts_file_writable) {
      showNotification(
        'Session started with limited blocking. Grant permissions for full protection.',
        'warning'
      );
    }
  };

  return (
    <button onClick={handleStart}>
      Start Focus Session
    </button>
  );
}
```

## Testing Checklist

### Unit Tests

- [ ] Test `BlockingProvider` loads capabilities
- [ ] Test `PermissionBanner` shows when no permissions
- [ ] Test `PermissionBanner` hides when permissions granted
- [ ] Test `SetupModal` displays all instruction fields
- [ ] Test permission refresh functionality

### Integration Tests

- [ ] Test complete setup flow
- [ ] Test permission polling
- [ ] Test modal open/close
- [ ] Test error handling
- [ ] Test with different capability states

### Manual Tests

- [ ] macOS without Full Disk Access
- [ ] macOS with Full Disk Access
- [ ] Windows without admin
- [ ] Windows with admin
- [ ] Linux without permissions
- [ ] Linux with permissions

## Styling Guide

### Banner Styles

```css
.banner {
  padding: 1rem;
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  gap: 1rem;
}

.banner.warning {
  background: #fef3cd;
  border: 1px solid #ffc107;
  color: #664d03;
}

.banner-content {
  flex: 1;
}

.banner button {
  padding: 0.5rem 1rem;
  background: #ffc107;
  border: none;
  border-radius: 0.25rem;
  cursor: pointer;
}
```

### Modal Styles

```css
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 0.5rem;
  padding: 2rem;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal h2 {
  margin-top: 0;
}

.modal section {
  margin: 1.5rem 0;
}

.alert.info {
  padding: 1rem;
  background: #cfe2ff;
  border: 1px solid #0d6efd;
  border-radius: 0.25rem;
  color: #052c65;
}
```

## Common Patterns

### Pattern 1: Check Before Action

```typescript
async function performBlockingAction() {
  const hasPermission = await invoke<boolean>('check_hosts_file_permissions');

  if (!hasPermission) {
    showWarning('This works better with elevated permissions');
  }

  // Proceed anyway - graceful degradation
  await invoke('toggle_blocking', { enable: true });
}
```

### Pattern 2: Permission Polling

```typescript
function usePermissionPolling(onGranted: () => void) {
  useEffect(() => {
    const interval = setInterval(async () => {
      const hasPermission = await invoke<boolean>('check_hosts_file_permissions');

      if (hasPermission) {
        onGranted();
        clearInterval(interval);
      }
    }, 5000);

    return () => clearInterval(interval);
  }, [onGranted]);
}
```

### Pattern 3: Status Indicator

```typescript
function BlockingStatus() {
  const { capabilities } = useBlocking();

  const getStatus = () => {
    if (!capabilities) return { label: 'Checking...', variant: 'info' };

    if (capabilities.hosts_file_writable) {
      return { label: 'Full Blocking', variant: 'success' };
    }

    return { label: 'Limited Blocking', variant: 'warning' };
  };

  const status = getStatus();

  return (
    <div className={`status-badge ${status.variant}`}>
      {status.label}
    </div>
  );
}
```

## Troubleshooting

### Issue: Capabilities always null

**Check:**
- [ ] `BlockingProvider` is wrapping your app
- [ ] `invoke` is imported from `@tauri-apps/api/core`
- [ ] Backend commands are registered in `lib.rs`

### Issue: Modal doesn't show

**Check:**
- [ ] `get_elevation_instructions` is being called correctly
- [ ] State is being updated
- [ ] Modal z-index is high enough
- [ ] No CSS display: none on modal

### Issue: Permissions not detected after grant

**Solution:**
App needs to be restarted (on macOS and Windows). Add message:
```typescript
{instructions.requires_restart && (
  <div className="alert warning">
    ⚠️ Please restart the app after granting permissions
  </div>
)}
```

## Resources

### Documentation
- `PRIVILEGE_HANDLING.md` - Complete guide
- `API_REFERENCE_CAPABILITIES.md` - API documentation
- `QUICK_START_PRIVILEGE_HANDLING.md` - Quick reference
- `examples/frontend-integration.ts` - Full examples

### Support
- GitHub Issues: For bugs or questions
- Team Chat: For implementation help
- Code Review: Share PR for review

## Timeline Estimate

- **Basic Integration:** 1 day
- **Enhanced UX:** 1 day
- **Polish & Testing:** 1 day
- **Total:** 3 days

## Success Criteria

✅ **Must Have:**
- [ ] Banner shows when permissions needed
- [ ] Setup instructions are accessible
- [ ] Users can start sessions regardless of permissions
- [ ] Clear status indicator in settings

✅ **Should Have:**
- [ ] Permission polling after setup
- [ ] Success notification when permissions granted
- [ ] Help documentation
- [ ] Error handling

✅ **Nice to Have:**
- [ ] Animations/transitions
- [ ] Keyboard shortcuts
- [ ] Analytics/telemetry
- [ ] Video tutorial link

---

**Questions?** Check the documentation or ask the team!
