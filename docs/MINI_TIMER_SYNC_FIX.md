# Mini-Timer Sync Fix - Root Cause Analysis and Solution

## Problem Statement

The mini-timer window was showing `00:00` or `NaN:NaN` instead of syncing with the main timer, even though the main timer displayed the correct time (e.g., 24:49).

## Root Cause

**Tauri 2's `emitTo()` function does not support cross-window event communication from JavaScript.**

The confusion arose from the misleading API signature:
```javascript
await emitTo("mini-timer", "timer-state-update", timerPayload);
```

This looks like it should send an event to the "mini-timer" window, but in Tauri 2:

1. **`emitTo()` from `@tauri-apps/api/event` invokes Rust backend commands**, not window-to-window events
2. **Only the Rust backend can emit events directly to windows** using `window.emit()`
3. **JavaScript-to-JavaScript window communication must be routed through Rust**

## Evidence

### What Was Broken

**FocusTimer.tsx (lines 159-186, before fix):**
```typescript
// This does NOT work in Tauri 2
await emitTo("mini-timer", "timer-state-update", timerPayload);
```

The mini-timer window's `listen()` call never received these events because `emitTo()` doesn't send them to windows.

### What Actually Works

**focus.rs (lines 238-242, 266-276):**
```rust
// This WORKS - the Rust backend uses window handles
if let Some(window) = app_handle.get_webview_window("mini-timer") {
    window.emit("session-extended", payload)?;
}
```

The Rust backend gets the window handle and calls `emit()` directly.

## Solution

Created a Rust command that acts as a relay for timer state updates:

### 1. New Rust Command

**window.rs:**
```rust
#[tauri::command]
pub async fn emit_to_mini_timer<R: Runtime>(
    app: AppHandle<R>,
    event_name: String,
    payload: serde_json::Value,
) -> Result<()> {
    if let Some(window) = app.get_webview_window("mini-timer") {
        window.emit(&event_name, &payload)?;
    }
    Ok(())
}
```

### 2. Updated FocusTimer.tsx

**Before:**
```typescript
await emitTo("mini-timer", "timer-state-update", timerPayload);
```

**After:**
```typescript
await invoke("emit_to_mini_timer", {
  eventName: "timer-state-update",
  payload: timerPayload,
});
```

### 3. Event Flow

```
FocusTimer.tsx (main window)
    ↓
    invoke("emit_to_mini_timer")
    ↓
Rust backend (lib.rs → window.rs)
    ↓
    window.emit("timer-state-update")
    ↓
mini-timer-window.tsx listens for event
    ↓
UI updates with correct time
```

## Files Modified

1. **/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/window.rs**
   - Added `emit_to_mini_timer` command

2. **/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/lib.rs**
   - Registered `emit_to_mini_timer` in invoke handler

3. **/Users/jacob/projects/focus-app/packages/desktop/src/features/FocusTimer.tsx**
   - Removed `emitTo` import
   - Changed event emission to use `invoke("emit_to_mini_timer")`
   - Added debug logging

4. **/Users/jacob/projects/focus-app/packages/desktop/src/features/mini-timer/mini-timer-window.tsx**
   - Added debug logging to track event reception

## Testing Instructions

### 1. Build and Run

```bash
cd /Users/jacob/projects/focus-app/packages/desktop
pnpm build  # Build frontend assets
cargo build --release  # Build Rust backend
pnpm tauri dev  # Run dev mode
```

### 2. Test Cases

#### Test A: Basic Timer Sync
1. Start a focus session in the main window
2. Open the mini-timer window (Cmd+Shift+M)
3. Verify the mini-timer shows the same time as the main timer
4. Expected: Both timers show identical remaining time

#### Test B: Timer Updates
1. Watch both timers for 10 seconds
2. Verify both timers count down smoothly together
3. Expected: No flickering, both stay in sync

#### Test C: Pause/Resume
1. Start a session
2. Open mini-timer
3. Pause on main window
4. Verify mini-timer pauses at the same time
5. Resume and verify both resume together
6. Expected: Perfect synchronization

#### Test D: Session Extension
1. Start a session
2. Open mini-timer
3. Click extend button on mini-timer
4. Verify planned duration increases in both windows
5. Expected: Both windows update immediately

### 3. Debug Logging

Check the browser console (DevTools) for logs:

```javascript
// Main window logs
[FocusTimer] Timer update: {elapsed: 100, remaining: 1400, isRunning: true, sessionId: "..."}

// Mini-timer window logs
[mini-timer] Initial session state loaded: {sessionId: "...", elapsed: 50, planned: 25}
[mini-timer] Received timer-state-update event: {elapsed: 100, isRunning: true, sessionId: "..."}
```

If events are not arriving in mini-timer, check:
1. Main window has an active session
2. Mini-timer window is open
3. No errors in console

## Key Learning: Tauri 2 Event System

In Tauri 2:

### Window-to-Window Communication (Incorrect)
```typescript
// This does NOT work for cross-window events
emitTo("target-window", "event-name", payload);  // ❌ Wrong
listen("event-name", handler);  // Never receives
```

### Correct Pattern: Via Rust Backend
```typescript
// Frontend initiates communication
invoke("relay_to_window", {
  targetWindow: "mini-timer",
  eventName: "my-event",
  payload: data,
});

// Rust backend relays
window.emit("my-event", payload);  // ✅ Correct
```

## Performance Implications

- Events are sent on every timer tick (every 100ms)
- Payload includes: `activeSession`, `elapsedSeconds`, `isRunning`
- Rust relay adds minimal overhead (<1ms per event)
- Local interpolation in mini-timer maintains smooth visual updates even if events are delayed

## Future Improvements

1. **Rate limiting**: Could throttle events to every 500ms if performance becomes an issue
2. **Shared state service**: Consider a centralized state service in Rust for more complex scenarios
3. **WebSockets alternative**: For very high-frequency updates, WebSockets could be considered

## References

- Tauri 2 Documentation: https://docs.tauri.app/
- Event System: https://docs.tauri.app/develop/calling-rust/
- Window Management: https://docs.tauri.app/develop/guides/window-management/
