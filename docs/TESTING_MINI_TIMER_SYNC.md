# Testing Guide: Mini-Timer Sync Fix

## Quick Start

### Build and Run
```bash
cd /Users/jacob/projects/focus-app/packages/desktop

# Build frontend
pnpm build

# Run in development mode
pnpm tauri dev
```

### Verify Compilation
```bash
# Check TypeScript
pnpm tsc --noEmit

# Check Rust
cd src-tauri
cargo check
```

## Test Scenarios

### Test 1: Basic Synchronization
**Goal**: Verify mini-timer receives and displays correct timer state

**Steps**:
1. Start the app
2. Create a new focus session (25 minutes)
3. Immediately open mini-timer (Cmd+Shift+M on macOS)
4. Observe both timers

**Expected Results**:
- Main timer and mini-timer show identical remaining time
- Example: Both show "24:49" not "00:00"
- Mini-timer updates smoothly (no NaN values)

**Where to Check Logs**:
- DevTools Console (F12 on main window)
- Separate DevTools for mini-timer window

**Expected Console Logs**:
```
[mini-timer] Initial session state loaded: {sessionId: "...", elapsed: 12, planned: 25}
[mini-timer] Received timer-state-update event: {elapsed: 12, isRunning: true, sessionId: "..."}
[FocusTimer] Timer update: {elapsed: 12, remaining: 1488, isRunning: true, sessionId: "..."}
```

### Test 2: Continuous Updates
**Goal**: Verify timer state updates propagate continuously

**Steps**:
1. Start session in main window
2. Open mini-timer
3. Watch both timers for 30 seconds
4. Check console logs

**Expected Results**:
- Both timers count down smoothly
- No "stuttering" or jumps in display
- Mini-timer updates every 100ms locally (smooth animation)
- Rust events arrive periodically with authoritative time

**Debug Check**:
- Main window console should show `[FocusTimer] Timer update` logs every 10 seconds
- Mini-timer console should show `[mini-timer] Received timer-state-update` periodically

### Test 3: Pause/Resume Synchronization
**Goal**: Verify pause and resume events sync correctly

**Steps**:
1. Start session, open mini-timer
2. Both timers running, showing same time
3. Click Pause in main window
4. Observe mini-timer
5. Click Resume
6. Observe sync again

**Expected Results**:
- When main pauses, mini-timer pauses at the SAME time
- Indicator dot changes from green to gray in mini-timer
- Resume: both resume together
- Times remain perfectly synchronized

### Test 4: Session Extension
**Goal**: Verify duration extensions sync to mini-timer

**Steps**:
1. Start 25-minute session
2. Open mini-timer
3. Click "+" button on mini-timer to extend by 5 minutes
4. Observe both windows

**Expected Results**:
- Main window shows new 30-minute planned duration
- Mini-timer displays new remaining time calculation
- Both show same remaining seconds

### Test 5: No Active Session
**Goal**: Verify mini-timer handles idle state

**Steps**:
1. Stop any running session in main window
2. Keep mini-timer open (or open it now)
3. Try to start a new session

**Expected Results**:
- Mini-timer shows "--:--" or "Idle" message
- No console errors
- Once session starts, mini-timer populates correctly

### Test 6: Mini-Timer Window Timing on Open
**Goal**: Verify mini-timer initializes with correct elapsed time

**Steps**:
1. Start a 10-minute session in main window
2. Wait 30 seconds
3. Open mini-timer
4. Check initial time displayed

**Expected Results**:
- Mini-timer shows approximately "9:30" (30 seconds have elapsed)
- Not "9:59" (wrong elapsed time)
- Initial state is calculated from session startTime

**Console Check**:
```
[mini-timer] Initial session state loaded: {
  sessionId: "abc123",
  elapsed: 30,      // About 30 seconds
  planned: 10
}
```

## Debugging Console Logs

### How to View Logs

**Main Window**:
- Press F12 to open DevTools
- Go to Console tab
- See logs with `[FocusTimer]` prefix

**Mini-Timer Window**:
- Right-click on mini-timer
- Select "Inspect" (if available)
- OR press F12 in main window first, then open mini-timer
- Mini-timer should appear in tab list or open separate DevTools

### What Each Log Means

#### Main Window (FocusTimer.tsx)
```
[FocusTimer] Timer update: {
  elapsed: 60,              // Seconds elapsed since session start
  remaining: 1440,          // Remaining seconds
  isRunning: true,          // Timer is running
  sessionId: "abc123"       // Current session ID
}
```
- Appears every 10 seconds
- Shows what's being sent to mini-timer

#### Mini-Timer (mini-timer-window.tsx)
```
[mini-timer] Initial session state loaded: {
  sessionId: "abc123",
  elapsed: 60,
  planned: 25              // Planned duration in minutes
}
```
- Shows initial state loaded from backend
- Should match active session

```
[mini-timer] Received timer-state-update event: {
  elapsed: 60,
  isRunning: true,
  sessionId: "abc123"
}
```
- Shows incoming event from main window
- Should update frequently while session runs

#### Errors to Look For
```
[mini-timer] Failed to get active session: ...
```
- Mini-timer couldn't fetch initial session from backend
- Check if backend is responding

```
Failed to emit to mini-timer: ...
```
- Main window couldn't send event to mini-timer
- Mini-timer window might be closed/broken

## Performance Testing

### Memory Usage
1. Start a 25-minute session
2. Open mini-timer
3. Let it run for 5 minutes
4. Monitor DevTools memory (Main & Mini-Timer window)
5. No excessive memory growth should occur

### CPU Usage
1. Open Activity Monitor (macOS) or Task Manager (Windows)
2. Run session with mini-timer for 2-3 minutes
3. Check CPU usage for both windows
4. Should be minimal (<1% per window during idle, slight spikes when events arrive)

### Event Frequency
- Events: One per FocusTimer tick (every 100ms)
- Payload size: ~200 bytes per event
- Expected: ~2KB per second when session is active

## Troubleshooting

### Mini-Timer Shows 00:00
**Cause**: Events not being received
**Check**:
1. Console shows `[mini-timer] Received timer-state-update` logs?
   - YES: Display calculation issue
   - NO: Event relay is broken
2. Is mini-timer window actually open?
3. Is main window running a session?

**Fix**:
- Rebuild: `pnpm build && cargo build`
- Check that `emit_to_mini_timer` command is registered in lib.rs

### Mini-Timer Shows NaN:NaN
**Cause**: Invalid elapsed seconds value
**Check**:
1. Console error messages?
2. What does `elapsedSeconds` show in logs?
3. Is `plannedDurationMinutes` zero?

**Fix**:
- Clear application cache
- Restart both windows
- Check database for valid session data

### No Events Arriving
**Cause**: Communication broken between windows
**Check**:
1. Is `emit_to_mini_timer` in invoke handler?
   - File: `/src-tauri/src/lib.rs` line 184
   - Should have: `commands::window::emit_to_mini_timer,`
2. Is `Emitter` trait imported in window.rs?
   - File: `/src-tauri/src/commands/window.rs` line 4
   - Should import `Emitter`
3. Console errors in main window?

**Fix**:
```bash
cd /Users/jacob/projects/focus-app/packages/desktop/src-tauri
cargo check  # See exact error
```

## Comparison: Before and After

### Before Fix (Broken)
```
Main Timer: 24:49 ✓ (correct)
Mini-Timer: 00:00 ✗ (wrong)
Logs: No events arriving in mini-timer
Cause: emitTo() doesn't support window-to-window events
```

### After Fix (Working)
```
Main Timer: 24:49 ✓
Mini-Timer: 24:49 ✓ (matches!)
Logs: Events arriving every tick
Flow: FocusTimer → invoke("emit_to_mini_timer") → Rust → window.emit() → mini-timer
```

## Submitting Results

If you find issues:
1. Collect console logs from both windows
2. Note the exact behavior (e.g., "Shows 00:00 instead of 24:49")
3. Include logs, Rust compiler output, and steps to reproduce
4. Check the MINI_TIMER_SYNC_FIX.md for technical explanation

## Key Files Changed

1. **window.rs**: Added `emit_to_mini_timer` command
2. **lib.rs**: Registered new command
3. **FocusTimer.tsx**: Changed from `emitTo()` to `invoke("emit_to_mini_timer")`
4. **mini-timer-window.tsx**: Added debug logging

## Next Steps

After verification:
1. Remove debug logging (optional - can keep for now)
2. Consider throttling events if performance is an issue
3. Monitor real-world usage for sync issues
4. Collect telemetry on mini-timer usage
