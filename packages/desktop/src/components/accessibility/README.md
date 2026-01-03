# Accessibility Components

This directory contains reusable accessibility components for WCAG 2.1 Level AA compliance.

## Components

### SkipLink
**File:** `skip-link.tsx`
**WCAG:** 2.4.1 Bypass Blocks (Level A)

Allows keyboard users to skip repetitive navigation and jump directly to main content.

**Usage:**
```tsx
import { SkipLink } from '@/components/accessibility/skip-link';

<SkipLink href="#main-content">Skip to main content</SkipLink>
```

**Features:**
- Visually hidden until focused
- First element in tab order
- Smooth scroll to target
- Sets focus on target element

**Behavior:**
- Tab to focus (appears at top-left)
- Enter/Click to activate
- Scrolls to and focuses #main-content

---

### ScreenReaderOnly
**File:** `screen-reader-only.tsx`
**Purpose:** Visually hide content while keeping it accessible to screen readers

**Usage:**
```tsx
import { ScreenReaderOnly } from '@/components/accessibility/screen-reader-only';

<button>
  <IconX />
  <ScreenReaderOnly>Close dialog</ScreenReaderOnly>
</button>
```

**Use Cases:**
- Descriptive labels for icon-only buttons
- Additional context for screen reader users
- Status messages that don't need visual display
- Skip link text

**Note:** Uses the `.sr-only` utility class defined in `index.css`

---

### FocusTrap
**File:** `focus-trap.tsx`
**WCAG:** 2.1.2 No Keyboard Trap (Level A)

Traps keyboard focus within a container, typically used for modal dialogs.

**Usage:**
```tsx
import { FocusTrap } from '@/components/accessibility/focus-trap';

<FocusTrap enabled={isOpen} restoreFocus={true}>
  <DialogContent>
    {/* Dialog content */}
  </DialogContent>
</FocusTrap>
```

**Props:**
```typescript
interface FocusTrapProps {
  children: React.ReactNode;
  enabled?: boolean;           // Enable/disable trap (default: true)
  restoreFocus?: boolean;      // Restore focus on unmount (default: true)
  initialFocus?: RefObject;    // Element to focus on mount
  className?: string;
}
```

**Features:**
- Auto-focus first focusable element on mount
- Tab/Shift+Tab cycles through elements (loops)
- Restores focus to previous element on unmount
- Handles dynamic content changes

**Keyboard:**
- Tab: Move to next focusable element
- Shift+Tab: Move to previous focusable element
- Loops: After last element, returns to first

**Implementation Details:**
- Queries for focusable elements: `a[href]`, `button:not([disabled])`, `input:not([disabled])`, etc.
- Filters out `disabled` and `aria-hidden` elements
- Stores previous active element for restoration

---

## Hooks

### useReducedMotion
**File:** `../hooks/use-reduced-motion.ts`
**WCAG:** 2.3.3 Animation from Interactions (Level AAA)

Detects if user prefers reduced motion (vestibular disorder support).

**Usage:**
```tsx
import { useReducedMotion } from '@/hooks/use-reduced-motion';

function Component() {
  const prefersReducedMotion = useReducedMotion();

  return (
    <div className={prefersReducedMotion ? '' : 'animate-pulse'}>
      {content}
    </div>
  );
}
```

**Returns:** `boolean` - true if user prefers reduced motion

**Media Query:** `(prefers-reduced-motion: reduce)`

---

### useKeyboardNavigation
**File:** `../hooks/use-keyboard-navigation.ts`
**WCAG:** 2.1.1 Keyboard (Level A)

Provides keyboard navigation for complex UI patterns (lists, grids, tabs).

**Usage:**
```tsx
import { useKeyboardNavigation } from '@/hooks/use-keyboard-navigation';

function List({ items }) {
  const { currentIndex, onKeyDown } = useKeyboardNavigation({
    itemCount: items.length,
    onSelect: (index) => handleSelect(items[index]),
    orientation: 'vertical',
  });

  return (
    <div onKeyDown={onKeyDown}>
      {items.map((item, index) => (
        <div
          key={item.id}
          tabIndex={index === currentIndex ? 0 : -1}
          className={index === currentIndex ? 'focused' : ''}
        >
          {item.name}
        </div>
      ))}
    </div>
  );
}
```

**Options:**
```typescript
interface UseKeyboardNavigationOptions {
  itemCount: number;           // Total number of items
  initialIndex?: number;       // Starting index (default: 0)
  onSelect?: (index) => void;  // Called on Enter/Space
  onEscape?: () => void;       // Called on Escape
  orientation?: Orientation;   // 'horizontal' | 'vertical' | 'both'
  loop?: boolean;              // Loop navigation (default: true)
  enabled?: boolean;           // Enable/disable (default: true)
}
```

**Keyboard Shortcuts:**
- **Arrow Keys:** Navigate (direction based on orientation)
- **Home:** Jump to first item
- **End:** Jump to last item
- **Enter/Space:** Select current item
- **Escape:** Cancel/close (if onEscape provided)

---

## Testing

### Keyboard Testing
1. Disconnect mouse
2. Use Tab/Shift+Tab to navigate
3. Verify all interactive elements reachable
4. Test component-specific shortcuts

### Screen Reader Testing
- **macOS:** VoiceOver (Cmd+F5)
- **Windows:** NVDA (free) or JAWS

### Automated Testing
```typescript
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

test('Component has no accessibility violations', async () => {
  const { container } = render(<Component />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

---

## Best Practices

### 1. Semantic HTML First
Use native HTML elements when possible:
```tsx
// ✅ Good - native button
<button onClick={handleClick}>Submit</button>

// ❌ Bad - div button
<div onClick={handleClick} role="button" tabIndex={0}>Submit</div>
```

### 2. ARIA as Enhancement
Only use ARIA when semantic HTML is insufficient:
```tsx
// ✅ Good - label association
<label htmlFor="email">Email</label>
<input id="email" type="email" />

// ❌ Bad - unnecessary ARIA
<div aria-label="Email">
  <input type="email" />
</div>
```

### 3. Focus Management
Always manage focus for dynamic content:
```tsx
// ✅ Good - focus management
function Modal({ onClose }) {
  const closeButtonRef = useRef();

  useEffect(() => {
    closeButtonRef.current?.focus();
  }, []);

  return (
    <FocusTrap>
      <button ref={closeButtonRef} onClick={onClose}>Close</button>
    </FocusTrap>
  );
}
```

### 4. Visible Focus Indicators
Never remove focus outlines without replacement:
```css
/* ❌ Bad - removes focus */
*:focus {
  outline: none;
}

/* ✅ Good - custom focus style */
*:focus-visible {
  outline: none;
  ring: 2px solid blue;
  ring-offset: 2px;
}
```

### 5. Live Regions for Status
Announce dynamic changes to screen readers:
```tsx
// ✅ Good - status announced
<div role="status" aria-live="polite">
  {statusMessage}
</div>
```

---

## ARIA Patterns Reference

### Modal Dialog
```tsx
<div
  role="dialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  aria-describedby="dialog-description"
>
  <h2 id="dialog-title">Title</h2>
  <p id="dialog-description">Description</p>
</div>
```

### Tabs
```tsx
<div role="tablist" aria-label="Settings tabs">
  <button role="tab" aria-selected="true" aria-controls="panel-1">
    Tab 1
  </button>
  <button role="tab" aria-selected="false" aria-controls="panel-2">
    Tab 2
  </button>
</div>
<div role="tabpanel" id="panel-1">Panel 1 content</div>
<div role="tabpanel" id="panel-2" hidden>Panel 2 content</div>
```

### Button with Icon
```tsx
<button aria-label="Close">
  <XIcon aria-hidden="true" />
</button>
```

### Live Region
```tsx
<div role="status" aria-live="polite" aria-atomic="true">
  {announcement}
</div>
```

---

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [WebAIM Screen Reader Testing](https://webaim.org/articles/screenreader_testing/)
- [axe DevTools](https://www.deque.com/axe/devtools/)

---

**Maintained by:** FocusFlow Accessibility Team
**Last Updated:** December 24, 2024
