# Mini-Timer Widget

A floating, always-on-top mini-timer window for FocusFlow that provides quick access to your active session.

## Features

- **Always On Top**: Stays above all other windows
- **Floating Widget**: Small, unobtrusive design (200x80px)
- **Semi-Transparent**: 40% opacity when idle, 100% on hover
- **Draggable**: Position anywhere on screen with saved preferences
- **Real-Time Sync**: Automatically syncs with main timer state
- **Quick Actions**: Pause, stop, and add time without switching windows
- **Keyboard Shortcut**: Toggle with `Cmd+Shift+M` (macOS) or `Ctrl+Shift+M` (Windows/Linux)

## Usage

### Opening the Mini-Timer

**Method 1: Keyboard Shortcut**
- Press `Cmd+Shift+M` (macOS) or `Ctrl+Shift+M` (Windows/Linux)

**Method 2: UI Button**
- Click the mini-timer icon (top-right corner) in the FocusTimer component when a session is active

**Method 3: Programmatically**
```typescript
import { invoke } from "@tauri-apps/api/core";

await invoke("open_mini_timer");
```

### Interacting with the Mini-Timer

**Drag to Move**
- Click and drag anywhere on the mini-timer to reposition
- Position is automatically saved

**Hover for Full Visibility**
- Move mouse over the widget to see it at 100% opacity
- Moves away to reduce to 40% opacity

**Double-Click**
- Double-click the timer to bring the main window to focus

**Right-Click** (Future)
- Right-click for context menu with settings

### Controls

**Pause/Resume Button**
- Toggle timer between running and paused states
- Syncs with main window instantly

**Stop Button**
- End the current session (marked as incomplete)
- Red-colored for visual distinction

**+5m Button**
- Add 5 minutes to the current session
- Useful for extending focus without interruption

**Close Button (X)**
- Close the mini-timer window
- Does not affect the active session

## Technical Implementation

### Architecture

```
┌─────────────────┐         Events         ┌─────────────────┐
│                 │◄─────────────────────►│                 │
│  Main Window    │  timer-state-update    │  Mini-Timer     │
│  (FocusTimer)   │                        │  Window         │
│                 │                        │                 │
└────────┬────────┘                        └────────┬────────┘
         │                                          │
         │                                          │
         └──────────►  Tauri Commands  ◄───────────┘
                    (Rust Backend)
```

### Files Created

**Frontend (TypeScript/React)**
- `/src/features/mini-timer/mini-timer-window.tsx` - Main mini-timer UI component
- `/src/features/mini-timer/mini-timer-controls.tsx` - Control buttons component
- `/src/mini-timer-main.tsx` - Entry point for mini-timer window
- `/src/hooks/useKeyboardShortcuts.ts` - Keyboard shortcut handler hook
- `/mini-timer.html` - HTML entry point for mini-timer

**Backend (Rust)**
- `/src-tauri/src/commands/window.rs` - Window management commands
- Updated `/src-tauri/src/commands/mod.rs` - Module registration
- Updated `/src-tauri/src/lib.rs` - Command handler registration
- Updated `/src-tauri/src/error.rs` - Window error type

**Configuration**
- Updated `/src-tauri/tauri.conf.json` - Main window label
- Updated `/vite.config.ts` - Multi-page build configuration

### Tauri Commands

**Window Management**
```rust
// Open or focus mini-timer window
open_mini_timer() -> Result<()>

// Close mini-timer window
close_mini_timer() -> Result<()>

// Toggle mini-timer visibility
toggle_mini_timer() -> Result<()>

// Save window position
set_mini_timer_position(position: WindowPosition) -> Result<()>

// Get current window position
get_mini_timer_position() -> Result<Option<WindowPosition>>

// Focus main window
focus_main_window() -> Result<()>
```

### Event System

**Timer State Updates**
```typescript
interface TimerEventPayload {
  activeSession: ActiveSession | null;
  elapsedSeconds: number;
  isRunning: boolean;
}

// Emitted by main window on every timer update
emit("timer-state-update", payload);

// Listened to by mini-timer window
listen("timer-state-update", (event) => {
  // Update mini-timer UI
});
```

## Customization

### Window Appearance

The mini-timer uses dynamic styling based on session type:

**Focus Sessions**
- Blue-tinted background (`bg-blue-500/20`)
- Blue border (`border-blue-500/30`)

**Break Sessions**
- Green-tinted background (`bg-green-500/20`)
- Green border (`border-green-500/30`)

### Opacity Behavior

```typescript
// Default opacity when idle
opacity: 0.4

// Full opacity on hover
opacity: 1.0

// Partial opacity when dragging
opacity: 0.8
```

### Window Configuration

```json
{
  "label": "mini-timer",
  "url": "mini-timer.html",
  "width": 200,
  "height": 80,
  "resizable": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true
}
```

## Future Enhancements

- [ ] Context menu for settings (right-click)
- [ ] Persistent position preferences across sessions
- [ ] Customizable opacity levels
- [ ] Theme customization
- [ ] Multiple timer displays (Pomodoro cycle view)
- [ ] Quick task notes
- [ ] Mini analytics snapshot
- [ ] Session type switcher
- [ ] Resize handle for user preference
- [ ] Snap to screen edges

## Accessibility

The mini-timer follows WCAG 2.1 Level AA guidelines:

- **Keyboard Navigation**: All controls accessible via keyboard
- **ARIA Labels**: Proper labels for screen readers
- **Focus Indicators**: Visible focus states
- **Color Contrast**: Sufficient contrast ratios
- **Screen Reader Announcements**: Status updates announced

## Performance

- **Event Throttling**: Timer updates throttled to prevent excessive re-renders
- **Minimal Re-renders**: Optimized state updates with React.memo
- **Low Memory**: Small window with minimal DOM elements
- **GPU Acceleration**: Transparent window uses hardware acceleration

## Troubleshooting

**Mini-timer doesn't open**
- Check browser console for errors
- Verify Tauri window permissions
- Ensure no conflicting keyboard shortcuts

**Position not saved**
- Check file system permissions
- Verify localStorage availability
- Review console for save errors

**Timer out of sync**
- Verify event listeners are registered
- Check for errors in event emission
- Restart both windows

**Performance issues**
- Reduce opacity transitions
- Check GPU acceleration settings
- Verify no background tasks blocking

## Development

### Running in Development

```bash
cd packages/desktop
pnpm tauri dev
```

The mini-timer window will be available alongside the main window.

### Building for Production

```bash
cd packages/desktop
pnpm tauri build
```

Both HTML entry points are bundled automatically via Vite's multi-page configuration.

### Testing

```typescript
// Test window opening
await invoke("open_mini_timer");

// Test position saving
await invoke("set_mini_timer_position", {
  position: { x: 100, y: 100 }
});

// Test event emission
await emit("timer-state-update", {
  activeSession: mockSession,
  elapsedSeconds: 300,
  isRunning: true,
});
```

## License

Part of FocusFlow - See main LICENSE file
