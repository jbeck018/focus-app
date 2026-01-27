// ai/system_prompts.rs - Modular system prompt architecture for AI Coach
//
// This module provides reusable prompt components that are consumed by the orchestrator
// and other parts of the AI system.
//
// ARCHITECTURE:
// - Core Identity: Who the AI is, personality, communication style
// - Framework Knowledge: Indistractable principles as structured reference
// - Response Guidelines: How to structure outputs for different scenarios
// - Tool Usage Instructions: How the AI should use available tools
// - PromptContext: Structured user context data
//
// USAGE:
// - For production: Use `GuidelineOrchestrator` which imports constants from this module
// - For testing/simple cases: Use `build_system_prompt()` or `build_minimal_prompt()`
//
// The orchestrator (orchestrator.rs) combines these components with dynamic guidelines
// from guideline_registry.rs to build context-aware prompts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// CORE IDENTITY PROMPT
// =============================================================================

/// The foundational identity of the AI Coach
pub const CORE_IDENTITY: &str = r#"You are the FocusFlow AI Coach, a supportive guide helping users build sustainable focus habits and overcome digital distractions.

PERSONALITY:
- Warm and encouraging, but not artificially enthusiastic
- Direct and practical - you value the user's time
- Curious about the user's experience - you ask thoughtful questions
- Non-judgmental - you meet users where they are
- Grounded in evidence-based techniques, not platitudes

COMMUNICATION STYLE:
- Conversational and natural, like a knowledgeable friend
- Concise by default (2-4 sentences for most responses)
- Specific over generic - reference concrete details when available
- Action-oriented - end with something the user can do
- Avoid: excessive punctuation, emojis, corporate speak, or being preachy
- IMPORTANT: Do not start messages with "Response:" - respond naturally without this prefix

VOICE EXAMPLES:
- Instead of "Great job!" use "That's solid progress."
- Instead of "You've got this!" use "You're building real momentum here."
- Instead of "Don't worry about setbacks!" use "Setbacks are information. What triggered this one?"
- Instead of "Amazing work!!!" use "Nice work on that session."

YOUR BOUNDARIES:
- You focus on productivity, focus, and habit-building
- You don't provide medical, psychological, or crisis intervention advice
- If users seem distressed, gently suggest professional support
- You're honest when you don't have enough information"#;

// =============================================================================
// FRAMEWORK KNOWLEDGE - Indistractable Principles
// =============================================================================

/// Structured knowledge of the Indistractable framework
pub const FRAMEWORK_KNOWLEDGE: &str = r#"THE INDISTRACTABLE FRAMEWORK (Nir Eyal)
You draw from these four core principles, applying them contextually:

1. MASTER INTERNAL TRIGGERS
   Core insight: All behavior is driven by a desire to escape discomfort
   Key techniques:
   - Identify the emotion/feeling preceding distraction (boredom, anxiety, fatigue)
   - Surf the urge: acknowledge it without acting (10-minute rule)
   - Reimagine the task: find curiosity or challenge within it
   - Reimagine your temperament: "I'm someone who stays focused"
   Application: When users report distraction, explore the internal trigger first

2. MAKE TIME FOR TRACTION
   Core insight: You can't call something a distraction unless you know what it distracted you from
   Key techniques:
   - Timeboxing: schedule time for focused work in advance
   - Turn values into time: allocate time for what matters most
   - Schedule reflection time to review and adjust
   Application: Help users plan specific focus sessions, not just "work more"

3. HACK BACK EXTERNAL TRIGGERS
   Core insight: External triggers (notifications, interruptions) can be managed
   Key techniques:
   - Audit triggers: which are serving you vs. distracting you?
   - Remove temptations before they appear (blocking apps, phone in another room)
   - Create "do not disturb" signals for others
   Application: Help users configure their environment before willpower is needed

4. PREVENT DISTRACTION WITH PACTS
   Core insight: Pre-commitment makes the right choice easier
   Types of pacts:
   - Effort pact: Add friction to unwanted behaviors
   - Price pact: Put something valuable at stake
   - Identity pact: "I am the kind of person who..."
   Application: Suggest concrete pre-commitments when users struggle with consistency

WHEN TO REFERENCE PRINCIPLES:
- Don't lecture - weave principles naturally into advice
- One principle per response is usually enough
- Connect principle to user's specific situation
- Offer the principle as a tool, not a rule"#;

// =============================================================================
// RESPONSE GUIDELINES
// =============================================================================

/// Guidelines for structuring responses
pub const RESPONSE_GUIDELINES: &str = "RESPONSE STRUCTURE GUIDELINES

MARKDOWN FORMATTING:
- Use proper markdown syntax for formatting
- Headers must be on their own line: separate from other text
- Don't put headers inline with other text
- Avoid stray backticks - only use backticks for inline code or code blocks
- For code blocks, use triple backticks on their own line
- Keep formatting simple and readable

LENGTH GUIDELINES:
- Chat/Quick questions: 2-4 sentences (50-100 words)
- Session advice: 3-4 points, each 1-2 sentences
- Pattern analysis: 2-3 insights with brief explanation (100-150 words)
- Reflection prompts: 2-3 questions with brief context
- Help/Navigation: Concise list format with 1-line descriptions

WHEN TO GO LONGER:
- User explicitly asks for detailed analysis
- Pattern analysis with significant data to discuss
- Complex situation requiring nuanced guidance
- User seems stuck and needs comprehensive help

WHEN TO STAY SHORT:
- Quick encouragement or acknowledgment
- Simple questions with clear answers
- User is about to start a session (don't delay them)
- Follow-up in an ongoing conversation

QUESTION GUIDELINES:
- Ask ONE follow-up question maximum per response
- Make questions specific, not generic
- Good: \"What usually triggers the social media urge?\"
- Bad: \"How do you feel about that?\"
- Don't ask questions when user needs action, not reflection

ACTION SUGGESTION GUIDELINES:
- Suggest actions that are immediately actionable
- Connect suggestions to available tools when relevant
- Prioritize: most impactful action first
- Maximum 3 suggestions per response

TONE CALIBRATION:
- Match user's energy level (don't be perky when they're frustrated)
- Acknowledge difficulty without dwelling on it
- Celebrate progress proportionally (small win = small acknowledgment)
- Be honest about uncertainty";

// =============================================================================
// CONTEXT VARIABLE SYSTEM
// =============================================================================

/// Template for injecting user context
#[allow(dead_code)] // Template constant for reference
pub const USER_CONTEXT_TEMPLATE: &str = r#"USER CONTEXT:
- Time: {time_of_day} on {day_of_week}
- Today's progress: {sessions_completed} sessions, {focus_hours:.1} hours focused
- Current streak: {streak_days} days
- Average session: {avg_session_minutes} minutes
- Top distraction trigger: {top_trigger}
{additional_context}"#;

/// Extended context for session-specific scenarios
#[allow(dead_code)] // Template constant for reference
pub const SESSION_CONTEXT_TEMPLATE: &str = r#"SESSION CONTEXT:
- Planned duration: {planned_duration} minutes
- Sessions today: {sessions_today}
- Last session: {last_session_info}
- Current energy note: {energy_level}"#;

// =============================================================================
// GUIDELINE INJECTION POINTS
// =============================================================================

/// Main system prompt template with injection points
#[allow(dead_code)] // Template constant for reference
pub const SYSTEM_PROMPT_TEMPLATE: &str = r#"{core_identity}

{framework_knowledge}

{response_guidelines}

{user_context}

{active_guidelines}

{available_tools}

{scenario_instructions}"#;

// =============================================================================
// TOOL USAGE INSTRUCTIONS
// =============================================================================

/// Instructions for how the AI should use tools
pub const TOOL_USAGE_INSTRUCTIONS: &str = r#"## Tools

You have these tools available:

- start_focus_session: Start a focus session. Optional: duration (minutes, default 25)
- end_focus_session: End current session. Optional: completed (true/false)
- log_trigger: Log a distraction. Required: trigger_type. Optional: notes, intensity (1-5)
- get_trigger_patterns: Show user's distraction patterns
- add_blocked_item: Block an app/website. Required: type (app/website), value
- get_blocking_stats: Show what's blocked
- get_session_stats: Show focus stats. Optional: period (today/week)
- get_streak_info: Show streak progress
- set_focus_goal: Set a goal. Required: type (daily/weekly), minutes

## Tool Format

Write tool calls as plain XML on their own line:
<tool name="start_focus_session" duration="25"/>

## CRITICAL RULES

1. Use AT MOST one tool per response
2. Do NOT wrap tools in code blocks or backticks - write them as plain text
3. Do NOT use headers like Response: - just respond naturally
4. Keep responses under 3 paragraphs
5. Stop after making your point
6. If user just wants to chat, skip the tool entirely"#;

// =============================================================================
// SCENARIO GUIDELINES (DEPRECATED - Use guideline_registry.rs instead)
// =============================================================================
//
// NOTE: These scenario guidelines are deprecated and have been removed.
// Use GuidelineRegistry with orchestrator instead (see guideline_registry.rs).

// =============================================================================
// SCENARIO ENUMERATION (Alternative to GuidelineMatch system)
// =============================================================================
//
// NOTE: This UserIntent system provides an alternative, simpler approach to intent
// detection compared to the full guideline matching system. It's currently available
// but not used by the orchestrator, which uses GuidelineMatch instead.
//
// Keep this if you want a lightweight intent detection option, or remove if the
// guideline system fully replaces it.

/// Detected user intent for guideline selection (alternative to GuidelineMatch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserIntent {
    FocusInquiry,
    DistractionConcern,
    MotivationEnergy,
    HelpNavigation,
    SessionPlanning,
    SessionReflection,
    PatternAnalysis,
    StreakProgress,
    GeneralChat,
}

impl UserIntent {
    /// Detect intent from user message (keyword-based, can be upgraded to NLU)
    pub fn detect(message: &str) -> Self {
        let msg_lower = message.to_lowercase();

        // Order matters - check more specific intents first
        // Session planning: explicit session mentions or planning focus time
        if (msg_lower.contains("session") && (msg_lower.contains("plan") || msg_lower.contains("start") || msg_lower.contains("begin")))
            || (msg_lower.contains("planning") && (msg_lower.contains("focus") || msg_lower.contains("work")))
            || msg_lower.contains("start a session")
            || msg_lower.contains("begin a session")
        {
            return UserIntent::SessionPlanning;
        }

        if msg_lower.contains("finished") || msg_lower.contains("completed") || msg_lower.contains("ended") || msg_lower.contains("done with") {
            return UserIntent::SessionReflection;
        }

        if msg_lower.contains("pattern") || msg_lower.contains("insight") || msg_lower.contains("analy") || msg_lower.contains("trend") {
            return UserIntent::PatternAnalysis;
        }

        if msg_lower.contains("streak") || msg_lower.contains("progress") || msg_lower.contains("achievement") || msg_lower.contains("how am i doing") {
            return UserIntent::StreakProgress;
        }

        if msg_lower.contains("focus") || msg_lower.contains("concentrate") || msg_lower.contains("attention") {
            return UserIntent::FocusInquiry;
        }

        if msg_lower.contains("distract") || msg_lower.contains("interrupt") || msg_lower.contains("off track") || msg_lower.contains("lost focus") {
            return UserIntent::DistractionConcern;
        }

        if msg_lower.contains("motivation") || msg_lower.contains("tired") || msg_lower.contains("exhausted")
            || msg_lower.contains("energy") || msg_lower.contains("burned out") || msg_lower.contains("can't focus") {
            return UserIntent::MotivationEnergy;
        }

        if msg_lower.contains("help") || msg_lower.contains("how do") || msg_lower.contains("what can")
            || msg_lower.contains("how to") || msg_lower.contains("what do") {
            return UserIntent::HelpNavigation;
        }

        UserIntent::GeneralChat
    }
}

// =============================================================================
// PROMPT BUILDER
// =============================================================================

/// User context for prompt generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    pub time_of_day: String,
    pub day_of_week: String,
    pub sessions_completed: i32,
    pub focus_hours: f32,
    pub streak_days: i32,
    pub avg_session_minutes: i32,
    pub top_trigger: Option<String>,
    pub additional_context: Option<String>,
}

impl Default for PromptContext {
    fn default() -> Self {
        Self {
            time_of_day: "morning".to_string(),
            day_of_week: "Monday".to_string(),
            sessions_completed: 0,
            focus_hours: 0.0,
            streak_days: 0,
            avg_session_minutes: 25,
            top_trigger: None,
            additional_context: None,
        }
    }
}

/// Build the complete system prompt with all components
///
/// NOTE: This function is kept for backward compatibility and simple use cases.
/// For production use, prefer `GuidelineOrchestrator::build_dynamic_prompt()` which
/// provides dynamic guideline matching and more sophisticated prompt construction.
#[deprecated(note = "Use GuidelineOrchestrator::orchestrate() for production use")]
#[allow(dead_code)] // Public API - may be used by external callers or tests
pub fn build_system_prompt(context: &PromptContext, _intent: UserIntent, include_tools: bool) -> String {
    let user_context = build_user_context(context);
    // Guidelines are now managed by the orchestrator - this function provides a minimal prompt
    let tools = if include_tools {
        format!("\n{}", TOOL_USAGE_INSTRUCTIONS)
    } else {
        String::new()
    };

    format!(
        "{}\n\n{}\n\n{}\n\n{}{}",
        CORE_IDENTITY,
        FRAMEWORK_KNOWLEDGE,
        RESPONSE_GUIDELINES,
        user_context,
        tools
    )
}

/// Build the user context section
#[allow(dead_code)] // Used by deprecated build_system_prompt
fn build_user_context(context: &PromptContext) -> String {
    let trigger_display = context.top_trigger.as_deref().unwrap_or("not yet identified");
    let additional = context.additional_context.as_deref().unwrap_or("");

    format!(
        r#"USER CONTEXT:
- Time: {} on {}
- Today's progress: {} sessions, {:.1} hours focused
- Current streak: {} days
- Average session: {} minutes
- Top distraction trigger: {}
{}"#,
        context.time_of_day,
        context.day_of_week,
        context.sessions_completed,
        context.focus_hours,
        context.streak_days,
        context.avg_session_minutes,
        trigger_display,
        additional
    )
}

/// Build a lightweight system prompt for simple interactions
///
/// NOTE: This function is kept for testing and simple use cases.
/// For production use, prefer `GuidelineOrchestrator::build_dynamic_prompt()`.
#[allow(dead_code)] // Public API - may be used by external callers or tests
pub fn build_minimal_prompt(context: &PromptContext) -> String {
    format!(
        "{}\n\n{}\n\n{}",
        CORE_IDENTITY,
        RESPONSE_GUIDELINES,
        build_user_context(context)
    )
}

/// Scenario-specific prompt builder for templated use cases
///
/// NOTE: This builder is kept for backward compatibility and simple scenarios.
/// For production use, prefer `GuidelineOrchestrator` which provides more sophisticated
/// context handling and guideline matching.
#[allow(dead_code)] // Public API - may be used by external callers
pub struct ScenarioPromptBuilder {
    context: PromptContext,
    scenario_overrides: HashMap<String, String>,
}

impl ScenarioPromptBuilder {
    #[allow(dead_code)] // Public API method
    pub fn new(context: PromptContext) -> Self {
        Self {
            context,
            scenario_overrides: HashMap::new(),
        }
    }

    /// Add session-specific context
    #[allow(dead_code)] // Public API method
    pub fn with_session_context(mut self, planned_duration: i32, last_session: Option<&str>) -> Self {
        let session_info = format!(
            "\nSESSION CONTEXT:\n- Planned duration: {} minutes\n- Last session: {}",
            planned_duration,
            last_session.unwrap_or("No recent session")
        );
        self.context.additional_context = Some(session_info);
        self
    }

    /// Add reflection-specific context
    #[allow(dead_code)] // Public API method
    pub fn with_reflection_context(mut self, completed: bool, actual_duration: i32) -> Self {
        let reflection_info = format!(
            "\nSESSION JUST ENDED:\n- Completed: {}\n- Duration: {} minutes",
            if completed { "Yes" } else { "No (ended early)" },
            actual_duration
        );
        self.context.additional_context = Some(reflection_info);
        self
    }

    /// Add custom scenario override
    #[allow(dead_code)] // Public API method
    pub fn with_override(mut self, key: &str, value: &str) -> Self {
        self.scenario_overrides.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the final prompt
    #[allow(dead_code)] // Public API method
    pub fn build(self, intent: UserIntent, _include_tools: bool) -> String {
        // Use GuidelineOrchestrator for production use
        // Note: The orchestrator includes tool instructions by default
        use crate::ai::orchestrator::GuidelineOrchestrator;
        use crate::commands::coach::UserContext;

        // Convert PromptContext to UserContext for orchestrator
        let user_context = UserContext {
            total_focus_hours_today: self.context.focus_hours,
            sessions_completed_today: self.context.sessions_completed,
            current_streak_days: self.context.streak_days,
            top_trigger: self.context.top_trigger.clone(),
            average_session_minutes: self.context.avg_session_minutes,
        };

        let orchestrator = GuidelineOrchestrator::new();

        // For ScenarioPromptBuilder, we want to use a simple message that triggers
        // the right intent. The orchestrator will match appropriate guidelines.
        let message = match intent {
            UserIntent::SessionPlanning => "planning a session",
            UserIntent::SessionReflection => "reflecting on session",
            UserIntent::PatternAnalysis => "analyzing patterns",
            UserIntent::FocusInquiry => "need focus help",
            UserIntent::DistractionConcern => "getting distracted",
            UserIntent::MotivationEnergy => "need motivation",
            UserIntent::StreakProgress => "checking progress",
            UserIntent::HelpNavigation => "need help",
            UserIntent::GeneralChat => "",
        };

        let result = orchestrator.orchestrate(message, &user_context);

        // If additional context was added, append it
        if let Some(additional) = &self.context.additional_context {
            format!("{}\n\n{}", result.dynamic_prompt, additional)
        } else {
            result.dynamic_prompt
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_context() -> PromptContext {
        PromptContext {
            time_of_day: "afternoon".to_string(),
            day_of_week: "Tuesday".to_string(),
            sessions_completed: 3,
            focus_hours: 2.5,
            streak_days: 5,
            avg_session_minutes: 25,
            top_trigger: Some("social media".to_string()),
            additional_context: None,
        }
    }

    #[test]
    fn test_intent_detection_focus() {
        assert_eq!(UserIntent::detect("How can I focus better?"), UserIntent::FocusInquiry);
        assert_eq!(UserIntent::detect("I need to concentrate"), UserIntent::FocusInquiry);
    }

    #[test]
    fn test_intent_detection_distraction() {
        assert_eq!(UserIntent::detect("I keep getting distracted"), UserIntent::DistractionConcern);
        assert_eq!(UserIntent::detect("Too many interruptions today"), UserIntent::DistractionConcern);
    }

    #[test]
    fn test_intent_detection_motivation() {
        assert_eq!(UserIntent::detect("I'm so tired today"), UserIntent::MotivationEnergy);
        assert_eq!(UserIntent::detect("No motivation to work"), UserIntent::MotivationEnergy);
    }

    #[test]
    fn test_intent_detection_help() {
        assert_eq!(UserIntent::detect("How do I use this?"), UserIntent::HelpNavigation);
        assert_eq!(UserIntent::detect("What can you help me with?"), UserIntent::HelpNavigation);
    }

    #[test]
    fn test_intent_detection_session() {
        assert_eq!(UserIntent::detect("I want to start a session"), UserIntent::SessionPlanning);
        assert_eq!(UserIntent::detect("Planning my focus time"), UserIntent::SessionPlanning);
    }

    #[test]
    fn test_intent_detection_reflection() {
        assert_eq!(UserIntent::detect("Just finished my session"), UserIntent::SessionReflection);
        assert_eq!(UserIntent::detect("I ended the session early"), UserIntent::SessionReflection);
    }

    #[test]
    fn test_intent_detection_patterns() {
        assert_eq!(UserIntent::detect("Show me my patterns"), UserIntent::PatternAnalysis);
        assert_eq!(UserIntent::detect("Analyze my focus trends"), UserIntent::PatternAnalysis);
    }

    #[test]
    fn test_intent_detection_streak() {
        assert_eq!(UserIntent::detect("What's my streak?"), UserIntent::StreakProgress);
        assert_eq!(UserIntent::detect("How am I doing?"), UserIntent::StreakProgress);
    }

    #[test]
    fn test_intent_detection_general() {
        assert_eq!(UserIntent::detect("Hello there"), UserIntent::GeneralChat);
        assert_eq!(UserIntent::detect("What's the weather?"), UserIntent::GeneralChat);
    }

    #[test]
    fn test_system_prompt_contains_core_elements() {
        let ctx = mock_context();
        #[allow(deprecated)]
        let prompt = build_system_prompt(&ctx, UserIntent::FocusInquiry, true);

        assert!(prompt.contains("FocusFlow AI Coach"));
        assert!(prompt.contains("INDISTRACTABLE")); // Framework title is in caps
        assert!(prompt.contains("RESPONSE STRUCTURE"));
        assert!(prompt.contains("afternoon on Tuesday"));
        assert!(prompt.contains("5 days"));
        assert!(prompt.contains("social media"));
        // Note: Tool format now uses XML-style syntax
        assert!(prompt.contains("<tool name=\"start_focus_session\""));
    }

    #[test]
    fn test_minimal_prompt_excludes_extras() {
        let ctx = mock_context();
        let prompt = build_minimal_prompt(&ctx);

        assert!(prompt.contains("FocusFlow AI Coach"));
        assert!(!prompt.contains("AVAILABLE TOOLS"));
        assert!(!prompt.contains("SCENARIO:"));
    }

    #[test]
    fn test_scenario_builder_with_session_context() {
        let ctx = mock_context();
        let prompt = ScenarioPromptBuilder::new(ctx)
            .with_session_context(45, Some("25-min session, completed"))
            .build(UserIntent::SessionPlanning, false);

        assert!(prompt.contains("Planned duration: 45 minutes"));
        assert!(prompt.contains("25-min session, completed"));
    }

    #[test]
    fn test_scenario_builder_with_reflection_context() {
        let ctx = mock_context();
        let prompt = ScenarioPromptBuilder::new(ctx)
            .with_reflection_context(true, 30)
            .build(UserIntent::SessionReflection, false);

        assert!(prompt.contains("Completed: Yes"));
        assert!(prompt.contains("Duration: 30 minutes"));
    }

}
