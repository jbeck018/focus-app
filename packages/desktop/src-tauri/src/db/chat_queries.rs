// db/chat_queries.rs - Chat history and LLM memory queries
//
// Lower-level database query utilities. These functions are available for
// internal use but currently wrapped by the chat_history command module.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::Result;

// ============================================================================
// Chat & Conversation Models
// ============================================================================

/// Conversation database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: String,
    pub user_id: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub message_count: i64,
    pub total_tokens: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
}

/// Message database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub token_count: Option<i64>,
    pub tool_calls: Option<String>,
    pub model_used: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Memory database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Memory {
    pub id: String,
    pub user_id: Option<String>,
    pub key: String,
    pub value: String,
    pub category: String,
    pub confidence: f64,
    pub source_conversation_id: Option<String>,
    pub source_message_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub access_count: i64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Conversation summary database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationSummary {
    pub id: String,
    pub conversation_id: String,
    pub summary_text: String,
    pub key_topics: Option<String>,
    pub summary_tokens: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub summarized_messages_count: i64,
}

// ============================================================================
// Conversation Queries
// ============================================================================

/// Create a new conversation
pub async fn create_conversation(
    pool: &SqlitePool,
    id: &str,
    user_id: Option<&str>,
    title: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO conversations (id, user_id, title)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get conversation by ID
pub async fn get_conversation(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Conversation>> {
    let conversation = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT id, user_id, title, summary, message_count, total_tokens,
               created_at, updated_at, archived
        FROM conversations
        WHERE id = ? AND deleted = 0
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(conversation)
}

/// Get recent conversations (last 30 days, non-archived)
pub async fn get_recent_conversations(
    pool: &SqlitePool,
    user_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Conversation>> {
    let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);

    let conversations = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT id, user_id, title, summary, message_count, total_tokens,
               created_at, updated_at, archived
        FROM conversations
        WHERE deleted = 0
            AND archived = 0
            AND created_at >= ?
            AND (user_id IS NULL OR user_id = ?)
        ORDER BY updated_at DESC
        LIMIT ?
        "#,
    )
    .bind(thirty_days_ago)
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(conversations)
}

/// Get all conversations (including archived, for history view)
pub async fn get_all_conversations(
    pool: &SqlitePool,
    user_id: Option<&str>,
    include_archived: bool,
    limit: i64,
) -> Result<Vec<Conversation>> {
    let conversations = if include_archived {
        sqlx::query_as::<_, Conversation>(
            r#"
            SELECT id, user_id, title, summary, message_count, total_tokens,
                   created_at, updated_at, archived
            FROM conversations
            WHERE deleted = 0
                AND (user_id IS NULL OR user_id = ?)
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Conversation>(
            r#"
            SELECT id, user_id, title, summary, message_count, total_tokens,
                   created_at, updated_at, archived
            FROM conversations
            WHERE deleted = 0
                AND archived = 0
                AND (user_id IS NULL OR user_id = ?)
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(conversations)
}

/// Update conversation metadata (title, summary, counts)
pub async fn update_conversation(
    pool: &SqlitePool,
    id: &str,
    title: Option<&str>,
    summary: Option<&str>,
    message_count_delta: i64,
    token_count_delta: i64,
) -> Result<()> {
    if let Some(title) = title {
        sqlx::query(
            r#"
            UPDATE conversations
            SET title = ?,
                updated_at = CURRENT_TIMESTAMP,
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(title)
        .bind(id)
        .execute(pool)
        .await?;
    }

    if let Some(summary) = summary {
        sqlx::query(
            r#"
            UPDATE conversations
            SET summary = ?,
                updated_at = CURRENT_TIMESTAMP,
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(summary)
        .bind(id)
        .execute(pool)
        .await?;
    }

    if message_count_delta != 0 || token_count_delta != 0 {
        sqlx::query(
            r#"
            UPDATE conversations
            SET message_count = message_count + ?,
                total_tokens = total_tokens + ?,
                updated_at = CURRENT_TIMESTAMP,
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(message_count_delta)
        .bind(token_count_delta)
        .bind(id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Archive a conversation
pub async fn archive_conversation(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE conversations
        SET archived = 1,
            updated_at = CURRENT_TIMESTAMP,
            last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a conversation (soft delete)
pub async fn delete_conversation(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE conversations
        SET deleted = 1,
            last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================================
// Message Queries
// ============================================================================

/// Insert a new message
pub async fn insert_message(
    pool: &SqlitePool,
    id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
    token_count: Option<i64>,
    tool_calls: Option<&str>,
    model_used: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO messages (id, conversation_id, role, content, token_count, tool_calls, model_used)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(token_count)
    .bind(tool_calls)
    .bind(model_used)
    .execute(pool)
    .await?;

    // Update conversation metadata
    update_conversation(
        pool,
        conversation_id,
        None,
        None,
        1,
        token_count.unwrap_or(0),
    )
    .await?;

    Ok(())
}

/// Get messages for a conversation
pub async fn get_messages_for_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    limit: Option<i64>,
) -> Result<Vec<Message>> {
    let messages = if let Some(limit) = limit {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT id, conversation_id, role, content, token_count, tool_calls, model_used, created_at
            FROM messages
            WHERE conversation_id = ? AND deleted = 0
            ORDER BY created_at ASC
            LIMIT ?
            "#,
        )
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT id, conversation_id, role, content, token_count, tool_calls, model_used, created_at
            FROM messages
            WHERE conversation_id = ? AND deleted = 0
            ORDER BY created_at ASC
            "#,
        )
        .bind(conversation_id)
        .fetch_all(pool)
        .await?
    };

    Ok(messages)
}

/// Get recent messages across all conversations (for context building)
pub async fn get_recent_messages(
    pool: &SqlitePool,
    user_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT m.id, m.conversation_id, m.role, m.content, m.token_count,
               m.tool_calls, m.model_used, m.created_at
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        WHERE m.deleted = 0
            AND c.deleted = 0
            AND (c.user_id IS NULL OR c.user_id = ?)
        ORDER BY m.created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

/// Get message by ID
pub async fn get_message(pool: &SqlitePool, id: &str) -> Result<Option<Message>> {
    let message = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, conversation_id, role, content, token_count, tool_calls, model_used, created_at
        FROM messages
        WHERE id = ? AND deleted = 0
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(message)
}

/// Delete a message (soft delete)
pub async fn delete_message(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE messages
        SET deleted = 1,
            last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get token count for conversation (for context window management)
pub async fn get_conversation_token_count(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<i64> {
    let count: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT SUM(token_count)
        FROM messages
        WHERE conversation_id = ? AND deleted = 0
        "#,
    )
    .bind(conversation_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0.unwrap_or(0))
}

// ============================================================================
// Memory Queries
// ============================================================================

/// Create or update memory
pub async fn upsert_memory(
    pool: &SqlitePool,
    id: &str,
    user_id: Option<&str>,
    key: &str,
    value: &str,
    category: &str,
    confidence: f64,
    source_conversation_id: Option<&str>,
    source_message_id: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO memory (
            id, user_id, key, value, category, confidence,
            source_conversation_id, source_message_id, expires_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            value = excluded.value,
            confidence = excluded.confidence,
            last_updated = CURRENT_TIMESTAMP,
            last_modified = CURRENT_TIMESTAMP,
            expires_at = excluded.expires_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(key)
    .bind(value)
    .bind(category)
    .bind(confidence)
    .bind(source_conversation_id)
    .bind(source_message_id)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get memory by key
pub async fn get_memory_by_key(
    pool: &SqlitePool,
    key: &str,
    user_id: Option<&str>,
) -> Result<Option<Memory>> {
    let memory = sqlx::query_as::<_, Memory>(
        r#"
        SELECT id, user_id, key, value, category, confidence,
               source_conversation_id, source_message_id,
               created_at, last_updated, access_count, last_accessed, expires_at
        FROM memory
        WHERE key = ?
            AND (user_id IS NULL OR user_id = ?)
            AND deleted = 0
            AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
        ORDER BY confidence DESC, last_updated DESC
        LIMIT 1
        "#,
    )
    .bind(key)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    // Update access count if found
    if let Some(ref mem) = memory {
        sqlx::query(
            r#"
            UPDATE memory
            SET access_count = access_count + 1,
                last_accessed = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(&mem.id)
        .execute(pool)
        .await?;
    }

    Ok(memory)
}

/// Get all memories by category
pub async fn get_memories_by_category(
    pool: &SqlitePool,
    user_id: Option<&str>,
    category: &str,
    limit: i64,
) -> Result<Vec<Memory>> {
    let memories = sqlx::query_as::<_, Memory>(
        r#"
        SELECT id, user_id, key, value, category, confidence,
               source_conversation_id, source_message_id,
               created_at, last_updated, access_count, last_accessed, expires_at
        FROM memory
        WHERE category = ?
            AND (user_id IS NULL OR user_id = ?)
            AND deleted = 0
            AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
        ORDER BY confidence DESC, last_updated DESC
        LIMIT ?
        "#,
    )
    .bind(category)
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(memories)
}

/// Get all active memories (for LLM context building)
pub async fn get_all_active_memories(
    pool: &SqlitePool,
    user_id: Option<&str>,
    min_confidence: f64,
) -> Result<Vec<Memory>> {
    let memories = sqlx::query_as::<_, Memory>(
        r#"
        SELECT id, user_id, key, value, category, confidence,
               source_conversation_id, source_message_id,
               created_at, last_updated, access_count, last_accessed, expires_at
        FROM memory
        WHERE (user_id IS NULL OR user_id = ?)
            AND deleted = 0
            AND confidence >= ?
            AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
        ORDER BY confidence DESC, access_count DESC, last_updated DESC
        "#,
    )
    .bind(user_id)
    .bind(min_confidence)
    .fetch_all(pool)
    .await?;

    Ok(memories)
}

/// Update memory confidence
pub async fn update_memory_confidence(
    pool: &SqlitePool,
    id: &str,
    confidence: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE memory
        SET confidence = ?,
            last_updated = CURRENT_TIMESTAMP,
            last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(confidence)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete memory (soft delete)
pub async fn delete_memory(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE memory
        SET deleted = 1,
            last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Clean up expired memories
pub async fn cleanup_expired_memories(pool: &SqlitePool) -> Result<i64> {
    let result = sqlx::query(
        r#"
        UPDATE memory
        SET deleted = 1,
            last_modified = CURRENT_TIMESTAMP
        WHERE expires_at IS NOT NULL
            AND expires_at <= CURRENT_TIMESTAMP
            AND deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as i64)
}

// ============================================================================
// Conversation Summary Queries
// ============================================================================

/// Create conversation summary
pub async fn create_conversation_summary(
    pool: &SqlitePool,
    id: &str,
    conversation_id: &str,
    summary_text: &str,
    key_topics: Option<&str>,
    summary_tokens: Option<i64>,
    summarized_messages_count: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO conversation_summaries (
            id, conversation_id, summary_text, key_topics,
            summary_tokens, summarized_messages_count
        )
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(conversation_id)
    .bind(summary_text)
    .bind(key_topics)
    .bind(summary_tokens)
    .bind(summarized_messages_count)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get summaries for conversation
pub async fn get_conversation_summaries(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ConversationSummary>> {
    let summaries = sqlx::query_as::<_, ConversationSummary>(
        r#"
        SELECT id, conversation_id, summary_text, key_topics,
               summary_tokens, created_at, summarized_messages_count
        FROM conversation_summaries
        WHERE conversation_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;

    Ok(summaries)
}

/// Get latest summary for conversation
pub async fn get_latest_conversation_summary(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<ConversationSummary>> {
    let summary = sqlx::query_as::<_, ConversationSummary>(
        r#"
        SELECT id, conversation_id, summary_text, key_topics,
               summary_tokens, created_at, summarized_messages_count
        FROM conversation_summaries
        WHERE conversation_id = ?
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    Ok(summary)
}
