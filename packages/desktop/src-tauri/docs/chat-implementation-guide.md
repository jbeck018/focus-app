# FocusFlow Chat Implementation Guide

## Quick Start

### Files Created/Modified

**New Files:**
1. `/src/db/chat_queries.rs` - All database query functions for chat/memory
2. `/docs/chat-schema-design.md` - Comprehensive schema documentation
3. `/docs/chat-schema-erd.md` - Entity relationship diagrams and query analysis

**Modified Files:**
1. `/src/db/migrations.rs` - Added migrations 17-21 for chat schema
2. `/src/db/mod.rs` - Exposed `chat_queries` module

### Running Migrations

Migrations run automatically when the database is initialized:

```rust
use focusflow_lib::db::Database;

// Database initialization (already in your code)
let db = Database::new(&db_path).await?;
// Migrations 17-21 run here automatically
```

### Verify Schema

```bash
# Check that tables were created
sqlite3 ~/.local/share/focusflow/focusflow.db ".tables"

# Should see:
# conversations
# messages
# memory
# conversation_summaries

# Verify indexes
sqlite3 ~/.local/share/focusflow/focusflow.db ".indices conversations"
```

## Usage Examples

### 1. Basic Chat Flow

```rust
use focusflow_lib::db::chat_queries::*;
use uuid::Uuid;

// Create new conversation
let conv_id = Uuid::new_v4().to_string();
create_conversation(
    &db.pool,
    &conv_id,
    Some("user-123"),  // user_id
    "Focus Tips Chat"
).await?;

// Add user message
let msg_id = Uuid::new_v4().to_string();
insert_message(
    &db.pool,
    &msg_id,
    &conv_id,
    "user",
    "How can I improve my focus?",
    Some(8),  // token count
    None,     // no tool calls
    None      // user message (no model)
).await?;

// Add LLM response
let response_id = Uuid::new_v4().to_string();
insert_message(
    &db.pool,
    &response_id,
    &conv_id,
    "assistant",
    "Here are three proven techniques: 1) Pomodoro...",
    Some(45),
    None,
    Some("gpt-4")
).await?;

// Fetch conversation with all messages
let conv = get_conversation(&db.pool, &conv_id).await?.unwrap();
let messages = get_messages_for_conversation(&db.pool, &conv_id, None).await?;

println!("Conversation: {}", conv.title);
println!("Messages: {}", conv.message_count);
println!("Total tokens: {}", conv.total_tokens);

for msg in messages {
    println!("[{}]: {}", msg.role, msg.content);
}
```

### 2. Memory Management

```rust
use chrono::{Duration, Utc};

// Store user preference
let mem_id = Uuid::new_v4().to_string();
upsert_memory(
    &db.pool,
    &mem_id,
    Some("user-123"),
    "preferred_focus_duration",  // key
    "25 minutes",                // value
    "preference",                // category
    0.95,                        // confidence
    Some(&conv_id),              // source conversation
    Some(&msg_id),               // source message
    None                         // no expiration
).await?;

// Store temporary context (expires in 7 days)
let temp_mem_id = Uuid::new_v4().to_string();
let expires = Utc::now() + Duration::days(7);
upsert_memory(
    &db.pool,
    &temp_mem_id,
    Some("user-123"),
    "current_project",
    "Building FocusFlow app",
    "context",
    0.8,
    Some(&conv_id),
    None,
    Some(expires)
).await?;

// Retrieve memory by key
if let Some(pref) = get_memory_by_key(&db.pool, "preferred_focus_duration", Some("user-123")).await? {
    println!("User prefers: {}", pref.value);
    println!("Confidence: {:.0}%", pref.confidence * 100.0);
    println!("Learned from conversation: {:?}", pref.source_conversation_id);
    println!("Accessed {} times", pref.access_count);
}

// Get all high-confidence memories
let memories = get_all_active_memories(&db.pool, Some("user-123"), 0.7).await?;
for mem in memories {
    println!("{} ({}): {} [confidence: {:.2}]",
        mem.key, mem.category, mem.value, mem.confidence);
}

// Get memories by category
let preferences = get_memories_by_category(&db.pool, Some("user-123"), "preference", 50).await?;
let facts = get_memories_by_category(&db.pool, Some("user-123"), "fact", 50).await?;
let patterns = get_memories_by_category(&db.pool, Some("user-123"), "pattern", 50).await?;
```

### 3. Building LLM Context

```rust
async fn build_llm_context(
    pool: &SqlitePool,
    user_id: &str,
    conversation_id: Option<&str>,
    max_tokens: i64,
) -> Result<String> {
    let mut context = String::new();
    let mut tokens_used = 0;

    // 1. System prompt
    context.push_str("You are FocusFlow AI, a productivity coach.\n\n");

    // 2. Add high-confidence memories
    context.push_str("## What I know about you:\n");
    let memories = get_all_active_memories(pool, Some(user_id), 0.7).await?;

    for mem in memories {
        let line = format!("- {}: {}\n", mem.key.replace('_', " "), mem.value);
        tokens_used += estimate_tokens(&line);
        context.push_str(&line);
    }

    // 3. Add recent conversation history
    context.push_str("\n## Recent conversation:\n");
    let remaining_tokens = max_tokens - tokens_used;

    if let Some(conv_id) = conversation_id {
        // Get messages from this conversation
        let messages = get_messages_for_conversation(pool, conv_id, None).await?;

        for msg in messages.iter().rev().take(20).rev() {
            if let Some(tokens) = msg.token_count {
                if tokens_used + tokens > remaining_tokens {
                    // Check if we have a summary
                    if let Some(summary) = get_latest_conversation_summary(pool, conv_id).await? {
                        context.push_str(&format!("\n[Earlier: {}]\n\n", summary.summary_text));
                    }
                    break;
                }
                tokens_used += tokens;
            }

            context.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
    } else {
        // Get recent messages across all conversations
        let recent = get_recent_messages(pool, Some(user_id), 20).await?;

        for msg in recent.iter().rev() {
            if let Some(tokens) = msg.token_count {
                if tokens_used + tokens > remaining_tokens {
                    break;
                }
                tokens_used += tokens;
            }

            context.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
    }

    context.push_str(&format!("\n[Context tokens: {}/{}]\n", tokens_used, max_tokens));

    Ok(context)
}

// Helper function to estimate tokens (rough approximation)
fn estimate_tokens(text: &str) -> i64 {
    (text.len() / 4) as i64  // ~4 chars per token on average
}

// Usage
let context = build_llm_context(&db.pool, "user-123", Some(&conv_id), 8000).await?;
let llm_response = your_llm_client.chat(&context, user_query).await?;
```

### 4. Conversation Summarization

```rust
async fn summarize_old_conversations(pool: &SqlitePool) -> Result<()> {
    use chrono::{Duration, Utc};

    // Find conversations older than 7 days with >50 messages and no recent summary
    let seven_days_ago = Utc::now() - Duration::days(7);

    let conversations = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT c.id, c.user_id, c.title, c.summary, c.message_count,
               c.total_tokens, c.created_at, c.updated_at, c.archived
        FROM conversations c
        LEFT JOIN conversation_summaries cs ON c.id = cs.conversation_id
        WHERE c.created_at < ?
          AND c.message_count > 50
          AND (cs.created_at IS NULL OR cs.created_at < ?)
          AND c.deleted = 0
        "#
    )
    .bind(seven_days_ago)
    .bind(seven_days_ago)
    .fetch_all(pool)
    .await?;

    for conv in conversations {
        // Get all messages for this conversation
        let messages = get_messages_for_conversation(pool, &conv.id, None).await?;

        // Build summary prompt
        let mut prompt = format!(
            "Summarize this conversation titled '{}' in 2-3 sentences:\n\n",
            conv.title
        );

        for msg in messages.iter().take(50) {  // Summarize first 50 messages
            prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }

        // Call LLM to generate summary
        let summary = your_llm_client.summarize(&prompt).await?;

        // Extract key topics (simple keyword extraction)
        let topics = extract_key_topics(&summary);
        let topics_json = serde_json::to_string(&topics)?;

        // Store summary
        let summary_id = Uuid::new_v4().to_string();
        create_conversation_summary(
            pool,
            &summary_id,
            &conv.id,
            &summary,
            Some(&topics_json),
            Some(estimate_tokens(&summary)),
            messages.len() as i64
        ).await?;

        // Update conversation summary field
        update_conversation(
            pool,
            &conv.id,
            None,
            Some(&summary),
            0,
            0
        ).await?;

        tracing::info!("Summarized conversation: {} ({})", conv.title, conv.id);
    }

    Ok(())
}

fn extract_key_topics(text: &str) -> Vec<String> {
    // Simple keyword extraction (in production, use NLP)
    let keywords = ["focus", "productivity", "blocking", "pomodoro", "goals", "habits"];

    keywords.iter()
        .filter(|&kw| text.to_lowercase().contains(kw))
        .map(|&kw| kw.to_string())
        .collect()
}
```

### 5. Memory Extraction from Conversations

```rust
async fn extract_memories_from_message(
    pool: &SqlitePool,
    message: &Message,
    conversation_id: &str,
    user_id: &str,
) -> Result<Vec<Memory>> {
    let mut extracted = Vec::new();

    // Simple pattern matching (in production, use LLM to extract)
    let content_lower = message.content.to_lowercase();

    // Extract focus duration preference
    if content_lower.contains("prefer") && content_lower.contains("minute") {
        // Parse preference (simplified)
        if let Some(duration) = parse_duration(&message.content) {
            let mem_id = Uuid::new_v4().to_string();
            upsert_memory(
                pool,
                &mem_id,
                Some(user_id),
                "preferred_focus_duration",
                &duration,
                "preference",
                0.85,  // moderate confidence from single mention
                Some(conversation_id),
                Some(&message.id),
                None
            ).await?;

            extracted.push(
                get_memory_by_key(pool, "preferred_focus_duration", Some(user_id))
                    .await?
                    .unwrap()
            );
        }
    }

    // Extract work schedule
    if content_lower.contains("work") && (content_lower.contains("morning") || content_lower.contains("evening")) {
        let schedule = if content_lower.contains("morning") {
            "morning person"
        } else {
            "evening person"
        };

        let mem_id = Uuid::new_v4().to_string();
        upsert_memory(
            pool,
            &mem_id,
            Some(user_id),
            "work_schedule_preference",
            schedule,
            "pattern",
            0.75,
            Some(conversation_id),
            Some(&message.id),
            None
        ).await?;
    }

    // Extract goals
    if content_lower.contains("goal") || content_lower.contains("want to") {
        let mem_id = Uuid::new_v4().to_string();
        upsert_memory(
            pool,
            &mem_id,
            Some(user_id),
            &format!("goal_{}", Uuid::new_v4()),  // unique key per goal
            &message.content,
            "goal",
            0.9,  // high confidence - user explicitly stated
            Some(conversation_id),
            Some(&message.id),
            None
        ).await?;
    }

    Ok(extracted)
}

fn parse_duration(text: &str) -> Option<String> {
    // Simple regex to find "X minutes" pattern
    let re = regex::Regex::new(r"(\d+)\s*minute").ok()?;
    re.captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| format!("{} minutes", m.as_str()))
}
```

### 6. Maintenance Operations

```rust
// Background task to run daily
async fn daily_maintenance(pool: &SqlitePool) -> Result<()> {
    // 1. Clean up expired memories
    let expired_count = cleanup_expired_memories(pool).await?;
    tracing::info!("Cleaned up {} expired memories", expired_count);

    // 2. Archive old inactive conversations
    let result = sqlx::query(
        r#"
        UPDATE conversations
        SET archived = 1,
            updated_at = CURRENT_TIMESTAMP,
            last_modified = CURRENT_TIMESTAMP
        WHERE updated_at < datetime('now', '-30 days')
          AND archived = 0
          AND deleted = 0
        "#
    )
    .execute(pool)
    .await?;

    tracing::info!("Archived {} old conversations", result.rows_affected());

    // 3. Summarize conversations older than 7 days
    summarize_old_conversations(pool).await?;

    // 4. Update memory confidence based on usage
    sqlx::query(
        r#"
        UPDATE memory
        SET confidence = CASE
            WHEN access_count > 10 THEN MIN(1.0, confidence + 0.05)
            WHEN access_count = 0 AND created_at < datetime('now', '-30 days') THEN MAX(0.5, confidence - 0.1)
            ELSE confidence
        END,
        last_modified = CURRENT_TIMESTAMP
        WHERE deleted = 0
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Run in background task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400)); // 24 hours
    loop {
        interval.tick().await;
        if let Err(e) = daily_maintenance(&pool).await {
            tracing::error!("Daily maintenance failed: {}", e);
        }
    }
});
```

## Integration with Tauri Commands

### Example Command Implementation

```rust
// In src/commands/chat.rs (create this file)

use crate::db::chat_queries::*;
use crate::Result;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateConversationRequest {
    title: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AddMessageRequest {
    conversation_id: String,
    role: String,
    content: String,
    token_count: Option<i64>,
}

#[tauri::command]
pub async fn create_conversation(
    db: State<'_, crate::db::Database>,
    user_id: Option<String>,
    req: CreateConversationRequest,
) -> Result<Conversation> {
    let conv_id = Uuid::new_v4().to_string();

    create_conversation(
        &db.pool,
        &conv_id,
        user_id.as_deref(),
        &req.title
    ).await?;

    let conv = get_conversation(&db.pool, &conv_id)
        .await?
        .ok_or_else(|| crate::Error::NotFound("Conversation not found".into()))?;

    Ok(conv)
}

#[tauri::command]
pub async fn get_conversations(
    db: State<'_, crate::db::Database>,
    user_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<Conversation>> {
    get_recent_conversations(&db.pool, user_id.as_deref(), limit.unwrap_or(50)).await
}

#[tauri::command]
pub async fn add_message(
    db: State<'_, crate::db::Database>,
    req: AddMessageRequest,
) -> Result<Message> {
    let msg_id = Uuid::new_v4().to_string();

    insert_message(
        &db.pool,
        &msg_id,
        &req.conversation_id,
        &req.role,
        &req.content,
        req.token_count,
        None,
        None
    ).await?;

    let msg = get_message(&db.pool, &msg_id)
        .await?
        .ok_or_else(|| crate::Error::NotFound("Message not found".into()))?;

    Ok(msg)
}

#[tauri::command]
pub async fn get_conversation_messages(
    db: State<'_, crate::db::Database>,
    conversation_id: String,
    limit: Option<i64>,
) -> Result<Vec<Message>> {
    get_messages_for_conversation(&db.pool, &conversation_id, limit).await
}

#[tauri::command]
pub async fn get_memories(
    db: State<'_, crate::db::Database>,
    user_id: Option<String>,
    category: Option<String>,
    min_confidence: Option<f64>,
) -> Result<Vec<Memory>> {
    if let Some(cat) = category {
        get_memories_by_category(&db.pool, user_id.as_deref(), &cat, 100).await
    } else {
        get_all_active_memories(&db.pool, user_id.as_deref(), min_confidence.unwrap_or(0.7)).await
    }
}

// Register commands in main.rs
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        create_conversation,
        get_conversations,
        add_message,
        get_conversation_messages,
        get_memories,
        // ... other commands
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        crate::db::migrations::run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_conversation_create_and_retrieve() {
        let pool = setup_test_db().await;
        let conv_id = "test-conv-1";

        create_conversation(&pool, conv_id, None, "Test Chat").await.unwrap();

        let conv = get_conversation(&pool, conv_id).await.unwrap().unwrap();
        assert_eq!(conv.title, "Test Chat");
        assert_eq!(conv.message_count, 0);
        assert_eq!(conv.total_tokens, 0);
    }

    #[tokio::test]
    async fn test_message_insert_updates_conversation() {
        let pool = setup_test_db().await;
        let conv_id = "test-conv-2";

        create_conversation(&pool, conv_id, None, "Test").await.unwrap();

        insert_message(&pool, "msg-1", conv_id, "user", "Hello", Some(5), None, None)
            .await.unwrap();

        let conv = get_conversation(&pool, conv_id).await.unwrap().unwrap();
        assert_eq!(conv.message_count, 1);
        assert_eq!(conv.total_tokens, 5);
    }

    #[tokio::test]
    async fn test_memory_upsert() {
        let pool = setup_test_db().await;

        upsert_memory(&pool, "mem-1", None, "pref_key", "value1", "preference", 0.8, None, None, None)
            .await.unwrap();

        let mem = get_memory_by_key(&pool, "pref_key", None).await.unwrap().unwrap();
        assert_eq!(mem.value, "value1");
        assert_eq!(mem.confidence, 0.8);

        // Update
        upsert_memory(&pool, "mem-1", None, "pref_key", "value2", "preference", 0.9, None, None, None)
            .await.unwrap();

        let mem = get_memory_by_key(&pool, "pref_key", None).await.unwrap().unwrap();
        assert_eq!(mem.value, "value2");
        assert_eq!(mem.confidence, 0.9);
    }
}
```

## Performance Considerations

### Optimize for Common Queries

```rust
// GOOD: Use limit to avoid large result sets
get_recent_conversations(&pool, user_id, 20).await?;

// BAD: Could return thousands of conversations
get_all_conversations(&pool, user_id, true, i64::MAX).await?;
```

### Batch Operations

```rust
// GOOD: Use transaction for multiple inserts
let mut tx = pool.begin().await?;

for msg in messages {
    insert_message(&mut tx, &msg.id, &conv_id, &msg.role, &msg.content, msg.tokens, None, None).await?;
}

tx.commit().await?;

// BAD: Individual transactions
for msg in messages {
    insert_message(&pool, &msg.id, &conv_id, &msg.role, &msg.content, msg.tokens, None, None).await?;
}
```

### Token Budget Management

```rust
// Always check token count before building context
let conversation_tokens = get_conversation_token_count(&pool, &conv_id).await?;

if conversation_tokens > max_context_tokens {
    // Use summary instead
    let summary = get_latest_conversation_summary(&pool, &conv_id).await?;
} else {
    // Use full messages
    let messages = get_messages_for_conversation(&pool, &conv_id, None).await?;
}
```

## Next Steps

1. **Create Tauri Commands**: Implement commands in `src/commands/chat.rs`
2. **Frontend Integration**: Build UI components to display conversations/messages
3. **LLM Integration**: Connect to your LLM provider for responses
4. **Memory Extraction**: Implement intelligent memory extraction from conversations
5. **Background Tasks**: Set up daily maintenance task
6. **Testing**: Add comprehensive unit and integration tests
7. **Analytics**: Track conversation metrics and memory usage patterns

## Troubleshooting

### Migrations Not Running

```bash
# Check migration status
sqlite3 ~/.local/share/focusflow/focusflow.db "SELECT * FROM _migrations ORDER BY id;"

# Should include:
# 17|create_conversations_table|...
# 18|create_messages_table|...
# 19|create_memory_table|...
# 20|create_conversation_summaries_table|...
# 21|create_chat_indices|...
```

### Foreign Key Violations

```rust
// Ensure foreign key constraints are enabled
// Already set in Database::new() with .foreign_keys(true)

// Verify
sqlx::query("PRAGMA foreign_keys").fetch_one(&pool).await?;
// Should return: foreign_keys = 1
```

### Performance Issues

```bash
# Analyze query plans
EXPLAIN QUERY PLAN SELECT * FROM messages WHERE conversation_id = 'xxx';

# Should use: SEARCH messages USING INDEX idx_messages_conversation
```

## Support

For questions or issues:
1. Review `/docs/chat-schema-design.md` for detailed schema documentation
2. Review `/docs/chat-schema-erd.md` for relationship diagrams
3. Check migration code in `/src/db/migrations.rs`
4. Examine query implementations in `/src/db/chat_queries.rs`
