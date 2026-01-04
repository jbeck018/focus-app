// ai/tools.rs - Tool definition, registration, and orchestration
//
// This module provides a LangGraph-inspired tool system that allows the AI coach
// to take real actions in the app, such as starting sessions, logging triggers,
// and retrieving statistics.
//
// Design Goals:
// - Tools are defined declaratively with metadata for LLM prompting
// - Tools are executed safely with proper state access
// - Results are formatted for both LLM consumption and user display
// - The system supports an agentic loop (LLM -> tool -> result -> LLM)

use crate::{AppState, Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Core Types
// ============================================================================

/// The result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool executed successfully
    pub success: bool,
    /// Human-readable result message
    pub message: String,
    /// Structured data returned by the tool (for programmatic use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Error details if the tool failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result with a message
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            error: None,
        }
    }

    /// Create a successful result with message and data
    pub fn success_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(message: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            error: Some(error.into()),
        }
    }

    /// Format result for LLM consumption
    pub fn to_llm_format(&self) -> String {
        if self.success {
            if let Some(ref data) = self.data {
                format!(
                    "Tool executed successfully.\nResult: {}\nData: {}",
                    self.message,
                    serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string())
                )
            } else {
                format!("Tool executed successfully.\nResult: {}", self.message)
            }
        } else {
            format!(
                "Tool execution failed.\nMessage: {}\nError: {}",
                self.message,
                self.error.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

/// Parameter type for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    /// Array of a specific type
    Array(Box<ParameterType>),
    /// Optional parameter
    Optional(Box<ParameterType>),
}

impl ParameterType {
    /// Get the type name for display
    pub fn type_name(&self) -> String {
        match self {
            ParameterType::String => "string".to_string(),
            ParameterType::Integer => "integer".to_string(),
            ParameterType::Float => "float".to_string(),
            ParameterType::Boolean => "boolean".to_string(),
            ParameterType::Array(inner) => format!("array<{}>", inner.type_name()),
            ParameterType::Optional(inner) => format!("{}?", inner.type_name()),
        }
    }
}

/// A parameter definition for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name (used in the call)
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Human-readable description
    pub description: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Example values for the LLM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
}

impl ToolParameter {
    /// Create a required string parameter
    #[allow(dead_code)] // Public API method - used by tool registration
    pub fn required_string(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::String,
            description: description.into(),
            required: true,
            default: None,
            examples: None,
        }
    }

    /// Create an optional string parameter
    pub fn optional_string(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::Optional(Box::new(ParameterType::String)),
            description: description.into(),
            required: false,
            default: None,
            examples: None,
        }
    }

    /// Create a required integer parameter
    pub fn required_int(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::Integer,
            description: description.into(),
            required: true,
            default: None,
            examples: None,
        }
    }

    /// Create an optional integer parameter with default
    pub fn optional_int(
        name: impl Into<String>,
        description: impl Into<String>,
        default: i64,
    ) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::Optional(Box::new(ParameterType::Integer)),
            description: description.into(),
            required: false,
            default: Some(serde_json::json!(default)),
            examples: None,
        }
    }

    /// Create a required boolean parameter
    #[allow(dead_code)] // Public API method - may be used for future tools
    pub fn required_bool(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::Boolean,
            description: description.into(),
            required: true,
            default: None,
            examples: None,
        }
    }

    /// Create an optional boolean parameter with default
    pub fn optional_bool(
        name: impl Into<String>,
        description: impl Into<String>,
        default: bool,
    ) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::Optional(Box::new(ParameterType::Boolean)),
            description: description.into(),
            required: false,
            default: Some(serde_json::json!(default)),
            examples: None,
        }
    }

    /// Add examples to the parameter
    pub fn with_examples(mut self, examples: Vec<&str>) -> Self {
        self.examples = Some(examples.into_iter().map(String::from).collect());
        self
    }
}

/// Trait for tool handlers
///
/// Implementations define the actual logic for each tool.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with the given parameters
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult>;
}

/// A complete tool definition
pub struct Tool {
    /// Unique tool name (used in calls)
    pub name: String,
    /// Human-readable description for the LLM
    pub description: String,
    /// Category for organization
    pub category: ToolCategory,
    /// Parameter definitions
    pub parameters: Vec<ToolParameter>,
    /// The handler that executes this tool
    pub handler: Arc<dyn ToolHandler>,
    /// Example usage for the LLM
    pub examples: Vec<ToolExample>,
}

impl Tool {
    /// Create a new tool
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: ToolCategory,
        handler: impl ToolHandler + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category,
            parameters: Vec::new(),
            handler: Arc::new(handler),
            examples: Vec::new(),
        }
    }

    /// Add parameters to the tool
    pub fn with_parameters(mut self, params: Vec<ToolParameter>) -> Self {
        self.parameters = params;
        self
    }

    /// Add examples to the tool
    pub fn with_examples(mut self, examples: Vec<ToolExample>) -> Self {
        self.examples = examples;
        self
    }

    /// Generate documentation for this tool (for LLM system prompt)
    pub fn to_documentation(&self) -> String {
        let mut doc = format!("### {}\n", self.name);
        doc.push_str(&format!("**Description:** {}\n", self.description));
        doc.push_str(&format!("**Category:** {:?}\n", self.category));

        if !self.parameters.is_empty() {
            doc.push_str("**Parameters:**\n");
            for param in &self.parameters {
                let required = if param.required { " (required)" } else { " (optional)" };
                doc.push_str(&format!(
                    "  - `{}` ({}){}: {}\n",
                    param.name,
                    param.param_type.type_name(),
                    required,
                    param.description
                ));
                if let Some(ref default) = param.default {
                    doc.push_str(&format!("    Default: {}\n", default));
                }
                if let Some(ref examples) = param.examples {
                    doc.push_str(&format!("    Examples: {}\n", examples.join(", ")));
                }
            }
        }

        if !self.examples.is_empty() {
            doc.push_str("**Examples:**\n");
            for example in &self.examples {
                doc.push_str(&format!("  - {}\n", example.description));
                doc.push_str(&format!("    Call: `{}`\n", example.call));
            }
        }

        doc
    }
}

/// Tool category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// Session management tools
    Session,
    /// Analytics and statistics tools
    Analytics,
    /// Journal and trigger logging tools
    Journal,
    /// Blocking management tools
    Blocking,
    /// Streak and goal management tools
    Goals,
}

/// Example usage of a tool
#[derive(Debug, Clone)]
pub struct ToolExample {
    /// Description of what this example does
    pub description: String,
    /// The tool call syntax
    pub call: String,
}

impl ToolExample {
    pub fn new(description: impl Into<String>, call: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            call: call.into(),
        }
    }
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Registry for all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// Get all tool names
    ///
    /// Public API method
    #[allow(dead_code)]
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get all tools in a category
    ///
    /// Public API method - used by generate_documentation
    pub fn tools_in_category(&self, category: ToolCategory) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Generate documentation for all tools (for LLM system prompt)
    pub fn generate_documentation(&self) -> String {
        let mut doc = String::from("# Tool Reference\n\n");
        doc.push_str("Format: <tool name=\"NAME\" param=\"value\"/>\n\n");
        doc.push_str("RULES: One tool max per response. No code blocks. No repeating.\n\n");

        // Group by category
        let categories = [
            (ToolCategory::Session, "Session Management"),
            (ToolCategory::Analytics, "Analytics & Statistics"),
            (ToolCategory::Journal, "Journal & Triggers"),
            (ToolCategory::Blocking, "Blocking Management"),
            (ToolCategory::Goals, "Goals & Streaks"),
        ];

        for (category, category_name) in categories {
            let tools = self.tools_in_category(category);
            if !tools.is_empty() {
                doc.push_str(&format!("## {}\n\n", category_name));
                for tool in tools {
                    doc.push_str(&tool.to_documentation());
                    doc.push('\n');
                }
            }
        }

        doc
    }

    /// Create a registry with all default tools registered
    pub fn with_default_tools() -> Self {
        let mut registry = Self::new();
        register_all_tools(&mut registry);
        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_default_tools()
    }
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// Handler for starting a focus session
pub struct StartFocusSessionHandler;

#[async_trait]
impl ToolHandler for StartFocusSessionHandler {
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        // Extract parameters
        let duration = params
            .get("duration")
            .and_then(|v| v.as_i64())
            .unwrap_or(25) as i32;

        // Check if session already active
        {
            let active = state.active_session.read().await;
            if active.is_some() {
                return Ok(ToolResult::failure(
                    "Cannot start session",
                    "A focus session is already active. End the current session first.",
                ));
            }
        }

        // Import the session types we need
        use crate::state::{ActiveSession, SessionType, TimerState};
        use crate::db::queries;

        // Check session limits for free tier
        {
            let auth_state = state.auth_state.read().await;
            let subscription_tier = auth_state
                .user
                .as_ref()
                .map(|u| u.subscription_tier.as_str())
                .unwrap_or("free");

            if subscription_tier == "free" || subscription_tier.is_empty() {
                let sessions_today = queries::count_todays_sessions(state.pool()).await?;
                if sessions_today >= 3 {
                    return Ok(ToolResult::failure(
                        "Session limit reached",
                        "Daily session limit reached (3/3). Upgrade to Pro for unlimited sessions.",
                    ));
                }
            }
        }

        // Get blocked items from database
        let blocked_items = queries::get_blocked_items(state.pool(), None).await?;
        let blocked_apps: Vec<String> = blocked_items
            .iter()
            .filter(|i| i.item_type == "app")
            .map(|i| i.value.clone())
            .collect();
        let blocked_websites: Vec<String> = blocked_items
            .iter()
            .filter(|i| i.item_type == "website")
            .map(|i| i.value.clone())
            .collect();

        // Create new session
        let session = ActiveSession::new(
            duration,
            SessionType::Focus,
            blocked_apps.clone(),
            blocked_websites.clone(),
        );

        // Insert into database
        queries::insert_session(
            state.pool(),
            &session.id,
            session.start_time,
            session.planned_duration_minutes,
            "focus",
        )
        .await?;

        // Enable blocking
        {
            let mut blocking = state.blocking_state.write().await;
            blocking.enable();
            blocking.update_blocked_websites(blocked_websites.clone());
        }

        let session_id = session.id.clone();

        // Set active session
        {
            let mut active = state.active_session.write().await;
            *active = Some(session);
        }

        // Initialize timer state
        {
            let mut timer_state = state.timer_state.write().await;
            *timer_state = TimerState::new_running();
        }

        // Start timer loop
        crate::commands::timer::start_timer_loop(state.app_handle.clone(), state.clone());

        Ok(ToolResult::success_with_data(
            format!(
                "Started a {}-minute focus session. Stay focused! Blocking {} apps and {} websites.",
                duration,
                blocked_apps.len(),
                blocked_websites.len()
            ),
            serde_json::json!({
                "session_id": session_id,
                "duration_minutes": duration,
                "blocked_apps_count": blocked_apps.len(),
                "blocked_websites_count": blocked_websites.len(),
            }),
        ))
    }
}

/// Handler for ending a focus session
pub struct EndFocusSessionHandler;

#[async_trait]
impl ToolHandler for EndFocusSessionHandler {
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let completed = params
            .get("completed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Get and remove active session
        let session = {
            let mut active = state.active_session.write().await;
            match active.take() {
                Some(s) => s,
                None => {
                    return Ok(ToolResult::failure(
                        "No active session",
                        "There is no active focus session to end.",
                    ));
                }
            }
        };

        use crate::db::queries;
        use crate::state::TimerState;

        // Stop timer
        {
            let mut timer_state = state.timer_state.write().await;
            timer_state.stop();
            *timer_state = TimerState::default();
        }

        let end_time = chrono::Utc::now();
        let duration_seconds = session.elapsed_seconds();
        let duration_minutes = duration_seconds / 60;

        // Update database
        queries::end_session(state.pool(), &session.id, end_time, completed).await?;

        // Disable blocking
        {
            let mut blocking = state.blocking_state.write().await;
            blocking.disable();
            blocking.update_blocked_websites(Vec::new());
        }

        // Update analytics
        let date = end_time.format("%Y-%m-%d").to_string();
        queries::upsert_daily_analytics(
            state.pool(),
            &date,
            duration_seconds,
            0,
            if completed { 1 } else { 0 },
            if completed { 0 } else { 1 },
        )
        .await?;

        let message = if completed {
            format!(
                "Completed {} minute focus session. Great work!",
                duration_minutes
            )
        } else {
            format!(
                "Ended session after {} minutes. Every bit of focus counts!",
                duration_minutes
            )
        };

        Ok(ToolResult::success_with_data(
            message,
            serde_json::json!({
                "session_id": session.id,
                "duration_minutes": duration_minutes,
                "completed": completed,
            }),
        ))
    }
}

/// Handler for getting session statistics
pub struct GetSessionStatsHandler;

#[async_trait]
impl ToolHandler for GetSessionStatsHandler {
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let period = params
            .get("period")
            .and_then(|v| v.as_str())
            .unwrap_or("today");

        use crate::db::queries;

        match period {
            "today" => {
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let analytics = queries::get_daily_analytics(state.pool(), &today).await?;

                let (focus_minutes, sessions_completed, sessions_abandoned) = analytics
                    .map(|a| {
                        (
                            a.total_focus_seconds / 60,
                            a.sessions_completed,
                            a.sessions_abandoned,
                        )
                    })
                    .unwrap_or((0, 0, 0));

                Ok(ToolResult::success_with_data(
                    format!(
                        "Today: {} minutes of focus across {} sessions ({} completed, {} ended early).",
                        focus_minutes,
                        sessions_completed + sessions_abandoned,
                        sessions_completed,
                        sessions_abandoned
                    ),
                    serde_json::json!({
                        "period": "today",
                        "focus_minutes": focus_minutes,
                        "sessions_completed": sessions_completed,
                        "sessions_abandoned": sessions_abandoned,
                    }),
                ))
            }
            "week" => {
                let end_date = chrono::Utc::now();
                let start_date = end_date - chrono::Duration::days(6);
                let start_str = start_date.format("%Y-%m-%d").to_string();
                let end_str = end_date.format("%Y-%m-%d").to_string();

                let analytics =
                    queries::get_analytics_range(state.pool(), &start_str, &end_str).await?;

                let total_focus_minutes: i64 =
                    analytics.iter().map(|a| a.total_focus_seconds / 60).sum();
                let total_completed: i64 = analytics.iter().map(|a| a.sessions_completed).sum();
                let total_abandoned: i64 = analytics.iter().map(|a| a.sessions_abandoned).sum();
                let active_days = analytics.len();

                Ok(ToolResult::success_with_data(
                    format!(
                        "This week: {} hours {} minutes of focus across {} sessions over {} days.",
                        total_focus_minutes / 60,
                        total_focus_minutes % 60,
                        total_completed + total_abandoned,
                        active_days
                    ),
                    serde_json::json!({
                        "period": "week",
                        "focus_hours": total_focus_minutes / 60,
                        "focus_minutes": total_focus_minutes % 60,
                        "sessions_completed": total_completed,
                        "sessions_abandoned": total_abandoned,
                        "active_days": active_days,
                    }),
                ))
            }
            _ => Ok(ToolResult::failure(
                "Invalid period",
                "Period must be 'today' or 'week'.",
            )),
        }
    }
}

/// Handler for logging a trigger
pub struct LogTriggerHandler;

#[async_trait]
impl ToolHandler for LogTriggerHandler {
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let trigger_type = params
            .get("trigger_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidInput("trigger_type is required".into()))?;

        let notes = params.get("notes").and_then(|v| v.as_str());
        let intensity = params.get("intensity").and_then(|v| v.as_i64()).map(|i| i as i32);
        let emotion = params.get("emotion").and_then(|v| v.as_str());

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Get active session ID if any
        let session_id = {
            let active = state.active_session.read().await;
            active.as_ref().map(|s| s.id.clone())
        };

        // Get device_id from settings
        let device_id: Option<String> = sqlx::query_scalar(
            "SELECT value FROM user_settings WHERE key = 'device_id'"
        )
        .fetch_optional(state.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO journal_entries (id, session_id, trigger_type, emotion, notes, intensity, created_at, device_id, last_modified)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&id)
        .bind(&session_id)
        .bind(trigger_type)
        .bind(emotion)
        .bind(notes)
        .bind(intensity)
        .bind(&now)
        .bind(&device_id)
        .bind(&now)
        .execute(state.pool())
        .await?;

        let context = if session_id.is_some() {
            "during your focus session"
        } else {
            "outside of a session"
        };

        Ok(ToolResult::success_with_data(
            format!(
                "Logged '{}' trigger {}. Tracking these patterns helps build awareness.",
                trigger_type, context
            ),
            serde_json::json!({
                "id": id,
                "trigger_type": trigger_type,
                "session_id": session_id,
                "emotion": emotion,
                "intensity": intensity,
            }),
        ))
    }
}

/// Handler for getting trigger patterns
pub struct GetTriggerPatternsHandler;

#[async_trait]
impl ToolHandler for GetTriggerPatternsHandler {
    async fn execute(&self, _params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        // Get frequency by trigger type
        let insights: Vec<(String, i32)> = sqlx::query_as(
            r#"
            SELECT trigger_type, COUNT(*) as frequency
            FROM journal_entries
            WHERE deleted = 0 AND created_at >= datetime('now', '-30 days')
            GROUP BY trigger_type
            ORDER BY frequency DESC
            LIMIT 5
            "#,
        )
        .fetch_all(state.pool())
        .await?;

        if insights.is_empty() {
            return Ok(ToolResult::success(
                "No trigger data in the last 30 days. Start logging triggers to identify patterns.",
            ));
        }

        let mut message = String::from("Trigger patterns (last 30 days):\n");
        let mut data = Vec::new();

        for (trigger_type, frequency) in &insights {
            message.push_str(&format!("- {}: {} times\n", trigger_type, frequency));
            data.push(serde_json::json!({
                "trigger_type": trigger_type,
                "frequency": frequency,
            }));
        }

        Ok(ToolResult::success_with_data(message, serde_json::json!(data)))
    }
}

/// Handler for getting blocking statistics
pub struct GetBlockingStatsHandler;

#[async_trait]
impl ToolHandler for GetBlockingStatsHandler {
    async fn execute(&self, _params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        use crate::db::queries;

        let items = queries::get_blocked_items(state.pool(), None).await?;

        let apps: Vec<&str> = items
            .iter()
            .filter(|i| i.item_type == "app")
            .map(|i| i.value.as_str())
            .collect();

        let websites: Vec<&str> = items
            .iter()
            .filter(|i| i.item_type == "website")
            .map(|i| i.value.as_str())
            .collect();

        let blocking_enabled = {
            let blocking_state = state.blocking_state.read().await;
            blocking_state.enabled
        };

        let status = if blocking_enabled {
            "active (session in progress)"
        } else {
            "inactive"
        };

        Ok(ToolResult::success_with_data(
            format!(
                "Blocking {}: {} apps and {} websites configured.\nApps: {}\nWebsites: {}",
                status,
                apps.len(),
                websites.len(),
                if apps.is_empty() {
                    "none".to_string()
                } else {
                    apps.join(", ")
                },
                if websites.is_empty() {
                    "none".to_string()
                } else {
                    websites.join(", ")
                },
            ),
            serde_json::json!({
                "enabled": blocking_enabled,
                "apps": apps,
                "websites": websites,
                "apps_count": apps.len(),
                "websites_count": websites.len(),
            }),
        ))
    }
}

/// Handler for adding a blocked item
pub struct AddBlockedItemHandler;

#[async_trait]
impl ToolHandler for AddBlockedItemHandler {
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let item_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidInput("type is required (app or website)".into()))?;

        let value = params
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidInput("value is required".into()))?;

        if item_type != "app" && item_type != "website" {
            return Ok(ToolResult::failure(
                "Invalid type",
                "Type must be 'app' or 'website'.",
            ));
        }

        use crate::db::queries;

        queries::insert_blocked_item(state.pool(), item_type, value).await?;

        Ok(ToolResult::success(format!(
            "Added '{}' to blocked {} list. It will be blocked during focus sessions.",
            value, item_type
        )))
    }
}

/// Handler for getting streak information
pub struct GetStreakInfoHandler;

#[async_trait]
impl ToolHandler for GetStreakInfoHandler {
    async fn execute(&self, _params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let user_id = state.get_user_id().await;

        // Get streak history
        let history: Vec<(String, i32, bool)> = sqlx::query_as(
            r#"
            SELECT date, sessions_count, was_frozen
            FROM streak_history
            WHERE user_id IS NULL OR user_id = ?
            ORDER BY date DESC
            LIMIT 400
            "#,
        )
        .bind(&user_id)
        .fetch_all(state.pool())
        .await?;

        if history.is_empty() {
            return Ok(ToolResult::success_with_data(
                "No streak data yet. Complete at least one focus session to start building your streak!",
                serde_json::json!({
                    "current_streak": 0,
                    "longest_streak": 0,
                }),
            ));
        }

        // Calculate current streak
        use chrono::{Duration, Local, NaiveDate};
        let today = Local::now().date_naive();
        let mut current_streak = 0;
        let mut expected_date = today;

        for (date_str, sessions, was_frozen) in &history {
            if let Ok(entry_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                if entry_date == expected_date {
                    if *sessions >= 1 || *was_frozen {
                        current_streak += 1;
                        expected_date -= Duration::days(1);
                    } else {
                        break;
                    }
                } else if entry_date < expected_date {
                    break;
                }
            }
        }

        // Calculate longest streak (simplified)
        let longest_streak = history
            .iter()
            .filter(|(_, sessions, frozen)| *sessions >= 1 || *frozen)
            .count() as i32;

        let message = if current_streak > 0 {
            format!(
                "Current streak: {} days. Longest recorded: {} days. Keep it up!",
                current_streak, longest_streak
            )
        } else {
            format!(
                "No active streak. Your longest was {} days. Start a session to begin a new streak!",
                longest_streak
            )
        };

        Ok(ToolResult::success_with_data(
            message,
            serde_json::json!({
                "current_streak": current_streak,
                "longest_streak": longest_streak,
            }),
        ))
    }
}

/// Handler for setting a focus goal
pub struct SetFocusGoalHandler;

#[async_trait]
impl ToolHandler for SetFocusGoalHandler {
    async fn execute(&self, params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let goal_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("daily");

        let minutes = params
            .get("minutes")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| Error::InvalidInput("minutes is required".into()))? as i32;

        if !(5..=480).contains(&minutes) {
            return Ok(ToolResult::failure(
                "Invalid goal",
                "Focus goal must be between 5 and 480 minutes.",
            ));
        }

        let key = match goal_type {
            "daily" => "focus_goal_daily_minutes",
            "weekly" => "focus_goal_weekly_minutes",
            _ => {
                return Ok(ToolResult::failure(
                    "Invalid goal type",
                    "Goal type must be 'daily' or 'weekly'.",
                ));
            }
        };

        // Upsert the setting
        sqlx::query(
            r#"
            INSERT INTO user_settings (key, value, updated_at)
            VALUES (?, ?, datetime('now'))
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')
            "#,
        )
        .bind(key)
        .bind(minutes.to_string())
        .execute(state.pool())
        .await?;

        let hours = minutes / 60;
        let remaining_mins = minutes % 60;
        let time_str = if hours > 0 {
            format!("{} hour{} {} minutes", hours, if hours > 1 { "s" } else { "" }, remaining_mins)
        } else {
            format!("{} minutes", minutes)
        };

        Ok(ToolResult::success_with_data(
            format!("Set {} focus goal to {}.", goal_type, time_str),
            serde_json::json!({
                "goal_type": goal_type,
                "minutes": minutes,
            }),
        ))
    }
}

/// Handler for getting the current active session
pub struct GetActiveSessionHandler;

#[async_trait]
impl ToolHandler for GetActiveSessionHandler {
    async fn execute(&self, _params: serde_json::Value, state: &AppState) -> Result<ToolResult> {
        let active = state.active_session.read().await;

        match active.as_ref() {
            Some(session) => {
                let elapsed_seconds = session.elapsed_seconds();
                let elapsed_minutes = elapsed_seconds / 60;
                let remaining_minutes = session.planned_duration_minutes as i64 - elapsed_minutes;

                let timer_state = state.timer_state.read().await;
                let status = if timer_state.is_paused {
                    "paused"
                } else {
                    "running"
                };

                Ok(ToolResult::success_with_data(
                    format!(
                        "Active session ({}): {} minutes elapsed, {} minutes remaining.",
                        status, elapsed_minutes, remaining_minutes.max(0)
                    ),
                    serde_json::json!({
                        "session_id": session.id,
                        "status": status,
                        "elapsed_minutes": elapsed_minutes,
                        "remaining_minutes": remaining_minutes.max(0),
                        "planned_duration_minutes": session.planned_duration_minutes,
                        "blocked_apps": session.blocked_apps.len(),
                        "blocked_websites": session.blocked_websites.len(),
                    }),
                ))
            }
            None => Ok(ToolResult::success_with_data(
                "No active focus session. Would you like to start one?",
                serde_json::json!({
                    "has_active_session": false,
                }),
            )),
        }
    }
}

// ============================================================================
// Tool Registration
// ============================================================================

/// Register all default tools with the registry
pub fn register_all_tools(registry: &mut ToolRegistry) {
    // Start focus session
    registry.register(
        Tool::new(
            "start_focus_session",
            "Start a new focus session with optional duration. Activates blocking for configured apps/websites.",
            ToolCategory::Session,
            StartFocusSessionHandler,
        )
        .with_parameters(vec![
            ToolParameter::optional_int("duration", "Session duration in minutes", 25)
                .with_examples(vec!["15", "25", "45", "60"]),
        ])
        .with_examples(vec![
            ToolExample::new(
                "Start a default 25-minute session",
                "<tool name=\"start_focus_session\"/>",
            ),
            ToolExample::new(
                "Start a 45-minute session",
                "<tool name=\"start_focus_session\" duration=\"45\"/>",
            ),
        ]),
    );

    // End focus session
    registry.register(
        Tool::new(
            "end_focus_session",
            "End the current focus session. Can mark as completed or ended early.",
            ToolCategory::Session,
            EndFocusSessionHandler,
        )
        .with_parameters(vec![ToolParameter::optional_bool(
            "completed",
            "Whether the session was completed successfully",
            true,
        )])
        .with_examples(vec![
            ToolExample::new(
                "End session as completed",
                "<tool name=\"end_focus_session\" completed=\"true\"/>",
            ),
            ToolExample::new(
                "End session early",
                "<tool name=\"end_focus_session\" completed=\"false\"/>",
            ),
        ]),
    );

    // Get active session
    registry.register(
        Tool::new(
            "get_active_session",
            "Check if there's an active focus session and get its status.",
            ToolCategory::Session,
            GetActiveSessionHandler,
        )
        .with_examples(vec![ToolExample::new(
            "Check current session",
            "<tool name=\"get_active_session\"/>",
        )]),
    );

    // Get session stats
    registry.register(
        Tool::new(
            "get_session_stats",
            "Get focus session statistics for today or the past week.",
            ToolCategory::Analytics,
            GetSessionStatsHandler,
        )
        .with_parameters(vec![
            ToolParameter::optional_string("period", "Time period: 'today' or 'week'")
                .with_examples(vec!["today", "week"]),
        ])
        .with_examples(vec![
            ToolExample::new("Get today's stats", "<tool name=\"get_session_stats\"/>"),
            ToolExample::new(
                "Get weekly stats",
                "<tool name=\"get_session_stats\" period=\"week\"/>",
            ),
        ]),
    );

    // Log trigger
    registry.register(
        Tool::new(
            "log_trigger",
            "Log a distraction trigger to the journal. Helps identify patterns over time.",
            ToolCategory::Journal,
            LogTriggerHandler,
        )
        .with_parameters(vec![
            ToolParameter::required_string(
                "trigger_type",
                "Type of trigger: boredom, anxiety, stress, fatigue, notification, person, environment, other",
            )
            .with_examples(vec![
                "boredom",
                "anxiety",
                "notification",
                "social_media",
            ]),
            ToolParameter::optional_string("notes", "Additional notes about the trigger"),
            ToolParameter::optional_int("intensity", "Intensity of the urge (1-5)", 3),
            ToolParameter::optional_string(
                "emotion",
                "How you're feeling: frustrated, anxious, tired, distracted, curious, bored, overwhelmed, neutral",
            ),
        ])
        .with_examples(vec![
            ToolExample::new(
                "Log a social media urge",
                "<tool name=\"log_trigger\" trigger_type=\"notification\" notes=\"Saw a phone notification\"/>",
            ),
            ToolExample::new(
                "Log boredom with intensity",
                "<tool name=\"log_trigger\" trigger_type=\"boredom\" intensity=\"4\" emotion=\"restless\"/>",
            ),
        ]),
    );

    // Get trigger patterns
    registry.register(
        Tool::new(
            "get_trigger_patterns",
            "Analyze trigger patterns from the last 30 days to identify common distractions.",
            ToolCategory::Journal,
            GetTriggerPatternsHandler,
        )
        .with_examples(vec![ToolExample::new(
            "View trigger patterns",
            "<tool name=\"get_trigger_patterns\"/>",
        )]),
    );

    // Get blocking stats
    registry.register(
        Tool::new(
            "get_blocking_stats",
            "Get information about currently blocked apps and websites.",
            ToolCategory::Blocking,
            GetBlockingStatsHandler,
        )
        .with_examples(vec![ToolExample::new(
            "Check blocked items",
            "<tool name=\"get_blocking_stats\"/>",
        )]),
    );

    // Add blocked item
    registry.register(
        Tool::new(
            "add_blocked_item",
            "Add an app or website to the block list. Will be blocked during focus sessions.",
            ToolCategory::Blocking,
            AddBlockedItemHandler,
        )
        .with_parameters(vec![
            ToolParameter::required_string("type", "Type of item: 'app' or 'website'")
                .with_examples(vec!["app", "website"]),
            ToolParameter::required_string("value", "App name or website domain to block")
                .with_examples(vec!["twitter.com", "instagram.com", "slack", "discord"]),
        ])
        .with_examples(vec![
            ToolExample::new(
                "Block Twitter",
                "<tool name=\"add_blocked_item\" type=\"website\" value=\"twitter.com\"/>",
            ),
            ToolExample::new(
                "Block Slack app",
                "<tool name=\"add_blocked_item\" type=\"app\" value=\"slack\"/>",
            ),
        ]),
    );

    // Get streak info
    registry.register(
        Tool::new(
            "get_streak_info",
            "Get current streak information and longest streak record.",
            ToolCategory::Goals,
            GetStreakInfoHandler,
        )
        .with_examples(vec![ToolExample::new(
            "Check streak",
            "<tool name=\"get_streak_info\"/>",
        )]),
    );

    // Set focus goal
    registry.register(
        Tool::new(
            "set_focus_goal",
            "Set a daily or weekly focus time goal in minutes.",
            ToolCategory::Goals,
            SetFocusGoalHandler,
        )
        .with_parameters(vec![
            ToolParameter::optional_string("type", "Goal type: 'daily' or 'weekly'")
                .with_examples(vec!["daily", "weekly"]),
            ToolParameter::required_int("minutes", "Goal in minutes"),
        ])
        .with_examples(vec![
            ToolExample::new(
                "Set 2 hour daily goal",
                "<tool name=\"set_focus_goal\" type=\"daily\" minutes=\"120\"/>",
            ),
            ToolExample::new(
                "Set 10 hour weekly goal",
                "<tool name=\"set_focus_goal\" type=\"weekly\" minutes=\"600\"/>",
            ),
        ]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_creation() {
        let success = ToolResult::success("Test message");
        assert!(success.success);
        assert_eq!(success.message, "Test message");
        assert!(success.data.is_none());
        assert!(success.error.is_none());

        let failure = ToolResult::failure("Failed", "Error details");
        assert!(!failure.success);
        assert_eq!(failure.error, Some("Error details".to_string()));
    }

    #[test]
    fn test_tool_result_llm_format() {
        let success = ToolResult::success_with_data(
            "Completed",
            serde_json::json!({"count": 5}),
        );
        let formatted = success.to_llm_format();
        assert!(formatted.contains("successfully"));
        assert!(formatted.contains("count"));
    }

    #[test]
    fn test_parameter_type_name() {
        assert_eq!(ParameterType::String.type_name(), "string");
        assert_eq!(ParameterType::Integer.type_name(), "integer");
        assert_eq!(
            ParameterType::Optional(Box::new(ParameterType::String)).type_name(),
            "string?"
        );
        assert_eq!(
            ParameterType::Array(Box::new(ParameterType::Integer)).type_name(),
            "array<integer>"
        );
    }

    #[test]
    fn test_registry_with_default_tools() {
        let registry = ToolRegistry::with_default_tools();
        assert!(registry.get("start_focus_session").is_some());
        assert!(registry.get("end_focus_session").is_some());
        assert!(registry.get("get_session_stats").is_some());
        assert!(registry.get("log_trigger").is_some());
    }

    #[test]
    fn test_generate_documentation() {
        let registry = ToolRegistry::with_default_tools();
        let docs = registry.generate_documentation();

        assert!(docs.contains("Tool Reference"));
        assert!(docs.contains("start_focus_session"));
        assert!(docs.contains("Session Management"));
    }
}
