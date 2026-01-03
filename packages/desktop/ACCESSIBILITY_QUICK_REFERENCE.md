# Accessibility Quick Reference Card

A cheat sheet for developers to implement accessible components in FocusFlow.

---

## Common ARIA Patterns

### Buttons

```tsx
// ✅ Text button (no ARIA needed)
<button onClick={handleClick}>Save</button>

// ✅ Icon button
<button aria-label="Close dialog" onClick={handleClose}>
  <XIcon aria-hidden="true" />
</button>

// ✅ Toggle button
<button
  aria-pressed={isActive}
  onClick={toggle}
>
  {isActive ? 'On' : 'Off'}
</button>

// ✅ Loading state
<button disabled aria-busy={isLoading}>
  {isLoading ? 'Saving...' : 'Save'}
</button>
```

---

### Links

```tsx
// ✅ Standard link
<a href="/dashboard">Dashboard</a>

// ✅ Link with icon
<a href="/settings" aria-label="Go to settings">
  <SettingsIcon aria-hidden="true" />
  Settings
</a>

// ✅ External link
<a href="https://example.com" target="_blank" rel="noopener noreferrer">
  Example
  <ScreenReaderOnly>(opens in new window)</ScreenReaderOnly>
</a>
```

---

### Forms

```tsx
// ✅ Input with label
<label htmlFor="email">Email address</label>
<input id="email" type="email" required />

// ✅ Input with description
<label htmlFor="password">Password</label>
<input
  id="password"
  type="password"
  aria-describedby="password-hint"
  required
/>
<span id="password-hint">At least 8 characters</span>

// ✅ Input with error
<label htmlFor="username">Username</label>
<input
  id="username"
  type="text"
  aria-invalid={hasError}
  aria-describedby={hasError ? "username-error" : undefined}
/>
{hasError && (
  <span id="username-error" role="alert">
    Username is required
  </span>
)}
```

---

### Dialogs/Modals

```tsx
import { Dialog, DialogContent, DialogTitle, DialogDescription } from '@/components/ui/dialog';

// ✅ Accessible dialog
<Dialog open={isOpen} onOpenChange={setIsOpen}>
  <DialogContent aria-describedby="dialog-description">
    <DialogTitle>Confirm action</DialogTitle>
    <DialogDescription id="dialog-description">
      Are you sure you want to delete this item?
    </DialogDescription>
    <button onClick={handleConfirm}>Confirm</button>
    <button onClick={handleCancel}>Cancel</button>
  </DialogContent>
</Dialog>
```

---

### Tabs

```tsx
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';

// ✅ Accessible tabs
<Tabs value={activeTab} onValueChange={setActiveTab}>
  <TabsList aria-label="Settings tabs">
    <TabsTrigger value="account" aria-label="Account settings">
      Account
    </TabsTrigger>
    <TabsTrigger value="privacy" aria-label="Privacy settings">
      Privacy
    </TabsTrigger>
  </TabsList>
  <TabsContent value="account">
    <h2>Account Settings</h2>
    {/* Content */}
  </TabsContent>
  <TabsContent value="privacy">
    <h2>Privacy Settings</h2>
    {/* Content */}
  </TabsContent>
</Tabs>
```

---

### Live Regions

```tsx
// ✅ Polite announcement (doesn't interrupt)
<div role="status" aria-live="polite" aria-atomic="true">
  {statusMessage}
</div>

// ✅ Assertive announcement (interrupts)
<div role="alert" aria-live="assertive" aria-atomic="true">
  {errorMessage}
</div>

// ✅ Visually hidden live region
<div
  role="status"
  aria-live="polite"
  aria-atomic="true"
  className="sr-only"
>
  {announcement}
</div>
```

---

### Lists

```tsx
// ✅ Unordered list
<ul>
  <li>Item 1</li>
  <li>Item 2</li>
</ul>

// ✅ Ordered list
<ol>
  <li>Step 1</li>
  <li>Step 2</li>
</ol>

// ✅ Description list
<dl>
  <dt>Total sessions</dt>
  <dd>127</dd>
  <dt>Current streak</dt>
  <dd>5 days</dd>
</dl>
```

---

### Progress Indicators

```tsx
import { Progress } from '@/components/ui/progress';

// ✅ Progress bar
<Progress
  value={progress}
  aria-label={`Upload progress: ${progress}%`}
/>

// ✅ Loading spinner
<div role="status" aria-label="Loading">
  <Spinner aria-hidden="true" />
  <ScreenReaderOnly>Loading...</ScreenReaderOnly>
</div>
```

---

## Keyboard Navigation

### Standard Shortcuts

| Key | Action |
|-----|--------|
| Tab | Focus next element |
| Shift+Tab | Focus previous element |
| Enter | Activate button/link |
| Space | Activate button, toggle checkbox |
| Escape | Close dialog/modal, cancel action |
| Arrow keys | Navigate tabs, lists, menus |
| Home | First item |
| End | Last item |

### Custom Shortcuts

```tsx
import { useEffect } from 'react';

function Component() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Ctrl+S or Cmd+S to save
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);
}
```

---

## Focus Management

### Focus Trap (for dialogs)

```tsx
import { FocusTrap } from '@/components/accessibility/focus-trap';

<FocusTrap enabled={isOpen} restoreFocus={true}>
  <DialogContent>
    {/* Content */}
  </DialogContent>
</FocusTrap>
```

### Manual Focus

```tsx
import { useRef, useEffect } from 'react';

function Component() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Focus input on mount
    inputRef.current?.focus();
  }, []);

  return <input ref={inputRef} />;
}
```

### Focus Visible

```tsx
// Focus only on keyboard interaction (not mouse click)
<button className="focus-visible:ring-2">
  Click me
</button>
```

---

## Screen Reader Support

### Visually Hidden

```tsx
import { ScreenReaderOnly } from '@/components/accessibility/screen-reader-only';

// Icon button with SR text
<button>
  <TrashIcon aria-hidden="true" />
  <ScreenReaderOnly>Delete item</ScreenReaderOnly>
</button>
```

### Skip Links

```tsx
import { SkipLink } from '@/components/accessibility/skip-link';

<SkipLink href="#main-content">
  Skip to main content
</SkipLink>

{/* Later in the document */}
<main id="main-content" tabIndex={-1}>
  {/* Main content */}
</main>
```

### Landmarks

```tsx
// Header
<header role="banner">
  <h1>FocusFlow</h1>
</header>

// Navigation
<nav role="navigation" aria-label="Main navigation">
  {/* Nav items */}
</nav>

// Main content
<main role="main">
  {/* Main content */}
</main>

// Sidebar
<aside role="complementary" aria-label="Related information">
  {/* Sidebar content */}
</aside>

// Footer
<footer role="contentinfo">
  {/* Footer content */}
</footer>
```

---

## Reduced Motion

### CSS

```css
@media (prefers-reduced-motion: reduce) {
  .animated {
    animation: none !important;
    transition: none !important;
  }
}
```

### React Hook

```tsx
import { useReducedMotion } from '@/hooks/use-reduced-motion';

function Component() {
  const prefersReducedMotion = useReducedMotion();

  return (
    <div className={prefersReducedMotion ? '' : 'animate-pulse'}>
      Content
    </div>
  );
}
```

---

## Color & Contrast

### Minimum Contrast Ratios

| Element | Minimum Ratio | Level |
|---------|--------------|-------|
| Normal text | 4.5:1 | AA |
| Large text (18pt+) | 3:1 | AA |
| UI components | 3:1 | AA |
| Focus indicators | 3:1 | AA |

### Checking Contrast

1. **Chrome DevTools:**
   - Inspect element
   - Accessibility pane
   - Check "Contrast" section

2. **Browser Extensions:**
   - axe DevTools
   - WAVE

### Don't Rely on Color Alone

```tsx
// ❌ Bad - color only
<span style={{ color: 'red' }}>Error</span>

// ✅ Good - icon + color + text
<span className="text-destructive">
  <AlertIcon aria-hidden="true" />
  Error: Invalid input
</span>
```

---

## Testing Checklist

### Quick Test (5 minutes)

- [ ] Tab through page (keyboard only)
- [ ] All interactive elements focusable?
- [ ] Focus indicators visible?
- [ ] No keyboard traps?
- [ ] Run axe DevTools scan

### Full Test (30 minutes)

- [ ] Complete keyboard navigation
- [ ] Screen reader test (VoiceOver/NVDA)
- [ ] Color contrast check
- [ ] Enable reduced motion
- [ ] Test with zoom (200%)

---

## Common Mistakes

### ❌ Missing alt text

```tsx
// ❌ Bad
<img src="logo.png" />

// ✅ Good
<img src="logo.png" alt="FocusFlow logo" />

// ✅ Decorative
<img src="divider.png" alt="" aria-hidden="true" />
```

### ❌ Placeholder as label

```tsx
// ❌ Bad
<input placeholder="Email address" />

// ✅ Good
<label htmlFor="email">Email address</label>
<input id="email" placeholder="you@example.com" />
```

### ❌ Click div

```tsx
// ❌ Bad
<div onClick={handleClick}>Click me</div>

// ✅ Good
<button onClick={handleClick}>Click me</button>
```

### ❌ Removing focus outline

```css
/* ❌ Bad */
*:focus {
  outline: none;
}

/* ✅ Good */
*:focus-visible {
  outline: none;
  ring: 2px solid blue;
}
```

### ❌ Auto-playing animations

```tsx
// ❌ Bad
<div className="animate-spin">Loading</div>

// ✅ Good
const prefersReducedMotion = useReducedMotion();
<div className={prefersReducedMotion ? '' : 'animate-spin'}>
  Loading
</div>
```

---

## Resources

### Tools
- [axe DevTools](https://www.deque.com/axe/devtools/) - Automated testing
- [WAVE](https://wave.webaim.org/extension/) - Visual feedback
- [Lighthouse](https://developers.google.com/web/tools/lighthouse) - Audits

### Guidelines
- [WCAG 2.1 Quick Reference](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [Radix UI Accessibility](https://www.radix-ui.com/primitives/docs/overview/accessibility)

### Screen Readers
- **macOS:** VoiceOver (Cmd+F5)
- **Windows:** [NVDA](https://www.nvaccess.org/) (free)
- **Chrome:** ChromeVox extension

---

## Need Help?

1. Check `/packages/desktop/ACCESSIBILITY.md` for detailed documentation
2. Use `/packages/desktop/ACCESSIBILITY_TESTING_CHECKLIST.md` for testing
3. Review component examples in `/src/components/accessibility/README.md`
4. Ask in #accessibility Slack channel
5. Email: accessibility@focusflow.app

---

**Last Updated:** December 24, 2024
**Version:** 1.0.0
