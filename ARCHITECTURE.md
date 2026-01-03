# FocusFlow Backend Architecture

## Overview

FocusFlow is built with Rust and Tauri 2.0, providing a high-performance, memory-safe desktop application for productivity tracking and focus management.

## Core Design Principles

### 1. **Type Safety & Zero-Cost Abstractions**
- Compile-time SQL query validation with SQLx macros
- Strongly-typed error handling with `thiserror`
- Generic state management with `Arc<RwLock<T>>` for thread-safe sharing
- Zero-overhead async/await with Tokio

### 2. **Ownership & Lifetime Management**
- `AppState` uses `Arc` for cheap cloning across async task boundaries
- `RwLock` provides reader-writer lock semantics for optimal concurrency
- Database queries return owned types to avoid lifetime complexity
- Clone trait for serializable state types

### 3. **Async Patterns**
- All I/O operations are async to prevent UI blocking
- Background monitoring loop runs in separate Tokio task
- Database operations use connection pooling (max 5 connections)
- Non-blocking notification sending

### 4. **Error Handling Strategy**

```rust
pub enum Error {
    Database(String),        // SQLx errors
    Io(String),             // File system errors
    PermissionDenied(String), // Hosts file access
    ProcessNotFound(String), // Process monitoring
    InvalidSession(String),  // Session state errors
    // ... etc
}
```

- All errors implement `From<T>` for ergonomic `?` propagation
- Errors are serializable for transmission to frontend
- Custom error types for domain-specific failures

### 5. **Permission Escalation Strategy**

**Hosts File Modification:**
- Check permissions before attempting write
- Provide clear error messages with platform-specific instructions
- Atomic write pattern: temp file → rename
- Graceful degradation if permissions unavailable

**Process Termination:**
- Uses sysinfo crate's cross-platform process APIs
- Graceful termination attempted first
- Warning notification before kill (3-second grace period)

## Module Structure

### `/commands` - Tauri Command Handlers

**focus.rs** - Session management
- `start_focus_session`: Creates session, enables blocking
- `end_focus_session`: Completes session, updates analytics
- `get_active_session`: Returns current session state
- `get_session_history`: Queries historical sessions

**blocking.rs** - Block list management
- `add_blocked_app`: Add process name to block list
- `add_blocked_website`: Add domain to hosts file blocking
- `toggle_blocking`: Manual enable/disable

**analytics.rs** - Productivity metrics
- `get_daily_stats`: Today's focus statistics
- `get_weekly_stats`: 7-day aggregated data
- `get_productivity_score`: Calculated score (0-100)

**sync.rs** - Data import/export
- `export_data`: JSON export of all user data
- `import_data`: Merge imported data with existing

### `/db` - Database Layer

**Schema Design:**
```sql
sessions (
    id TEXT PRIMARY KEY,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    planned_duration_minutes INTEGER,
    actual_duration_seconds INTEGER,
    session_type TEXT,  -- focus, break, custom
    completed BOOLEAN
)

blocked_items (
    id INTEGER PRIMARY KEY,
    item_type TEXT,  -- app, website
    value TEXT,
    enabled BOOLEAN,
    UNIQUE(item_type, value)
)

daily_analytics (
    date DATE PRIMARY KEY,
    total_focus_seconds INTEGER,
    sessions_completed INTEGER,
    productivity_score REAL
)
```

**Query Patterns:**
- All queries use SQLx compile-time checking
- Prepared statements for SQL injection prevention
- Indices on frequently queried columns
- Soft deletes for blocked items (enable/disable flag)

### `/blocking` - Blocking Implementation

**hosts.rs** - Hosts file manipulation
- Atomic read-modify-write pattern
- Platform-specific paths (Windows vs Unix)
- Marker-based insertion for clean updates
- DNS cache flushing after modification

**process.rs** - Process monitoring
- Efficient delta updates using sysinfo
- 2-second monitoring interval (configurable)
- Hash-based deduplication of warnings
- Graceful shutdown handling

**dns.rs** - Future DNS-level blocking
- Placeholder for local DNS proxy
- Would provide defense-in-depth
- No elevated privileges required

### `/system` - System Integration

**tray.rs** - System tray management
- Dynamic menu based on session state
- Click handlers for quick actions
- Icon updates for visual feedback

**notifications.rs** - Cross-platform notifications
- Tauri notification API wrapper
- Consistent message formatting
- Scheduled break reminders

## State Management

### AppState Architecture

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,                          // Shared database pool
    pub active_session: Arc<RwLock<Option<ActiveSession>>>,  // Current session
    pub blocking_state: Arc<RwLock<BlockingState>>, // Blocking enabled/disabled
    pub app_handle: tauri::AppHandle,               // Tauri app instance
}
```

**Thread Safety:**
- `Arc` allows multiple ownership across async tasks
- `RwLock` permits multiple readers OR one writer
- Read-heavy workload optimization (process monitoring reads frequently)

**State Updates:**
```rust
// Read pattern (multiple concurrent readers)
let active = state.active_session.read().await;

// Write pattern (exclusive access)
let mut active = state.active_session.write().await;
*active = Some(new_session);
```

## Process Monitoring Loop Design

### Efficiency Optimizations

1. **Delta Updates**: sysinfo only refreshes changed processes
2. **Interval-based**: 2-second checks (not continuous polling)
3. **Early exit**: Skip monitoring when blocking disabled
4. **Hash-based tracking**: Avoid duplicate warnings per PID

### Flow

```
Every 2 seconds:
├─ Check if blocking enabled → exit if not
├─ Fetch blocked apps from DB
├─ Refresh process list (delta only)
├─ For each process:
│  ├─ Match against blocked list
│  ├─ Send warning notification (first time)
│  ├─ Schedule termination after 3 seconds
│  └─ Track in warned_processes HashSet
└─ Clean up exited processes from tracking
```

### CPU Usage Considerations

- System::refresh_processes() uses platform APIs efficiently
- No string allocations in hot path
- HashSet lookups are O(1)
- Async interval doesn't block executor

## Security Considerations

### Privilege Escalation
- Never auto-elevate privileges
- Provide clear instructions to user
- Graceful degradation if unavailable

### Data Protection
- SQLite database in app data directory
- No network transmission (sync is local export/import)
- Sensitive process data stays in memory briefly

### SQL Injection Prevention
- All queries use SQLx parameterized statements
- Compile-time query validation
- No dynamic SQL construction

## Performance Characteristics

### Database
- WAL mode for better concurrency
- Connection pooling reduces overhead
- Indices on start_time, item_type
- Prepared statement caching

### Memory Usage
- ~10-20MB baseline (depends on system process count)
- Database connection pool: 5 connections
- Process monitoring: O(n) memory where n = blocked processes
- String cloning minimized with borrows

### Startup Time
- Database migration check: ~50ms
- Initial process scan: ~100ms
- Total to ready: <500ms

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_focusflow_entries() {
        // Test hosts file parsing
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        // Test session start → end flow
    }
}
```

### Integration Tests
- In-memory SQLite for database tests
- Mock system processes for monitoring
- Temporary hosts file for modification tests

## Future Enhancements

1. **DNS-level blocking**: Local DNS proxy for additional security
2. **Machine learning**: Predict optimal focus times
3. **Cloud sync**: Optional encrypted cloud backup
4. **Browser extensions**: Deep integration with web blocking
5. **Screen time API**: Native macOS/Windows screen time integration

## Build & Deployment

### Dependencies
- Rust 1.70+ (2021 edition)
- SQLx CLI for migrations
- Tauri CLI 2.0

### Compilation
```bash
# Development build
cargo tauri dev

# Production build with optimizations
cargo tauri build --release
```

### Release Profile Optimizations
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
strip = true           # Remove debug symbols
```

## Conclusion

FocusFlow's backend leverages Rust's strengths:
- **Memory safety** without garbage collection
- **Fearless concurrency** with ownership system
- **Zero-cost abstractions** for performance
- **Rich type system** for correctness

The architecture prioritizes user experience through non-blocking async operations, efficient resource usage, and graceful error handling.
