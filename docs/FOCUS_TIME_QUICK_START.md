# Focus Time - Quick Start Guide

## For Developers

### Install Dependencies

```bash
cd packages/desktop
pnpm add @radix-ui/react-scroll-area@^1.2.2 @radix-ui/react-alert-dialog@^1.1.4 @radix-ui/react-accordion@^1.2.3
```

### Add to Main Layout

```tsx
// packages/desktop/src/App.tsx
import { FocusTimeOverlay } from "@/components/FocusTimeOverlay";

export function App() {
  return (
    <>
      <YourExistingLayout />
      <FocusTimeOverlay /> {/* Add this line */}
    </>
  );
}
```

### Backend Implementation Checklist

1. Create `packages/desktop/src-tauri/src/commands/focus_time.rs`
2. Add 10 Tauri commands (see FOCUS_TIME_DEPENDENCIES.md)
3. Register commands in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::focus_time::get_focus_time_events,
    commands::focus_time::get_active_focus_time,
    commands::focus_time::get_focus_time_state,
    commands::focus_time::get_focus_time_allowed_apps,
    commands::focus_time::add_focus_time_allowed_app,
    commands::focus_time::remove_focus_time_allowed_app,
    commands::focus_time::end_focus_time_early,
    commands::focus_time::start_focus_time_now,
    commands::focus_time::override_focus_time,
    commands::focus_time::refresh_focus_time_events,
])
```

4. Integrate with existing calendar sync system
5. Add Focus Time blocking logic to process monitor

## For Users

### Setup Instructions

1. **Connect Calendar**
   - Go to Calendar > Connections
   - Connect Google Calendar or Microsoft Outlook

2. **Create Focus Time Event**
   - Open your calendar (Google/Outlook)
   - Create new event: `🎯 Focus Time: Deep Work`
   - Set start and end time
   - In description, add allowed apps:
     ```
     @coding @communication @music
     ```
     OR list specific apps:
     ```
     Visual Studio Code, Slack, Spotify
     ```

3. **During Focus Time**
   - Green overlay appears in top-right
   - Shows countdown timer
   - Click "Modify" to add/remove apps
   - Click "End" to finish early

### Available Categories

Quick setup with predefined categories:

| Category | Apps Included |
|----------|---------------|
| `@coding` | VS Code, IntelliJ, Xcode, Sublime Text, Vim |
| `@communication` | Slack, Teams, Discord, Zoom, Telegram |
| `@browser` | Chrome, Firefox, Safari, Edge |
| `@design` | Figma, Sketch, Photoshop, Illustrator |
| `@productivity` | Notion, Obsidian, Evernote, OneNote |
| `@terminal` | Terminal, iTerm2, Alacritty, Hyper |
| `@music` | Spotify, Apple Music, YouTube Music |

### Example Calendar Events

**Deep Work Session**
```
Title: 🎯 Focus Time: Deep Work
Time: 9:00 AM - 11:00 AM
Description: @coding @terminal @music
```

**Meeting Prep**
```
Title: 🎯 Focus Time: Meeting Prep
Time: 2:00 PM - 2:30 PM
Description: @productivity @browser Google Chrome
```

**Design Time**
```
Title: 🎯 Focus Time: UI Design
Time: 3:00 PM - 5:00 PM
Description: @design @browser Figma
```

## Testing During Development

### Mock Data for Frontend

Until backend is ready, you can add mock data:

```tsx
// In useFocusTime.ts, replace invoke calls with mock data
const mockEvents: FocusTimeEvent[] = [
  {
    id: "1",
    title: "🎯 Focus Time: Deep Work",
    start_time: new Date(Date.now() + 3600000).toISOString(),
    end_time: new Date(Date.now() + 7200000).toISOString(),
    provider: "google",
    allowed_apps: ["Visual Studio Code", "Terminal", "Spotify"],
    allowed_categories: ["@coding", "@terminal", "@music"],
    is_active: false,
    created_from_calendar: true,
  },
];
```

### Test States

1. **No Calendar Connected**: View Calendar > Focus Time
2. **Calendar Connected, No Events**: Create test event in calendar
3. **Upcoming Events**: Verify events list shows correctly
4. **Active Session**: Mock an active session to test overlay
5. **Override Flow**: Test adding/removing apps during active session

## Troubleshooting

### Overlay Not Showing
- Check `useFocusTimeState()` returns `active: true`
- Verify `FocusTimeOverlay` is in main layout
- Check z-index conflicts

### Events Not Loading
- Verify calendar is connected
- Check backend command is implemented
- Check calendar event format (must start with "🎯 Focus Time")

### Categories Not Working
- Ensure category name matches exactly (e.g., `@coding` not `@code`)
- Check FOCUS_TIME_CATEGORIES constant is imported
- Verify backend expands categories correctly

## Files Reference

```
packages/
├── types/src/
│   ├── focus-time.ts          # Types & constants
│   └── index.ts               # Export types
├── desktop/src/
│   ├── hooks/
│   │   └── useFocusTime.ts    # React Query hooks
│   ├── components/
│   │   ├── AppSelector.tsx    # App selection UI
│   │   ├── FocusTimeOverlay.tsx # Floating overlay
│   │   └── ui/
│   │       ├── scroll-area.tsx
│   │       ├── alert-dialog.tsx
│   │       └── accordion.tsx
│   └── features/
│       ├── FocusTime.tsx      # Main feature page
│       └── Calendar.tsx       # Updated with Focus Time tab
└── docs/
    ├── FOCUS_TIME_IMPLEMENTATION.md  # Full implementation details
    ├── FOCUS_TIME_QUICK_START.md     # This file
    └── FOCUS_TIME_DEPENDENCIES.md    # Backend requirements
```

## API Reference

### Hooks

```tsx
// Get upcoming events
const { data: events } = useFocusTimeEvents();

// Get active session
const { data: active } = useActiveFocusTime();

// Get complete state
const { data: state } = useFocusTimeState();

// Actions
const { addApp, removeApp, endEarly, startNow } = useFocusTimeActions();

// Add allowed app
addApp.mutate("Visual Studio Code");

// End early
endEarly.mutate("Taking a break");
```

### Components

```tsx
// Overlay (add to layout)
<FocusTimeOverlay />

// App selector (used in overlay)
<AppSelector
  selectedApps={["VS Code", "Slack"]}
  onToggleApp={(app) => console.log(app)}
  onToggleCategory={(cat) => console.log(cat)}
  onClose={() => {}}
/>

// Feature page
<FocusTime />
```

## Support

For issues or questions:
1. Check FOCUS_TIME_IMPLEMENTATION.md for detailed info
2. Review FOCUS_TIME_DEPENDENCIES.md for backend requirements
3. Open GitHub issue with `focus-time` label
