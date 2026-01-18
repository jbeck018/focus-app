# Permission System Testing Guide

## 🧪 Testing Strategy

This guide covers comprehensive testing for the permission system including unit tests, integration tests, and manual testing scenarios.

## 📋 Test Coverage Checklist

### Context Provider
- [ ] Loads permission status on mount
- [ ] Handles loading state correctly
- [ ] Exposes correct computed values (hasFullPermissions, isDegraded)
- [ ] recheckPermissions() triggers new check
- [ ] Handles backend errors gracefully
- [ ] Updates all consumers when state changes

### Permission Modal
- [ ] Auto-shows when degraded (uncontrolled mode)
- [ ] Respects "don't show again" localStorage setting
- [ ] Controlled mode works correctly
- [ ] Platform detection works for all OS
- [ ] "Check Again" button triggers recheck
- [ ] "Open Guide" opens external URL
- [ ] "Continue Anyway" closes modal
- [ ] "Don't show again" persists to localStorage
- [ ] Displays correct error messages
- [ ] Shows correct status icons (check/X)

### Degraded Mode Banner
- [ ] Shows when permissions degraded
- [ ] Hides when permissions fully functional
- [ ] Animates in smoothly
- [ ] Correct color for degraded vs non-functional
- [ ] "Fix This" button triggers callback
- [ ] Displays correct missing features

### Hook (usePermissions)
- [ ] Returns correct permission status
- [ ] Throws error when used outside provider
- [ ] Re-renders components when status changes

## 🔬 Unit Tests

### Context Provider Tests

```tsx
// permission-status-context.test.tsx
import { renderHook, waitFor } from '@testing-library/react';
import { PermissionStatusProvider } from './permission-status-context';
import { usePermissions } from './use-permissions';

// Mock Tauri invoke
const mockInvoke = jest.fn();
jest.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('PermissionStatusProvider', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <PermissionStatusProvider>{children}</PermissionStatusProvider>
  );

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should load permissions on mount', async () => {
    const mockStatus = {
      hosts_file_writable: true,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'fully_functional',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.permissionStatus).toEqual(mockStatus);
    expect(result.current.hasFullPermissions).toBe(true);
    expect(result.current.isDegraded).toBe(false);
  });

  it('should handle degraded state', async () => {
    const mockStatus = {
      hosts_file_writable: true,
      hosts_file_error: null,
      process_monitoring_available: false,
      process_monitoring_error: 'Cannot access processes',
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.hasFullPermissions).toBe(false);
    expect(result.current.isDegraded).toBe(true);
  });

  it('should handle non-functional state', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: 'Permission denied',
      process_monitoring_available: false,
      process_monitoring_error: 'Cannot access processes',
      overall_status: 'non_functional',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.hasFullPermissions).toBe(false);
    expect(result.current.isDegraded).toBe(true);
  });

  it('should handle backend errors gracefully', async () => {
    mockInvoke.mockRejectedValue(new Error('Backend error'));

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.permissionStatus?.overall_status).toBe('non_functional');
  });

  it('should recheck permissions when requested', async () => {
    const mockStatus1 = {
      hosts_file_writable: false,
      hosts_file_error: 'Permission denied',
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    const mockStatus2 = {
      hosts_file_writable: true,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'fully_functional',
    };

    mockInvoke
      .mockResolvedValueOnce(mockStatus1)
      .mockResolvedValueOnce(mockStatus2);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.isDegraded).toBe(true);

    // Trigger recheck
    await result.current.recheckPermissions();

    await waitFor(() => {
      expect(result.current.hasFullPermissions).toBe(true);
    });

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });
});
```

### Permission Modal Tests

```tsx
// permission-modal.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { PermissionModal } from './permission-modal';
import { PermissionStatusProvider } from './permission-status-context';

const mockInvoke = jest.fn();
jest.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

const mockOpen = jest.fn();
jest.mock('@tauri-apps/plugin-shell', () => ({
  open: mockOpen,
}));

describe('PermissionModal', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    localStorage.clear();
  });

  const renderWithProvider = (ui: React.ReactElement) => {
    return render(
      <PermissionStatusProvider>{ui}</PermissionStatusProvider>
    );
  };

  it('should not show when permissions are fully functional', async () => {
    const mockStatus = {
      hosts_file_writable: true,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'fully_functional',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    renderWithProvider(<PermissionModal />);

    await waitFor(() => {
      expect(screen.queryByText(/Permission Required/i)).not.toBeInTheDocument();
    });
  });

  it('should auto-show when degraded (uncontrolled)', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: 'Permission denied',
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    renderWithProvider(<PermissionModal />);

    await waitFor(() => {
      expect(screen.getByText(/Permission Required/i)).toBeInTheDocument();
    });
  });

  it('should respect "don\'t show again" setting', async () => {
    localStorage.setItem('focusflow_dont_show_permission_modal', 'true');

    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: 'Permission denied',
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    renderWithProvider(<PermissionModal />);

    await waitFor(() => {
      expect(screen.queryByText(/Permission Required/i)).not.toBeInTheDocument();
    });
  });

  it('should persist "don\'t show again" when checked', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: null,
      process_monitoring_available: false,
      process_monitoring_error: null,
      overall_status: 'non_functional',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    renderWithProvider(<PermissionModal />);

    await waitFor(() => {
      expect(screen.getByText(/Permission Required/i)).toBeInTheDocument();
    });

    // Check the checkbox
    const checkbox = screen.getByLabelText(/Don't show this warning again/i);
    fireEvent.click(checkbox);

    // Close modal
    const continueButton = screen.getByText(/Continue Anyway/i);
    fireEvent.click(continueButton);

    // Check localStorage
    expect(localStorage.getItem('focusflow_dont_show_permission_modal')).toBe('true');
  });

  it('should trigger recheck on "Check Again"', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    renderWithProvider(<PermissionModal />);

    await waitFor(() => {
      expect(screen.getByText(/Permission Required/i)).toBeInTheDocument();
    });

    const checkAgainButton = screen.getByText(/Check Again/i);
    fireEvent.click(checkAgainButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });
  });

  it('should open external guide on "Open Setup Guide"', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    renderWithProvider(<PermissionModal />);

    await waitFor(() => {
      expect(screen.getByText(/Permission Required/i)).toBeInTheDocument();
    });

    const guideButton = screen.getByText(/Open Setup Guide/i);
    fireEvent.click(guideButton);

    expect(mockOpen).toHaveBeenCalledWith('https://focusflow.app/docs/permissions');
  });

  it('should work in controlled mode', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const onOpenChange = jest.fn();

    renderWithProvider(
      <PermissionModal open={true} onOpenChange={onOpenChange} />
    );

    await waitFor(() => {
      expect(screen.getByText(/Permission Required/i)).toBeInTheDocument();
    });

    const continueButton = screen.getByText(/Continue Anyway/i);
    fireEvent.click(continueButton);

    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
```

### Degraded Mode Banner Tests

```tsx
// degraded-mode-banner.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { DegradedModeBanner } from './degraded-mode-banner';
import { PermissionStatusProvider } from './permission-status-context';

const mockInvoke = jest.fn();
jest.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('DegradedModeBanner', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  const renderWithProvider = (ui: React.ReactElement) => {
    return render(
      <PermissionStatusProvider>{ui}</PermissionStatusProvider>
    );
  };

  it('should not show when fully functional', async () => {
    const mockStatus = {
      hosts_file_writable: true,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'fully_functional',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const onFixClick = jest.fn();
    renderWithProvider(<DegradedModeBanner onFixClick={onFixClick} />);

    await waitFor(() => {
      expect(screen.queryByText(/Limited/i)).not.toBeInTheDocument();
    });
  });

  it('should show when degraded', async () => {
    const mockStatus = {
      hosts_file_writable: true,
      hosts_file_error: null,
      process_monitoring_available: false,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const onFixClick = jest.fn();
    renderWithProvider(<DegradedModeBanner onFixClick={onFixClick} />);

    await waitFor(() => {
      expect(screen.getByText(/limited/i)).toBeInTheDocument();
    });
  });

  it('should show correct status for non-functional', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: null,
      process_monitoring_available: false,
      process_monitoring_error: null,
      overall_status: 'non_functional',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const onFixClick = jest.fn();
    renderWithProvider(<DegradedModeBanner onFixClick={onFixClick} />);

    await waitFor(() => {
      expect(screen.getByText(/unavailable/i)).toBeInTheDocument();
    });
  });

  it('should trigger onFixClick when button clicked', async () => {
    const mockStatus = {
      hosts_file_writable: false,
      hosts_file_error: null,
      process_monitoring_available: true,
      process_monitoring_error: null,
      overall_status: 'degraded',
    };

    mockInvoke.mockResolvedValue(mockStatus);

    const onFixClick = jest.fn();
    renderWithProvider(<DegradedModeBanner onFixClick={onFixClick} />);

    await waitFor(() => {
      expect(screen.getByText(/Fix This/i)).toBeInTheDocument();
    });

    const fixButton = screen.getByText(/Fix This/i);
    fireEvent.click(fixButton);

    expect(onFixClick).toHaveBeenCalledTimes(1);
  });
});
```

## 🎭 Manual Testing Scenarios

### Scenario 1: Fresh Install (No Permissions)

**Setup:**
1. Fresh app install
2. No hosts file permissions
3. No process monitoring permissions

**Expected:**
- [ ] Permission modal shows on startup
- [ ] Shows "NON_FUNCTIONAL" status
- [ ] Both features show red X icons
- [ ] Platform-specific instructions displayed
- [ ] "Check Again" button works
- [ ] "Open Setup Guide" opens browser
- [ ] "Continue Anyway" closes modal
- [ ] Degraded banner appears at bottom
- [ ] Banner shows both missing features
- [ ] Banner is red/destructive style

### Scenario 2: Partial Permissions (Hosts Only)

**Setup:**
1. Hosts file permission granted
2. No process monitoring permission

**Expected:**
- [ ] Permission modal shows on startup
- [ ] Shows "DEGRADED" status
- [ ] Hosts file shows green check
- [ ] Process monitoring shows red X
- [ ] Only process monitoring instructions shown
- [ ] Banner shows amber/warning style
- [ ] Banner text mentions "app blocking"

### Scenario 3: All Permissions Granted

**Setup:**
1. All permissions granted
2. First time startup

**Expected:**
- [ ] No modal shown
- [ ] No banner shown
- [ ] App works normally

### Scenario 4: "Don't Show Again" Persistence

**Setup:**
1. Start with degraded state
2. Check "Don't show again"
3. Close modal
4. Restart app

**Expected:**
- [ ] Modal doesn't show on restart
- [ ] Banner still shows (persistent)
- [ ] LocalStorage has correct value
- [ ] Clicking "Fix This" opens modal
- [ ] Modal works in controlled mode

### Scenario 5: Permission Fix Flow

**Setup:**
1. Start with no permissions
2. Follow instructions to grant permissions
3. Click "Check Again"

**Expected:**
- [ ] Button shows loading spinner
- [ ] Backend rechecks permissions
- [ ] UI updates to show granted permissions
- [ ] If all granted: modal auto-closes
- [ ] If all granted: banner auto-hides
- [ ] Toast/success message (optional)

### Scenario 6: Platform Detection

**Test on each platform:**
- [ ] macOS: Shows macOS-specific instructions
- [ ] Windows: Shows Windows-specific instructions
- [ ] Linux: Shows Linux-specific instructions
- [ ] Instructions are accurate and actionable

### Scenario 7: Error Display

**Setup:**
1. Backend returns specific error messages

**Expected:**
- [ ] Error messages displayed under each permission
- [ ] Messages are user-friendly
- [ ] Technical details visible but not scary
- [ ] Helps user understand the issue

### Scenario 8: Responsive Design

**Test on different screen sizes:**
- [ ] Desktop (1920x1080): Full layout
- [ ] Laptop (1366x768): Compact layout
- [ ] Tablet (768x1024): Stacked buttons
- [ ] Mobile (375x667): Full mobile layout
- [ ] Modal scrolls if content too tall

### Scenario 9: Dark Mode

**Test in both themes:**
- [ ] Modal readable in dark mode
- [ ] Banner readable in dark mode
- [ ] Colors appropriate for theme
- [ ] Icons visible in both modes

### Scenario 10: Keyboard Navigation

**Test with keyboard only:**
- [ ] Tab to all interactive elements
- [ ] Enter/Space activates buttons
- [ ] Escape closes modal
- [ ] Focus trap works in modal
- [ ] Focus restored on modal close
- [ ] Banner has proper focus management

## ♿ Accessibility Testing

### Screen Reader Testing

**Test with VoiceOver (macOS) or NVDA (Windows):**
- [ ] Modal announced when shown
- [ ] Permission status clearly read
- [ ] Instructions readable and logical
- [ ] Buttons have clear labels
- [ ] Banner has proper role/aria-live
- [ ] Error messages announced

### Keyboard Testing
- [ ] All functionality accessible via keyboard
- [ ] No keyboard traps (except modal)
- [ ] Logical tab order
- [ ] Visual focus indicators clear

### Color Contrast
- [ ] All text meets WCAG AA standards
- [ ] Icons distinguishable without color
- [ ] Status communicated beyond color

## 🔄 Integration Testing

### With Real Backend

```bash
# Test with mock Rust responses
cargo test permission_checks

# Test full integration
npm run dev
```

**Test cases:**
1. Backend returns fully functional → No UI shown
2. Backend returns degraded → Banner + Modal shown
3. Backend returns non-functional → Red warning shown
4. Backend throws error → Graceful error state
5. Backend slow response → Loading state shown

## 📊 Performance Testing

### Metrics to Monitor
- [ ] Initial permission check: < 100ms
- [ ] Recheck: < 100ms
- [ ] Modal render time: < 50ms
- [ ] Banner animation: smooth 60fps
- [ ] Context re-renders: Only when state changes

### Tools
```bash
# React DevTools Profiler
# Check for unnecessary re-renders

# Lighthouse
# Check accessibility score (should be 100)

# Chrome DevTools Performance
# Check for jank during animations
```

## 🐛 Bug Testing

### Edge Cases
- [ ] Rapid clicking "Check Again"
- [ ] Opening/closing modal quickly
- [ ] Multiple instances of hook (should share state)
- [ ] Provider unmount during check
- [ ] Network errors during Tauri invoke
- [ ] LocalStorage quota exceeded
- [ ] Browser blocks localStorage
- [ ] Platform detection fails (fallback to macOS)

### Error Handling
- [ ] Backend unreachable
- [ ] Invalid response format
- [ ] Null/undefined values
- [ ] Extremely long error messages
- [ ] Unicode in error messages
- [ ] Simultaneous recheck calls

## 📝 Test Checklist Summary

### Before Release
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Manual testing completed on all platforms
- [ ] Accessibility audit complete (WCAG AA)
- [ ] Performance metrics within targets
- [ ] Edge cases handled gracefully
- [ ] Error states tested
- [ ] Documentation accurate
- [ ] Backend integration verified
- [ ] LocalStorage handling tested

### Regression Testing
When making changes, re-test:
- [ ] Core functionality (provider, hook, modal, banner)
- [ ] Platform detection
- [ ] LocalStorage persistence
- [ ] Accessibility features
- [ ] One manual test on each platform

## 🎯 Success Criteria

A successful permission system should:
1. ✅ Never block the user from using the app
2. ✅ Clearly communicate what's not working
3. ✅ Provide actionable steps to fix issues
4. ✅ Respect user preferences (dismissal)
5. ✅ Be accessible to all users
6. ✅ Perform well (no jank)
7. ✅ Handle errors gracefully
8. ✅ Work on all supported platforms
