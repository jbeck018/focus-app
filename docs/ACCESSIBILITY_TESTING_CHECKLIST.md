# FocusFlow Accessibility Testing Checklist

Use this checklist before each release to ensure WCAG 2.1 Level AA compliance.

---

## 🎯 Pre-Test Setup

- [ ] **Environment:** Test on macOS, Windows, and Linux
- [ ] **Browsers:** Chrome, Firefox, Safari, Edge (latest versions)
- [ ] **Screen Readers:**
  - [ ] macOS VoiceOver
  - [ ] Windows NVDA (free)
  - [ ] Windows JAWS (if available)
- [ ] **Tools Installed:**
  - [ ] axe DevTools browser extension
  - [ ] WAVE browser extension
  - [ ] Chrome Lighthouse

---

## 1️⃣ Keyboard Navigation (WCAG 2.1.1, 2.1.2)

### Tab Navigation
- [ ] Press Tab repeatedly to navigate through all interactive elements
- [ ] All buttons, links, inputs are reachable
- [ ] Tab order is logical (left-to-right, top-to-bottom)
- [ ] Shift+Tab navigates in reverse order
- [ ] Skip to main content link appears first (when focused)

### Focus Indicators
- [ ] All focused elements show visible focus ring
- [ ] Focus ring has at least 2px width
- [ ] Focus ring has 3:1 contrast against background
- [ ] Focus ring not obscured by other elements

### Navigation Tabs
- [ ] **Tab key** moves focus to tab list
- [ ] **Arrow Left/Right** navigates between tabs
- [ ] **Home** key jumps to first tab
- [ ] **End** key jumps to last tab
- [ ] **Enter/Space** activates selected tab
- [ ] Active tab indicated visually and programmatically

### Timer Controls
- [ ] "Start Session" button activatable with Enter/Space
- [ ] Dialog opens and focus moves to first element
- [ ] Tab cycles through dialog elements (focus trap)
- [ ] Escape closes dialog
- [ ] Focus returns to "Start Session" button after close
- [ ] "Pause/Resume" button activatable with Enter/Space
- [ ] "Stop" button activatable with Enter/Space

### Forms & Inputs
- [ ] Tab moves between form fields
- [ ] Labels clearly associated with inputs
- [ ] Error messages appear on invalid input
- [ ] Submit buttons activatable with Enter
- [ ] No keyboard traps (can always escape)

---

## 2️⃣ Screen Reader Testing (WCAG 4.1.2, 4.1.3)

### VoiceOver (macOS)
**Activate:** Cmd+F5

#### General Navigation
- [ ] Start VoiceOver
- [ ] Navigate to FocusFlow app (Ctrl+Option+Arrow keys)
- [ ] "Skip to main content" link announced first
- [ ] Link text clearly describes destination
- [ ] Activating link jumps to main content

#### Landmarks
- [ ] Header landmark announced: "banner"
- [ ] Navigation landmark announced: "navigation, Main navigation"
- [ ] Main content landmark announced: "main"
- [ ] Landmarks allow quick navigation (Ctrl+Option+U, then arrows)

#### Tabs Navigation
- [ ] Tab role announced: "tab, [Name]"
- [ ] Selected state announced: "selected"
- [ ] Tab count announced: "1 of 7"
- [ ] Tab panel role announced: "tabpanel"

#### Timer Screen
- [ ] "Ready to Focus?" heading announced
- [ ] "Start Session" button role and label clear
- [ ] Activate button (Ctrl+Option+Space)
- [ ] Dialog role announced: "dialog, Start a Focus Session"
- [ ] Dialog description read: "[X] sessions remaining today"

#### Dialog Interaction
- [ ] Focus moved to first interactive element
- [ ] Session type tabs:
  - [ ] "Focus" tab announced with role and state
  - [ ] "Break" tab announced with role and state
- [ ] Duration buttons:
  - [ ] Button role announced
  - [ ] Label includes duration (e.g., "Set duration to 25 min")
  - [ ] Pressed state announced for selected button
- [ ] Custom duration input:
  - [ ] Label announced: "Custom (minutes)"
  - [ ] Input type announced: "number"
  - [ ] Current value announced

#### Live Regions (Critical)
- [ ] Start timer
- [ ] Announcement: "Focus session started, 25 minutes" (without interruption)
- [ ] Wait for time update (5 min interval)
- [ ] Announcement: "5 minutes remaining"
- [ ] Pause timer
- [ ] Announcement: "Timer paused"
- [ ] Resume timer
- [ ] Announcement: "Focus session started, [X] minutes" (resume)
- [ ] Let timer complete
- [ ] Announcement: "Focus session completed"

#### Timer Display
- [ ] Time display role: "timer, Focus timer"
- [ ] Time remaining announced: "Time remaining: 24:59"
- [ ] Progress bar announced: "Session progress: 5%"

#### Session Limit
- [ ] Status announced: "2 of 3 sessions remaining today"
- [ ] Role: "status" (polite live region)

### NVDA (Windows)
**Activate:** Ctrl+Alt+N

- [ ] Repeat all VoiceOver tests above
- [ ] Verify landmarks navigable (Insert+F7)
- [ ] Verify headings navigable (Insert+F7, then arrows)
- [ ] Test browse mode (default) and focus mode (Insert+Space)

---

## 3️⃣ Color Contrast (WCAG 1.4.3, 1.4.11)

### Chrome DevTools Method
1. Right-click element > Inspect
2. Go to "Accessibility" pane
3. Check "Contrast" section

### Test All Text Elements
- [ ] **Body text** (foreground on background): **4.5:1** minimum
- [ ] **Large text** (18pt+ or 14pt bold): **3:1** minimum
- [ ] **Primary button text**: **4.5:1** minimum
- [ ] **Secondary button text**: **4.5:1** minimum
- [ ] **Destructive button text**: **4.5:1** minimum
- [ ] **Tab labels** (inactive): **4.5:1** minimum
- [ ] **Tab labels** (active): **4.5:1** minimum
- [ ] **Placeholder text**: **4.5:1** minimum
- [ ] **Error messages**: **4.5:1** minimum
- [ ] **Success messages**: **4.5:1** minimum

### Test UI Component Contrast
- [ ] **Focus ring** vs background: **3:1** minimum
- [ ] **Button borders**: **3:1** minimum
- [ ] **Input borders**: **3:1** minimum
- [ ] **Progress bar**: **3:1** minimum
- [ ] **Card borders**: **3:1** minimum
- [ ] **Dividers**: **3:1** minimum

### Dark Mode
- [ ] Repeat all contrast tests in dark mode
- [ ] Toggle: System Preferences > Appearance > Dark

### High Contrast Mode (Windows)
- [ ] Enable: Settings > Ease of Access > High Contrast
- [ ] Verify all UI elements visible
- [ ] Verify borders rendered
- [ ] Verify focus indicators visible

---

## 4️⃣ Reduced Motion (WCAG 2.3.3)

### macOS
1. System Preferences > Accessibility > Display
2. Check "Reduce motion"
3. Reload FocusFlow

### Windows
1. Settings > Ease of Access > Display
2. Turn off "Show animations in Windows"
3. Reload FocusFlow

### Verification
- [ ] No pulsing timer animation
- [ ] No zoom/fade dialog animations
- [ ] No smooth scrolling (instant jumps)
- [ ] Progress bar updates instantly
- [ ] Tab transitions instant
- [ ] App remains fully functional

---

## 5️⃣ Automated Testing

### axe DevTools
1. Install browser extension
2. Open DevTools > axe DevTools tab
3. Click "Scan ALL of my page"

**Expected:**
- [ ] **0 Critical issues**
- [ ] **0 Serious issues**
- [ ] Review/dismiss minor issues (if justified)

### Lighthouse
1. Open Chrome DevTools
2. Go to "Lighthouse" tab
3. Select "Accessibility" only
4. Click "Generate report"

**Expected:**
- [ ] **Accessibility score ≥ 95%**
- [ ] No failed audits in "Accessibility" section

### WAVE
1. Install browser extension
2. Click WAVE icon
3. Review report

**Expected:**
- [ ] **0 Errors**
- [ ] **0 Contrast Errors**
- [ ] Review Alerts (may have false positives)

---

## 6️⃣ Visual Inspection

### Focus States
- [ ] All buttons have visible focus ring
- [ ] All links have visible focus ring
- [ ] All inputs have visible focus ring
- [ ] Tab triggers have visible focus ring
- [ ] Focus ring not clipped by overflow

### Hover States
- [ ] All interactive elements respond to hover
- [ ] Hover doesn't hide essential content
- [ ] Cursor changes to pointer on interactive elements

### Active/Pressed States
- [ ] Buttons show pressed state on click
- [ ] Tab triggers show active state when selected
- [ ] Pressed state visible before state change completes

### Disabled States
- [ ] Disabled buttons visually distinct
- [ ] Disabled inputs visually distinct
- [ ] Disabled elements not in tab order
- [ ] `aria-disabled` attribute present (or `disabled`)

### Error States
- [ ] Invalid inputs show error styling
- [ ] Error messages visible near input
- [ ] Error icon/indicator present
- [ ] `aria-invalid="true"` attribute set

---

## 7️⃣ Mobile Responsiveness

### Touch Targets (WCAG 2.5.5)
- [ ] All buttons ≥ 44x44 CSS pixels
- [ ] Adequate spacing between touch targets
- [ ] No overlapping interactive elements

### Screen Reader (iOS VoiceOver)
- [ ] Triple-click home button (or side button)
- [ ] Navigate with left/right swipes
- [ ] Verify all elements announced correctly

---

## 8️⃣ Forms & Input Validation

### Label Associations
- [ ] Click label, input receives focus
- [ ] Screen reader announces label with input
- [ ] Visual association clear

### Required Fields
- [ ] Visual indicator (asterisk or text)
- [ ] `aria-required="true"` or `required` attribute
- [ ] Screen reader announces "required"

### Error Messages
- [ ] Displayed on blur or submit
- [ ] Associated via `aria-describedby`
- [ ] `aria-invalid="true"` when error present
- [ ] Clear instructions to fix error
- [ ] Screen reader announces error

### Success States
- [ ] Confirmation message displayed
- [ ] Success announced to screen reader (live region)
- [ ] Visual success indicator (checkmark, green border)

---

## 9️⃣ Dialogs & Modals

### Focus Management
- [ ] Opening dialog moves focus to first element
- [ ] Focus trapped within dialog
- [ ] Tab cycles through focusable elements
- [ ] Shift+Tab cycles in reverse
- [ ] Cannot tab to background content

### Escape Functionality
- [ ] Escape key closes dialog
- [ ] Close button visible and focusable
- [ ] Clicking overlay closes dialog (optional)
- [ ] Focus returns to trigger element on close

### ARIA Attributes
- [ ] `role="dialog"` present
- [ ] `aria-modal="true"` present
- [ ] `aria-labelledby` references title
- [ ] `aria-describedby` references description (if present)

### Screen Reader
- [ ] Dialog role announced
- [ ] Dialog title announced
- [ ] Dialog description announced (if present)
- [ ] Close button label clear ("Close dialog")

---

## 🔟 Page-Specific Tests

### Timer Page
- [ ] Keyboard navigate to "Start Session" button
- [ ] Activate with Enter/Space
- [ ] Navigate dialog with Tab
- [ ] Select session type with arrows
- [ ] Select duration preset with Enter/Space
- [ ] Enter custom duration (keyboard only)
- [ ] Submit and start timer
- [ ] Pause/Resume with keyboard
- [ ] Stop with keyboard
- [ ] Verify all actions announced to screen reader

### Dashboard Page
- [ ] Charts have accessible labels
- [ ] Data tables keyboard navigable
- [ ] Statistics announced by screen reader
- [ ] No information conveyed by color alone

### Calendar Page
- [ ] Date picker keyboard accessible
- [ ] Arrow keys navigate dates
- [ ] Enter selects date
- [ ] Screen reader announces selected date

### Journal Page
- [ ] Text editor keyboard accessible
- [ ] Save button keyboard accessible
- [ ] Entries list keyboard navigable
- [ ] Screen reader reads entry content

### Settings Page
- [ ] All settings keyboard accessible
- [ ] Toggle switches operable with Space
- [ ] Dropdowns operable with arrows
- [ ] Changes announced to screen reader

---

## 1️⃣1️⃣ Documentation Review

- [ ] ACCESSIBILITY.md is up to date
- [ ] All components documented
- [ ] ARIA patterns explained
- [ ] Keyboard shortcuts listed
- [ ] Known issues documented
- [ ] Testing procedures accurate

---

## 1️⃣2️⃣ Regression Testing

Run automated tests:
```bash
npm run test:a11y
```

Expected output:
```
✓ App has no accessibility violations
✓ FocusTimer has no accessibility violations
✓ Dialog has no accessibility violations
✓ Tabs have no accessibility violations
```

---

## 🎉 Sign-Off

**Tester Name:** ___________________________

**Date:** ___________________________

**Version Tested:** ___________________________

**Overall Result:**
- [ ] ✅ **PASS** - All critical items passed
- [ ] ⚠️ **PASS WITH ISSUES** - Minor issues documented below
- [ ] ❌ **FAIL** - Critical issues must be resolved

**Issues Found:**
```
1. [Issue description]
   - Severity: Critical / Serious / Minor
   - WCAG Criterion: [e.g., 2.1.1]
   - Steps to reproduce: [...]
   - Expected: [...]
   - Actual: [...]

2. [Issue description]
   ...
```

**Notes:**
```
[Additional observations, recommendations, or context]
```

---

## 📚 Quick Reference

### WCAG Levels
- **Level A:** Essential (must have)
- **Level AA:** Recommended (should have)
- **Level AAA:** Enhanced (nice to have)

### Contrast Ratios
- **Normal text:** 4.5:1 (AA), 7:1 (AAA)
- **Large text:** 3:1 (AA), 4.5:1 (AAA)
- **UI components:** 3:1 (AA)

### Common Keyboard Shortcuts
- **Tab:** Next element
- **Shift+Tab:** Previous element
- **Enter/Space:** Activate button/link
- **Arrow keys:** Navigate within component
- **Escape:** Close dialog/cancel
- **Home/End:** First/last item

### Screen Reader Commands

#### VoiceOver (macOS)
- **Start:** Cmd+F5
- **Navigate:** Ctrl+Option+Arrow
- **Activate:** Ctrl+Option+Space
- **Landmarks:** Ctrl+Option+U

#### NVDA (Windows)
- **Start:** Ctrl+Alt+N
- **Navigate:** Arrow keys (browse mode)
- **Activate:** Enter
- **Elements list:** Insert+F7

---

**Last Updated:** December 24, 2024
**Version:** 1.0.0
