// ai/orchestrator.rs - Main orchestration logic for Parlant-style guideline matching
//
// The orchestrator evaluates ALL guidelines against each user turn and dynamically
// builds prompts with only the relevant guidelines injected. This enables coherent
// multi-topic responses without hardcoded routing logic.
//
// INTEGRATION WITH SYSTEM_PROMPTS:
// This orchestrator uses the modular prompt components from system_prompts.rs:
// - CORE_IDENTITY: Base personality and communication style
// - FRAMEWORK_KNOWLEDGE: Indistractable principles
// - RESPONSE_GUIDELINES: Response structure and tone guidelines
// - TOOL_USAGE_INSTRUCTIONS: How to use available tools
// - PromptContext: Structured user context data
//
// The orchestrator combines these base components with dynamic guidelines from
// guideline_registry.rs to create context-aware, personalized prompts.

use crate::ai::guideline_registry::GuidelineRegistry;
use crate::ai::guidelines::{GuidelineCategory, GuidelineMatch};
use crate::ai::system_prompts::{
    CORE_IDENTITY, FRAMEWORK_KNOWLEDGE, RESPONSE_GUIDELINES, TOOL_USAGE_INSTRUCTIONS,
    PromptContext,
};
use crate::commands::coach::UserContext;
use chrono::{Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for the orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum number of guidelines to include in a single prompt
    pub max_guidelines: usize,
    /// Minimum confidence threshold for including a guideline
    pub min_confidence: f32,
    /// Whether to include contextual guidelines that match based on state
    pub include_contextual: bool,
    /// Whether to deduplicate similar guidelines
    pub deduplicate: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_guidelines: 5,
            min_confidence: 0.3,
            include_contextual: true,
            deduplicate: true,
        }
    }
}

/// Result of orchestration containing matched guidelines and built prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// Guidelines that were matched and included
    pub matched_guidelines: Vec<MatchedGuidelineInfo>,
    /// The complete dynamic prompt ready for the LLM
    pub dynamic_prompt: String,
    /// Suggested actions aggregated from all matched guidelines
    pub suggested_tools: Vec<String>,
    /// Keywords that triggered the matches
    pub matched_keywords: Vec<String>,
}

/// Information about a matched guideline (serializable subset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedGuidelineInfo {
    pub id: String,
    pub name: String,
    pub priority: i32,
    pub confidence: f32,
    pub category: GuidelineCategory,
}

/// The main orchestrator for dynamic guideline matching
pub struct GuidelineOrchestrator {
    registry: GuidelineRegistry,
    config: OrchestratorConfig,
}

impl GuidelineOrchestrator {
    /// Create a new orchestrator with default configuration
    pub fn new() -> Self {
        Self {
            registry: GuidelineRegistry::new(),
            config: OrchestratorConfig::default(),
        }
    }

    /// Create an orchestrator with custom configuration
    #[allow(dead_code)] // Public API for custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self {
            registry: GuidelineRegistry::new(),
            config,
        }
    }

    /// Evaluate all guidelines against the user message and context
    ///
    /// Returns matching guidelines sorted by priority and confidence
    pub fn evaluate_guidelines(
        &self,
        user_message: &str,
        context: &UserContext,
    ) -> Vec<GuidelineMatch> {
        let mut matches: Vec<GuidelineMatch> = self
            .registry
            .all_guidelines()
            .iter()
            .filter_map(|g| {
                // Skip contextual guidelines if disabled
                if !self.config.include_contextual && g.category == GuidelineCategory::Contextual {
                    return None;
                }

                g.evaluate(user_message, context)
            })
            .filter(|m| m.confidence >= self.config.min_confidence)
            .collect();

        // Sort by priority (descending), then confidence (descending)
        matches.sort_by(|a, b| {
            let priority_cmp = b.guideline.priority.cmp(&a.guideline.priority);
            if priority_cmp == std::cmp::Ordering::Equal {
                b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                priority_cmp
            }
        });

        // Deduplicate if enabled (keep highest priority version of similar guidelines)
        if self.config.deduplicate {
            matches = self.deduplicate_guidelines(matches);
        }

        // Limit number of guidelines
        matches.truncate(self.config.max_guidelines);

        matches
    }

    /// Build a complete dynamic prompt with matched guidelines injected
    pub fn build_dynamic_prompt(
        &self,
        user_message: &str,
        context: &UserContext,
        matched_guidelines: &[GuidelineMatch],
    ) -> String {
        self.build_dynamic_prompt_with_options(user_message, context, matched_guidelines, true)
    }

    /// Build dynamic prompt with option to include/exclude tool instructions
    pub fn build_dynamic_prompt_with_options(
        &self,
        user_message: &str,
        context: &UserContext,
        matched_guidelines: &[GuidelineMatch],
        include_tools: bool,
    ) -> String {
        let system_prompt = self.build_system_prompt(context);
        let guidelines_section = self.build_guidelines_section(matched_guidelines, context);
        let tool_section = if include_tools {
            format!("\n\n{}", TOOL_USAGE_INSTRUCTIONS)
        } else {
            String::new()
        };
        let user_section = self.build_user_section(user_message, matched_guidelines);

        format!(
            "{}{}\n\n{}\n\n{}",
            system_prompt, tool_section, guidelines_section, user_section
        )
    }

    /// Full orchestration: evaluate guidelines and build prompt
    pub fn orchestrate(
        &self,
        user_message: &str,
        context: &UserContext,
    ) -> OrchestrationResult {
        let matches = self.evaluate_guidelines(user_message, context);
        let dynamic_prompt = self.build_dynamic_prompt(user_message, context, &matches);

        // Collect all tools from matched guidelines
        let suggested_tools: Vec<String> = matches
            .iter()
            .flat_map(|m| m.guideline.tools.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Collect all matched keywords
        let matched_keywords: Vec<String> = matches
            .iter()
            .flat_map(|m| m.matched_keywords.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let matched_guidelines = matches
            .iter()
            .map(|m| MatchedGuidelineInfo {
                id: m.guideline.id.clone(),
                name: m.guideline.name.clone(),
                priority: m.guideline.priority,
                confidence: m.confidence,
                category: m.guideline.category,
            })
            .collect();

        OrchestrationResult {
            matched_guidelines,
            dynamic_prompt,
            suggested_tools,
            matched_keywords,
        }
    }

    /// Orchestrate for a specific prompt type (daily tip, session advice, etc.)
    pub fn orchestrate_for_type(
        &self,
        prompt_type: PromptType,
        context: &UserContext,
        additional_message: Option<&str>,
    ) -> OrchestrationResult {
        // Get type-specific guidelines
        let type_guidelines = self.get_type_specific_guidelines(prompt_type, context);

        // Also match against additional message if provided
        let message_matches = additional_message
            .map(|msg| self.evaluate_guidelines(msg, context))
            .unwrap_or_default();

        // Combine and deduplicate
        let mut all_matches = type_guidelines;
        for m in message_matches {
            if !all_matches.iter().any(|existing| existing.guideline.id == m.guideline.id) {
                all_matches.push(m);
            }
        }

        // Sort and limit
        all_matches.sort_by(|a, b| {
            b.guideline.priority.cmp(&a.guideline.priority)
        });
        all_matches.truncate(self.config.max_guidelines);

        let dynamic_prompt = self.build_dynamic_prompt(
            additional_message.unwrap_or(""),
            context,
            &all_matches,
        );

        let suggested_tools: Vec<String> = all_matches
            .iter()
            .flat_map(|m| m.guideline.tools.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let matched_keywords: Vec<String> = all_matches
            .iter()
            .flat_map(|m| m.matched_keywords.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let matched_guidelines = all_matches
            .iter()
            .map(|m| MatchedGuidelineInfo {
                id: m.guideline.id.clone(),
                name: m.guideline.name.clone(),
                priority: m.guideline.priority,
                confidence: m.confidence,
                category: m.guideline.category,
            })
            .collect();

        OrchestrationResult {
            matched_guidelines,
            dynamic_prompt,
            suggested_tools,
            matched_keywords,
        }
    }

    /// Get guidelines specific to a prompt type
    fn get_type_specific_guidelines(
        &self,
        prompt_type: PromptType,
        context: &UserContext,
    ) -> Vec<GuidelineMatch> {
        // Get all guidelines and filter by relevance to prompt type
        let all_guidelines = self.registry.all_guidelines();

        let relevant: Vec<_> = match prompt_type {
            PromptType::DailyTip => {
                // For daily tips, use contextual guidelines
                all_guidelines
                    .iter()
                    .filter(|g| g.category == GuidelineCategory::Contextual)
                    .filter_map(|g| g.evaluate("", context))
                    .collect()
            }
            PromptType::SessionAdvice => {
                all_guidelines
                    .iter()
                    .filter(|g| {
                        g.category == GuidelineCategory::Session
                            || g.id == "session_planning"
                    })
                    .filter_map(|g| {
                        // Create a synthetic match for session guidelines
                        Some(GuidelineMatch {
                            guideline: g.clone(),
                            confidence: 1.0,
                            matched_keywords: vec!["session".into()],
                        })
                    })
                    .collect()
            }
            PromptType::Reflection => {
                all_guidelines
                    .iter()
                    .filter(|g| {
                        g.id.contains("reflection")
                            || g.category == GuidelineCategory::Session
                    })
                    .filter_map(|g| {
                        Some(GuidelineMatch {
                            guideline: g.clone(),
                            confidence: 1.0,
                            matched_keywords: vec!["reflection".into()],
                        })
                    })
                    .collect()
            }
            PromptType::PatternAnalysis => {
                all_guidelines
                    .iter()
                    .filter(|g| g.category == GuidelineCategory::Analytics)
                    .filter_map(|g| g.evaluate("", context))
                    .chain(
                        // Also include contextual guidelines that match
                        all_guidelines
                            .iter()
                            .filter(|g| g.category == GuidelineCategory::Contextual)
                            .filter_map(|g| g.evaluate("", context)),
                    )
                    .collect()
            }
            PromptType::GeneralCoaching => {
                // No specific filtering - use message matching
                vec![]
            }
        };

        relevant
    }

    /// Deduplicate guidelines that are too similar
    fn deduplicate_guidelines(&self, matches: Vec<GuidelineMatch>) -> Vec<GuidelineMatch> {
        let mut seen_categories: HashSet<(GuidelineCategory, String)> = HashSet::new();
        let mut deduplicated = Vec::new();

        for m in matches {
            // Create a key based on category and primary keyword
            // If no keywords matched (e.g., context-based matches), use the guideline ID
            // to prevent deduplication of distinct context-based guidelines
            let primary_keyword = m.matched_keywords.first().cloned()
                .unwrap_or_else(|| m.guideline.id.clone());
            let key = (m.guideline.category, primary_keyword);

            // Allow multiple from same category if they have different triggers
            // High priority (>=90) guidelines are always included
            if !seen_categories.contains(&key) || m.guideline.priority >= 90 {
                seen_categories.insert(key);
                deduplicated.push(m);
            }
        }

        deduplicated
    }

    /// Build the core system prompt using modular components from system_prompts
    fn build_system_prompt(&self, context: &UserContext) -> String {
        let prompt_context = Self::convert_user_context_to_prompt_context(context);
        let user_context_section = Self::build_user_context_section(&prompt_context);

        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            CORE_IDENTITY,
            FRAMEWORK_KNOWLEDGE,
            RESPONSE_GUIDELINES,
            user_context_section
        )
    }

    /// Convert UserContext to PromptContext
    fn convert_user_context_to_prompt_context(context: &UserContext) -> PromptContext {
        let now = Utc::now();
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

    /// Build user context section from PromptContext
    fn build_user_context_section(context: &PromptContext) -> String {
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

    /// Build the guidelines section of the prompt
    fn build_guidelines_section(
        &self,
        matches: &[GuidelineMatch],
        context: &UserContext,
    ) -> String {
        if matches.is_empty() {
            return String::from(
                "ACTIVE GUIDELINES:\nNo specific guidelines matched. Provide general coaching support.",
            );
        }

        let mut section = String::from("ACTIVE GUIDELINES:\n");
        section.push_str("The following guidelines are relevant to this interaction. Apply them coherently.\n\n");

        for (i, m) in matches.iter().enumerate() {
            section.push_str(&format!(
                "--- Guideline {} ({}, priority: {}, confidence: {:.0}%) ---\n",
                i + 1,
                m.guideline.name,
                m.guideline.priority,
                m.confidence * 100.0
            ));
            section.push_str(&m.get_action_with_context());
            section.push_str("\n\n");
        }

        // Add context-specific notes
        if let Some(trigger) = &context.top_trigger {
            section.push_str(&format!(
                "USER'S TOP TRIGGER: {} - reference this when relevant.\n",
                trigger
            ));
        }

        section
    }

    /// Build the user section of the prompt
    fn build_user_section(&self, user_message: &str, matches: &[GuidelineMatch]) -> String {
        let mut section = String::new();

        // Add detected topics
        if !matches.is_empty() {
            let topics: Vec<_> = matches
                .iter()
                .map(|m| m.guideline.name.as_str())
                .collect();
            section.push_str(&format!(
                "DETECTED TOPICS: {}\n",
                topics.join(", ")
            ));
        }

        if !user_message.is_empty() {
            section.push_str(&format!("\nUSER MESSAGE: {}\n", user_message));
        }

        section.push_str("\nProvide a helpful, personalized response that addresses all relevant topics coherently.");

        section
    }

    /// Get the registry for direct access (useful for testing)
    #[allow(dead_code)] // Public API for testing and introspection
    pub fn registry(&self) -> &GuidelineRegistry {
        &self.registry
    }
}

/// Types of prompts that can be orchestrated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // All variants are part of the public API
pub enum PromptType {
    GeneralCoaching,
    DailyTip,
    SessionAdvice,
    Reflection,
    PatternAnalysis,
}

impl Default for GuidelineOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Build suggested actions from matched guidelines
pub fn build_suggestions_from_guidelines(
    matches: &[GuidelineMatch],
    context: &UserContext,
) -> Vec<String> {
    // Default suggestions if no guidelines matched
    if matches.is_empty() {
        return if context.sessions_completed_today == 0 {
            vec![
                "Start a focus session".into(),
                "View my blocked apps".into(),
                "Check trigger insights".into(),
            ]
        } else {
            vec![
                "Start another session".into(),
                "Log a trigger".into(),
                "View my progress".into(),
            ]
        };
    }

    // Extract suggestions from guidelines based on their actions
    let mut suggestions: Vec<String> = Vec::new();

    for m in matches.iter().take(2) {
        // Parse suggestions from the guideline action text
        let action = &m.guideline.action;
        if let Some(suggestions_start) = action.find("Suggestions to offer:") {
            let suggestions_text = &action[suggestions_start..];
            for line in suggestions_text.lines().skip(1) {
                let line = line.trim();
                if line.starts_with("- ") {
                    suggestions.push(line[2..].to_string());
                }
                if suggestions.len() >= 3 {
                    break;
                }
            }
        }
    }

    // Fallback if no suggestions were parsed
    if suggestions.is_empty() {
        suggestions = vec![
            "Start a focus session".into(),
            "Log a trigger".into(),
            "View my progress".into(),
        ];
    }

    // Limit to 3 suggestions
    suggestions.truncate(3);
    suggestions
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
    fn test_orchestrate_focus_query() {
        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        let result = orchestrator.orchestrate("How can I focus better?", &ctx);

        assert!(!result.matched_guidelines.is_empty(), "Expected at least one matched guideline");
        assert!(
            result.matched_guidelines.iter().any(|g| g.id == "focus_coaching"),
            "Expected focus_coaching guideline to match. Got: {:?}",
            result.matched_guidelines.iter().map(|g| &g.id).collect::<Vec<_>>()
        );
        assert!(!result.dynamic_prompt.is_empty());
    }

    #[test]
    fn test_orchestrate_multi_topic() {
        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        let result = orchestrator.orchestrate(
            "How do I focus and what's my progress?",
            &ctx,
        );

        // Should match both focus and progress guidelines
        assert!(result.matched_guidelines.len() >= 2);
        assert!(result.dynamic_prompt.contains("DETECTED TOPICS"));
    }

    #[test]
    fn test_orchestrate_for_type_daily_tip() {
        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        let result = orchestrator.orchestrate_for_type(PromptType::DailyTip, &ctx, None);

        // Should have contextual guidelines
        assert!(!result.matched_guidelines.is_empty());
        assert!(!result.dynamic_prompt.is_empty());
    }

    #[test]
    fn test_priority_ordering() {
        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        let matches = orchestrator.evaluate_guidelines(
            "I'm tired and can't focus",
            &ctx,
        );

        // Higher priority guidelines should come first
        if matches.len() >= 2 {
            assert!(matches[0].guideline.priority >= matches[1].guideline.priority);
        }
    }

    #[test]
    fn test_contextual_guidelines_match() {
        let orchestrator = GuidelineOrchestrator::new();

        // Test with high productivity context
        let high_ctx = UserContext {
            total_focus_hours_today: 5.0,
            sessions_completed_today: 8,
            current_streak_days: 10,
            top_trigger: Some("email".into()),
            average_session_minutes: 30,
        };

        let matches = orchestrator.evaluate_guidelines("Hello", &high_ctx);

        // Should match high productivity and strong streak guidelines
        assert!(matches.iter().any(|m| m.guideline.id == "high_productivity_day"));
        assert!(matches.iter().any(|m| m.guideline.id == "strong_streak"));
    }

    #[test]
    fn test_build_suggestions() {
        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        let matches = orchestrator.evaluate_guidelines("How can I focus?", &ctx);
        let suggestions = build_suggestions_from_guidelines(&matches, &ctx);

        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn test_empty_message_uses_context() {
        let orchestrator = GuidelineOrchestrator::new();

        let new_user_ctx = UserContext {
            total_focus_hours_today: 0.0,
            sessions_completed_today: 0,
            current_streak_days: 0,
            top_trigger: None,
            average_session_minutes: 25,
        };

        let matches = orchestrator.evaluate_guidelines("", &new_user_ctx);

        // Should still match contextual guidelines like new_user_encouragement
        assert!(matches.iter().any(|m| m.guideline.id == "new_user_encouragement"));
    }

    #[test]
    fn test_system_prompts_integration() {
        use crate::ai::system_prompts::{CORE_IDENTITY, FRAMEWORK_KNOWLEDGE, RESPONSE_GUIDELINES};

        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        let prompt = orchestrator.build_dynamic_prompt("Hello", &ctx, &[]);

        // Verify that the prompt contains the core components from system_prompts
        assert!(prompt.contains("FocusFlow AI Coach"), "Should contain CORE_IDENTITY");
        assert!(prompt.contains("INDISTRACTABLE"), "Should contain FRAMEWORK_KNOWLEDGE");
        assert!(prompt.contains("RESPONSE STRUCTURE"), "Should contain RESPONSE_GUIDELINES");
        assert!(prompt.contains("USER CONTEXT"), "Should contain user context section");

        // Verify user-specific data is included
        assert!(prompt.contains("3 sessions"), "Should include sessions_completed");
        assert!(prompt.contains("5 days"), "Should include streak_days");
        assert!(prompt.contains("social media"), "Should include top_trigger");
    }

    #[test]
    fn test_tool_instructions_inclusion() {
        use crate::ai::system_prompts::TOOL_USAGE_INSTRUCTIONS;

        let orchestrator = GuidelineOrchestrator::new();
        let ctx = mock_context();

        // With tools enabled (default)
        let prompt_with_tools = orchestrator.build_dynamic_prompt("Help me focus", &ctx, &[]);
        assert!(prompt_with_tools.contains("[[start_focus_session]]"),
                "Should include tool instructions by default");

        // With tools disabled
        let prompt_without_tools = orchestrator.build_dynamic_prompt_with_options(
            "Help me focus", &ctx, &[], false
        );
        assert!(!prompt_without_tools.contains("[[start_focus_session]]"),
                "Should not include tool instructions when disabled");
    }
}
