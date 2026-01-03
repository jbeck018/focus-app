// ai/prompt_templates.rs - Prompt building utilities for AI coach
//
// This module provides the interface for building prompts using the modular
// system_prompts architecture. It handles template selection and context
// conversion for the coaching scenarios.

use crate::commands::coach::UserContext;
use crate::ai::system_prompts::{
    build_minimal_prompt, PromptContext, ScenarioPromptBuilder,
};
use crate::ai::orchestrator::GuidelineOrchestrator;
use crate::ai::tools::ToolRegistry;
pub use crate::ai::system_prompts::UserIntent;
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

/// Prompt template type (maintained for API compatibility)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)] // All variants used by template system
pub enum PromptTemplate {
    GeneralCoaching,
    DailyTip,
    SessionAdvice,
    ReflectionPrompt,
    PatternAnalysis,
}

impl PromptTemplate {
    /// Convert template type to user intent for guideline selection
    #[allow(dead_code)] // Used by deprecated prompt building functions
    fn to_intent(&self) -> UserIntent {
        match self {
            PromptTemplate::GeneralCoaching => UserIntent::GeneralChat,
            PromptTemplate::DailyTip => UserIntent::GeneralChat, // Uses custom instructions
            PromptTemplate::SessionAdvice => UserIntent::SessionPlanning,
            PromptTemplate::ReflectionPrompt => UserIntent::SessionReflection,
            PromptTemplate::PatternAnalysis => UserIntent::PatternAnalysis,
        }
    }
}

/// Convert UserContext to PromptContext
#[allow(dead_code)] // Used by deprecated prompt building functions
fn to_prompt_context(context: &UserContext) -> PromptContext {
    let now = chrono::Utc::now();
    let hour = now.hour();
    let day_name = now.weekday().to_string();

    let time_of_day = match hour {
        0..=5 => "very early morning".to_string(),
        6..=11 => "morning".to_string(),
        12..=17 => "afternoon".to_string(),
        18..=21 => "evening".to_string(),
        _ => "late night".to_string(),
    };

    PromptContext {
        time_of_day,
        day_of_week: day_name,
        sessions_completed: context.sessions_completed_today,
        focus_hours: context.total_focus_hours_today,
        streak_days: context.current_streak_days,
        avg_session_minutes: context.average_session_minutes,
        top_trigger: context.top_trigger.clone(),
        additional_context: None,
    }
}

/// Build a complete prompt for the AI coach
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_coach_prompt(
    template: PromptTemplate,
    user_message: &str,
    context: &UserContext,
) -> String {
    let orchestrator = GuidelineOrchestrator::new();
    let user_prompt = build_user_prompt(template, user_message, context);

    // Use orchestrate to build the dynamic prompt with matched guidelines
    let result = orchestrator.orchestrate(user_message, context);

    format!("{}\n\n{}", result.dynamic_prompt, user_prompt)
}

/// Build a complete prompt with tool documentation for agentic interactions
///
/// This prompt enables the LLM to call tools to take actions in the app.
/// Use this for interactions where the AI should be able to start sessions,
/// log triggers, check stats, etc.
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_agentic_prompt(
    user_message: &str,
    context: &UserContext,
    include_all_tools: bool,
) -> String {
    let orchestrator = GuidelineOrchestrator::new();
    let result = orchestrator.orchestrate(user_message, context);
    let tool_docs = build_tool_documentation(include_all_tools);

    format!(
        "{}\n\n{}\n\n\
         TOOL USAGE INSTRUCTIONS:\n\
         - When the user asks you to take an action (start session, log trigger, etc.), \
           use the appropriate tool to perform it\n\
         - You can use multiple tools in a single response if needed\n\
         - Always explain what you're doing before or after using a tool\n\
         - If a tool fails, explain what went wrong and suggest alternatives\n\n\
         USER MESSAGE: {}\n\n\
         Respond helpfully, using tools when appropriate to take actions.",
        result.dynamic_prompt,
        tool_docs,
        user_message
    )
}

/// Build a chat prompt with tool support for multi-turn conversations
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_agentic_chat_prompt(
    messages: &[(String, String)],
    context: &UserContext,
    tool_results: Option<&str>,
) -> String {
    let orchestrator = GuidelineOrchestrator::new();

    // Detect intent from the most recent user message
    let last_user_message = messages
        .iter()
        .rev()
        .find(|(role, _)| role == "user")
        .map(|(_, content)| content.as_str())
        .unwrap_or("");

    let result = orchestrator.orchestrate(last_user_message, context);
    let tool_docs = build_tool_documentation(true);

    let mut prompt = format!(
        "{}\n\n{}\n\n\
         TOOL USAGE INSTRUCTIONS:\n\
         - Use tools to take actions when the user requests them\n\
         - Explain your actions clearly\n\
         - After tool execution, acknowledge the result and continue the conversation\n\n\
         CONVERSATION HISTORY:",
        result.dynamic_prompt,
        tool_docs
    );

    for (role, content) in messages {
        match role.as_str() {
            "user" => prompt.push_str(&format!("\n\nUSER: {}", content)),
            "assistant" => prompt.push_str(&format!("\n\nASSISTANT: {}", content)),
            "tool" => prompt.push_str(&format!("\n\n[TOOL RESULT]\n{}", content)),
            _ => {}
        }
    }

    // Add any pending tool results
    if let Some(results) = tool_results {
        prompt.push_str(&format!("\n\n[TOOL RESULT]\n{}", results));
    }

    prompt.push_str("\n\nASSISTANT:");
    prompt
}

/// Build tool documentation for the system prompt
#[allow(dead_code)] // Used by agentic prompt builders
fn build_tool_documentation(include_all: bool) -> String {
    let registry = ToolRegistry::with_default_tools();

    if include_all {
        registry.generate_documentation()
    } else {
        // Minimal tool documentation for lighter prompts
        let mut doc = String::from("# Available Actions\n\n");
        doc.push_str("You can take these actions using XML-style tool calls:\n\n");

        let essential_tools = [
            ("start_focus_session", "Start a focus session", "duration (optional, default 25)"),
            ("end_focus_session", "End current session", "completed (optional, default true)"),
            ("log_trigger", "Log a distraction trigger", "trigger_type (required), notes (optional)"),
            ("get_session_stats", "Get focus statistics", "period (optional: today/week)"),
            ("get_streak_info", "Check streak status", "none"),
        ];

        for (name, desc, params) in essential_tools {
            doc.push_str(&format!("- `{}`: {} (params: {})\n", name, desc, params));
        }

        doc.push_str("\nExample: `<tool name=\"start_focus_session\" duration=\"25\"/>`\n");
        doc
    }
}

/// Build a prompt specifically for action-oriented requests
///
/// Use this when the user clearly wants to take an action (start session,
/// log something, check stats, etc.)
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_action_prompt(
    action_request: &str,
    context: &UserContext,
) -> String {
    let prompt_context = to_prompt_context(context);
    let tool_docs = build_tool_documentation(false);

    format!(
        "You are a helpful focus coach assistant. You help users take actions in the FocusFlow app.\n\n\
         CURRENT CONTEXT:\n\
         - Sessions today: {} ({:.1} hours of focus)\n\
         - Current streak: {} days\n\
         - Top trigger: {}\n\n\
         {}\n\n\
         USER REQUEST: {}\n\n\
         Respond by:\n\
         1. Acknowledging what the user wants to do\n\
         2. Using the appropriate tool to take the action\n\
         3. Adding a brief, encouraging note\n\n\
         Keep your response concise (2-3 sentences plus the tool call).",
        prompt_context.sessions_completed,
        prompt_context.focus_hours,
        prompt_context.streak_days,
        prompt_context.top_trigger.as_deref().unwrap_or("not identified"),
        tool_docs,
        action_request
    )
}

/// Detect intent from both the template type and user message
#[allow(dead_code)] // Used by deprecated prompt building functions
fn detect_intent_from_message_and_template(template: PromptTemplate, message: &str) -> UserIntent {
    // Template takes precedence for specific scenarios
    match template {
        PromptTemplate::SessionAdvice => UserIntent::SessionPlanning,
        PromptTemplate::ReflectionPrompt => UserIntent::SessionReflection,
        PromptTemplate::PatternAnalysis => UserIntent::PatternAnalysis,
        // For general coaching, detect from message
        PromptTemplate::GeneralCoaching | PromptTemplate::DailyTip => {
            if message.is_empty() {
                template.to_intent()
            } else {
                UserIntent::detect(message)
            }
        }
    }
}

/// Build user-facing prompt based on template and context
#[allow(dead_code)] // Used by deprecated prompt building functions
fn build_user_prompt(
    template: PromptTemplate,
    user_message: &str,
    context: &UserContext,
) -> String {
    match template {
        PromptTemplate::GeneralCoaching => {
            format!(
                "USER MESSAGE: {}\n\n\
                 Respond according to the guidelines above. Keep your response concise and actionable.",
                user_message
            )
        }

        PromptTemplate::DailyTip => {
            let focus_area = determine_daily_tip_focus(context);

            format!(
                "TASK: Generate a daily productivity tip.\n\n\
                 FOCUS AREA: {}\n\n\
                 REQUIREMENTS:\n\
                 - 2-3 sentences maximum\n\
                 - Reference one Indistractable principle naturally (don't name it explicitly)\n\
                 - Make it actionable for today specifically\n\
                 - Do NOT start with greetings like 'Good morning'\n\
                 - Do NOT use exclamation marks excessively\n\n\
                 Generate the tip directly.",
                focus_area
            )
        }

        PromptTemplate::SessionAdvice => {
            format!(
                "TASK: Provide pre-session advice.\n\n\
                 USER INPUT: {}\n\n\
                 REQUIREMENTS:\n\
                 - 2-3 specific preparation tips\n\
                 - Reference their top trigger if available\n\
                 - Consider their average session length\n\
                 - Include environmental setup suggestions\n\
                 - End with readiness to start (use [[start_focus_session]] tool)\n\n\
                 Provide the advice directly.",
                user_message
            )
        }

        PromptTemplate::ReflectionPrompt => {
            let completed = !user_message.to_lowercase().contains("early");
            let reflection_context = if completed {
                "The user completed their session successfully."
            } else {
                "The user ended their session early. Be supportive, not judgmental."
            };

            format!(
                "TASK: Generate a post-session reflection prompt.\n\n\
                 SESSION DETAILS: {}\n\
                 CONTEXT: {}\n\n\
                 REQUIREMENTS:\n\
                 - 2-3 brief reflection questions\n\
                 - Focus on learning and pattern identification\n\
                 - If ended early, explore what happened without judgment\n\
                 - Suggest logging triggers if relevant (use [[log_trigger]] tool)\n\n\
                 Provide the reflection prompt directly.",
                user_message,
                reflection_context
            )
        }

        PromptTemplate::PatternAnalysis => {
            let streak_context = describe_streak(context.current_streak_days);
            let trigger_context = describe_trigger(&context.top_trigger);

            format!(
                "TASK: Analyze the user's productivity patterns.\n\n\
                 PATTERN DATA:\n\
                 - Streak status: {}\n\
                 - Sessions today: {} ({:.1} hours)\n\
                 - Average session: {} minutes\n\
                 - Trigger pattern: {}\n\n\
                 REQUIREMENTS:\n\
                 - 2-3 specific insights based on their data\n\
                 - Identify one strength (what's working)\n\
                 - Identify one growth opportunity (with concrete suggestion)\n\
                 - Reference actual numbers from their data\n\
                 - End with a forward-looking suggestion or question\n\n\
                 Provide the analysis directly.",
                streak_context,
                context.sessions_completed_today,
                context.total_focus_hours_today,
                context.average_session_minutes,
                trigger_context
            )
        }
    }
}

/// Determine the focus area for daily tips based on context
#[allow(dead_code)] // Used by build_user_prompt
fn determine_daily_tip_focus(context: &UserContext) -> String {
    if context.sessions_completed_today == 0 {
        "Getting started with focused work today".to_string()
    } else if context.total_focus_hours_today > 3.0 {
        "Maintaining energy and avoiding burnout after significant deep work".to_string()
    } else if context.current_streak_days > 7 {
        "Building on strong consistency and deepening focus habits".to_string()
    } else if context.current_streak_days == 0 {
        "Rebuilding momentum and starting fresh".to_string()
    } else {
        "Building momentum through the day".to_string()
    }
}

/// Describe streak status for pattern analysis
#[allow(dead_code)] // Used by build_user_prompt
fn describe_streak(streak_days: i32) -> String {
    match streak_days {
        0 => "No active streak - looking to start fresh".to_string(),
        1..=3 => format!("{}-day streak - building initial momentum", streak_days),
        4..=7 => format!("{}-day streak - developing consistency", streak_days),
        8..=14 => format!("{}-day streak - habit is forming", streak_days),
        15..=30 => format!("{}-day streak - strong consistency", streak_days),
        _ => format!("{}-day streak - excellent long-term habit", streak_days),
    }
}

/// Describe trigger pattern for analysis
#[allow(dead_code)] // Used by build_user_prompt
fn describe_trigger(trigger: &Option<String>) -> String {
    match trigger {
        Some(t) => format!("'{}' is the most common trigger", t),
        None => "No triggers logged yet - encourage tracking".to_string(),
    }
}

/// Build a conversational prompt for chat-style interactions
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_chat_prompt(
    messages: &[(String, String)], // (role, content) pairs
    context: &UserContext,
) -> String {
    let orchestrator = GuidelineOrchestrator::new();

    // Detect intent from the most recent user message
    let last_user_message = messages
        .iter()
        .rev()
        .find(|(role, _)| role == "user")
        .map(|(_, content)| content.as_str())
        .unwrap_or("");

    let result = orchestrator.orchestrate(last_user_message, context);
    let mut prompt = result.dynamic_prompt;
    prompt.push_str("\n\nCONVERSATION HISTORY:");

    for (role, content) in messages {
        match role.as_str() {
            "user" => prompt.push_str(&format!("\n\nUSER: {}", content)),
            "assistant" => prompt.push_str(&format!("\n\nASSISTANT: {}", content)),
            _ => {}
        }
    }

    prompt.push_str("\n\nASSISTANT:");
    prompt
}

/// Build a minimal prompt for simple interactions (lower token usage)
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_lightweight_prompt(
    user_message: &str,
    context: &UserContext,
) -> String {
    let prompt_context = to_prompt_context(context);
    let system_prompt = build_minimal_prompt(&prompt_context);

    format!(
        "{}\n\n\
         USER MESSAGE: {}\n\n\
         Respond concisely (2-3 sentences). Be helpful and actionable.",
        system_prompt,
        user_message
    )
}

/// Build a prompt with session-specific context
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_session_prompt(
    context: &UserContext,
    planned_duration: i32,
    last_session_summary: Option<&str>,
) -> String {
    let prompt_context = to_prompt_context(context);

    ScenarioPromptBuilder::new(prompt_context)
        .with_session_context(planned_duration, last_session_summary)
        .build(UserIntent::SessionPlanning, true)
}

/// Build a prompt for post-session reflection
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_reflection_prompt_detailed(
    context: &UserContext,
    completed: bool,
    actual_duration: i32,
) -> String {
    let prompt_context = to_prompt_context(context);

    let prompt = ScenarioPromptBuilder::new(prompt_context)
        .with_reflection_context(completed, actual_duration)
        .build(UserIntent::SessionReflection, true);

    let task = if completed {
        format!(
            "TASK: The user completed a {}-minute focus session. \
             Generate 2-3 reflection questions to help them learn from the experience.",
            actual_duration
        )
    } else {
        format!(
            "TASK: The user ended their session early after {} minutes. \
             Be supportive and help them identify what happened without judgment.",
            actual_duration
        )
    };

    format!("{}\n\n{}", prompt, task)
}

/// Extract suggested actions from context (quick actions for UI)
#[allow(dead_code)] // Public API - may be used by external callers
pub fn build_suggestions(template: PromptTemplate, context: &UserContext) -> Vec<String> {
    match template {
        PromptTemplate::GeneralCoaching => {
            if context.sessions_completed_today == 0 {
                vec![
                    "Start a focus session".to_string(),
                    "View my blocked apps".to_string(),
                    "Check trigger insights".to_string(),
                ]
            } else {
                vec![
                    "Start another session".to_string(),
                    "Log a trigger".to_string(),
                    "View my progress".to_string(),
                ]
            }
        }

        PromptTemplate::DailyTip => {
            if context.current_streak_days == 0 {
                vec![
                    "Start a session now".to_string(),
                    "Set a focus goal".to_string(),
                ]
            } else {
                vec![
                    "Continue my streak".to_string(),
                    "Review blocking rules".to_string(),
                ]
            }
        }

        PromptTemplate::SessionAdvice => vec![
            "Start the session".to_string(),
            "Adjust duration".to_string(),
            "Update blocked apps".to_string(),
        ],

        PromptTemplate::ReflectionPrompt => vec![
            "Log a trigger".to_string(),
            "Start next session".to_string(),
            "Take a break".to_string(),
        ],

        PromptTemplate::PatternAnalysis => vec![
            "Start a session".to_string(),
            "View full analytics".to_string(),
            "Set a new goal".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_context() -> UserContext {
        UserContext {
            total_focus_hours_today: 2.5,
            sessions_completed_today: 3,
            current_streak_days: 5,
            top_trigger: Some("social media".to_string()),
            average_session_minutes: 25,
        }
    }

    #[test]
    fn test_prompt_context_conversion() {
        let ctx = mock_context();
        let prompt_ctx = to_prompt_context(&ctx);

        assert_eq!(prompt_ctx.sessions_completed, 3);
        assert_eq!(prompt_ctx.focus_hours, 2.5);
        assert_eq!(prompt_ctx.streak_days, 5);
        assert_eq!(prompt_ctx.top_trigger, Some("social media".to_string()));
    }

    #[test]
    fn test_system_prompt_includes_context() {
        let ctx = mock_context();
        let prompt = build_coach_prompt(PromptTemplate::GeneralCoaching, "How can I focus?", &ctx);

        assert!(prompt.contains("FocusFlow AI Coach"));
        assert!(prompt.contains("INDISTRACTABLE")); // Framework title is in caps
        assert!(prompt.contains("5 days"));
        assert!(prompt.contains("social media"));
    }

    #[test]
    fn test_daily_tip_prompt() {
        let ctx = mock_context();
        let prompt = build_coach_prompt(PromptTemplate::DailyTip, "", &ctx);

        assert!(prompt.contains("daily productivity tip"));
        assert!(prompt.contains("Do NOT start with greetings"));
    }

    #[test]
    fn test_session_advice_prompt() {
        let ctx = mock_context();
        let prompt = build_coach_prompt(
            PromptTemplate::SessionAdvice,
            "Planning a 45-minute session",
            &ctx,
        );

        assert!(prompt.contains("pre-session advice"));
        assert!(prompt.contains("45-minute session"));
    }

    #[test]
    fn test_pattern_analysis_prompt() {
        let ctx = mock_context();
        let prompt = build_coach_prompt(PromptTemplate::PatternAnalysis, "", &ctx);

        assert!(prompt.contains("productivity patterns"));
        assert!(prompt.contains("5-day streak"));
        assert!(prompt.contains("social media"));
    }

    #[test]
    fn test_chat_prompt_formatting() {
        let ctx = mock_context();
        let messages = vec![
            ("user".to_string(), "How can I focus better?".to_string()),
            (
                "assistant".to_string(),
                "Great question! Let's explore your triggers.".to_string(),
            ),
            ("user".to_string(), "I get distracted by social media".to_string()),
        ];

        let prompt = build_chat_prompt(&messages, &ctx);

        assert!(prompt.contains("USER: How can I focus better?"));
        assert!(prompt.contains("ASSISTANT: Great question!"));
        assert!(prompt.contains("USER: I get distracted by social media"));
        assert!(prompt.ends_with("\n\nASSISTANT:"));
    }

    #[test]
    fn test_lightweight_prompt_is_shorter() {
        let ctx = mock_context();
        let full = build_coach_prompt(PromptTemplate::GeneralCoaching, "Hello", &ctx);
        let light = build_lightweight_prompt("Hello", &ctx);

        assert!(light.len() < full.len());
    }

    #[test]
    fn test_session_prompt_includes_duration() {
        let ctx = mock_context();
        let prompt = build_session_prompt(&ctx, 45, Some("25-min completed session"));

        assert!(prompt.contains("45 minutes"));
        assert!(prompt.contains("25-min completed session"));
    }

    #[test]
    fn test_reflection_prompt_completed() {
        let ctx = mock_context();
        let prompt = build_reflection_prompt_detailed(&ctx, true, 30);

        assert!(prompt.contains("completed a 30-minute"));
        assert!(prompt.contains("reflection questions"));
    }

    #[test]
    fn test_reflection_prompt_early_end() {
        let ctx = mock_context();
        let prompt = build_reflection_prompt_detailed(&ctx, false, 15);

        assert!(prompt.contains("ended their session early"));
        assert!(prompt.contains("15 minutes"));
        assert!(prompt.contains("without judgment"));
    }

    #[test]
    fn test_suggestions_vary_by_template() {
        let ctx = mock_context();

        let general = build_suggestions(PromptTemplate::GeneralCoaching, &ctx);
        let session = build_suggestions(PromptTemplate::SessionAdvice, &ctx);

        assert!(!general.is_empty());
        assert!(!session.is_empty());
        assert_ne!(general, session);
    }

    #[test]
    fn test_suggestions_vary_by_context() {
        let mut ctx = mock_context();
        let with_sessions = build_suggestions(PromptTemplate::GeneralCoaching, &ctx);

        ctx.sessions_completed_today = 0;
        let no_sessions = build_suggestions(PromptTemplate::GeneralCoaching, &ctx);

        assert_ne!(with_sessions, no_sessions);
    }

    #[test]
    fn test_intent_detection_in_chat() {
        let ctx = mock_context();
        let messages = vec![
            ("user".to_string(), "I keep getting distracted".to_string()),
        ];

        let prompt = build_chat_prompt(&messages, &ctx);

        // Should include distraction-specific guidelines
        assert!(prompt.contains("DISTRACTION") || prompt.contains("distraction"));
    }

    #[test]
    fn test_daily_tip_focus_determination() {
        let mut ctx = mock_context();

        ctx.sessions_completed_today = 0;
        assert!(determine_daily_tip_focus(&ctx).contains("Getting started"));

        ctx.sessions_completed_today = 5;
        ctx.total_focus_hours_today = 4.0;
        assert!(determine_daily_tip_focus(&ctx).contains("burnout"));

        ctx.total_focus_hours_today = 1.0;
        ctx.current_streak_days = 10;
        assert!(determine_daily_tip_focus(&ctx).contains("consistency"));
    }

    #[test]
    fn test_streak_description() {
        assert!(describe_streak(0).contains("No active streak"));
        assert!(describe_streak(2).contains("building initial"));
        assert!(describe_streak(7).contains("developing consistency"));
        assert!(describe_streak(14).contains("forming"));
        assert!(describe_streak(30).contains("strong consistency"));
        assert!(describe_streak(60).contains("long-term"));
    }
}
