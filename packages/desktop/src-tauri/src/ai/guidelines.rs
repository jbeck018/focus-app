// ai/guidelines.rs - Core guideline types and matching logic for Parlant-style orchestration
//
// This module implements dynamic guideline matching where ALL guidelines are evaluated
// at each user turn, and only relevant ones are loaded into context. This enables
// coherent responses across multiple topics without routing between specialized agents.

use crate::commands::coach::UserContext;
use chrono::{Timelike, Utc};
use serde::{Deserialize, Serialize};

/// A guideline that shapes the AI coach's behavior when activated
///
/// Guidelines are dynamically matched against user input and context,
/// allowing multiple guidelines to activate simultaneously for multi-topic responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guideline {
    /// Unique identifier for this guideline
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// When this guideline should activate
    pub condition: GuidelineCondition,
    /// What the agent should do when this guideline is active (instruction text)
    pub action: String,
    /// Priority for ordering when multiple guidelines match (higher = more important)
    pub priority: i32,
    /// Associated tools/capabilities this guideline enables
    pub tools: Vec<String>,
    /// Additional context to inject into the prompt
    pub context_injection: Option<String>,
    /// Category for grouping related guidelines
    pub category: GuidelineCategory,
}

/// Categories for organizing guidelines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)] // All variants used by guideline system
pub enum GuidelineCategory {
    /// Core coaching interactions
    Coaching,
    /// Session-related (pre, during, post)
    Session,
    /// Analytics and pattern analysis
    Analytics,
    /// Navigation and help
    Navigation,
    /// Time-based tips and reminders
    Contextual,
    /// Emotional support and motivation
    Motivation,
}

/// Conditions that determine when a guideline activates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuidelineCondition {
    /// Matches if any of the keywords are present in the message
    KeywordMatch { keywords: Vec<String> },

    /// Matches based on detected intent (semantic matching)
    IntentMatch { intent: String },

    /// Matches based on context field values
    ContextMatch { field: String, operator: MatchOperator, value: String },

    /// Matches during specific time windows (24-hour format)
    TimeOfDay { start_hour: u32, end_hour: u32 },

    /// Matches based on user state
    UserState { state_key: String, expected: String },

    /// Matches based on numeric context values
    NumericCondition { field: String, operator: NumericOperator, value: f32 },

    /// Composite condition combining multiple conditions
    Composite { conditions: Vec<GuidelineCondition>, operator: LogicOperator },

    /// Always matches (for base guidelines)
    Always,

    /// Never matches (for disabled guidelines)
    Never,
}

/// Operators for string matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
    IsNone,
    IsSome,
}

/// Operators for numeric comparisons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericOperator {
    Equals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Logic operators for composite conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicOperator {
    And,
    Or,
    Not,
}

/// Result of evaluating a guideline against context
#[derive(Debug, Clone)]
pub struct GuidelineMatch {
    pub guideline: Guideline,
    pub confidence: f32,  // 0.0 to 1.0
    pub matched_keywords: Vec<String>,
}

impl Guideline {
    /// Create a new guideline with the given parameters
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        condition: GuidelineCondition,
        action: impl Into<String>,
        priority: i32,
        category: GuidelineCategory,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            condition,
            action: action.into(),
            priority,
            tools: Vec::new(),
            context_injection: None,
            category,
        }
    }

    /// Add tools to this guideline
    #[allow(dead_code)] // Public API method
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    /// Add context injection to this guideline
    #[allow(dead_code)] // Public API method
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context_injection = Some(context.into());
        self
    }

    /// Evaluate this guideline against the given message and context
    pub fn evaluate(&self, message: &str, context: &UserContext) -> Option<GuidelineMatch> {
        let (matches, confidence, matched_keywords) =
            evaluate_condition(&self.condition, message, context);

        if matches {
            Some(GuidelineMatch {
                guideline: self.clone(),
                confidence,
                matched_keywords,
            })
        } else {
            None
        }
    }
}

/// Evaluate a condition against message and context
fn evaluate_condition(
    condition: &GuidelineCondition,
    message: &str,
    context: &UserContext,
) -> (bool, f32, Vec<String>) {
    match condition {
        GuidelineCondition::KeywordMatch { keywords } => {
            let msg_lower = message.to_lowercase();
            let matched: Vec<String> = keywords
                .iter()
                .filter(|kw| msg_lower.contains(&kw.to_lowercase()))
                .cloned()
                .collect();

            if matched.is_empty() {
                (false, 0.0, vec![])
            } else {
                // Any keyword match gives high confidence (0.8 base + boost for more matches)
                // This ensures keyword-based guidelines activate reliably
                let base_confidence = 0.8;
                let match_boost = (matched.len() as f32 - 1.0) * 0.05; // Small boost for multiple matches
                let confidence = (base_confidence + match_boost).min(1.0);
                (true, confidence, matched)
            }
        }

        GuidelineCondition::IntentMatch { intent } => {
            // Simple intent matching based on semantic similarity keywords
            // In production, this could use embeddings or a classifier
            let intent_keywords = get_intent_keywords(intent);
            let msg_lower = message.to_lowercase();
            let matches: Vec<String> = intent_keywords
                .iter()
                .filter(|kw| msg_lower.contains(&kw.to_lowercase()))
                .cloned()
                .collect();

            if matches.is_empty() {
                (false, 0.0, vec![])
            } else {
                let confidence = (matches.len() as f32 / intent_keywords.len() as f32).min(1.0);
                (true, confidence, matches)
            }
        }

        GuidelineCondition::ContextMatch { field, operator, value } => {
            let field_value = get_context_field(context, field);
            let matches = match operator {
                MatchOperator::Equals => field_value.as_deref() == Some(value.as_str()),
                MatchOperator::Contains => field_value
                    .as_ref()
                    .map(|v| v.contains(value))
                    .unwrap_or(false),
                MatchOperator::StartsWith => field_value
                    .as_ref()
                    .map(|v| v.starts_with(value))
                    .unwrap_or(false),
                MatchOperator::EndsWith => field_value
                    .as_ref()
                    .map(|v| v.ends_with(value))
                    .unwrap_or(false),
                MatchOperator::IsNone => field_value.is_none(),
                MatchOperator::IsSome => field_value.is_some(),
            };
            (matches, if matches { 1.0 } else { 0.0 }, vec![])
        }

        GuidelineCondition::TimeOfDay { start_hour, end_hour } => {
            let current_hour = Utc::now().hour();
            let matches = if start_hour <= end_hour {
                current_hour >= *start_hour && current_hour < *end_hour
            } else {
                // Handle overnight ranges (e.g., 22-6)
                current_hour >= *start_hour || current_hour < *end_hour
            };
            (matches, if matches { 1.0 } else { 0.0 }, vec![])
        }

        GuidelineCondition::UserState { state_key, expected } => {
            let current_value = get_user_state(context, state_key);
            let matches = current_value.as_deref() == Some(expected.as_str());
            (matches, if matches { 1.0 } else { 0.0 }, vec![])
        }

        GuidelineCondition::NumericCondition { field, operator, value } => {
            let field_value = get_numeric_field(context, field);
            let matches = field_value
                .map(|fv| match operator {
                    NumericOperator::Equals => (fv - value).abs() < f32::EPSILON,
                    NumericOperator::GreaterThan => fv > *value,
                    NumericOperator::GreaterThanOrEqual => fv >= *value,
                    NumericOperator::LessThan => fv < *value,
                    NumericOperator::LessThanOrEqual => fv <= *value,
                })
                .unwrap_or(false);
            (matches, if matches { 1.0 } else { 0.0 }, vec![])
        }

        GuidelineCondition::Composite { conditions, operator } => {
            let results: Vec<_> = conditions
                .iter()
                .map(|c| evaluate_condition(c, message, context))
                .collect();

            match operator {
                LogicOperator::And => {
                    let all_match = results.iter().all(|(m, _, _)| *m);
                    if all_match {
                        let avg_confidence = results.iter().map(|(_, c, _)| c).sum::<f32>()
                            / results.len() as f32;
                        let all_keywords: Vec<String> = results
                            .into_iter()
                            .flat_map(|(_, _, kw)| kw)
                            .collect();
                        (true, avg_confidence, all_keywords)
                    } else {
                        (false, 0.0, vec![])
                    }
                }
                LogicOperator::Or => {
                    let any_match = results.iter().any(|(m, _, _)| *m);
                    if any_match {
                        let max_confidence = results
                            .iter()
                            .filter(|(m, _, _)| *m)
                            .map(|(_, c, _)| *c)
                            .fold(0.0_f32, |a, b| a.max(b));
                        let all_keywords: Vec<String> = results
                            .into_iter()
                            .filter(|(m, _, _)| *m)
                            .flat_map(|(_, _, kw)| kw)
                            .collect();
                        (true, max_confidence, all_keywords)
                    } else {
                        (false, 0.0, vec![])
                    }
                }
                LogicOperator::Not => {
                    // NOT applies to the first condition only
                    if let Some((matches, _, _)) = results.first() {
                        (!matches, if !matches { 1.0 } else { 0.0 }, vec![])
                    } else {
                        (true, 1.0, vec![])
                    }
                }
            }
        }

        GuidelineCondition::Always => (true, 1.0, vec![]),
        GuidelineCondition::Never => (false, 0.0, vec![]),
    }
}

/// Get keywords associated with an intent for simple semantic matching
fn get_intent_keywords(intent: &str) -> Vec<String> {
    match intent {
        "focus_planning" => vec![
            "focus".into(), "concentrate".into(), "session".into(),
            "work".into(), "start".into(), "begin".into(), "plan".into(),
        ],
        "distraction_analysis" => vec![
            "distract".into(), "interrupt".into(), "trigger".into(),
            "pattern".into(), "why".into(), "problem".into(),
        ],
        "motivation_support" => vec![
            "motivation".into(), "tired".into(), "exhausted".into(),
            "can't".into(), "hard".into(), "difficult".into(), "give up".into(),
            "struggle".into(), "unmotivated".into(),
        ],
        "progress_inquiry" => vec![
            "progress".into(), "how am i".into(), "doing".into(),
            "streak".into(), "stats".into(), "analytics".into(),
        ],
        "help_navigation" => vec![
            "help".into(), "how".into(), "what".into(), "can you".into(),
            "feature".into(), "explain".into(),
        ],
        "session_reflection" => vec![
            "reflect".into(), "review".into(), "went".into(),
            "finished".into(), "completed".into(), "done".into(),
        ],
        _ => vec![intent.to_lowercase()],
    }
}

/// Get a string field value from user context
#[allow(dead_code)] // Used by evaluate_condition for ContextMatch
fn get_context_field(context: &UserContext, field: &str) -> Option<String> {
    match field {
        "top_trigger" => context.top_trigger.clone(),
        _ => None,
    }
}

/// Get a numeric field value from user context
#[allow(dead_code)] // Used by evaluate_condition for NumericCondition
fn get_numeric_field(context: &UserContext, field: &str) -> Option<f32> {
    match field {
        "total_focus_hours_today" => Some(context.total_focus_hours_today),
        "sessions_completed_today" => Some(context.sessions_completed_today as f32),
        "current_streak_days" => Some(context.current_streak_days as f32),
        "average_session_minutes" => Some(context.average_session_minutes as f32),
        _ => None,
    }
}

/// Get derived user state from context
#[allow(dead_code)] // Used by evaluate_condition for UserState
fn get_user_state(context: &UserContext, state_key: &str) -> Option<String> {
    match state_key {
        "has_sessions_today" => Some(if context.sessions_completed_today > 0 { "true" } else { "false" }.into()),
        "has_streak" => Some(if context.current_streak_days > 0 { "true" } else { "false" }.into()),
        "has_trigger_identified" => Some(if context.top_trigger.is_some() { "true" } else { "false" }.into()),
        "productivity_level" => {
            if context.total_focus_hours_today > 4.0 {
                Some("high".into())
            } else if context.total_focus_hours_today > 2.0 {
                Some("medium".into())
            } else if context.sessions_completed_today > 0 {
                Some("low".into())
            } else {
                Some("none".into())
            }
        }
        _ => None,
    }
}

impl GuidelineMatch {
    /// Get the combined action text with any context injections
    #[allow(dead_code)] // Public API method
    pub fn get_action_with_context(&self) -> String {
        match &self.guideline.context_injection {
            Some(ctx) => format!("{}\n\nAdditional Context:\n{}", self.guideline.action, ctx),
            None => self.guideline.action.clone(),
        }
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
    fn test_keyword_match() {
        let guideline = Guideline::new(
            "test",
            "Test Guideline",
            GuidelineCondition::KeywordMatch {
                keywords: vec!["focus".into(), "concentrate".into()],
            },
            "Help with focus",
            10,
            GuidelineCategory::Coaching,
        );

        let ctx = mock_context();
        let result = guideline.evaluate("How can I focus better?", &ctx);
        assert!(result.is_some());
        assert!(result.unwrap().matched_keywords.contains(&"focus".to_string()));

        let result = guideline.evaluate("What's the weather?", &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_numeric_condition() {
        let guideline = Guideline::new(
            "high_achiever",
            "High Achiever",
            GuidelineCondition::NumericCondition {
                field: "total_focus_hours_today".into(),
                operator: NumericOperator::GreaterThan,
                value: 2.0,
            },
            "Celebrate achievements",
            10,
            GuidelineCategory::Motivation,
        );

        let ctx = mock_context();
        let result = guideline.evaluate("How am I doing?", &ctx);
        assert!(result.is_some());

        let low_ctx = UserContext {
            total_focus_hours_today: 1.0,
            ..mock_context()
        };
        let result = guideline.evaluate("How am I doing?", &low_ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_composite_and() {
        let guideline = Guideline::new(
            "focused_with_trigger",
            "Focused with Trigger",
            GuidelineCondition::Composite {
                conditions: vec![
                    GuidelineCondition::KeywordMatch {
                        keywords: vec!["focus".into()],
                    },
                    GuidelineCondition::ContextMatch {
                        field: "top_trigger".into(),
                        operator: MatchOperator::IsSome,
                        value: String::new(),
                    },
                ],
                operator: LogicOperator::And,
            },
            "Address focus with trigger awareness",
            10,
            GuidelineCategory::Coaching,
        );

        let ctx = mock_context();
        let result = guideline.evaluate("Help me focus", &ctx);
        assert!(result.is_some());

        let no_trigger_ctx = UserContext {
            top_trigger: None,
            ..mock_context()
        };
        let result = guideline.evaluate("Help me focus", &no_trigger_ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_user_state() {
        let guideline = Guideline::new(
            "new_user",
            "New User",
            GuidelineCondition::UserState {
                state_key: "productivity_level".into(),
                expected: "none".into(),
            },
            "Encourage first session",
            10,
            GuidelineCategory::Motivation,
        );

        let new_ctx = UserContext {
            total_focus_hours_today: 0.0,
            sessions_completed_today: 0,
            current_streak_days: 0,
            top_trigger: None,
            average_session_minutes: 25,
        };

        let result = guideline.evaluate("Hello", &new_ctx);
        assert!(result.is_some());

        let active_ctx = mock_context();
        let result = guideline.evaluate("Hello", &active_ctx);
        assert!(result.is_none());
    }
}
