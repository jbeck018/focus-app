# FocusFlow Frontend Architecture

## Overview

This document describes the TypeScript/React frontend architecture for FocusFlow, a Tauri-based desktop productivity application.

## Technology Stack

- **React 18.2+** - UI library with hooks and concurrent features
- **TypeScript 5.3+** - Strict mode with advanced type system features
- **Zustand 4.4+** - Lightweight state management with persistence
- **TanStack Query 5.17+** - Async state management and caching
- **Tailwind CSS 3.4+** - Utility-first styling
- **Tauri 2.0** - Desktop app framework with Rust backend
- **Vitest** - Fast unit testing with React Testing Library
- **Vite 5** - Build tool optimized for Tauri

## Architecture Principles

### 1. Type Safety First

All code uses TypeScript's strictest settings:
- Branded types for domain primitives
- Discriminated unions for state modeling
- Result types for error handling (no exceptions)
- Template literal types for type-safe string operations
- Mapped types for flexible transformations

### 2. Functional Core, Imperative Shell

- Pure business logic in utilities and validators
- Side effects isolated to hooks and Tauri commands
- Immutable data structures with Immer
- Composable functions with clear contracts

### 3. Component Composition

- Small, focused components with single responsibility
- Custom hooks for reusable logic
- Render props and compound components for flexibility
- Controlled components with unidirectional data flow

### 4. Performance Optimization

- React Query for intelligent caching and background updates
- Zustand selectors for granular re-renders
- Code splitting with lazy loading
- Memoization where beneficial (not premature)

## Directory Structure

```
src/
├── components/          # React components
│   ├── FocusTimer/      # Pomodoro timer UI
│   ├── Dashboard/       # Analytics dashboard
│   ├── BlockList/       # Block rules management
│   ├── TriggerJournal/  # Distraction tracking
│   ├── Settings/        # App preferences
│   └── common/          # Shared UI components
│
├── hooks/               # Custom React hooks
│   ├── useTauri.ts      # Type-safe Tauri command wrapper
│   ├── useFocusSession.ts  # Session management
│   ├── useBlockRules.ts    # Blocking rules CRUD
│   └── useAnalytics.ts     # Analytics queries
│
├── stores/              # Zustand global state
│   ├── focusStore.ts    # Active session & timer state
│   ├── settingsStore.ts # App settings & preferences
│   └── analyticsStore.ts # Analytics UI state
│
├── types/               # TypeScript type definitions
│   ├── session.ts       # Session types with branded types
│   ├── blocking.ts      # Blocking rules & events
│   ├── analytics.ts     # Analytics & metrics types
│   └── tauri.ts         # Tauri command type mappings
│
├── utils/               # Pure utility functions
│   ├── time.ts          # Time calculations & formatting
│   ├── formatters.ts    # Display formatters
│   └── validators.ts    # Input validation with type guards
│
└── test/                # Test utilities
    ├── setup.ts         # Vitest configuration
    ├── mocks.ts         # Mock data generators
    └── helpers.tsx      # Test helpers & wrappers
```

## Type System Design

### Branded Types for Domain Safety

```typescript
// Instead of primitive types, use branded types
export type SessionId = string & { readonly __brand: 'SessionId' };
export type Minutes = number & { readonly __brand: 'Minutes' };
export type Domain = string & { readonly __brand: 'Domain' };

// With constructors that enforce invariants
export const Minutes = (m: number): Minutes => {
  if (m < 0) throw new Error('Minutes cannot be negative');
  return m as Minutes;
};
```

**Benefits:**
- Compile-time prevention of mixing incompatible values
- Self-documenting code (type name explains intent)
- Runtime validation at construction
- IDE autocomplete distinguishes types

### Discriminated Unions for State

```typescript
// Session status as discriminated union
export type SessionStatus =
  | { type: 'idle' }
  | { type: 'running'; startedAt: Timestamp }
  | { type: 'paused'; pausedAt: Timestamp; elapsedMs: number }
  | { type: 'completed'; completedAt: Timestamp };

// Type-safe exhaustive matching
function handleStatus(status: SessionStatus) {
  switch (status.type) {
    case 'idle':
      return 'Not started';
    case 'running':
      return `Started at ${status.startedAt}`;
    case 'paused':
      return `Paused with ${status.elapsedMs}ms elapsed`;
    case 'completed':
      return `Completed at ${status.completedAt}`;
  }
}
```

### Result Types for Error Handling

```typescript
// Instead of throwing exceptions
export type SessionResult<T> =
  | { success: true; data: T }
  | { success: false; error: SessionError };

// Discriminated error types
export type SessionError =
  | { type: 'validation'; field: string; message: string }
  | { type: 'not_found'; sessionId: SessionId }
  | { type: 'database'; message: string };

// Usage forces error handling
const result = await startSession(params);
if (!result.success) {
  // Handle specific error types
  switch (result.error.type) {
    case 'validation':
      showError(`${result.error.field}: ${result.error.message}`);
      break;
    case 'not_found':
      showError('Session not found');
      break;
  }
  return;
}

// Type narrowing ensures data is available
const session = result.data;
```

## State Management Strategy

### Zustand for Global State

Use Zustand for:
- User preferences (persisted to localStorage)
- Active session state (ephemeral)
- UI state (theme, sidebar visibility)
- Sound settings

**Example:**

```typescript
export const useFocusStore = create<FocusStore>()(
  persist(
    immer((set) => ({
      activeSession: null,
      setActiveSession: (session) =>
        set((state) => {
          state.activeSession = session;
        }),
    })),
    {
      name: 'focus-store',
      partialize: (state) => ({
        // Only persist preferences, not session
        defaultFocusDuration: state.defaultFocusDuration,
      }),
    }
  )
);
```

### TanStack Query for Server State

Use TanStack Query for:
- Fetching from Tauri backend
- Caching with smart invalidation
- Background refetching
- Optimistic updates

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
    refetchInterval: 1000, // Update timer every second
  });
}
```

### When to Use Which?

| State Type | Solution | Example |
|------------|----------|---------|
| User preferences | Zustand + persist | Theme, notification settings |
| Server/Tauri data | TanStack Query | Session history, analytics |
| UI state (local) | useState | Form inputs, modals |
| UI state (global) | Zustand | Sidebar open/closed |
| Derived data | useMemo | Computed metrics |

## Tauri Integration Pattern

### Type-Safe Command Invocation

All Tauri commands are typed using mapped types:

```typescript
// Define command registry
export const TauriCommand = {
  START_SESSION: 'start_session',
  GET_ANALYTICS: 'get_analytics',
} as const;

// Map commands to parameters
export type CommandParams = {
  [TauriCommand.START_SESSION]: CreateSessionDTO;
  [TauriCommand.GET_ANALYTICS]: AnalyticsQuery;
};

// Map commands to return types
export type CommandReturn = {
  [TauriCommand.START_SESSION]: SessionResult<FocusSession>;
  [TauriCommand.GET_ANALYTICS]: SessionResult<AnalyticsOverview>;
};

// Type-safe wrapper
export function useTauriCommand<T extends TauriCommand>(command: T) {
  return useCallback(
    async (params: CommandParams[T]): Promise<CommandReturn[T]> => {
      return await invoke<CommandReturn[T]>(command, params);
    },
    [command]
  );
}
```

**Benefits:**
- Compile-time verification of parameters
- Autocomplete for command names
- Type inference for return values
- Centralized command registry

### Event Listener Pattern

```typescript
export function useTauriEvent<T extends TauriEvent>(
  event: T,
  handler: (payload: EventPayload[T]) => void
) {
  useEffect(() => {
    const unlisten = listen<EventPayload[T]>(event, (e) => {
      handler(e.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [event, handler]);
}

// Usage
useTauriEvent('session:completed', (session) => {
  showNotification(`Session completed: ${session.id}`);
});
```

## Testing Strategy

### Unit Tests with Vitest

Test utilities and pure functions in isolation:

```typescript
import { describe, it, expect } from 'vitest';
import { validateDomain } from '@utils/validators';

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
    if (!result.valid) {
      expect(result.error).toContain('Invalid');
    }
  });
});
```

### Component Tests with React Testing Library

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { FocusTimer } from '@components/FocusTimer';
import { mockInvoke } from '@test/setup';

describe('FocusTimer', () => {
  it('starts a session', async () => {
    const queryClient = new QueryClient();

    mockInvoke.mockResolvedValue({
      success: true,
      data: { id: '1', plannedMinutes: 25 },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <FocusTimer />
      </QueryClientProvider>
    );

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

### Integration Tests

Test hooks with React Query:

```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useFocusSession } from '@hooks/useFocusSession';
import { mockInvoke } from '@test/setup';

describe('useFocusSession', () => {
  it('fetches and starts a session', async () => {
    const queryClient = new QueryClient();
    const wrapper = ({ children }) => (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );

    mockInvoke.mockResolvedValue({
      success: true,
      data: { id: '1', plannedMinutes: 25 },
    });

    const { result } = renderHook(() => useFocusSession(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await result.current.start({ plannedMinutes: 25, sessionType: 'focus' });

    expect(result.current.session).toBeDefined();
  });
});
```

## Error Boundary Strategy

### Global Error Boundary

```typescript
class GlobalErrorBoundary extends React.Component<Props, State> {
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log to error tracking service (e.g., Sentry)
    console.error('Uncaught error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return <ErrorFallback error={this.state.error} />;
    }

    return this.props.children;
  }
}
```

### Query Error Boundaries

```typescript
import { QueryErrorResetBoundary } from '@tanstack/react-query';
import { ErrorBoundary } from 'react-error-boundary';

function App() {
  return (
    <QueryErrorResetBoundary>
      {({ reset }) => (
        <ErrorBoundary
          onReset={reset}
          fallbackRender={({ error, resetErrorBoundary }) => (
            <div>
              Error: {error.message}
              <button onClick={resetErrorBoundary}>Try again</button>
            </div>
          )}
        >
          <Dashboard />
        </ErrorBoundary>
      )}
    </QueryErrorResetBoundary>
  );
}
```

## Performance Best Practices

### 1. Zustand Selectors

```typescript
// Bad: Subscribes to entire store
const store = useFocusStore();

// Good: Subscribe to specific slice
const activeSession = useFocusStore((state) => state.activeSession);

// Better: Use predefined selectors
const activeSession = useFocusStore(selectActiveSession);
```

### 2. React Query Caching

```typescript
// Configure intelligent defaults
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60000, // 1 minute
      cacheTime: 300000, // 5 minutes
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});
```

### 3. Code Splitting

```typescript
const Dashboard = lazy(() => import('@components/Dashboard'));
const Settings = lazy(() => import('@components/Settings'));

<Suspense fallback={<LoadingSpinner />}>
  <Routes>
    <Route path="/dashboard" element={<Dashboard />} />
    <Route path="/settings" element={<Settings />} />
  </Routes>
</Suspense>
```

## Next Steps

1. Implement core components (FocusTimer, Dashboard, BlockList)
2. Add form handling with react-hook-form
3. Implement routing with react-router-dom
4. Add animation with Framer Motion
5. Create comprehensive test suite
6. Set up CI/CD pipeline
7. Implement error tracking (Sentry)

## Additional Resources

- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)
- [React Query Docs](https://tanstack.com/query/latest)
- [Zustand Guide](https://docs.pmnd.rs/zustand)
- [Tauri Guides](https://tauri.app/v1/guides/)
- [Testing Library](https://testing-library.com/react)
