// commands/chat_history.rs - Chat history and memory management
//
// This module provides persistent conversation storage, message management,
// and memory persistence for the AI coach, enabling context-aware conversations
// across sessions.

use crate::{AppState, Error, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Maximum number of messages to keep in a conversation before auto-archiving
const MAX_CONVERSATION_MESSAGES: i64 = 500;

/// Number of days after which inactive conversations are auto-archived
const CONVERSATION_ARCHIVE_DAYS: i64 = 30;

/// Represents a conversation with the AI coach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user_id: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub message_count: i32,
    pub total_tokens: i32,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
}

/// Lightweight conversation summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub message_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents a single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub token_count: Option<i32>,
    pub tool_calls: Option<String>,
    pub model_used: Option<String>,
    pub created_at: String,
}

/// Message role in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    fn as_str(&self) -> &str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }
}

/// Represents stored memory for context persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub user_id: Option<String>,
    pub key: String,
    pub value: String,
    pub category: MemoryCategory,
    pub confidence: f64,
    pub source_conversation_id: Option<String>,
    pub created_at: String,
    pub last_updated: String,
    pub access_count: i32,
    pub last_accessed: Option<String>,
}

/// Categories for memory classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryCategory {
    Preference,
    Fact,
    Pattern,
    Goal,
    Context,
}

impl MemoryCategory {
    fn as_str(&self) -> &str {
        match self {
            MemoryCategory::Preference => "preference",
            MemoryCategory::Fact => "fact",
            MemoryCategory::Pattern => "pattern",
            MemoryCategory::Goal => "goal",
            MemoryCategory::Context => "context",
        }
    }
}

/// Full conversation with all messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationWithMessages {
    pub conversation: Conversation,
    pub messages: Vec<Message>,
}

/// Context built from conversation history and memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub recent_messages: Vec<Message>,
    pub relevant_memories: Vec<Memory>,
    pub conversation_summary: Option<String>,
}

/// Create a new conversation
///
/// Initializes a new conversation thread. If title is not provided,
/// it defaults to a timestamp-based title.
#[tauri::command]
pub async fn create_conversation(
    state: State<'_, AppState>,
    title: Option<String>,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let user_id = state.get_user_id().await;
    let title = title.unwrap_or_else(|| {
        format!("Conversation {}", Utc::now().format("%Y-%m-%d %H:%M"))
    });

    sqlx::query(
        r#"
        INSERT INTO conversations (id, user_id, title, message_count, total_tokens, created_at, updated_at)
        VALUES (?, ?, ?, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&title)
    .execute(state.pool())
    .await?;

    info!("Created conversation: {} ({})", id, title);
    Ok(id)
}

/// List conversations with pagination
///
/// Returns recent conversations ordered by last update time.
/// Supports pagination and excludes archived/deleted conversations by default.
#[tauri::command]
pub async fn list_conversations(
    state: State<'_, AppState>,
    limit: Option<i32>,
    offset: Option<i32>,
    include_archived: Option<bool>,
) -> Result<Vec<ConversationSummary>> {
    let limit = limit.unwrap_or(20).min(100);
    let offset = offset.unwrap_or(0);
    let include_archived = include_archived.unwrap_or(false);
    let user_id = state.get_user_id().await;

    let conversations = if include_archived {
        sqlx::query_as::<_, (String, String, Option<String>, i32, String, String)>(
            r#"
            SELECT id, title, summary, message_count, created_at, updated_at
            FROM conversations
            WHERE deleted = 0 AND (user_id = ? OR user_id IS NULL)
            ORDER BY updated_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.pool())
        .await?
    } else {
        sqlx::query_as::<_, (String, String, Option<String>, i32, String, String)>(
            r#"
            SELECT id, title, summary, message_count, created_at, updated_at
            FROM conversations
            WHERE deleted = 0 AND archived = 0 AND (user_id = ? OR user_id IS NULL)
            ORDER BY updated_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.pool())
        .await?
    };

    let summaries = conversations
        .into_iter()
        .map(|(id, title, summary, message_count, created_at, updated_at)| {
            ConversationSummary {
                id,
                title,
                summary,
                message_count,
                created_at,
                updated_at,
            }
        })
        .collect();

    Ok(summaries)
}

/// Get a conversation with all its messages
///
/// Retrieves the full conversation including metadata and all messages
/// ordered chronologically.
#[tauri::command]
pub async fn get_conversation(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<ConversationWithMessages> {
    // Fetch conversation metadata
    let conversation = sqlx::query_as::<_, (String, Option<String>, String, Option<String>, i32, i32, String, String, bool)>(
        r#"
        SELECT id, user_id, title, summary, message_count, total_tokens, created_at, updated_at, archived
        FROM conversations
        WHERE id = ? AND deleted = 0
        "#,
    )
    .bind(&conversation_id)
    .fetch_optional(state.pool())
    .await?
    .ok_or_else(|| Error::NotFound(format!("Conversation not found: {}", conversation_id)))?;

    let conversation = Conversation {
        id: conversation.0,
        user_id: conversation.1,
        title: conversation.2,
        summary: conversation.3,
        message_count: conversation.4,
        total_tokens: conversation.5,
        created_at: conversation.6,
        updated_at: conversation.7,
        archived: conversation.8,
    };

    // Fetch messages
    let messages = sqlx::query_as::<_, (String, String, String, String, Option<i32>, Option<String>, Option<String>, String)>(
        r#"
        SELECT id, conversation_id, role, content, token_count, tool_calls, model_used, created_at
        FROM messages
        WHERE conversation_id = ? AND deleted = 0
        ORDER BY created_at ASC
        "#,
    )
    .bind(&conversation_id)
    .fetch_all(state.pool())
    .await?
    .into_iter()
    .map(|(id, conversation_id, role, content, token_count, tool_calls, model_used, created_at)| {
        let role = match role.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => MessageRole::User,
        };

        Message {
            id,
            conversation_id,
            role,
            content,
            token_count,
            tool_calls,
            model_used,
            created_at,
        }
    })
    .collect();

    Ok(ConversationWithMessages {
        conversation,
        messages,
    })
}

/// Delete a conversation (soft delete)
///
/// Marks the conversation and all its messages as deleted without
/// physically removing them from the database.
#[tauri::command]
pub async fn delete_conversation(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<()> {
    let rows_affected = sqlx::query(
        r#"
        UPDATE conversations
        SET deleted = 1, last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&conversation_id)
    .execute(state.pool())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(Error::NotFound(format!(
            "Conversation not found: {}",
            conversation_id
        )));
    }

    // Soft delete all messages in the conversation
    sqlx::query(
        r#"
        UPDATE messages
        SET deleted = 1, last_modified = CURRENT_TIMESTAMP
        WHERE conversation_id = ?
        "#,
    )
    .bind(&conversation_id)
    .execute(state.pool())
    .await?;

    info!("Deleted conversation: {}", conversation_id);
    Ok(())
}

/// Archive a conversation
///
/// Archives the conversation, removing it from active lists but keeping it accessible.
#[tauri::command]
pub async fn archive_conversation(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<()> {
    let rows_affected = sqlx::query(
        r#"
        UPDATE conversations
        SET archived = 1, last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&conversation_id)
    .execute(state.pool())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(Error::NotFound(format!(
            "Conversation not found: {}",
            conversation_id
        )));
    }

    info!("Archived conversation: {}", conversation_id);
    Ok(())
}

/// Add a message to a conversation
///
/// Appends a new message to the conversation and updates conversation metadata.
#[tauri::command]
pub async fn add_message(
    state: State<'_, AppState>,
    conversation_id: String,
    role: String,
    content: String,
    token_count: Option<i32>,
    model_used: Option<String>,
) -> Result<Message> {
    let id = Uuid::new_v4().to_string();

    // Parse role
    let role_enum = match role.as_str() {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        _ => {
            return Err(Error::InvalidInput(format!("Invalid role: {}", role)));
        }
    };

    // Insert message
    sqlx::query(
        r#"
        INSERT INTO messages (id, conversation_id, role, content, token_count, model_used, created_at)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(role_enum.as_str())
    .bind(&content)
    .bind(token_count)
    .bind(&model_used)
    .execute(state.pool())
    .await?;

    // Update conversation metadata
    let token_increment = token_count.unwrap_or(0);
    sqlx::query(
        r#"
        UPDATE conversations
        SET message_count = message_count + 1,
            total_tokens = total_tokens + ?,
            updated_at = CURRENT_TIMESTAMP,
            last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(token_increment)
    .bind(&conversation_id)
    .execute(state.pool())
    .await?;

    // Check if conversation needs auto-archiving
    let message_count: (i64,) = sqlx::query_as(
        "SELECT message_count FROM conversations WHERE id = ?",
    )
    .bind(&conversation_id)
    .fetch_one(state.pool())
    .await?;

    if message_count.0 > MAX_CONVERSATION_MESSAGES {
        warn!(
            "Conversation {} has {} messages, consider archiving",
            conversation_id, message_count.0
        );
    }

    let created_at = Utc::now().to_rfc3339();

    Ok(Message {
        id,
        conversation_id,
        role: role_enum,
        content,
        token_count,
        tool_calls: None,
        model_used,
        created_at,
    })
}

/// Get recent messages from a conversation
///
/// Retrieves the most recent N messages, useful for building context windows.
#[tauri::command]
pub async fn get_recent_messages(
    state: State<'_, AppState>,
    conversation_id: String,
    limit: Option<i32>,
) -> Result<Vec<Message>> {
    let limit = limit.unwrap_or(50).min(200);

    let messages = sqlx::query_as::<_, (String, String, String, String, Option<i32>, Option<String>, Option<String>, String)>(
        r#"
        SELECT id, conversation_id, role, content, token_count, tool_calls, model_used, created_at
        FROM messages
        WHERE conversation_id = ? AND deleted = 0
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(&conversation_id)
    .bind(limit)
    .fetch_all(state.pool())
    .await?
    .into_iter()
    .rev() // Reverse to get chronological order
    .map(|(id, conversation_id, role, content, token_count, tool_calls, model_used, created_at)| {
        let role = match role.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => MessageRole::User,
        };

        Message {
            id,
            conversation_id,
            role,
            content,
            token_count,
            tool_calls,
            model_used,
            created_at,
        }
    })
    .collect();

    Ok(messages)
}

/// Save a memory entry
///
/// Stores persistent information about the user that can be recalled
/// in future conversations.
#[tauri::command]
pub async fn save_memory(
    state: State<'_, AppState>,
    key: String,
    value: String,
    category: String,
    confidence: Option<f64>,
    source_conversation_id: Option<String>,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let user_id = state.get_user_id().await;
    let confidence = confidence.unwrap_or(1.0).clamp(0.0, 1.0);

    // Parse category
    let category_enum = match category.as_str() {
        "preference" => MemoryCategory::Preference,
        "fact" => MemoryCategory::Fact,
        "pattern" => MemoryCategory::Pattern,
        "goal" => MemoryCategory::Goal,
        "context" => MemoryCategory::Context,
        _ => {
            return Err(Error::InvalidInput(format!("Invalid category: {}", category)));
        }
    };

    // Check if memory with this key already exists
    let existing: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT id FROM memory
        WHERE key = ? AND user_id = ? AND category = ? AND deleted = 0
        "#,
    )
    .bind(&key)
    .bind(&user_id)
    .bind(category_enum.as_str())
    .fetch_optional(state.pool())
    .await?;

    if let Some((existing_id,)) = existing {
        // Update existing memory
        sqlx::query(
            r#"
            UPDATE memory
            SET value = ?, confidence = ?, last_updated = CURRENT_TIMESTAMP,
                source_conversation_id = COALESCE(?, source_conversation_id),
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(&value)
        .bind(confidence)
        .bind(&source_conversation_id)
        .bind(&existing_id)
        .execute(state.pool())
        .await?;

        debug!("Updated memory: {} = {}", key, value);
        Ok(existing_id)
    } else {
        // Insert new memory
        sqlx::query(
            r#"
            INSERT INTO memory (id, user_id, key, value, category, confidence, source_conversation_id, created_at, last_updated)
            VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&key)
        .bind(&value)
        .bind(category_enum.as_str())
        .bind(confidence)
        .bind(&source_conversation_id)
        .execute(state.pool())
        .await?;

        debug!("Created memory: {} = {}", key, value);
        Ok(id)
    }
}

/// Get memories by category
///
/// Retrieves all memories in a specific category, ordered by confidence
/// and recency.
#[tauri::command]
pub async fn get_memories(
    state: State<'_, AppState>,
    category: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<Memory>> {
    let user_id = state.get_user_id().await;
    let limit = limit.unwrap_or(50).min(200);

    let memories = if let Some(cat) = category {
        sqlx::query_as::<_, (String, Option<String>, String, String, String, f64, Option<String>, String, String, i32, Option<String>)>(
            r#"
            SELECT id, user_id, key, value, category, confidence, source_conversation_id, created_at, last_updated, access_count, last_accessed
            FROM memory
            WHERE category = ? AND (user_id = ? OR user_id IS NULL) AND deleted = 0
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            ORDER BY confidence DESC, last_updated DESC
            LIMIT ?
            "#,
        )
        .bind(&cat)
        .bind(&user_id)
        .bind(limit)
        .fetch_all(state.pool())
        .await?
    } else {
        sqlx::query_as::<_, (String, Option<String>, String, String, String, f64, Option<String>, String, String, i32, Option<String>)>(
            r#"
            SELECT id, user_id, key, value, category, confidence, source_conversation_id, created_at, last_updated, access_count, last_accessed
            FROM memory
            WHERE (user_id = ? OR user_id IS NULL) AND deleted = 0
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            ORDER BY confidence DESC, last_updated DESC
            LIMIT ?
            "#,
        )
        .bind(&user_id)
        .bind(limit)
        .fetch_all(state.pool())
        .await?
    };

    let memories = memories
        .into_iter()
        .map(|(id, user_id, key, value, category, confidence, source_conversation_id, created_at, last_updated, access_count, last_accessed)| {
            let category = match category.as_str() {
                "preference" => MemoryCategory::Preference,
                "fact" => MemoryCategory::Fact,
                "pattern" => MemoryCategory::Pattern,
                "goal" => MemoryCategory::Goal,
                "context" => MemoryCategory::Context,
                _ => MemoryCategory::Context,
            };

            Memory {
                id,
                user_id,
                key,
                value,
                category,
                confidence,
                source_conversation_id,
                created_at,
                last_updated,
                access_count,
                last_accessed,
            }
        })
        .collect();

    Ok(memories)
}

/// Get relevant memories based on context
///
/// Simple keyword-based relevance matching. In production, this could use
/// semantic search with embeddings.
#[tauri::command]
pub async fn get_relevant_memories(
    state: State<'_, AppState>,
    context: String,
    limit: Option<i32>,
) -> Result<Vec<Memory>> {
    let user_id = state.get_user_id().await;
    let limit = limit.unwrap_or(10).min(50);

    // Simple keyword matching - could be enhanced with semantic search
    let search_pattern = format!("%{}%", context.to_lowercase());

    let memories = sqlx::query_as::<_, (String, Option<String>, String, String, String, f64, Option<String>, String, String, i32, Option<String>)>(
        r#"
        SELECT id, user_id, key, value, category, confidence, source_conversation_id, created_at, last_updated, access_count, last_accessed
        FROM memory
        WHERE (user_id = ? OR user_id IS NULL) AND deleted = 0
          AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
          AND (LOWER(key) LIKE ? OR LOWER(value) LIKE ?)
        ORDER BY confidence DESC, access_count DESC, last_updated DESC
        LIMIT ?
        "#,
    )
    .bind(&user_id)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(limit)
    .fetch_all(state.pool())
    .await?;

    // Update access count and last accessed time for returned memories
    for (id, _, _, _, _, _, _, _, _, _, _) in &memories {
        let _ = sqlx::query(
            r#"
            UPDATE memory
            SET access_count = access_count + 1,
                last_accessed = CURRENT_TIMESTAMP,
                last_modified = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(state.pool())
        .await;
    }

    let memories = memories
        .into_iter()
        .map(|(id, user_id, key, value, category, confidence, source_conversation_id, created_at, last_updated, access_count, last_accessed)| {
            let category = match category.as_str() {
                "preference" => MemoryCategory::Preference,
                "fact" => MemoryCategory::Fact,
                "pattern" => MemoryCategory::Pattern,
                "goal" => MemoryCategory::Goal,
                "context" => MemoryCategory::Context,
                _ => MemoryCategory::Context,
            };

            Memory {
                id,
                user_id,
                key,
                value,
                category,
                confidence,
                source_conversation_id,
                created_at,
                last_updated,
                access_count,
                last_accessed,
            }
        })
        .collect();

    Ok(memories)
}

/// Auto-archive old conversations
///
/// Background task to archive conversations that haven't been updated
/// in CONVERSATION_ARCHIVE_DAYS days.
#[tauri::command]
pub async fn auto_archive_old_conversations(state: State<'_, AppState>) -> Result<i32> {
    let result = sqlx::query(
        r#"
        UPDATE conversations
        SET archived = 1, last_modified = CURRENT_TIMESTAMP
        WHERE deleted = 0 AND archived = 0
          AND datetime(updated_at) < datetime('now', ? || ' days')
        "#,
    )
    .bind(format!("-{}", CONVERSATION_ARCHIVE_DAYS))
    .execute(state.pool())
    .await?;

    let archived_count = result.rows_affected() as i32;
    if archived_count > 0 {
        info!("Auto-archived {} old conversations", archived_count);
    }

    Ok(archived_count)
}

/// Cleanup expired memories
///
/// Removes memories that have passed their expiration time.
#[tauri::command]
pub async fn cleanup_expired_memories(state: State<'_, AppState>) -> Result<i32> {
    let result = sqlx::query(
        r#"
        UPDATE memory
        SET deleted = 1, last_modified = CURRENT_TIMESTAMP
        WHERE deleted = 0 AND expires_at IS NOT NULL
          AND datetime(expires_at) < datetime('now')
        "#,
    )
    .execute(state.pool())
    .await?;

    let cleaned_count = result.rows_affected() as i32;
    if cleaned_count > 0 {
        info!("Cleaned up {} expired memories", cleaned_count);
    }

    Ok(cleaned_count)
}

/// Build conversation context for AI prompts
///
/// Constructs a context object containing recent messages and relevant
/// memories for use in AI prompt generation.
#[tauri::command]
pub async fn build_conversation_context(
    state: State<'_, AppState>,
    conversation_id: String,
    user_message: Option<String>,
    max_messages: Option<i32>,
) -> Result<ConversationContext> {
    let max_messages = max_messages.unwrap_or(10).min(50);

    // Get recent messages
    let recent_messages = get_recent_messages(state.clone(), conversation_id.clone(), Some(max_messages))
        .await?;

    // Get relevant memories based on user message
    let relevant_memories = if let Some(msg) = user_message {
        get_relevant_memories(state.clone(), msg, Some(5)).await?
    } else {
        Vec::new()
    };

    // Get conversation summary if available
    let summary: Option<String> = sqlx::query_scalar(
        "SELECT summary FROM conversations WHERE id = ? AND deleted = 0",
    )
    .bind(&conversation_id)
    .fetch_optional(state.pool())
    .await?
    .flatten();

    Ok(ConversationContext {
        recent_messages,
        relevant_memories,
        conversation_summary: summary,
    })
}

/// Update conversation title based on content
///
/// Allows updating the conversation title, useful for auto-generating
/// titles from the first message.
#[tauri::command]
pub async fn update_conversation_title(
    state: State<'_, AppState>,
    conversation_id: String,
    title: String,
) -> Result<()> {
    let rows_affected = sqlx::query(
        r#"
        UPDATE conversations
        SET title = ?, last_modified = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&title)
    .bind(&conversation_id)
    .execute(state.pool())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(Error::NotFound(format!(
            "Conversation not found: {}",
            conversation_id
        )));
    }

    Ok(())
}
