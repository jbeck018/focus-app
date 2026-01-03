# FocusFlow Chat History & LLM Memory Schema Design

## Overview

This document describes the database schema design for chat history and LLM memory persistence in FocusFlow. The schema is built using SQLite with sqlx for async operations, following the existing FocusFlow patterns.

## Architecture Decisions

### Technology Choice: SQLite
- **Rationale**: Already in use throughout FocusFlow, zero-configuration, embedded database
- **Benefits**: No additional dependencies, ACID compliance, excellent performance for local storage
- **Trade-offs**: Single-writer model (acceptable for desktop app), limited concurrent access

### Data Model: Relational with JSON Fields
- **Core tables**: Normalized relational design for structured data
- **JSON fields**: `tool_calls` and `key_topics` stored as JSON for flexibility
- **Indexing strategy**: Partial indexes on filtered queries, composite indexes for common patterns

### Retention Policy: 30 Days Active + Summarization
- Active conversations kept for 30 days with full message history
- Older conversations can be summarized to reduce storage
- Soft deletes throughout for data recovery and sync

## Schema Tables

### 1. conversations

Stores conversation metadata and aggregate statistics.

```sql
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    title TEXT NOT NULL,
    summary TEXT,                          -- Auto-generated conversation summary
    message_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,  -- For context window management
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived BOOLEAN NOT NULL DEFAULT 0,
    device_id TEXT,                        -- For multi-device sync
    synced_at TEXT,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted BOOLEAN DEFAULT 0
)
```

**Design Rationale**:
- `message_count` and `total_tokens`: Pre-computed aggregates to avoid expensive COUNT queries
- `archived`: Separate from `deleted` to support user-initiated archival
- Sync columns (`device_id`, `synced_at`, `last_modified`): Future-proofing for cloud sync
- Soft delete pattern: Maintains referential integrity and enables data recovery

**Indexes**:
```sql
-- Primary access pattern: Recent conversations for user
CREATE INDEX idx_conversations_user_updated
ON conversations(user_id, updated_at DESC)
WHERE deleted = 0;

-- History view: All recent non-archived conversations
CREATE INDEX idx_conversations_created
ON conversations(created_at DESC)
WHERE deleted = 0 AND archived = 0;

-- Sync queries: Find conversations needing sync
CREATE INDEX idx_conversations_sync
ON conversations(user_id, last_modified)
WHERE synced_at IS NULL OR last_modified > synced_at;
```

### 2. messages

Stores individual chat messages with full content and metadata.

```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    token_count INTEGER,                   -- For context window tracking
    tool_calls TEXT,                       -- JSON: Tool invocations made by LLM
    model_used TEXT,                       -- Track which model generated response
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    device_id TEXT,
    synced_at TEXT,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted BOOLEAN DEFAULT 0,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
)
```

**Design Rationale**:
- `role` constraint: Enforces valid OpenAI-style message roles
- `tool_calls` as JSON: Stores complex tool invocation data without additional tables
- `token_count`: Critical for LLM context window management (e.g., 8K, 16K, 128K limits)
- `model_used`: Audit trail for which model version generated responses
- Cascade delete: When conversation is deleted, messages go with it

**Indexes**:
```sql
-- Primary access: Get messages for conversation in chronological order
CREATE INDEX idx_messages_conversation
ON messages(conversation_id, created_at ASC)
WHERE deleted = 0;

-- Filter by role: Useful for building context with only assistant/user messages
CREATE INDEX idx_messages_role
ON messages(conversation_id, role, created_at DESC)
WHERE deleted = 0;

-- Recent messages across all conversations (for global context)
CREATE INDEX idx_messages_recent
ON messages(created_at DESC)
WHERE deleted = 0;
```

### 3. memory

Stores extracted user preferences, facts, and patterns for LLM context persistence.

```sql
CREATE TABLE memory (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    key TEXT NOT NULL,                     -- e.g., "preferred_break_duration", "work_schedule"
    value TEXT NOT NULL,                   -- The actual memory value
    category TEXT NOT NULL CHECK(category IN ('preference', 'fact', 'pattern', 'goal', 'context')),
    confidence REAL NOT NULL DEFAULT 1.0 CHECK(confidence >= 0.0 AND confidence <= 1.0),
    source_conversation_id TEXT,           -- Traceability: where was this learned?
    source_message_id TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 0,  -- Track usage for importance
    last_accessed TEXT,
    expires_at TEXT,                       -- Optional expiration for temporary context
    device_id TEXT,
    synced_at TEXT,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted BOOLEAN DEFAULT 0,
    FOREIGN KEY (source_conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (source_message_id) REFERENCES messages(id) ON DELETE SET NULL
)
```

**Design Rationale**:
- **Categories**:
  - `preference`: User preferences (e.g., "prefers 25-minute focus sessions")
  - `fact`: Concrete user facts (e.g., "works as a software engineer")
  - `pattern`: Observed behavioral patterns (e.g., "most productive in mornings")
  - `goal`: User-stated goals (e.g., "wants to reduce phone usage")
  - `context`: Temporary context (e.g., "working on project X")
- `confidence`: Probabilistic memory - can be updated as evidence changes
- `access_count`: Tracks how often memory is used (proxy for importance)
- `expires_at`: Allows temporary context to auto-expire
- Source tracking: Full traceability to originating conversation/message

**Indexes**:
```sql
-- Primary access: Get memories by category
CREATE INDEX idx_memory_user_category
ON memory(user_id, category, last_updated DESC)
WHERE deleted = 0;

-- Lookup by key
CREATE INDEX idx_memory_key
ON memory(key, user_id)
WHERE deleted = 0;

-- Get high-confidence active memories (for LLM context)
CREATE INDEX idx_memory_confidence
ON memory(user_id, confidence DESC, last_updated DESC)
WHERE deleted = 0 AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP);

-- Recently accessed memories
CREATE INDEX idx_memory_accessed
ON memory(user_id, last_accessed DESC)
WHERE deleted = 0;
```

### 4. conversation_summaries

Stores summaries of older conversations to reduce context window usage.

```sql
CREATE TABLE conversation_summaries (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    summary_text TEXT NOT NULL,
    key_topics TEXT,                       -- JSON array: ["focus", "blocking", "goals"]
    summary_tokens INTEGER,                -- Token count of summary
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    summarized_messages_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
)
```

**Design Rationale**:
- **Summarization strategy**: When conversation exceeds certain age/size, create summary
- **Key topics**: JSON array enables topic-based filtering without full-text search
- **Token tracking**: Ensure summaries are actually more efficient than full messages
- Multiple summaries per conversation: Support incremental summarization

**Indexes**:
```sql
-- Get summaries for conversation
CREATE INDEX idx_summaries_conversation
ON conversation_summaries(conversation_id, created_at DESC);
```

## Query Patterns & Performance

### Common Access Patterns

#### 1. Get Recent Conversations (Last 30 Days)
```rust
get_recent_conversations(pool, user_id, limit)
```
- **Query**: Filters by `created_at >= 30 days ago`, non-deleted, non-archived
- **Index used**: `idx_conversations_created`
- **Performance**: O(log n) index scan + O(limit) rows

#### 2. Load Conversation with Messages
```rust
let conv = get_conversation(pool, conversation_id).await?;
let messages = get_messages_for_conversation(pool, conversation_id, None).await?;
```
- **Query 1**: Single primary key lookup - O(1)
- **Query 2**: Range scan on `idx_messages_conversation` - O(message_count)
- **Optimization**: Messages ordered ASC for chronological display

#### 3. Build LLM Context
```rust
let memories = get_all_active_memories(pool, user_id, 0.7).await?;
let recent_msgs = get_recent_messages(pool, user_id, 50).await?;
```
- **Query 1**: Uses `idx_memory_confidence` for high-confidence memories
- **Query 2**: Uses `idx_messages_recent` across all conversations
- **Token management**: Calculate total tokens to fit context window

#### 4. Store New Message
```rust
insert_message(pool, id, conv_id, role, content, tokens, tools, model).await?;
```
- **Transaction**: Inserts message + updates conversation aggregates
- **Atomicity**: Both succeed or both fail (ACID guarantee)
- **Side effects**: `message_count++`, `total_tokens += tokens`, `updated_at = now()`

### Performance Characteristics

| Operation | Complexity | Index Used |
|-----------|-----------|------------|
| Get conversation by ID | O(1) | Primary key |
| Get recent conversations | O(log n + k) | `idx_conversations_user_updated` |
| Get messages for conversation | O(m) | `idx_messages_conversation` |
| Get high-confidence memories | O(log n + k) | `idx_memory_confidence` |
| Insert message | O(log n) | Primary key + index updates |
| Search by memory key | O(log n) | `idx_memory_key` |

Where:
- n = total rows in table
- k = limit (number of rows returned)
- m = messages in conversation

## Context Window Management

### Token Tracking Strategy

1. **Per-Message Tokens**: Stored in `messages.token_count`
2. **Conversation Totals**: Pre-computed in `conversations.total_tokens`
3. **Summary Tokens**: Tracked in `conversation_summaries.summary_tokens`

### Context Building Algorithm

```rust
fn build_llm_context(pool: &SqlitePool, user_id: &str, max_tokens: i64) -> Result<String> {
    let mut context = String::new();
    let mut tokens_used = 0;

    // 1. Add high-confidence memories (preferences, facts)
    let memories = get_all_active_memories(pool, user_id, 0.8).await?;
    for mem in memories {
        // Add memory to context
        tokens_used += estimate_tokens(&mem.value);
    }

    // 2. Add recent messages (up to remaining token budget)
    let remaining = max_tokens - tokens_used;
    let messages = get_recent_messages(pool, user_id, 100).await?;

    for msg in messages.iter().rev() {
        if tokens_used + msg.token_count > remaining {
            break;
        }
        context.push_str(&msg.content);
        tokens_used += msg.token_count;
    }

    // 3. If old conversation, use summary instead of full history
    if conversation_age > 7 days {
        let summary = get_latest_conversation_summary(pool, conv_id).await?;
        // Use summary instead of all messages
    }

    Ok(context)
}
```

## Data Lifecycle & Maintenance

### Automatic Cleanup

```rust
// Run periodically (e.g., daily via background task)
async fn maintain_chat_database(pool: &SqlitePool) -> Result<()> {
    // 1. Clean up expired memories
    let expired = cleanup_expired_memories(pool).await?;
    tracing::info!("Cleaned up {} expired memories", expired);

    // 2. Summarize old conversations (> 7 days with >50 messages)
    let old_convs = find_conversations_to_summarize(pool).await?;
    for conv in old_convs {
        summarize_conversation(pool, &conv.id).await?;
    }

    // 3. Archive conversations with no activity in 30 days
    archive_inactive_conversations(pool, 30).await?;

    Ok(())
}
```

### Storage Estimates

**Per Message**: ~500 bytes average (content + metadata)
**Per Memory**: ~200 bytes average
**Per Conversation**: ~300 bytes + (messages * 500)

**Example**: 100 conversations with 50 messages each = 2.5MB + overhead

## Migration & Versioning

Migrations are idempotent and versioned using the existing FocusFlow pattern:

```rust
// In migrations.rs
run_if_needed(pool, 17, "create_conversations_table").await?;
run_if_needed(pool, 18, "create_messages_table").await?;
run_if_needed(pool, 19, "create_memory_table").await?;
run_if_needed(pool, 20, "create_conversation_summaries_table").await?;
run_if_needed(pool, 21, "create_chat_indices").await?;
```

### Schema Evolution Strategy

1. **Additive changes**: New columns with DEFAULT values (no migration needed)
2. **Breaking changes**: Create new migration + backfill + deprecation period
3. **Index changes**: Separate migration from table changes for rollback safety

## Security & Privacy Considerations

### Local-First Architecture
- All data stored locally in SQLite (no cloud upload by default)
- User controls data retention and deletion
- Soft deletes enable recovery but also require explicit purge operation

### Sensitive Data Handling
- No encryption at rest (relies on OS-level disk encryption)
- Messages stored as plaintext for full-text search capability
- Consider: Future enhancement to encrypt `content` field with user key

### Multi-User Support
- `user_id` field supports multiple users on same device
- Queries always filter by `user_id` to prevent data leakage
- NULL user_id = local-only data

## Future Enhancements

### Phase 2: Cloud Sync
- Leverage existing sync columns (`device_id`, `synced_at`, `last_modified`)
- Conflict resolution: Last-write-wins with `last_modified` timestamp
- Incremental sync: Query `WHERE last_modified > last_sync_time`

### Phase 3: Full-Text Search
```sql
-- Virtual table for FTS5
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    conversation_id UNINDEXED,
    content='messages',
    content_rowid='id'
);
```

### Phase 4: Vector Embeddings
```sql
-- Store embeddings for semantic search
ALTER TABLE messages ADD COLUMN embedding BLOB;
CREATE INDEX idx_messages_embedding ON messages(embedding) USING vss;
```

### Phase 5: Conversation Analytics
```sql
CREATE TABLE conversation_analytics (
    conversation_id TEXT PRIMARY KEY,
    avg_response_time_ms INTEGER,
    user_satisfaction_score REAL,
    topics_discussed TEXT,  -- JSON array
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
```

## Usage Examples

### Create New Conversation & Add Messages
```rust
use uuid::Uuid;
use focusflow_lib::db::chat_queries::*;

// Create conversation
let conv_id = Uuid::new_v4().to_string();
create_conversation(&pool, &conv_id, Some(user_id), "How to focus better").await?;

// Add user message
let msg_id = Uuid::new_v4().to_string();
insert_message(
    &pool,
    &msg_id,
    &conv_id,
    "user",
    "What's the best focus technique?",
    Some(10),  // token count
    None,      // no tool calls
    None,      // user message
).await?;

// Add assistant response
let msg_id2 = Uuid::new_v4().to_string();
insert_message(
    &pool,
    &msg_id2,
    &conv_id,
    "assistant",
    "The Pomodoro Technique is highly effective...",
    Some(50),
    None,
    Some("gpt-4"),
).await?;
```

### Store & Retrieve Memory
```rust
// Extract and store memory from conversation
let mem_id = Uuid::new_v4().to_string();
upsert_memory(
    &pool,
    &mem_id,
    Some(user_id),
    "preferred_focus_duration",
    "25 minutes",
    "preference",
    0.95,  // high confidence
    Some(&conv_id),
    Some(&msg_id),
    None,  // no expiration
).await?;

// Retrieve memory when needed
if let Some(mem) = get_memory_by_key(&pool, "preferred_focus_duration", Some(user_id)).await? {
    println!("User prefers: {}", mem.value);
    println!("Confidence: {}", mem.confidence);
    println!("Used {} times", mem.access_count);
}
```

### Build LLM Context
```rust
// Get all relevant context for LLM
let memories = get_all_active_memories(&pool, Some(user_id), 0.7).await?;
let recent = get_recent_messages(&pool, Some(user_id), 50).await?;

let mut context = String::from("## User Memories\n");
for mem in memories {
    context.push_str(&format!("{}: {}\n", mem.key, mem.value));
}

context.push_str("\n## Recent Conversations\n");
for msg in recent.iter().rev() {
    context.push_str(&format!("{}: {}\n", msg.role, msg.content));
}

// Use context in LLM call
let response = llm_client.chat(&context, user_query).await?;
```

## Files Modified/Created

### New Files
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/db/chat_queries.rs`
  - All query functions for chat, messages, memory, summaries
  - Rust structs with `sqlx::FromRow` derive macros
  - Full CRUD operations with soft delete support

### Modified Files
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/db/migrations.rs`
  - Added migrations 17-21 for chat schema
  - Created tables with proper constraints and foreign keys
  - Created optimized indexes for common query patterns

- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/db/mod.rs`
  - Added `pub mod chat_queries;` to expose new module

## Testing & Validation

### Unit Tests (Future)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conversation_lifecycle() {
        let pool = setup_test_db().await;

        // Create conversation
        create_conversation(&pool, "test-1", None, "Test").await.unwrap();

        // Verify it exists
        let conv = get_conversation(&pool, "test-1").await.unwrap();
        assert_eq!(conv.title, "Test");

        // Add messages
        insert_message(&pool, "msg-1", "test-1", "user", "Hello", Some(5), None, None).await.unwrap();

        // Verify counts updated
        let conv = get_conversation(&pool, "test-1").await.unwrap();
        assert_eq!(conv.message_count, 1);
        assert_eq!(conv.total_tokens, 5);
    }
}
```

### Performance Testing
```rust
// Benchmark: Insert 1000 messages
let start = Instant::now();
for i in 0..1000 {
    insert_message(&pool, &format!("msg-{}", i), conv_id, "user", "test", Some(10), None, None).await?;
}
println!("1000 inserts: {:?}", start.elapsed());

// Benchmark: Query recent conversations
let start = Instant::now();
let convs = get_recent_conversations(&pool, Some(user_id), 100).await?;
println!("Query 100 conversations: {:?}", start.elapsed());
```

## Conclusion

This schema provides a solid foundation for chat history and LLM memory in FocusFlow with:

- **Efficient storage**: Normalized design with pre-computed aggregates
- **Flexible querying**: Comprehensive indexes for all access patterns
- **Future-proof**: Sync columns, soft deletes, extensible design
- **Context-aware**: Token tracking and summarization for LLM integration
- **Privacy-first**: Local storage with user control over data

The implementation follows FocusFlow's existing patterns and integrates seamlessly with the sqlx-based architecture.
