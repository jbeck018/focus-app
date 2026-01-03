# TrailBase Backend Integration

This module provides a comprehensive TrailBase backend integration for team collaboration features in the FocusFlow app.

## Architecture Overview

The integration follows a trait-based, offline-first architecture with the following components:

### Core Components

1. **TrailBaseClient** (`client.rs`)
   - HTTP client for TrailBase API
   - Authentication (login, register, token refresh)
   - RESTful API methods (GET, POST, PUT, DELETE)
   - Entity sync operations

2. **SyncableEntity Trait** (`sync.rs`)
   - Generic trait for entities that can sync with TrailBase
   - Conflict resolution strategies (last-write-wins, custom merge)
   - Validation before sync
   - Automatic timestamp management

3. **Data Models** (`models.rs`)
   - `Team`: Team entity with invite codes
   - `Member`: Team member with roles (Owner, Admin, Member)
   - `SharedSession`: Shared focus sessions with sync status
   - All models implement `SyncableEntity`

4. **Sync Queue** (`sync.rs`)
   - Offline-first sync queue with retry logic
   - Queues operations when offline
   - Automatic retry with exponential backoff
   - Failed operation tracking and manual retry

## Database Schema

### team_connection
Stores the current team connection state:
```sql
CREATE TABLE team_connection (
    id TEXT PRIMARY KEY,
    server_url TEXT NOT NULL,
    team_id TEXT,
    user_id TEXT,
    api_key TEXT,
    connected_at INTEGER NOT NULL
);
```

### shared_sessions
Tracks shared focus sessions:
```sql
CREATE TABLE shared_sessions (
    id TEXT PRIMARY KEY,
    local_session_id TEXT NOT NULL,
    remote_session_id TEXT,
    team_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    planned_duration_minutes INTEGER NOT NULL,
    actual_duration_seconds INTEGER,
    completed BOOLEAN NOT NULL DEFAULT 0,
    shared_at INTEGER NOT NULL,
    last_modified TEXT NOT NULL,
    sync_status TEXT DEFAULT 'pending' CHECK(sync_status IN ('pending', 'synced', 'failed')),
    FOREIGN KEY (local_session_id) REFERENCES sessions(id)
);
```

### sync_queue
Offline-first sync queue:
```sql
CREATE TABLE sync_queue (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK(operation IN ('create', 'update', 'delete')),
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL,
    retry_count INTEGER DEFAULT 0,
    last_error TEXT,
    UNIQUE(entity_type, entity_id, operation)
);
```

## Tauri Commands

### Team Connection
- `connect_team(server_url, credentials)` - Authenticate with TrailBase
- `disconnect_team()` - Clear team connection

### Team Collaboration
- `get_team_members_sync()` - Fetch team members from TrailBase
- `share_session(session_id, team_id)` - Share a focus session
- `get_team_activity()` - Get team's recent activity

### Sync Management
- `sync_with_team()` - Trigger bi-directional sync
- `get_sync_status()` - Get sync queue statistics
- `retry_failed_syncs()` - Retry all failed sync operations
- `clear_failed_syncs()` - Clear permanently failed operations

## Usage Example

### Frontend (TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Connect to team
const authResponse = await invoke('connect_team', {
  serverUrl: 'https://api.trailbase.io',
  credentials: {
    email: 'user@example.com',
    password: 'password123'
  }
});

// Share a session
const sharedId = await invoke('share_session', {
  sessionId: 'local-session-123',
  teamId: 'team-456'
});

// Trigger sync
const stats = await invoke('sync_with_team');
console.log(`Synced: ${stats.synced}, Failed: ${stats.failed}, Pending: ${stats.pending}`);

// Get team activity
const activity = await invoke('get_team_activity');
console.log(`Total sessions: ${activity.total_sessions}`);
console.log(`Focus hours: ${activity.total_focus_hours}`);
```

### Backend (Rust)

```rust
use crate::trailbase::{TrailBaseClient, SyncableEntity, SyncQueue};
use crate::trailbase::models::SharedSession;

// Create client
let mut client = TrailBaseClient::new("https://api.trailbase.io".to_string())?;

// Authenticate
let auth = client.authenticate(credentials).await?;

// Sync an entity
let result = client.sync_entity(&shared_session).await?;

// Use sync queue for offline support
let sync_queue = SyncQueue::new(pool.clone());
sync_queue.enqueue(&shared_session, SyncOperationType::Create).await?;
```

## Implementing SyncableEntity

To make a new entity syncable:

```rust
use crate::trailbase::sync::SyncableEntity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyEntity {
    pub local_id: String,
    pub remote_id: Option<String>,
    pub data: String,
    pub last_modified: DateTime<Utc>,
}

impl SyncableEntity for MyEntity {
    fn entity_type() -> &'static str {
        "my_entities"
    }

    fn local_id(&self) -> &str {
        &self.local_id
    }

    fn remote_id(&self) -> Option<&str> {
        self.remote_id.as_deref()
    }

    fn set_remote_id(&mut self, remote_id: String) {
        self.remote_id = Some(remote_id);
    }

    fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }

    fn set_last_modified(&mut self, timestamp: DateTime<Utc>) {
        self.last_modified = timestamp;
    }

    // Optional: Custom validation
    fn validate(&self) -> Result<()> {
        if self.data.is_empty() {
            return Err(Error::Validation("Data cannot be empty".to_string()));
        }
        Ok(())
    }

    // Optional: Custom conflict resolution
    fn merge_with(&self, remote: &Self) -> Self {
        // Custom merge logic
        // Default is last-write-wins based on timestamp
        if self.last_modified > remote.last_modified {
            self.clone()
        } else {
            remote.clone()
        }
    }
}
```

## Conflict Resolution

The sync system supports multiple conflict resolution strategies:

1. **LastWriteWins** (default): Uses timestamp to determine winner
2. **LocalWins**: Always use local version
3. **RemoteWins**: Always use remote version
4. **CustomMerge**: Use entity-specific `merge_with()` method

Example:
```rust
use crate::trailbase::sync::{resolve_conflict, ConflictStrategy};

let merged = resolve_conflict(&local, &remote, ConflictStrategy::CustomMerge);
```

## Offline-First Sync Flow

1. **User Action** → Entity is created/updated locally
2. **Queue Operation** → Operation added to sync queue
3. **Sync Triggered** → Manual or periodic sync
4. **Process Queue** → Each operation is sent to TrailBase
5. **Handle Result**:
   - Success → Remove from queue, update local record
   - Failure → Increment retry count, store error
   - Conflict → Apply resolution strategy

## Error Handling

All operations return `Result<T>` with proper error types:

```rust
pub enum Error {
    Network(String),      // HTTP/network errors
    Auth(String),         // Authentication failures
    Sync(String),         // Sync-specific errors
    Validation(String),   // Entity validation errors
    NotFound(String),     // Entity not found
    // ... other variants
}
```

## Security Considerations

- API keys stored in SQLite database (should be encrypted in production)
- HTTPS required for all TrailBase communication
- JWT tokens automatically refreshed before expiration
- User data only shared with explicit consent

## Performance

- Sync queue batches operations (max 50 per sync)
- Indices on team_id, user_id, and sync_status for fast queries
- Connection pooling for database operations
- Async/await for non-blocking I/O

## Testing

Run tests with:
```bash
cargo test --lib trailbase
```

Key test coverage:
- Client authentication flow
- Sync queue operations
- Conflict resolution strategies
- Entity validation
- Model serialization/deserialization

## Future Enhancements

- [ ] Automatic background sync (every N minutes)
- [ ] Websocket support for real-time updates
- [ ] Incremental sync (only changed entities)
- [ ] Encryption for sensitive data
- [ ] Compression for large payloads
- [ ] Rate limiting and backoff strategies
- [ ] Multi-team support
- [ ] Team invitation flow
- [ ] Permission-based access control

## Dependencies

- `reqwest` - HTTP client with async support
- `serde` - Serialization/deserialization
- `sqlx` - Database operations
- `chrono` - Timestamp handling
- `uuid` - Unique ID generation
- `tokio` - Async runtime

## API Endpoints (TrailBase)

Expected TrailBase API structure:

```
POST   /auth/login           - Authenticate user
POST   /auth/register        - Register new user
POST   /auth/refresh         - Refresh access token
GET    /api/teams            - List teams
POST   /api/teams            - Create team
GET    /api/team/members     - List team members
GET    /api/team/activity    - Get team activity
POST   /api/shared_sessions  - Create shared session
PUT    /api/shared_sessions/:id - Update shared session
GET    /api/shared_sessions  - List shared sessions
```

## License

Same as parent project (FocusFlow)
