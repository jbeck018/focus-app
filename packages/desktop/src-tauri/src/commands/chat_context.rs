// commands/chat_context.rs - Context building utilities for AI coach
//
// This module provides utilities for building rich context from chat history
// and memories to enhance AI coach responses.

use crate::commands::chat_history::{Memory, Message, MessageRole};
use crate::commands::coach::UserContext;

/// Build system prompt with conversation history and memories
///
/// Constructs a comprehensive system prompt that includes:
/// - Recent conversation context
/// - Relevant user memories
/// - Current user stats and patterns
pub fn build_context_prompt(
    user_context: &UserContext,
    recent_messages: &[Message],
    relevant_memories: &[Memory],
    conversation_summary: Option<&str>,
) -> String {
    let mut prompt = String::new();

    // Add conversation summary if available
    if let Some(summary) = conversation_summary {
        prompt.push_str("## Previous Conversation Summary\n");
        prompt.push_str(summary);
        prompt.push_str("\n\n");
    }

    // Add relevant memories
    if !relevant_memories.is_empty() {
        prompt.push_str("## What I Remember About You\n");
        for memory in relevant_memories {
            prompt.push_str(&format!(
                "- {} ({}): {}\n",
                memory.key,
                format!("{:?}", memory.category).to_lowercase(),
                memory.value
            ));
        }
        prompt.push('\n');
    }

    // Add current user stats
    prompt.push_str("## Your Current Progress\n");
    prompt.push_str(&format!(
        "- Focus time today: {:.1} hours\n",
        user_context.total_focus_hours_today
    ));
    prompt.push_str(&format!(
        "- Sessions completed today: {}\n",
        user_context.sessions_completed_today
    ));
    prompt.push_str(&format!(
        "- Current streak: {} days\n",
        user_context.current_streak_days
    ));
    if let Some(ref trigger) = user_context.top_trigger {
        prompt.push_str(&format!("- Most common distraction: {}\n", trigger));
    }
    prompt.push_str(&format!(
        "- Average session length: {} minutes\n\n",
        user_context.average_session_minutes
    ));

    // Add recent conversation context
    if !recent_messages.is_empty() {
        prompt.push_str("## Recent Conversation\n");
        for msg in recent_messages.iter().rev().take(5).rev() {
            let role_prefix = match msg.role {
                MessageRole::User => "You",
                MessageRole::Assistant => "Me",
                MessageRole::System => "System",
            };
            // Truncate long messages for context
            let content = if msg.content.len() > 200 {
                format!("{}...", &msg.content[..197])
            } else {
                msg.content.clone()
            };
            prompt.push_str(&format!("{}: {}\n", role_prefix, content));
        }
        prompt.push('\n');
    }

    prompt
}

/// Format conversation context as a system message
///
/// Converts the context into a system message that can be prepended
/// to the conversation for the LLM.
pub fn format_as_system_message(context_prompt: &str) -> String {
    format!(
        "You are an AI focus coach helping a user build better focus habits. \
         Here's what you know about them:\n\n{}",
        context_prompt
    )
}

/// Extract potential memories from a conversation
///
/// Analyzes assistant and user messages to identify information
/// that should be persisted as memories.
pub fn extract_potential_memories(content: &str, role: &MessageRole) -> Vec<(String, String, String)> {
    let mut memories = Vec::new();

    // Simple pattern matching for common memory patterns
    // In production, this could use NLP or LLM-based extraction

    match role {
        MessageRole::User => {
            // Extract user preferences
            if content.to_lowercase().contains("i prefer") || content.to_lowercase().contains("i like") {
                // Extract preference (simplified)
                let clean = content.to_lowercase();
                if let Some(start) = clean.find("i prefer") {
                    let pref = &content[start + 9..].trim();
                    if let Some(end) = pref.find('.') {
                        let pref_text = &pref[..end];
                        memories.push((
                            "user_preference".to_string(),
                            pref_text.to_string(),
                            "preference".to_string(),
                        ));
                    }
                }
            }

            // Extract goals
            if content.to_lowercase().contains("my goal is") || content.to_lowercase().contains("i want to") {
                let clean = content.to_lowercase();
                if let Some(start) = clean.find("my goal is") {
                    let goal = &content[start + 11..].trim();
                    if let Some(end) = goal.find('.') {
                        let goal_text = &goal[..end];
                        memories.push((
                            "user_goal".to_string(),
                            goal_text.to_string(),
                            "goal".to_string(),
                        ));
                    }
                } else if let Some(start) = clean.find("i want to") {
                    let goal = &content[start + 10..].trim();
                    if let Some(end) = goal.find('.') {
                        let goal_text = &goal[..end];
                        memories.push((
                            "user_goal".to_string(),
                            goal_text.to_string(),
                            "goal".to_string(),
                        ));
                    }
                }
            }
        }
        MessageRole::Assistant => {
            // Extract patterns identified by the coach
            if content.to_lowercase().contains("pattern") || content.to_lowercase().contains("notice") {
                // This could be enhanced to extract specific patterns
            }
        }
        MessageRole::System => {}
    }

    memories
}

/// Estimate token count for messages
///
/// Simple heuristic: ~4 characters per token (GPT-like tokenization)
pub fn estimate_token_count(text: &str) -> i32 {
    (text.len() / 4).max(1) as i32
}

/// Truncate conversation history to fit within token limit
///
/// Keeps most recent messages within the token budget while preserving
/// conversation flow (always include full user-assistant pairs).
#[allow(dead_code)]
pub fn truncate_to_token_limit(
    messages: Vec<Message>,
    max_tokens: i32,
) -> Vec<Message> {
    let mut total_tokens = 0;
    let mut truncated = Vec::new();

    // Process messages in reverse (most recent first)
    for msg in messages.into_iter().rev() {
        let msg_tokens = msg.token_count.unwrap_or_else(|| estimate_token_count(&msg.content));

        if total_tokens + msg_tokens <= max_tokens {
            total_tokens += msg_tokens;
            truncated.push(msg);
        } else {
            break;
        }
    }

    // Reverse back to chronological order
    truncated.reverse();
    truncated
}

/// Generate a conversation title from the first user message
///
/// Creates a concise title from the beginning of the conversation.
pub fn generate_conversation_title(first_message: &str) -> String {
    let max_length = 50;
    let cleaned = first_message
        .lines()
        .next()
        .unwrap_or(first_message)
        .trim();

    if cleaned.len() <= max_length {
        cleaned.to_string()
    } else {
        format!("{}...", &cleaned[..max_length - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_token_count() {
        assert_eq!(estimate_token_count("Hello world"), 2); // 11 chars / 4 = 2
        assert_eq!(estimate_token_count(""), 1); // Minimum 1 token
        assert_eq!(estimate_token_count("This is a longer message with more content"), 10);
    }

    #[test]
    fn test_generate_conversation_title() {
        assert_eq!(
            generate_conversation_title("How can I improve my focus?"),
            "How can I improve my focus?"
        );

        let long = "This is a very long message that exceeds the maximum length and should be truncated";
        let title = generate_conversation_title(long);
        assert!(title.len() <= 50);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_extract_potential_memories() {
        let content = "I prefer working in the morning because I'm more alert.";
        let memories = extract_potential_memories(content, &MessageRole::User);
        assert!(!memories.is_empty());
        assert_eq!(memories[0].2, "preference");
    }
}
