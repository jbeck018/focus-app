# FocusFlow Onboarding Implementation

This document describes the complete onboarding flow implementation for FocusFlow, a Tauri 2.0 desktop productivity app built on the Indistractable framework by Nir Eyal.

## Overview

The onboarding wizard is a multi-step process that:
1. Welcomes users and collects their name
2. Explains the 4 pillars of the Indistractable framework
3. Sets up initial app and website blocking
4. Configures default focus session preferences
5. Provides a quick tutorial of core features
6. Saves all settings to SQLite database and localStorage

## Architecture

### Frontend (React + TypeScript)

**Hook**: `/packages/desktop/src/hooks/use-onboarding.ts`
- Custom React hook managing onboarding state
- Handles navigation between steps
- Manages data persistence to localStorage
- Communicates with Rust backend via Tauri commands
- Includes `useNeedsOnboarding()` hook to check completion status

**Main Wizard**: `/packages/desktop/src/features/onboarding/onboarding-wizard.tsx`
- Orchestrates the entire onboarding flow
- Renders appropriate step component based on current state
- Shows completion screen with summary
- Handles final submission to backend

**Step Wrapper**: `/packages/desktop/src/features/onboarding/step-wrapper.tsx`
- Reusable wrapper component for all steps (DRY pattern)
- Provides consistent layout, progress bar, and navigation
- Props: title, description, progress, navigation handlers

**Step Components**: `/packages/desktop/src/features/onboarding/steps/`

1. **welcome-step.tsx**
   - Collects user's name
   - Displays welcome message and feature overview
   - Shows 4 feature cards with icons
   - Validates name input before allowing next

2. **pillars-step.tsx**
   - Explains the 4 pillars of Indistractable
   - Interactive expandable cards for each pillar
   - Shows examples and action items
   - Educational step with no data collection

3. **blocklist-step.tsx**
   - Tabbed interface for apps and websites
   - Preset suggestions for common distractions
   - Custom input for additional items
   - Visual badges for selected items
   - Can be skipped

4. **preferences-step.tsx**
   - Focus duration selection (15-90 minutes)
   - Break duration selection (5-20 minutes)
   - Notification preferences toggle
   - Auto-start breaks toggle
   - Summary of selected preferences

5. **tutorial-step.tsx**
   - Overview of 6 core features
   - Interactive checkboxes to mark as viewed
   - Distinguishes free vs Pro features
   - Expandable content with usage tips
   - Can proceed without viewing all

### Backend (Rust)

**Commands**: `/packages/desktop/src-tauri/src/commands/onboarding.rs`

Functions:
- `complete_onboarding(state, data)` - Saves all onboarding data to database
- `is_onboarding_complete(state)` - Checks if user has completed onboarding
- `get_onboarding_data(state)` - Retrieves partial onboarding data for resuming
- `reset_onboarding(state)` - Resets onboarding state (for testing)

Database operations:
- Saves user preferences to `user_settings` table
- Creates blocked items in `blocked_items` table
- Marks onboarding as complete with timestamp
- Uses transactions for atomic operations

**Integration**:
- Added to `/packages/desktop/src-tauri/src/commands/mod.rs`
- Registered in `/packages/desktop/src-tauri/src/lib.rs` invoke handler

### Types

**Types Package**: `/packages/types/src/onboarding.ts`

Exports:
- `OnboardingStep` - Union type of all step names
- `OnboardingState` - Current state and completion status
- `OnboardingData` - All collected user data
- `IndistractablePillar` - Structure for framework pillars
- `TutorialTopic` - Structure for tutorial items
- `OnboardingSaveRequest` - Backend request payload
- `OnboardingCompleteResponse` - Backend response
- `INDISTRACTABLE_PILLARS` - Array of pillar data
- `SUGGESTED_APPS` - Common app blocklist suggestions
- `SUGGESTED_WEBSITES` - Common website blocklist suggestions
- `TUTORIAL_TOPICS` - Array of tutorial topics

## Database Schema

Uses existing `user_settings` table (Migration 7):

```sql
CREATE TABLE user_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
)
```

Keys stored:
- `onboarding_completed` - "true" when complete
- `onboarding_completed_at` - Timestamp
- `user_name` - User's name
- `default_focus_minutes` - Default focus duration
- `default_break_minutes` - Default break duration
- `notification_sound` - Notification preference
- `auto_start_breaks` - Auto-start breaks preference

Blocked items stored in existing `blocked_items` table.

## Data Flow

1. **Initial Check** (`App.tsx`)
   - `useNeedsOnboarding()` checks localStorage and backend
   - If needed, renders `OnboardingWizard`
   - Otherwise renders main app

2. **User Input** (Step components)
   - User enters data in each step
   - Data stored in hook state via `updateData()`
   - Automatically synced to localStorage

3. **Navigation** (useOnboarding hook)
   - `nextStep()` / `previousStep()` methods
   - Progress calculated as percentage
   - Can jump to specific steps via `goToStep()`

4. **Completion** (OnboardingWizard)
   - Calls `completeOnboarding()` hook method
   - Sends all data to Rust backend
   - Backend saves to SQLite database
   - Creates blocked items
   - Marks onboarding complete
   - Clears localStorage
   - Triggers onComplete callback
   - Shows main app

5. **Persistence**
   - localStorage: In-progress state (resumable)
   - SQLite: Final completed state (permanent)
   - Both checked on app startup

## Integration Points

### App.tsx
```typescript
import { OnboardingWizard } from "@/features/onboarding/onboarding-wizard";
import { useNeedsOnboarding } from "@/hooks/use-onboarding";

// Show onboarding if needed
if (showOnboarding) {
  return <OnboardingWizard onComplete={() => setShowOnboarding(false)} />;
}
```

### Rust Commands Registration
```rust
// In lib.rs
commands::onboarding::complete_onboarding,
commands::onboarding::is_onboarding_complete,
commands::onboarding::get_onboarding_data,
commands::onboarding::reset_onboarding,
```

## Features

### User Experience
- Clean, modern UI with shadcn/ui components
- Progress indicator on every step
- Can navigate backwards to change answers
- Data persists if user closes app mid-onboarding
- Keyboard navigation support
- Responsive design for different window sizes

### Educational Content
- Comprehensive explanation of Indistractable framework
- Interactive elements to engage users
- Clear call-to-action on each step
- Visual hierarchy with icons and colors
- Practical examples and actionable tips

### Validation
- Name input required before proceeding
- All other steps optional (skip-friendly)
- Smart defaults (25 min focus, 5 min break)
- Type-safe data handling throughout

### Performance
- Lazy loading of step components
- Minimal re-renders with React hooks
- Efficient state management
- Database transactions for atomic writes
- LocalStorage for instant resume

## Design Patterns

### Component Patterns
- **DRY**: StepWrapper eliminates repetitive layout code
- **Composition**: Steps compose wrapper with custom content
- **Controlled Components**: Parent manages all state
- **Type Safety**: Full TypeScript coverage

### State Management
- **Single Source of Truth**: useOnboarding hook
- **Derived State**: Progress, validation computed
- **Persistence**: Dual localStorage + SQLite
- **Optimistic Updates**: UI updates immediately

### Code Organization
- **Feature-based**: All onboarding code in one directory
- **Separation of Concerns**: UI, logic, and data separate
- **Shared Types**: packages/types for type safety
- **Modular Steps**: Each step is independent

## Testing Considerations

### Manual Testing
1. Complete onboarding normally
2. Close app mid-flow and resume
3. Navigate backwards and change selections
4. Skip optional steps
5. Test with empty selections
6. Verify database persistence
7. Test reset functionality

### Automated Testing (TODO)
- Unit tests for useOnboarding hook
- Integration tests for wizard flow
- E2E tests with Playwright
- Rust tests for commands (included)

## Accessibility

- Semantic HTML elements
- ARIA labels where needed
- Keyboard navigation
- Focus management
- Screen reader friendly
- Color contrast compliance
- Clear visual hierarchy

## Future Enhancements

### Potential Additions
- Analytics tracking of onboarding completion rate
- A/B testing different flows
- Video tutorials in tutorial step
- Import settings from other apps
- More granular pillar selection
- Gamification (progress animations)
- Skip entire onboarding option
- Re-run onboarding from settings

### Pro Features
- Sync onboarding preferences across devices
- Team onboarding templates
- Custom company branding
- Advanced blocking rules setup

## Files Created

### Frontend
1. `/packages/desktop/src/features/onboarding/onboarding-wizard.tsx` - Main wizard
2. `/packages/desktop/src/features/onboarding/step-wrapper.tsx` - Reusable wrapper
3. `/packages/desktop/src/features/onboarding/steps/welcome-step.tsx` - Step 1
4. `/packages/desktop/src/features/onboarding/steps/pillars-step.tsx` - Step 2
5. `/packages/desktop/src/features/onboarding/steps/blocklist-step.tsx` - Step 3
6. `/packages/desktop/src/features/onboarding/steps/preferences-step.tsx` - Step 4
7. `/packages/desktop/src/features/onboarding/steps/tutorial-step.tsx` - Step 5
8. `/packages/desktop/src/hooks/use-onboarding.ts` - Hook and logic

### Backend
9. `/packages/desktop/src-tauri/src/commands/onboarding.rs` - Rust commands

### Types
10. `/packages/types/src/onboarding.ts` - TypeScript types

### Modified Files
- `/packages/desktop/src/App.tsx` - Integration
- `/packages/types/src/index.ts` - Export onboarding types
- `/packages/desktop/src-tauri/src/commands/mod.rs` - Export module
- `/packages/desktop/src-tauri/src/lib.rs` - Register commands

## Summary

This implementation provides a production-ready onboarding flow that:
- ✅ Educates users on Indistractable principles
- ✅ Collects essential preferences
- ✅ Sets up blocking immediately
- ✅ Persists data reliably
- ✅ Handles edge cases gracefully
- ✅ Follows best practices
- ✅ Uses modern React patterns
- ✅ Maintains type safety
- ✅ Provides excellent UX
- ✅ Is fully integrated with the app

The onboarding flow takes 3-5 minutes and ensures users understand the app's core value proposition before diving into features.
