# Focus Time Feature - Required Dependencies

The Focus Time feature requires the following Radix UI packages to be installed:

```bash
cd packages/desktop
pnpm add @radix-ui/react-scroll-area @radix-ui/react-alert-dialog @radix-ui/react-accordion
```

## Installation Command

```bash
pnpm add @radix-ui/react-scroll-area@^1.2.2 @radix-ui/react-alert-dialog@^1.1.4 @radix-ui/react-accordion@^1.2.3
```

## Files Created

### Types
- `/packages/types/src/focus-time.ts` - FocusTime types and constants
- Updated `/packages/types/src/index.ts` - Export FocusTime types

### Hooks
- `/packages/desktop/src/hooks/useFocusTime.ts` - React hooks for Focus Time functionality

### Components
- `/packages/desktop/src/components/AppSelector.tsx` - App selection UI
- `/packages/desktop/src/components/FocusTimeOverlay.tsx` - Floating overlay for active Focus Time
- `/packages/desktop/src/components/ui/scroll-area.tsx` - ScrollArea UI component
- `/packages/desktop/src/components/ui/alert-dialog.tsx` - AlertDialog UI component
- `/packages/desktop/src/components/ui/accordion.tsx` - Accordion UI component

### Features
- `/packages/desktop/src/features/FocusTime.tsx` - Main Focus Time feature component
- Updated `/packages/desktop/src/features/Calendar.tsx` - Added Focus Time tab

## Backend Commands Required

The following Tauri commands need to be implemented in the Rust backend:

```rust
// In packages/desktop/src-tauri/src/commands/focus_time.rs

pub async fn get_focus_time_events() -> Result<Vec<FocusTimeEvent>>;
pub async fn get_active_focus_time() -> Result<Option<FocusTimeEvent>>;
pub async fn get_focus_time_state() -> Result<FocusTimeState>;
pub async fn get_focus_time_allowed_apps() -> Result<Vec<String>>;
pub async fn add_focus_time_allowed_app(app_name: String) -> Result<()>;
pub async fn remove_focus_time_allowed_app(app_name: String) -> Result<()>;
pub async fn end_focus_time_early(reason: Option<String>) -> Result<()>;
pub async fn start_focus_time_now(event_id: String) -> Result<()>;
pub async fn override_focus_time(override_action: FocusTimeOverride) -> Result<()>;
pub async fn refresh_focus_time_events() -> Result<()>;
```

These commands should be added to the `invoke_handler!` in `lib.rs`.

## Integration with Main App

To use the Focus Time feature:

1. Install dependencies (see above)
2. Import and use `FocusTimeOverlay` in the main layout (it will only show when active)
3. The Calendar tab now includes a "Focus Time" sub-tab
4. Users can navigate to Calendar > Focus Time to see setup instructions

### Example Integration in Layout

```tsx
import { FocusTimeOverlay } from "@/components/FocusTimeOverlay";

export function Layout() {
  return (
    <>
      {/* Existing layout */}
      <FocusTimeOverlay />
    </>
  );
}
```

## How It Works

1. User creates calendar events with title "🎯 Focus Time: [description]"
2. In the event description/notes, they add allowed apps using:
   - Categories: `@coding @communication @music`
   - Specific apps: `Visual Studio Code, Slack, Spotify`
3. The backend parses these events and automatically blocks all apps except allowed ones
4. User can override rules during active session via the floating overlay
5. Focus Time ends automatically when calendar event ends, or user can end early

## UI Features

- Green accent color for Focus Time (matches the 🎯 emoji)
- Floating overlay shows remaining time and quick access to modify rules
- Setup instructions shown in Calendar tab
- Category-based app selection (faster than selecting individual apps)
- Real-time sync with calendar events
