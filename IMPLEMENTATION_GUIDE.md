# FocusFlow Frontend Implementation Guide

## Quick Start

### 1. Install Dependencies

Since the architecture plan mentions using Bun, but package.json is set up for npm:

```bash
# Using Bun (recommended)
bun install

# Or using npm
npm install
```

### 2. Project Structure Created

The following structure has been implemented:

```
src/
├── types/              # ✅ Complete - All type definitions
│   ├── session.ts
│   ├── blocking.ts
│   ├── analytics.ts
│   └── tauri.ts
│
├── hooks/              # ✅ Complete - Core hooks
│   ├── useTauri.ts
│   ├── useFocusSession.ts
│   ├── useBlockRules.ts
│   └── useAnalytics.ts
│
├── stores/             # ✅ Complete - Zustand stores
│   ├── focusStore.ts
│   ├── settingsStore.ts
│   └── analyticsStore.ts
│
├── utils/              # ✅ Complete - Utility functions
│   ├── time.ts
│   ├── formatters.ts
│   └── validators.ts
│
├── components/         # 🚧 Partial - FocusTimer example
│   └── FocusTimer/
│       ├── FocusTimer.tsx
│       ├── TimerDisplay.tsx
│       ├── TimerControls.tsx
│       └── SessionTypeSelector.tsx
│
└── test/               # ✅ Complete - Test setup
    └── setup.ts
```

## Key Architectural Features

### 1. Type-Safe Tauri Commands

All Tauri backend communication is fully typed:

```typescript
// Define command with type safety
const startSession = useTauriCommand('start_session');

// TypeScript knows the parameter and return types
const result = await startSession({
  plannedMinutes: Minutes(25),
  sessionType: 'focus',
});

// Result type enforces error handling
if (!result.success) {
  console.error(result.error);
  return;
}

const session = result.data; // Type: FocusSession
```

### 2. Branded Types for Safety

Domain primitives use branded types to prevent mixing:

```typescript
// These are type-safe at compile time
const duration: Minutes = Minutes(25);
const sessionId: SessionId = SessionId('abc-123');
const domain: Domain = Domain('twitter.com');

// This won't compile (type error)
const id: SessionId = duration; // ❌ Type error!
```

### 3. Discriminated Unions for State

Complex state is modeled with discriminated unions:

```typescript
// Session status is exhaustively typed
type SessionStatus =
  | { type: 'idle' }
  | { type: 'running'; startedAt: Timestamp }
  | { type: 'paused'; pausedAt: Timestamp };

// TypeScript ensures all cases are handled
function render(status: SessionStatus) {
  switch (status.type) {
    case 'idle':
      return <IdleView />;
    case 'running':
      return <RunningView startedAt={status.startedAt} />;
    case 'paused':
      return <PausedView pausedAt={status.pausedAt} />;
  }
}
```

### 4. Result Types Instead of Exceptions

Errors are values, not exceptions:

```typescript
// Backend returns Result type
type SessionResult<T> =
  | { success: true; data: T }
  | { success: false; error: SessionError };

// Forces error handling
const result = await createBlockRule(dto);
if (!result.success) {
  // Handle specific error types
  switch (result.error.type) {
    case 'validation':
      showError(result.error.message);
      break;
    case 'already_exists':
      showWarning('Rule already exists');
      break;
  }
  return;
}

// Success path
const rule = result.data;
```

### 5. Zustand with Immer and Persistence

State management is simple but powerful:

```typescript
export const useFocusStore = create<FocusStore>()(
  persist(
    immer((set) => ({
      // State
      activeSession: null,

      // Actions with Immer (mutable-style updates)
      setActiveSession: (session) =>
        set((state) => {
          state.activeSession = session; // Looks mutable, actually immutable
        }),
    })),
    {
      name: 'focus-store',
      // Only persist preferences, not ephemeral state
      partialize: (state) => ({
        defaultFocusDuration: state.defaultFocusDuration,
      }),
    }
  )
);
```

### 6. React Query for Async State

Server state is cached and synchronized:

```typescript
export function useActiveSession() {
  const getActiveSession = useTauriCommand('get_active_session');

  return useQuery({
    queryKey: ['sessions', 'active'],
    queryFn: async () => {
      const result = await getActiveSession();
      if (!result.success) throw new Error(result.error.type);
      return result.data;
    },
    refetchInterval: 1000, // Keep timer updated
    staleTime: 500,
  });
}
```

## Next Implementation Steps

### Phase 1: Complete Core Components

1. **Dashboard Component**
   - Analytics overview cards
   - Charts (Chart.js or Recharts)
   - Streak display
   - Goal progress

2. **BlockList Component**
   - Rule list with filtering
   - Add rule form with validation
   - Toggle enable/disable
   - Category templates

3. **TriggerJournal Component**
   - Journal entry form
   - Entry list with filtering
   - Insights display

4. **Settings Component**
   - Tabbed settings interface
   - Form validation
   - Tauri system integration

### Phase 2: Enhanced Features

1. **Add React Router**
   ```bash
   bun add react-router-dom
   ```

2. **Add Form Management**
   ```bash
   bun add react-hook-form @hookform/resolvers zod
   ```

3. **Add Animation**
   ```bash
   bun add framer-motion
   ```

4. **Add Charts**
   ```bash
   bun add recharts
   ```

### Phase 3: Testing

1. **Write Component Tests**
   - Test user interactions
   - Test error states
   - Test loading states

2. **Write Hook Tests**
   - Test state management
   - Test async operations
   - Test error handling

3. **Write Integration Tests**
   - Test complete user flows
   - Test Tauri integration

## Configuration Details

### TypeScript Configuration

The project uses the strictest TypeScript settings:

- `strict: true` - All strict checks enabled
- `noUncheckedIndexedAccess: true` - Array access returns `T | undefined`
- `exactOptionalPropertyTypes: true` - Distinguish `undefined` from missing
- `noImplicitReturns: true` - All code paths must return
- `noPropertyAccessFromIndexSignature: true` - Force bracket notation

### Vite Configuration

Optimized for Tauri:

- Fixed port (1420) for Tauri integration
- Path aliases for clean imports
- Source maps in debug mode
- Minification in production

### Vitest Configuration

Testing setup with:

- Happy DOM (faster than jsdom)
- React Testing Library helpers
- Tauri API mocks
- Coverage reporting

## Development Workflow

### 1. Start Development Server

```bash
# Frontend only
bun run dev

# With Tauri
bun run tauri:dev
```

### 2. Type Checking

```bash
# Check types without building
bun run typecheck
```

### 3. Run Tests

```bash
# Run all tests
bun test

# Watch mode
bun test --watch

# Coverage report
bun test:coverage

# UI mode
bun test:ui
```

### 4. Lint

```bash
bun run lint
```

### 5. Build

```bash
# Build frontend
bun run build

# Build Tauri app
bun run tauri:build
```

## Code Style Guidelines

### 1. Use Branded Types for Domain Primitives

```typescript
// ✅ Good
export type Minutes = number & { readonly __brand: 'Minutes' };
export const Minutes = (m: number): Minutes => {
  if (m < 0) throw new Error('Minutes cannot be negative');
  return m as Minutes;
};

// ❌ Bad
export type Minutes = number;
```

### 2. Use Discriminated Unions for Complex State

```typescript
// ✅ Good
type LoadingState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: Data }
  | { status: 'error'; error: Error };

// ❌ Bad
interface LoadingState {
  status: 'idle' | 'loading' | 'success' | 'error';
  data?: Data;
  error?: Error;
}
```

### 3. Use Result Types for Error Handling

```typescript
// ✅ Good
type Result<T, E> =
  | { success: true; data: T }
  | { success: false; error: E };

// ❌ Bad (throwing exceptions)
function riskyOperation(): Data {
  if (error) throw new Error('Failed');
  return data;
}
```

### 4. Prefer Readonly Types

```typescript
// ✅ Good
interface User {
  readonly id: string;
  readonly name: string;
}

// ❌ Bad
interface User {
  id: string;
  name: string;
}
```

### 5. Use Const Assertions

```typescript
// ✅ Good
export const SessionType = {
  FOCUS: 'focus',
  BREAK: 'break',
} as const;

export type SessionType = typeof SessionType[keyof typeof SessionType];

// ❌ Bad
export enum SessionType {
  FOCUS = 'focus',
  BREAK = 'break',
}
```

## Common Patterns

### Form Handling with Validation

```typescript
function AddBlockRuleForm() {
  const [domain, setDomain] = useState('');
  const [error, setError] = useState<string | null>(null);
  const createRule = useCreateBlockRule();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validate
    const validation = validateDomain(domain);
    if (!validation.valid) {
      setError(validation.error);
      return;
    }

    // Create rule
    try {
      await createRule.mutateAsync({
        ruleType: 'website',
        target: validation.data,
        scheduleType: 'always',
      });
      setDomain('');
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create rule');
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        value={domain}
        onChange={(e) => setDomain(e.target.value)}
        placeholder="example.com"
      />
      {error && <p className="text-red-500">{error}</p>}
      <button type="submit" disabled={createRule.isPending}>
        {createRule.isPending ? 'Adding...' : 'Add Rule'}
      </button>
    </form>
  );
}
```

### Optimistic Updates

```typescript
const updateRule = useMutation({
  mutationFn: async ({ ruleId, enabled }) => {
    const result = await updateBlockRule({ ruleId, updates: { enabled } });
    if (!result.success) throw new Error(result.error.type);
    return result.data;
  },
  onMutate: async ({ ruleId, enabled }) => {
    // Cancel outgoing queries
    await queryClient.cancelQueries({ queryKey: ['block-rules'] });

    // Snapshot previous value
    const previous = queryClient.getQueryData(['block-rules']);

    // Optimistically update
    queryClient.setQueryData(['block-rules'], (old) => {
      return old.map((rule) =>
        rule.id === ruleId ? { ...rule, enabled } : rule
      );
    });

    return { previous };
  },
  onError: (err, variables, context) => {
    // Rollback on error
    queryClient.setQueryData(['block-rules'], context.previous);
  },
});
```

## Troubleshooting

### Type Errors with Tauri

If you get type errors with Tauri commands, ensure:
1. Command names match between frontend and backend
2. Parameter types are correctly mapped in `tauri.ts`
3. Return types include Result wrapper

### Zustand Not Persisting

Check:
1. `name` is unique in persist config
2. `partialize` includes the fields you want to persist
3. localStorage is accessible in Tauri

### React Query Stale Data

Adjust:
1. `staleTime` - How long data is considered fresh
2. `cacheTime` - How long unused data stays in cache
3. `refetchInterval` - Auto-refetch frequency

## Resources

- [Architecture Documentation](./FRONTEND_ARCHITECTURE.md)
- [Type Definitions](./src/types/)
- [Backend Architecture Plan](./FocusFlow-Architecture-Plan.md)
