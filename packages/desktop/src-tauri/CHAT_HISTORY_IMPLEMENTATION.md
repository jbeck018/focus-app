# Chat History and Memory Management Implementation

## Overview

This implementation adds comprehensive chat history and memory management to FocusFlow's AI coach, enabling:
- Persistent conversation storage across sessions
- Context-aware responses using conversation history
- Long-term memory for user preferences, patterns, and goals
- Automatic conversation archiving and cleanup
- Token-efficient context building

## Architecture

### Database Schema

#### Conversations Table
```sql
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    title TEXT NOT NULL,
    summary TEXT,
    message_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived BOOLEAN NOT NULL DEFAULT 0,
    device_id TEXT,
    synced_at TEXT,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted BOOLEAN DEFAULT 0
)
```

**Purpose**: Tracks individual conversation threads with the AI coach.

**Key Features**:
- Auto-increments message_count and total_tokens
- Supports archiving for old conversations
- Soft delete for data recovery
- Sync support for future cloud features

#### Messages Table
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    token_count INTEGER,
    tool_calls TEXT,
    model_used TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    device_id TEXT,
    synced_at TEXT,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted BOOLEAN DEFAULT 0,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
)
```

**Purpose**: Stores individual messages within conversations.

**Key Features**:
- Tracks role (user, assistant, system) for proper context
- Records token count for context window management
- Stores tool calls for debugging and analysis
- Tracks which model generated the response

#### Memory Table
```sql
CREATE TABLE memory (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('preference', 'fact', 'pattern', 'goal', 'context')),
    confidence REAL NOT NULL DEFAULT 1.0 CHECK(confidence >= 0.0 AND confidence <= 1.0),
    source_conversation_id TEXT,
    source_message_id TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed TEXT,
    expires_at TEXT,
    device_id TEXT,
    synced_at TEXT,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    deleted BOOLEAN DEFAULT 0
)
```

**Purpose**: Long-term storage of user information for context enrichment.

**Categories**:
- **preference**: User preferences (e.g., "prefers morning sessions")
- **fact**: Factual information (e.g., "works as software engineer")
- **pattern**: Behavioral patterns (e.g., "gets distracted after 45 minutes")
- **goal**: User goals (e.g., "wants to increase focus to 4 hours daily")
- **context**: General context (e.g., "preparing for exam next week")

**Key Features**:
- Confidence scoring for reliability
- Source tracking for provenance
- Access counting for relevance ranking
- Expiration support for temporary context
- Upsert on (user_id, key, category) for updates

#### Conversation Summaries Table
```sql
CREATE TABLE conversation_summaries (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    summary_text TEXT NOT NULL,
    key_topics TEXT,
    summary_tokens INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    summarized_messages_count INTEGER NOT NULL DEFAULT 0
)
```

**Purpose**: Compress old conversation context for efficient retrieval.

## Modules

### 1. `commands/chat_history.rs`

Main command module for chat history management.

#### Conversation Management Commands

**`create_conversation(title: Option<String>) -> String`**
- Creates a new conversation thread
- Auto-generates timestamp-based title if not provided
- Returns conversation ID

**`list_conversations(limit: i32, offset: i32, include_archived: bool) -> Vec<ConversationSummary>`**
- Lists conversations with pagination
- Filters archived conversations by default
- Orders by most recent activity

**`get_conversation(conversation_id: String) -> ConversationWithMessages`**
- Retrieves complete conversation with all messages
- Includes conversation metadata and chronologically ordered messages

**`delete_conversation(conversation_id: String) -> ()`**
- Soft deletes conversation and all messages
- Preserves data for potential recovery

**`archive_conversation(conversation_id: String) -> ()`**
- Archives old conversations
- Removes from active list but keeps accessible

#### Message Management Commands

**`add_message(conversation_id: String, role: String, content: String, token_count: Option<i32>, model_used: Option<String>) -> Message`**
- Adds message to conversation
- Updates conversation metadata (count, tokens)
- Auto-checks for archiving threshold (500 messages)

**`get_recent_messages(conversation_id: String, limit: i32) -> Vec<Message>`**
- Retrieves recent N messages
- Limited to 200 messages max for performance
- Returns in chronological order

#### Memory Management Commands

**`save_memory(key: String, value: String, category: String, confidence: f64, source_conversation_id: Option<String>) -> String`**
- Upserts memory entry
- Updates existing memory or creates new
- Tracks source conversation for provenance

**`get_memories(category: Option<String>, limit: i32) -> Vec<Memory>`**
- Retrieves memories by category
- Filters expired memories
- Orders by confidence and recency

**`get_relevant_memories(context: String, limit: i32) -> Vec<Memory>`**
- Finds memories relevant to current context
- Uses keyword matching (can be enhanced with embeddings)
- Updates access count and timestamp

#### Maintenance Commands

**`auto_archive_old_conversations() -> i32`**
- Archives conversations inactive for 30+ days
- Returns count of archived conversations
- Run periodically via background task

**`cleanup_expired_memories() -> i32`**
- Removes memories past expiration
- Returns count of cleaned memories

#### Context Building Commands

**`build_conversation_context(conversation_id: String, user_message: Option<String>, max_messages: i32) -> ConversationContext`**
- Builds rich context for AI prompts
- Includes recent messages, relevant memories, and summary
- Limited to 50 messages for performance

**`update_conversation_title(conversation_id: String, title: String) -> ()`**
- Updates conversation title
- Used for auto-generated titles from first message

### 2. `commands/chat_context.rs`

Context building utilities for enhancing AI responses.

#### Key Functions

**`build_context_prompt(user_context: &UserContext, recent_messages: &[Message], relevant_memories: &[Memory], conversation_summary: Option<&str>) -> String`**
- Constructs comprehensive context prompt
- Includes:
  - Conversation summary
  - Relevant memories
  - Current user stats (focus time, streak, etc.)
  - Recent conversation history
- Returns formatted string for system prompt

**`format_as_system_message(context_prompt: &str) -> String`**
- Wraps context as system message
- Includes AI coach persona

**`extract_potential_memories(content: &str, role: &MessageRole) -> Vec<(String, String, String)>`**
- Extracts memory candidates from messages
- Pattern matches for preferences and goals
- Returns (key, value, category) tuples

**`estimate_token_count(text: &str) -> i32`**
- Estimates tokens using 4 chars per token heuristic
- Useful for context window management

**`truncate_to_token_limit(messages: Vec<Message>, max_tokens: i32) -> Vec<Message>`**
- Truncates messages to fit token budget
- Preserves most recent messages
- Maintains chronological order

**`generate_conversation_title(first_message: &str) -> String`**
- Generates concise title from first message
- Truncates to 50 characters max

### 3. Enhanced `commands/coach.rs`

Integration with existing coach commands.

#### New Commands

**`send_coach_message(conversation_id: Option<String>, message: String, context: Option<UserContext>) -> CoachResponse`**
- Enhanced coaching with conversation history
- Auto-creates conversation if none provided
- Saves both user and assistant messages
- Extracts and saves memories automatically
- Auto-generates title from first message
- Integrates with guideline orchestrator

**Flow**:
1. Get or create conversation
2. Build conversation context (messages + memories)
3. Enhance orchestrated prompt with context
4. Save user message
5. Generate response (LLM or template fallback)
6. Save assistant response
7. Extract and save memories
8. Update title if first message

**`get_or_create_active_conversation() -> String`**
- Helper to get most recent conversation
- Creates new if none exists
- Useful for frontend state management

## Usage Examples

### Creating a Conversation

```typescript
// Frontend TypeScript
const conversationId = await invoke('create_conversation', {
  title: 'Focus Strategy Discussion'
});
```

### Sending a Message with History

```typescript
const response = await invoke('send_coach_message', {
  conversationId: currentConversationId, // or null to create new
  message: 'How can I improve my focus during afternoon slumps?',
  context: null // auto-fetches if not provided
});

console.log(response.message);
console.log(response.suggestions);
```

### Listing Conversations

```typescript
const conversations = await invoke('list_conversations', {
  limit: 20,
  offset: 0,
  includeArchived: false
});

conversations.forEach(conv => {
  console.log(`${conv.title} - ${conv.message_count} messages`);
});
```

### Retrieving Full Conversation

```typescript
const full = await invoke('get_conversation', {
  conversationId: 'uuid-here'
});

full.messages.forEach(msg => {
  console.log(`${msg.role}: ${msg.content}`);
});
```

### Managing Memories

```typescript
// Save a preference
await invoke('save_memory', {
  key: 'preferred_session_length',
  value: '45 minutes',
  category: 'preference',
  confidence: 0.9,
  sourceConversationId: conversationId
});

// Get preferences
const prefs = await invoke('get_memories', {
  category: 'preference',
  limit: 50
});

// Get relevant memories for context
const relevant = await invoke('get_relevant_memories', {
  context: 'focus session planning',
  limit: 5
});
```

## Performance Considerations

### Indexing Strategy

All critical queries are covered by indices:

```sql
-- Conversations
CREATE INDEX idx_conversations_user_updated ON conversations(user_id, updated_at DESC);
CREATE INDEX idx_conversations_created ON conversations(created_at DESC);

-- Messages
CREATE INDEX idx_messages_conversation ON messages(conversation_id, created_at ASC);
CREATE INDEX idx_messages_role ON messages(conversation_id, role, created_at DESC);

-- Memory
CREATE INDEX idx_memory_user_category ON memory(user_id, category, last_updated DESC);
CREATE INDEX idx_memory_key ON memory(key, user_id);
CREATE INDEX idx_memory_confidence ON memory(user_id, confidence DESC, last_updated DESC);
```

### Query Optimization

- **Pagination**: All list queries use LIMIT/OFFSET
- **Soft Deletes**: WHERE clauses filter deleted=0 at index level
- **Token Counting**: Estimated heuristically to avoid tokenizer overhead
- **Context Window**: Limited to 50 messages max (configurable)

### Memory Management

- **Auto-archiving**: Conversations older than 30 days
- **Message Limit**: Warns at 500 messages per conversation
- **Expired Memories**: Cleaned up via background task
- **Confidence Decay**: Lower confidence memories rank lower

## Future Enhancements

### Semantic Search

Replace keyword matching with vector embeddings:

```rust
// Future implementation
async fn get_relevant_memories_semantic(
    embeddings: &EmbeddingModel,
    query: &str,
    limit: i32
) -> Result<Vec<Memory>> {
    let query_embedding = embeddings.embed(query).await?;
    // Vector similarity search in memory table
}
```

### Conversation Summarization

Automatically summarize old conversations:

```rust
async fn summarize_conversation(
    conversation_id: String,
    llm: &LlmEngine
) -> Result<String> {
    let messages = get_messages(conversation_id).await?;
    let summary = llm.summarize(&messages).await?;
    save_summary(conversation_id, summary).await
}
```

### Multi-turn Tool Execution

Enable the coach to use tools across multiple turns:

```rust
// Track tool execution state across conversation
struct ToolExecutionState {
    pending_confirmations: Vec<ToolCall>,
    executed_tools: Vec<ToolResult>,
}
```

### Privacy-Preserving Sync

Encrypt sensitive memories before sync:

```rust
async fn sync_memories(
    memories: Vec<Memory>,
    encryption_key: &[u8]
) -> Result<()> {
    for memory in memories {
        let encrypted = encrypt(&memory.value, encryption_key)?;
        sync_to_cloud(encrypted).await?;
    }
}
```

## Testing

All functions compile without warnings and pass clippy checks.

### Unit Tests

Located in `commands/chat_context.rs`:

```rust
#[test]
fn test_estimate_token_count() { /* ... */ }

#[test]
fn test_generate_conversation_title() { /* ... */ }

#[test]
fn test_extract_potential_memories() { /* ... */ }
```

### Integration Testing

Recommended test scenarios:

1. **Create conversation → Send messages → Retrieve conversation**
2. **Memory persistence across sessions**
3. **Context building with memories and history**
4. **Auto-archiving old conversations**
5. **Cleanup of expired memories**

## Security Considerations

### SQL Injection Protection

- All queries use parameterized bindings via sqlx
- No string concatenation in SQL
- Type-safe query builders

### Data Privacy

- User ID scoping on all queries
- Soft deletes preserve data for recovery
- Optional encryption at rest (future)
- Local-first architecture (no cloud by default)

### Memory Confidence

- Confidence scoring prevents low-quality memories
- Source tracking for provenance
- User can manually verify/edit memories (future UI)

## Migration Path

Migrations are idempotent and versioned:

- **Migration 17**: conversations table
- **Migration 18**: messages table
- **Migration 19**: memory table
- **Migration 20**: conversation_summaries table
- **Migration 21**: indices for all tables

Run automatically on app startup via `db::migrations::run()`.

## API Reference

All commands are exposed via Tauri's IPC system and registered in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // Chat History Commands
    commands::chat_history::create_conversation,
    commands::chat_history::list_conversations,
    commands::chat_history::get_conversation,
    commands::chat_history::delete_conversation,
    commands::chat_history::archive_conversation,
    commands::chat_history::add_message,
    commands::chat_history::get_recent_messages,
    commands::chat_history::save_memory,
    commands::chat_history::get_memories,
    commands::chat_history::get_relevant_memories,
    commands::chat_history::auto_archive_old_conversations,
    commands::chat_history::cleanup_expired_memories,
    commands::chat_history::build_conversation_context,
    commands::chat_history::update_conversation_title,

    // Enhanced Coach Commands
    commands::coach::send_coach_message,
    commands::coach::get_or_create_active_conversation,
])
```

## Files Modified/Created

### Created
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/chat_history.rs` (736 lines)
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/chat_context.rs` (238 lines)

### Modified
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/db/migrations.rs` (Added migrations 17-21)
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/coach.rs` (Added `send_coach_message` and `get_or_create_active_conversation`)
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/mod.rs` (Added chat_history and chat_context modules)
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/lib.rs` (Registered 16 new commands)
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/db/chat_queries.rs` (Added #[allow(dead_code)] for future use)

## Compilation Status

✅ Compiles without errors or warnings
✅ Passes clippy checks with no warnings in new modules
✅ All dependencies resolved
✅ Type-safe throughout
