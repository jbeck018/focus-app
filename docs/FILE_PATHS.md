# Mini-Timer Sync Fix - File Paths Reference

## Modified Source Files

### Rust Backend
1. `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/window.rs`
   - Modified: Added Emitter import (line 4)
   - Modified: Added emit_to_mini_timer command (lines 154-172)

2. `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/lib.rs`
   - Modified: Registered emit_to_mini_timer command (line 184)

### Frontend
1. `/Users/jacob/projects/focus-app/packages/desktop/src/features/FocusTimer.tsx`
   - Modified: Removed emitTo import (line 5)
   - Modified: Changed event emission method (lines 171-185)
   - Modified: Added debug logging (lines 194-201)

2. `/Users/jacob/projects/focus-app/packages/desktop/src/features/mini-timer/mini-timer-window.tsx`
   - Modified: Added debug logging for event reception (lines 51-55)
   - Modified: Added debug logging for initial state (lines 99-111)

## Documentation Files Created

All created in: `/Users/jacob/projects/focus-app/packages/desktop/`

1. **MINI_TIMER_SYNC_FIX.md**
   - Technical deep dive (complete explanation of root cause and solution)

2. **TESTING_MINI_TIMER_SYNC.md**
   - Comprehensive testing guide with 6 test scenarios

3. **FIX_SUMMARY.md**
   - Executive summary (quick overview)

4. **DEBUG_INVESTIGATION_REPORT.md**
   - Detailed investigation report (investigation process and findings)

5. **CHANGES.md**
   - Complete list of changes with before/after code

6. **FILE_PATHS.md** (this file)
   - Reference of all file locations

## Directory Structure

```
/Users/jacob/projects/focus-app/packages/desktop/
├── src/
│   └── features/
│       ├── FocusTimer.tsx (MODIFIED)
│       └── mini-timer/
│           └── mini-timer-window.tsx (MODIFIED)
├── src-tauri/
│   └── src/
│       ├── lib.rs (MODIFIED)
│       └── commands/
│           └── window.rs (MODIFIED)
├── MINI_TIMER_SYNC_FIX.md (CREATED)
├── TESTING_MINI_TIMER_SYNC.md (CREATED)
├── FIX_SUMMARY.md (CREATED)
├── DEBUG_INVESTIGATION_REPORT.md (CREATED)
├── CHANGES.md (CREATED)
└── FILE_PATHS.md (CREATED - this file)
```

## How to Build

From: `/Users/jacob/projects/focus-app/packages/desktop/`

```bash
# Verify compilation
cd src-tauri
cargo check

# Build frontend
cd ..
pnpm build

# Run development
pnpm tauri dev
```

## Key Absolute Paths

- Project root: `/Users/jacob/projects/focus-app/`
- Desktop package: `/Users/jacob/projects/focus-app/packages/desktop/`
- Frontend source: `/Users/jacob/projects/focus-app/packages/desktop/src/`
- Rust backend: `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/`
- Rust commands: `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/`
- Frontend features: `/Users/jacob/projects/focus-app/packages/desktop/src/features/`
- Mini-timer: `/Users/jacob/projects/focus-app/packages/desktop/src/features/mini-timer/`

## Import References in Modified Files

### window.rs imports
```rust
use tauri::{AppHandle, Emitter, LogicalPosition, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindowBuilder};
```

### FocusTimer.tsx imports
```typescript
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
```

### lib.rs registration (line 184)
```rust
commands::window::emit_to_mini_timer,
```

## Configuration Files

- Tauri config: `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/tauri.conf.json`
- Cargo manifest: `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/Cargo.toml`
- Package config: `/Users/jacob/projects/focus-app/packages/desktop/package.json`

## Testing Entry Points

- Main window: `/Users/jacob/projects/focus-app/packages/desktop/src/main.tsx`
- Mini-timer window: `/Users/jacob/projects/focus-app/packages/desktop/src/mini-timer-main.tsx`
- Main HTML: `/Users/jacob/projects/focus-app/packages/desktop/index.html`
- Mini-timer HTML: `/Users/jacob/projects/focus-app/packages/desktop/mini-timer.html`

## Verification Commands

```bash
# Rust compilation
cd /Users/jacob/projects/focus-app/packages/desktop/src-tauri
cargo check

# TypeScript checking
cd /Users/jacob/projects/focus-app/packages/desktop
pnpm tsc --noEmit

# Development build
pnpm build

# Run with Tauri
pnpm tauri dev
```

## Reference Documentation

All documentation files are in: `/Users/jacob/projects/focus-app/packages/desktop/`

For questions about:
- Root cause: See MINI_TIMER_SYNC_FIX.md
- Implementation: See specific modified files with inline comments
- Testing: See TESTING_MINI_TIMER_SYNC.md
- Changes: See CHANGES.md
- Investigation: See DEBUG_INVESTIGATION_REPORT.md
