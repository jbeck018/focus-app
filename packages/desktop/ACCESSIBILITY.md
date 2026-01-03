# FocusFlow Accessibility Documentation

## WCAG 2.1 Level AA Compliance

This document outlines the accessibility features implemented in FocusFlow to ensure compliance with WCAG 2.1 Level AA standards and EU EAA (European Accessibility Act) requirements.

---

## Table of Contents

1. [Overview](#overview)
2. [Implemented Features](#implemented-features)
3. [WCAG Success Criteria Coverage](#wcag-success-criteria-coverage)
4. [Testing Procedures](#testing-procedures)
5. [Known Issues](#known-issues)
6. [Component-Specific Accessibility](#component-specific-accessibility)
7. [Resources](#resources)

---

## Overview

FocusFlow is a productivity application built with accessibility as a core requirement. All interactive elements are keyboard-accessible, screen reader compatible, and provide appropriate visual feedback.

**Compliance Level:** WCAG 2.1 Level AA
**EU EAA Deadline:** June 2025
**Last Updated:** December 2024

---

## Implemented Features

### 1. Keyboard Navigation (WCAG 2.1.1, 2.1.2)

#### Skip to Main Content
- **Location:** `src/components/accessibility/skip-link.tsx`
- **Description:** First focusable element allows keyboard users to bypass navigation
- **Keyboard:** Tab to focus, Enter to activate
- **Implementation:** Visually hidden until focused, smooth scroll to main content

#### Tab Navigation
- **Pattern:** ARIA Tabs with roving tabindex
- **Keyboard Shortcuts:**
  - `Arrow Left/Right`: Navigate between tabs
  - `Home`: Jump to first tab
  - `End`: Jump to last tab
  - `Tab`: Move to tab panel content

#### Dialog/Modal Focus Trap
- **Location:** `src/components/accessibility/focus-trap.tsx`
- **Description:** Traps focus within dialogs, prevents keyboard navigation outside modal
- **Keyboard:**
  - `Tab`: Cycle through focusable elements (loops)
  - `Shift+Tab`: Reverse cycle
  - `Escape`: Close dialog (Radix UI built-in)

#### Button Controls
- **Keyboard:** `Enter` or `Space` to activate
- **Focus Indicators:** Visible 2px ring with 3:1 contrast ratio

### 2. Screen Reader Support (WCAG 4.1.2, 4.1.3)

#### Semantic HTML & ARIA Landmarks
```html
<header role="banner">       <!-- Application header -->
<nav role="navigation">      <!-- Main navigation tabs -->
<main role="main">           <!-- Primary content area -->
```

#### Live Regions for Dynamic Content
- **Timer Updates:** Announces countdown at intervals (5 min, 1 min, 10 sec)
- **Session Status:** Announces start, pause, completion
- **Remaining Sessions:** Polite announcement of daily limit

#### ARIA Labels & Descriptions
- Icon-only buttons: `aria-label` provides context
- Decorative icons: `aria-hidden="true"`
- Complex widgets: `aria-describedby` for additional context
- Form inputs: Explicit label associations

### 3. Visual Accessibility (WCAG 1.4.3, 1.4.11, 2.4.7)

#### Color Contrast
- **Text:** 4.5:1 minimum (7:1 for body text)
- **Interactive elements:** 3:1 minimum
- **Focus indicators:** 3:1 minimum against background
- **Tool:** Chrome DevTools contrast checker

#### Focus Indicators
```css
*:focus-visible {
  outline: none;
  ring: 2px solid var(--ring);
  ring-offset: 2px;
  transition: box-shadow 0.15s ease-in-out;
}
```

#### High Contrast Mode Support
```css
@media (prefers-contrast: high) {
  button, a, input { border: 2px solid currentColor; }
  *:focus-visible { outline: 3px solid currentColor; }
}
```

### 4. Motion & Animation (WCAG 2.3.3)

#### Reduced Motion Support
```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

#### React Hook
```typescript
import { useReducedMotion } from '@/hooks/use-reduced-motion';

const prefersReducedMotion = useReducedMotion();
const animation = prefersReducedMotion ? '' : 'animate-pulse';
```

### 5. Form Accessibility (WCAG 3.3.1, 3.3.2)

#### Label Associations
```tsx
<Label htmlFor="custom-duration">Custom (minutes)</Label>
<Input id="custom-duration" type="number" />
```

#### Error Messages
- Associated with inputs via `aria-describedby`
- Error state indicated with `aria-invalid="true"`
- Visual + programmatic error indication

#### Required Fields
- Visual indicator (asterisk)
- `aria-required="true"` attribute

---

## WCAG Success Criteria Coverage

### Level A (Essential)

| Criterion | Title | Status | Implementation |
|-----------|-------|--------|----------------|
| 1.1.1 | Non-text Content | ✅ | All images have alt text, icons have aria-labels |
| 1.3.1 | Info and Relationships | ✅ | Semantic HTML, ARIA landmarks, label associations |
| 1.3.2 | Meaningful Sequence | ✅ | Logical tab order, heading hierarchy |
| 1.3.3 | Sensory Characteristics | ✅ | No reliance on shape/size/color alone |
| 2.1.1 | Keyboard | ✅ | All functionality available via keyboard |
| 2.1.2 | No Keyboard Trap | ✅ | Focus trap in dialogs with escape hatch |
| 2.2.1 | Timing Adjustable | ✅ | User controls timer duration |
| 2.4.1 | Bypass Blocks | ✅ | Skip to main content link |
| 2.4.2 | Page Titled | ✅ | Document title reflects current view |
| 2.4.4 | Link Purpose | ✅ | All links have descriptive text |
| 3.1.1 | Language of Page | ✅ | `<html lang="en">` |
| 3.2.1 | On Focus | ✅ | No unexpected context changes |
| 3.2.2 | On Input | ✅ | No unexpected context changes |
| 3.3.1 | Error Identification | ✅ | Errors described in text |
| 3.3.2 | Labels or Instructions | ✅ | All inputs have labels |
| 4.1.1 | Parsing | ✅ | Valid HTML/ARIA |
| 4.1.2 | Name, Role, Value | ✅ | Proper ARIA attributes |

### Level AA (Recommended)

| Criterion | Title | Status | Implementation |
|-----------|-------|--------|----------------|
| 1.4.3 | Contrast (Minimum) | ✅ | 4.5:1 text, 3:1 UI components |
| 1.4.5 | Images of Text | ✅ | Text, not images |
| 1.4.11 | Non-text Contrast | ✅ | 3:1 for UI components |
| 1.4.12 | Text Spacing | ✅ | Responds to user text spacing |
| 1.4.13 | Content on Hover | ✅ | Dismissible, hoverable tooltips |
| 2.4.5 | Multiple Ways | ✅ | Tab navigation provides multiple paths |
| 2.4.6 | Headings and Labels | ✅ | Descriptive headings/labels |
| 2.4.7 | Focus Visible | ✅ | Visible focus indicators |
| 3.1.2 | Language of Parts | ✅ | No foreign language content |
| 3.2.3 | Consistent Navigation | ✅ | Tab order consistent |
| 3.2.4 | Consistent Identification | ✅ | Icons/buttons consistent |
| 3.3.3 | Error Suggestion | ✅ | Validation errors suggest fixes |
| 3.3.4 | Error Prevention | ✅ | Confirmation for destructive actions |
| 4.1.3 | Status Messages | ✅ | Live regions for status updates |

### Level AAA (Enhanced - Partial)

| Criterion | Title | Status | Implementation |
|-----------|-------|--------|----------------|
| 2.3.3 | Animation from Interactions | ✅ | Respects prefers-reduced-motion |
| 2.4.8 | Location | ⚠️ | Partial - tab indicates location |
| 2.4.10 | Section Headings | ✅ | Proper heading hierarchy |

**Legend:** ✅ Fully Implemented | ⚠️ Partially Implemented | ❌ Not Implemented

---

## Testing Procedures

### 1. Keyboard Navigation Testing

**Objective:** Verify all functionality is keyboard accessible

**Steps:**
1. Disconnect mouse/trackpad
2. Press `Tab` to navigate through all interactive elements
3. Verify visible focus indicators on each element
4. Test all buttons with `Enter` and `Space`
5. Test dialog navigation:
   - Open dialog
   - `Tab` through all focusable elements
   - Verify focus loops within dialog
   - Press `Escape` to close
   - Verify focus returns to trigger button

**Expected Results:**
- All interactive elements focusable
- Clear visual focus indicator (2px ring)
- No keyboard traps (except intentional in dialogs)
- Logical tab order

### 2. Screen Reader Testing

**Tools:**
- **macOS:** VoiceOver (`Cmd+F5`)
- **Windows:** NVDA (free) or JAWS
- **Chrome:** ChromeVox extension

**Test Scenarios:**

#### A. Navigation
```
1. Start screen reader
2. Navigate to FocusFlow app
3. Verify "Skip to main content" link is announced first
4. Navigate through tabs:
   - Verify role="tab" announced
   - Verify selected state announced
   - Verify tab labels read correctly
5. Navigate to main content
   - Verify landmark announced as "main"
```

#### B. Timer Functionality
```
1. Navigate to "Start Session" button
2. Verify button role and label announced
3. Activate button (Enter/Space)
4. Verify dialog opens and focus moves to dialog
5. Navigate through dialog options:
   - Session type tabs
   - Duration buttons
   - Custom duration input
6. Start timer
7. Verify live region announces:
   - "Focus session started, 25 minutes"
   - Periodic time updates
   - Completion announcement
```

#### C. Forms & Inputs
```
1. Navigate to custom duration input
2. Verify label association ("Custom (minutes)")
3. Enter invalid value (e.g., -5)
4. Verify error message announced
5. Navigate to other form fields
6. Verify all labels read correctly
```

**Expected Results:**
- All text content readable
- Interactive elements have clear labels
- State changes announced (selected, expanded, checked)
- Live regions announce updates without interruption
- No unlabeled buttons or inputs

### 3. Color Contrast Testing

**Tools:**
- Chrome DevTools (Inspect > Accessibility pane)
- WAVE Browser Extension
- axe DevTools

**Steps:**
1. Open Chrome DevTools
2. Inspect each text element
3. Check "Contrast" in Accessibility pane
4. Verify ratios meet minimums:
   - Body text: 4.5:1
   - Large text (18pt+): 3:1
   - UI components: 3:1
   - Focus indicators: 3:1

**Test Cases:**
- Default theme: Light mode text on background
- Dark mode: Dark mode text on background
- Primary button: Text on primary color
- Secondary button: Text on secondary color
- Destructive button: Text on red background
- Disabled states: 3:1 minimum (lower contrast acceptable)
- Focus rings: Against all background colors

### 4. Reduced Motion Testing

**Steps:**
1. **macOS:**
   - System Preferences > Accessibility > Display
   - Check "Reduce motion"
2. **Windows:**
   - Settings > Ease of Access > Display
   - Turn on "Show animations in Windows"
3. Reload FocusFlow app
4. Verify:
   - No pulsing animations
   - No zoom/fade animations
   - Smooth scrolling disabled
   - Instant state transitions

**Expected Results:**
- All animations disabled or significantly reduced
- App remains fully functional
- No flashing or rapid transitions

### 5. Visual Regression Testing

**Tools:**
- Storybook with Chromatic
- Percy.io
- Browser DevTools

**Test Cases:**
1. Focus states for all interactive elements
2. Hover states
3. Active/pressed states
4. Disabled states
5. Error states
6. Loading states

### 6. Automated Testing

**Tools & Scripts:**

```bash
# Install dependencies
npm install --save-dev @axe-core/react jest-axe

# Run accessibility tests
npm run test:a11y
```

**Example Test:**
```typescript
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { FocusTimer } from './FocusTimer';

expect.extend(toHaveNoViolations);

test('FocusTimer has no accessibility violations', async () => {
  const { container } = render(<FocusTimer />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

---

## Known Issues

### Current Limitations

1. **Tab Panel Focus:**
   - Tab panels receive focus (`tabIndex={0}`)
   - May cause confusion for some screen reader users
   - **Mitigation:** Skip links allow bypassing

2. **Dynamic Content Updates:**
   - Some rapid updates may be verbose for screen readers
   - **Mitigation:** Polite live regions, throttled announcements

3. **Color Scheme:**
   - System preference respected, but no manual toggle
   - **Future:** Add theme switcher control

### Planned Improvements

1. **Enhanced Live Regions:**
   - Configurable announcement frequency
   - User preference for verbosity level

2. **Keyboard Shortcuts:**
   - Global shortcuts for timer control
   - Customizable hotkeys
   - Shortcut reference modal (`?` key)

3. **Focus Management:**
   - More granular focus restoration
   - Focus history tracking

4. **High Contrast:**
   - Custom high contrast theme
   - Forced colors mode support

---

## Component-Specific Accessibility

### SkipLink (`skip-link.tsx`)
**WCAG:** 2.4.1 Bypass Blocks (Level A)

```tsx
<SkipLink href="#main-content">Skip to main content</SkipLink>
```

**Features:**
- First in tab order
- Visually hidden until focused
- Smooth scroll to target
- Sets focus on target element

---

### FocusTrap (`focus-trap.tsx`)
**WCAG:** 2.1.2 No Keyboard Trap (Level A)

```tsx
<FocusTrap enabled={true} restoreFocus={true}>
  {children}
</FocusTrap>
```

**Features:**
- Traps focus within container (for modals)
- Tab/Shift+Tab cycles through elements
- Restores focus on unmount
- Handles dynamic content

**Use Cases:**
- Modal dialogs
- Dropdown menus
- Popover panels

---

### useReducedMotion (`use-reduced-motion.ts`)
**WCAG:** 2.3.3 Animation from Interactions (Level AAA)

```tsx
const prefersReducedMotion = useReducedMotion();

return (
  <div className={prefersReducedMotion ? '' : 'animate-pulse'}>
    {content}
  </div>
);
```

**Features:**
- Detects `prefers-reduced-motion` media query
- Updates on system preference change
- SSR-safe (returns false on server)

---

### useKeyboardNavigation (`use-keyboard-navigation.ts`)
**WCAG:** 2.1.1 Keyboard (Level A)

```tsx
const { currentIndex, handlers } = useKeyboardNavigation({
  itemCount: items.length,
  onSelect: (index) => selectItem(index),
  orientation: 'vertical',
});
```

**Features:**
- Arrow key navigation
- Home/End support
- Enter/Space activation
- Escape to cancel
- Looping or bounded navigation

---

### Dialog Component (`dialog.tsx`)
**WCAG:** 2.4.3 Focus Order, 4.1.2 Name, Role, Value

**ARIA Attributes:**
- `role="dialog"`
- `aria-modal="true"`
- `aria-labelledby` (references DialogTitle)
- `aria-describedby` (references DialogDescription)

**Keyboard:**
- `Escape`: Close dialog
- `Tab`: Navigate focusable elements
- Focus trap active when open

---

### Tabs Component (`tabs.tsx`)
**WCAG:** 2.1.1 Keyboard, 4.1.2 Name, Role, Value

**ARIA Pattern:**
- `role="tablist"` on container
- `role="tab"` on triggers
- `role="tabpanel"` on content
- `aria-selected` state on active tab

**Keyboard:**
- `Arrow Left/Right`: Navigate tabs
- `Home`: First tab
- `End`: Last tab
- `Tab`: Move to panel

---

### FocusTimer Component (`FocusTimer.tsx`)
**WCAG:** 4.1.3 Status Messages, 1.3.1 Info and Relationships

**Live Regions:**
```tsx
<div role="status" aria-live="polite" aria-atomic="true">
  {announcement}
</div>
```

**Announcements:**
- Session start: "Focus session started, 25 minutes"
- Time updates: "5 minutes remaining"
- Completion: "Focus session completed"
- Pause: "Timer paused"

**Progress:**
```tsx
<Progress
  value={progress}
  aria-label="Session progress: 85%"
/>
```

---

## Resources

### WCAG Documentation
- [WCAG 2.1 Quick Reference](https://www.w3.org/WAI/WCAG21/quickref/)
- [Understanding WCAG 2.1](https://www.w3.org/WAI/WCAG21/Understanding/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)

### Testing Tools
- [axe DevTools](https://www.deque.com/axe/devtools/)
- [WAVE Browser Extension](https://wave.webaim.org/extension/)
- [Lighthouse (Chrome)](https://developers.google.com/web/tools/lighthouse)
- [VoiceOver User Guide](https://support.apple.com/guide/voiceover/welcome/mac)
- [NVDA Screen Reader](https://www.nvaccess.org/)

### EU EAA Resources
- [EU Accessibility Act Overview](https://ec.europa.eu/social/main.jsp?catId=1202)
- [EN 301 549 Standard](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf)

### Component Libraries
- [Radix UI Accessibility](https://www.radix-ui.com/primitives/docs/overview/accessibility)
- [shadcn/ui Documentation](https://ui.shadcn.com/)

---

## Maintenance

### Regular Testing Schedule
- **Weekly:** Automated axe-core tests in CI/CD
- **Bi-weekly:** Manual keyboard navigation review
- **Monthly:** Screen reader testing (VoiceOver/NVDA)
- **Quarterly:** Full WCAG audit with external auditor

### Accessibility Champion
**Role:** Ensure ongoing compliance and address issues
**Contact:** [accessibility@focusflow.app]

### Reporting Issues
Users can report accessibility issues via:
- In-app feedback form
- Email: accessibility@focusflow.app
- GitHub Issues (tag: `accessibility`)

**SLA:** Critical accessibility issues resolved within 48 hours

---

## Changelog

### v1.0.0 (December 2024)
- ✅ Initial WCAG 2.1 Level AA implementation
- ✅ Skip to main content link
- ✅ Focus trap for dialogs
- ✅ Live regions for timer announcements
- ✅ Reduced motion support
- ✅ Semantic HTML and ARIA landmarks
- ✅ Comprehensive keyboard navigation
- ✅ Screen reader optimizations
- ✅ Color contrast compliance
- ✅ Focus indicators (3:1 contrast minimum)

---

**Last Updated:** December 24, 2024
**Version:** 1.0.0
**Compliance:** WCAG 2.1 Level AA
