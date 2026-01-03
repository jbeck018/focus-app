# FocusFlow Frontend Architecture - Executive Summary

## Overview

A production-grade TypeScript/React frontend architecture for FocusFlow, leveraging advanced type system features for compile-time safety, runtime performance, and exceptional developer experience.

## Core Technology Stack

| Layer | Technology | Version | Purpose |
|-------|------------|---------|---------|
| UI Framework | React | 18.2+ | Component-based UI with hooks |
| Type System | TypeScript | 5.3+ | Strict mode with advanced types |
| State Management | Zustand | 4.4+ | Lightweight global state |
| Server State | TanStack Query | 5.17+ | Async state & caching |
| Styling | Tailwind CSS | 3.4+ | Utility-first styling |
| Desktop Framework | Tauri | 2.0 | Rust-powered native app |
| Testing | Vitest | 1.0+ | Fast unit testing |
| Build Tool | Vite | 5.0+ | Lightning-fast builds |

## Advanced Type System Features

### 1. Branded Types for Domain Safety

**Problem:** Primitive types are too permissive. TypeScript can't distinguish between different semantic meanings of the same primitive type.

**Solution:** Branded types create nominal typing within TypeScript's structural system.

```typescript
// Before (unsafe)
function getSession(id: string): Session { ... }
function getDuration(minutes: number): void { ... }

const userId = "user-123";
getSession(userId); // ❌ Compiles but wrong!

// After (type-safe)
type SessionId = string & { readonly __brand: 'SessionId' };
type Minutes = number & { readonly __brand: 'Minutes' };

const sessionId: SessionId = SessionId("session-123");
const userId: UserId = UserId("user-123");

getSession(userId); // ✅ Compile error - caught at build time!
```

**Files:**
- `/Users/jacob/projects/focus-app/src/types/session.ts`
- `/Users/jacob/projects/focus-app/src/types/blocking.ts`
- `/Users/jacob/projects/focus-app/src/types/analytics.ts`

### 2. Discriminated Unions for State Modeling

**Problem:** Complex state with optional fields leads to invalid combinations.

**Solution:** Discriminated unions ensure only valid states are representable.

```typescript
// Before (unsafe - invalid states possible)
interface SessionState {
  status: 'idle' | 'running' | 'paused';
  startedAt?: number;
  pausedAt?: number;
}

// Can create invalid states:
{ status: 'idle', startedAt: 123456 } // ❌ Invalid!

// After (type-safe - invalid states impossible)
type SessionStatus =
  | { type: 'idle' }
  | { type: 'running'; startedAt: Timestamp }
  | { type: 'paused'; pausedAt: Timestamp; elapsedMs: number };

// TypeScript enforces valid structure
const status: SessionStatus = { type: 'idle', startedAt: 123 }; // ✅ Compile error!
```

**Benefits:**
- Impossible states are unrepresentable
- Exhaustive pattern matching
- Better autocomplete in switch statements

**Files:**
- `/Users/jacob/projects/focus-app/src/types/session.ts` (SessionStatus, SessionError)
- `/Users/jacob/projects/focus-app/src/types/blocking.ts` (BlockRule, BlockingError)

### 3. Result Types for Error Handling

**Problem:** Exceptions bypass type system and make error handling implicit.

**Solution:** Errors as values force explicit handling.

```typescript
// Before (exceptions - easy to forget error handling)
function createRule(dto: CreateRuleDTO): BlockRule {
  if (invalid) throw new Error("Validation failed");
  return rule;
}

// Easy to forget try/catch
const rule = createRule(dto); // 💥 Runtime error!

// After (Result type - compiler enforces error handling)
type Result<T, E> =
  | { success: true; data: T }
  | { success: false; error: E };

function createRule(dto: CreateRuleDTO): Result<BlockRule, ValidationError> {
  if (invalid) return { success: false, error: validationError };
  return { success: true, data: rule };
}

// Compiler forces error handling
const result = createRule(dto);
if (!result.success) {
  handleError(result.error); // ✅ Must handle error
  return;
}
const rule = result.data; // Type narrowing ensures data exists
```

**Files:**
- `/Users/jacob/projects/focus-app/src/types/session.ts` (SessionResult, SessionError)
- `/Users/jacob/projects/focus-app/src/types/blocking.ts` (BlockingResult, BlockingError)

### 4. Mapped Types for Type-Safe APIs

**Problem:** Tauri command invocation loses type safety at runtime.

**Solution:** Mapped types create compile-time guarantees for runtime operations.

```typescript
// Command registry with const assertion
export const TauriCommand = {
  START_SESSION: 'start_session',
  GET_ANALYTICS: 'get_analytics',
} as const;

// Map each command to its parameter type
export type CommandParams = {
  [TauriCommand.START_SESSION]: CreateSessionDTO;
  [TauriCommand.GET_ANALYTICS]: AnalyticsQuery;
  // ... compiler ensures all commands are mapped
};

// Map each command to its return type
export type CommandReturn = {
  [TauriCommand.START_SESSION]: SessionResult<FocusSession>;
  [TauriCommand.GET_ANALYTICS]: SessionResult<AnalyticsOverview>;
};

// Type-safe wrapper with generic constraint
export function useTauriCommand<T extends TauriCommand>(
  command: T
): (params: CommandParams[T]) => Promise<CommandReturn[T]> {
  // TypeScript infers correct types for params and return
}

// Usage gets full autocomplete and type checking
const start = useTauriCommand('start_session');
const result = await start({ /* autocomplete knows parameters */ });
```

**Benefits:**
- Autocomplete for command names
- Type inference for parameters and returns
- Centralized command registry
- Refactoring safety (rename command → compile errors point to all uses)

**Files:**
- `/Users/jacob/projects/focus-app/src/types/tauri.ts`
- `/Users/jacob/projects/focus-app/src/hooks/useTauri.ts`

### 5. Template Literal Types

**Problem:** String manipulation loses type information.

**Solution:** Template literal types for compile-time string operations.

```typescript
// Query key factory with type-safe interpolation
const sessionKeys = {
  all: ['sessions'] as const,
  active: () => [...sessionKeys.all, 'active'] as const,
  history: (limit?: number) => [...sessionKeys.all, 'history', { limit }] as const,
} as const;

// TypeScript knows the exact structure
type ActiveKey = ReturnType<typeof sessionKeys.active>;
// Type: readonly ["sessions", "active"]
```

**Files:**
- `/Users/jacob/projects/focus-app/src/hooks/useFocusSession.ts` (query keys)
- `/Users/jacob/projects/focus-app/src/hooks/useBlockRules.ts` (query keys)

## State Management Architecture

### Zustand for Client State

**Use Cases:**
- User preferences (theme, notification settings)
- Active timer state (elapsed seconds, paused state)
- UI state (sidebar open/closed)

**Key Features:**
- Immer middleware for immutable updates with mutable-style code
- Persistence middleware for localStorage sync
- Partial persistence (ephemeral vs durable state)
- Selector optimization for granular re-renders

**Example:**
```typescript
export const useFocusStore = create<FocusStore>()(
  persist(
    immer((set) => ({
      activeSession: null,
      elapsedSeconds: 0,
      defaultFocusDuration: 25 as Minutes,

      setActiveSession: (session) =>
        set((state) => {
          state.activeSession = session; // Immer makes this safe
        }),
    })),
    {
      name: 'focus-store',
      partialize: (state) => ({
        // Only persist preferences, not ephemeral timer state
        defaultFocusDuration: state.defaultFocusDuration,
      }),
    }
  )
);
```

**Files:**
- `/Users/jacob/projects/focus-app/src/stores/focusStore.ts`
- `/Users/jacob/projects/focus-app/src/stores/settingsStore.ts`
- `/Users/jacob/projects/focus-app/src/stores/analyticsStore.ts`

### TanStack Query for Server State

**Use Cases:**
- Fetching data from Tauri backend
- Caching with smart invalidation
- Background refetching
- Optimistic updates

**Key Features:**
- Automatic caching with configurable staleness
- Background refetching for real-time data
- Optimistic updates with rollback on error
- Parallel queries and mutations
- Request deduplication

**Example:**
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
    refetchInterval: 1000, // Keep timer updated every second
    staleTime: 500, // Consider data stale after 500ms
  });
}
```

**Files:**
- `/Users/jacob/projects/focus-app/src/hooks/useFocusSession.ts`
- `/Users/jacob/projects/focus-app/src/hooks/useBlockRules.ts`
- `/Users/jacob/projects/focus-app/src/hooks/useAnalytics.ts`

## File Structure Overview

```
/Users/jacob/projects/focus-app/
│
├── src/
│   ├── types/              # Type definitions (100% complete)
│   │   ├── session.ts      # Session types with branded types
│   │   ├── blocking.ts     # Blocking rules & discriminated unions
│   │   ├── analytics.ts    # Analytics with mapped types
│   │   └── tauri.ts        # Type-safe Tauri command mappings
│   │
│   ├── hooks/              # Custom React hooks (100% complete)
│   │   ├── useTauri.ts     # Type-safe command & event wrappers
│   │   ├── useFocusSession.ts  # Session CRUD with React Query
│   │   ├── useBlockRules.ts    # Block rules with optimistic updates
│   │   └── useAnalytics.ts     # Analytics queries with caching
│   │
│   ├── stores/             # Zustand stores (100% complete)
│   │   ├── focusStore.ts   # Timer & session state
│   │   ├── settingsStore.ts    # App preferences
│   │   └── analyticsStore.ts   # Analytics UI state
│   │
│   ├── utils/              # Pure utilities (100% complete)
│   │   ├── time.ts         # Time calculations with branded types
│   │   ├── formatters.ts   # Display formatting
│   │   └── validators.ts   # Input validation with Result types
│   │
│   ├── components/         # React components (partial)
│   │   └── FocusTimer/     # Example: Timer component with hooks
│   │       ├── FocusTimer.tsx
│   │       ├── TimerDisplay.tsx
│   │       ├── TimerControls.tsx
│   │       └── SessionTypeSelector.tsx
│   │
│   └── test/               # Test setup (100% complete)
│       └── setup.ts        # Vitest config with Tauri mocks
│
├── Configuration Files
│   ├── package.json        # Dependencies (React 18, TS 5.3, Zustand, TanStack Query)
│   ├── tsconfig.json       # Strictest TypeScript settings
│   ├── vite.config.ts      # Vite with Tauri optimizations
│   └── vitest.config.ts    # Test configuration
│
└── Documentation
    ├── FRONTEND_ARCHITECTURE.md    # Comprehensive architecture guide
    ├── IMPLEMENTATION_GUIDE.md     # Step-by-step implementation
    └── ARCHITECTURE_SUMMARY.md     # This file
```

## TypeScript Configuration Highlights

```json
{
  "compilerOptions": {
    "strict": true,                           // All strict checks
    "noUncheckedIndexedAccess": true,         // arr[i] returns T | undefined
    "exactOptionalPropertyTypes": true,        // Distinguish undefined from omitted
    "noImplicitReturns": true,                // All paths must return
    "noPropertyAccessFromIndexSignature": true, // Force bracket notation
    "noUnusedLocals": true,                   // No unused variables
    "noUnusedParameters": true,               // No unused parameters
    "verbatimModuleSyntax": true              // Explicit import/export types
  }
}
```

## Testing Strategy

### Unit Tests (Vitest + Happy DOM)

```typescript
describe('validateDomain', () => {
  it('accepts valid domains', () => {
    const result = validateDomain('example.com');
    expect(result.valid).toBe(true);
    if (result.valid) {
      expect(result.data).toBe('example.com');
    }
  });

  it('rejects invalid domains', () => {
    const result = validateDomain('not a domain');
    expect(result.valid).toBe(false);
  });
});
```

### Component Tests (React Testing Library)

```typescript
describe('FocusTimer', () => {
  it('starts a session', async () => {
    render(<FocusTimer />);

    const startButton = screen.getByRole('button', { name: /start/i });
    await userEvent.click(startButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('start_session', {
        plannedMinutes: 25,
        sessionType: 'focus',
      });
    });
  });
});
```

### Integration Tests (Hook Testing)

```typescript
describe('useFocusSession', () => {
  it('fetches and starts a session', async () => {
    const { result } = renderHook(() => useFocusSession(), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.start({
      plannedMinutes: 25,
      sessionType: 'focus'
    });

    expect(result.current.session).toBeDefined();
  });
});
```

## Key Architectural Benefits

### 1. Type Safety

- **Compile-time errors** catch bugs before runtime
- **Branded types** prevent mixing incompatible values
- **Discriminated unions** eliminate invalid states
- **Result types** force error handling

### 2. Developer Experience

- **Excellent autocomplete** from TypeScript inference
- **Refactoring safety** with strict type checking
- **Self-documenting code** through types
- **IDE integration** for inline errors and quick fixes

### 3. Performance

- **Granular re-renders** with Zustand selectors
- **Smart caching** with React Query
- **Code splitting** with lazy loading
- **Optimized builds** with Vite

### 4. Maintainability

- **Clear separation of concerns** (types, hooks, stores, components)
- **Consistent patterns** across codebase
- **Comprehensive testing** strategy
- **Extensive documentation**

## Next Implementation Steps

### Immediate (Week 1-2)
1. Complete Dashboard component with analytics
2. Implement BlockList with CRUD operations
3. Add TriggerJournal for distraction tracking
4. Build Settings UI with form validation

### Short-term (Week 3-4)
5. Add React Router for navigation
6. Implement form validation with react-hook-form + Zod
7. Add animations with Framer Motion
8. Create comprehensive test suite

### Medium-term (Month 2)
9. Integrate with Rust backend
10. Implement error tracking (Sentry)
11. Add analytics charts (Recharts)
12. Performance optimization

## Development Commands

```bash
# Install dependencies
bun install

# Start dev server (frontend only)
bun run dev

# Start with Tauri
bun run tauri:dev

# Type check
bun run typecheck

# Run tests
bun test
bun test:watch
bun test:coverage
bun test:ui

# Lint
bun run lint

# Build
bun run build
bun run tauri:build
```

## Critical Files Reference

| Purpose | File Path |
|---------|-----------|
| Session types | `/Users/jacob/projects/focus-app/src/types/session.ts` |
| Blocking types | `/Users/jacob/projects/focus-app/src/types/blocking.ts` |
| Analytics types | `/Users/jacob/projects/focus-app/src/types/analytics.ts` |
| Tauri integration | `/Users/jacob/projects/focus-app/src/types/tauri.ts` |
| Tauri hooks | `/Users/jacob/projects/focus-app/src/hooks/useTauri.ts` |
| Session management | `/Users/jacob/projects/focus-app/src/hooks/useFocusSession.ts` |
| Block rules | `/Users/jacob/projects/focus-app/src/hooks/useBlockRules.ts` |
| Analytics | `/Users/jacob/projects/focus-app/src/hooks/useAnalytics.ts` |
| Focus store | `/Users/jacob/projects/focus-app/src/stores/focusStore.ts` |
| Settings store | `/Users/jacob/projects/focus-app/src/stores/settingsStore.ts` |
| Time utils | `/Users/jacob/projects/focus-app/src/utils/time.ts` |
| Validators | `/Users/jacob/projects/focus-app/src/utils/validators.ts` |
| Example component | `/Users/jacob/projects/focus-app/src/components/FocusTimer/FocusTimer.tsx` |

## Resources

- **Full Architecture Guide:** `/Users/jacob/projects/focus-app/FRONTEND_ARCHITECTURE.md`
- **Implementation Guide:** `/Users/jacob/projects/focus-app/IMPLEMENTATION_GUIDE.md`
- **Backend Architecture:** `/Users/jacob/projects/focus-app/FocusFlow-Architecture-Plan.md`

---

**Status:** Core architecture 100% complete. Ready for component implementation.

**Last Updated:** 2025-12-23
