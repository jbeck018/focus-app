# Mini-Timer Quick Start Guide

## Installation

The mini-timer is built-in to FocusFlow. No additional installation required.

## Usage

### Opening the Mini-Timer

**Keyboard Shortcut (Recommended)**
```
macOS:     Cmd + Shift + M
Windows:   Ctrl + Shift + M
Linux:     Ctrl + Shift + M
```

**UI Button**
1. Start a focus session
2. Click the maximize icon in the top-right corner of the timer

### Controls

| Button | Action |
|--------|--------|
| ▶️ / ⏸️ | Pause/Resume timer |
| ⏹️ | Stop session (incomplete) |
| +5m | Add 5 minutes to session |
| ✕ | Close mini-timer window |

### Window Behavior

**Opacity**
- Idle: 40% opacity (semi-transparent)
- Hover: 100% opacity (fully visible)

**Positioning**
- Drag anywhere to move
- Position automatically saved
- Defaults to top-right corner

**Always On Top**
- Stays above all other windows
- Perfect for multitasking

**Double-Click**
- Double-click to focus main window

## Visual Indicators

**Session Type Colors**
- 🔵 Blue: Focus session
- 🟢 Green: Break session
- ⚪ Gray: No active session

**Status Dot**
- 🟢 Pulsing green: Timer running
- ⚪ Gray: Timer paused

## Tips

1. **Keep it visible**: Position in corner of your screen
2. **Quick glance**: Check time without switching windows
3. **Fast actions**: Pause/extend without interrupting workflow
4. **Minimal distraction**: Semi-transparent design stays out of the way

## Troubleshooting

**Mini-timer won't open?**
- Ensure a session is active
- Try keyboard shortcut
- Check console for errors

**Not syncing with main timer?**
- Restart both windows
- Check for JavaScript errors
- Verify session is active

**Can't drag window?**
- Click and hold on timer area
- Avoid clicking on buttons
- Try from header section

## Advanced

### Programmatic Control

```typescript
import { invoke } from "@tauri-apps/api/core";

// Open mini-timer
await invoke("open_mini_timer");

// Close mini-timer
await invoke("close_mini_timer");

// Toggle visibility
await invoke("toggle_mini_timer");

// Set position
await invoke("set_mini_timer_position", {
  position: { x: 100, y: 100 }
});
```

### Event Listening

```typescript
import { listen } from "@tauri-apps/api/event";

// Listen for timer updates
const unlisten = await listen("timer-state-update", (event) => {
  console.log("Timer state:", event.payload);
});

// Clean up
unlisten();
```

## Keyboard Navigation

All controls are keyboard accessible:

- `Tab`: Navigate between controls
- `Space/Enter`: Activate button
- `Escape`: Close mini-timer

## Accessibility

The mini-timer supports:
- Screen readers (ARIA labels)
- Keyboard-only navigation
- High contrast mode
- Focus indicators

## Performance

**Minimal Resource Usage**
- Small window size (200x80px)
- Efficient event handling
- GPU-accelerated transparency
- ~5-10 MB memory footprint

## See Also

- [Complete Documentation](./MINI_TIMER.md)
- [Implementation Summary](../IMPLEMENTATION_SUMMARY.md)
- [Main Application Guide](../README.md)
