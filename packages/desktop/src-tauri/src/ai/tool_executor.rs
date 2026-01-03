// ai/tool_executor.rs - Execute tools and manage the agentic loop
//
// This module provides the execution layer for tools, including:
// - Single tool execution
// - Multi-tool orchestration
// - Agentic loop (LLM -> tool -> result -> LLM)
// - Result formatting for both LLM and user consumption

use crate::ai::tool_parser::{ParseResult, ParsedToolCall, ToolParser};
use crate::ai::tools::{ToolRegistry, ToolResult};
use crate::{AppState, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

// ============================================================================
// Execution Types
// ============================================================================

/// Result of executing one or more tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Results from each tool execution
    pub tool_results: Vec<ToolExecutionRecord>,
    /// Combined summary for the user
    pub summary: String,
    /// Whether all tools executed successfully
    pub all_succeeded: bool,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Record of a single tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRecord {
    /// The tool that was called
    pub tool_name: String,
    /// Parameters passed to the tool
    pub params: serde_json::Value,
    /// The result of execution
    pub result: ToolResult,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

impl ExecutionResult {
    /// Create an empty result
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn empty() -> Self {
        Self {
            tool_results: Vec::new(),
            summary: String::new(),
            all_succeeded: true,
            execution_time_ms: 0,
        }
    }

    /// Format for LLM consumption (to continue the conversation)
    pub fn to_llm_context(&self) -> String {
        if self.tool_results.is_empty() {
            return String::new();
        }

        let mut context = String::from("\n\n--- TOOL EXECUTION RESULTS ---\n");

        for record in &self.tool_results {
            context.push_str(&format!("\n[{}]\n", record.tool_name));
            context.push_str(&record.result.to_llm_format());
            context.push('\n');
        }

        context.push_str("\n--- END TOOL RESULTS ---\n");
        context
    }

    /// Get a combined message for user display
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn user_message(&self) -> String {
        if self.tool_results.is_empty() {
            return self.summary.clone();
        }

        self.tool_results
            .iter()
            .map(|r| r.result.message.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

// ============================================================================
// Tool Executor
// ============================================================================

/// Configuration for the tool executor
#[derive(Debug, Clone)]
#[allow(dead_code)] // All fields used by executor configuration
pub struct ExecutorConfig {
    /// Maximum number of tools to execute in sequence
    pub max_tools_per_turn: usize,
    /// Maximum depth for agentic loops
    pub max_loop_depth: usize,
    /// Timeout for individual tool execution (milliseconds)
    pub tool_timeout_ms: u64,
    /// Whether to continue on tool failure
    pub continue_on_failure: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_tools_per_turn: 5,
            max_loop_depth: 3,
            tool_timeout_ms: 30_000,
            continue_on_failure: true,
        }
    }
}

/// The main tool executor
pub struct ToolExecutor {
    /// Tool registry
    registry: Arc<ToolRegistry>,
    /// Parser for tool calls
    parser: ToolParser,
    /// Configuration
    config: ExecutorConfig,
}

impl ToolExecutor {
    /// Create a new executor with the default tool registry
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ToolRegistry::with_default_tools()),
            parser: ToolParser::new(),
            config: ExecutorConfig::default(),
        }
    }

    /// Create an executor with a custom registry
    #[allow(dead_code)] // Public API method - may be used for testing
    pub fn with_registry(registry: ToolRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
            parser: ToolParser::new(),
            config: ExecutorConfig::default(),
        }
    }

    /// Create an executor with custom configuration
    #[allow(dead_code)] // Public API method - may be used for testing
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self {
            registry: Arc::new(ToolRegistry::with_default_tools()),
            parser: ToolParser::new(),
            config,
        }
    }

    /// Get the tool registry
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Get tool documentation for system prompt
    pub fn get_tool_documentation(&self) -> String {
        self.registry.generate_documentation()
    }

    /// Parse LLM output for tool calls
    pub fn parse_output(&self, text: &str) -> ParseResult {
        self.parser.parse(text)
    }

    /// Execute a single parsed tool call
    pub async fn execute_tool(
        &self,
        call: &ParsedToolCall,
        state: &AppState,
    ) -> Result<ToolExecutionRecord> {
        let start = std::time::Instant::now();

        info!("Executing tool: {} with params: {}", call.name, call.params);

        // Look up the tool
        let tool = self.registry.get(&call.name).ok_or_else(|| {
            Error::NotFound(format!("Unknown tool: {}", call.name))
        })?;

        // Execute the handler
        let result = match tokio::time::timeout(
            std::time::Duration::from_millis(self.config.tool_timeout_ms),
            tool.handler.execute(call.params.clone(), state),
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                error!("Tool {} failed: {}", call.name, e);
                ToolResult::failure(
                    format!("Tool '{}' encountered an error", call.name),
                    e.to_string(),
                )
            }
            Err(_) => {
                warn!("Tool {} timed out", call.name);
                ToolResult::failure(
                    format!("Tool '{}' timed out", call.name),
                    "Execution exceeded time limit",
                )
            }
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        debug!(
            "Tool {} completed in {}ms: success={}",
            call.name, execution_time_ms, result.success
        );

        Ok(ToolExecutionRecord {
            tool_name: call.name.clone(),
            params: call.params.clone(),
            result,
            execution_time_ms,
        })
    }

    /// Execute all tool calls found in LLM output
    pub async fn execute_all(
        &self,
        text: &str,
        state: &AppState,
    ) -> Result<ExecutionResult> {
        let parse_result = self.parse_output(text);

        if !parse_result.has_tools {
            return Ok(ExecutionResult::empty());
        }

        let start = std::time::Instant::now();
        let mut tool_results = Vec::new();
        let mut all_succeeded = true;

        // Limit number of tools
        let tools_to_execute = parse_result
            .tool_calls
            .iter()
            .take(self.config.max_tools_per_turn);

        for call in tools_to_execute {
            match self.execute_tool(call, state).await {
                Ok(record) => {
                    if !record.result.success {
                        all_succeeded = false;
                    }
                    tool_results.push(record);

                    // Stop on failure if configured
                    if !all_succeeded && !self.config.continue_on_failure {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to execute tool {}: {}", call.name, e);
                    all_succeeded = false;

                    // Use a user-friendly message; include the tool name but hide internal error details
                    let user_message = format!("Unable to execute '{}'", call.name);
                    let error_details = e.to_string();

                    tool_results.push(ToolExecutionRecord {
                        tool_name: call.name.clone(),
                        params: call.params.clone(),
                        result: ToolResult::failure(user_message, error_details),
                        execution_time_ms: 0,
                    });

                    if !self.config.continue_on_failure {
                        break;
                    }
                }
            }
        }

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Build summary
        let summary = if all_succeeded {
            format!(
                "Executed {} tool(s) successfully in {}ms",
                tool_results.len(),
                execution_time_ms
            )
        } else {
            let failed = tool_results.iter().filter(|r| !r.result.success).count();
            format!(
                "Executed {} tool(s) with {} failure(s) in {}ms",
                tool_results.len(),
                failed,
                execution_time_ms
            )
        };

        Ok(ExecutionResult {
            tool_results,
            summary,
            all_succeeded,
            execution_time_ms,
        })
    }

    /// Execute a tool by name with given parameters
    pub async fn execute_by_name(
        &self,
        name: &str,
        params: serde_json::Value,
        state: &AppState,
    ) -> Result<ToolResult> {
        let call = ParsedToolCall::new(name, params, "", 0, 0);
        let record = self.execute_tool(&call, state).await?;
        Ok(record.result)
    }

    /// Check if a tool exists
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.get(name).is_some()
    }

    /// Get list of available tool names
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn available_tools(&self) -> Vec<&str> {
        self.registry.tool_names()
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Agentic Loop
// ============================================================================

/// State for an agentic conversation with tool use
///
/// Agentic loop utilities - for advanced multi-turn tool use
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Conversation history
    pub messages: Vec<AgentMessage>,
    /// Current loop depth
    pub depth: usize,
    /// Total tools executed
    pub total_tools_executed: usize,
    /// Whether the agent should continue
    pub should_continue: bool,
}

/// A message in the agent conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Agentic loop utilities - for advanced multi-turn tool use
pub struct AgentMessage {
    /// Role: "user", "assistant", or "tool"
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Tool results if this is a tool message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<ToolExecutionRecord>>,
}

impl AgentState {
    /// Create a new agent state with initial user message
    #[allow(dead_code)] // Agentic loop utilities - for advanced multi-turn tool use
    pub fn new(user_message: &str) -> Self {
        Self {
            messages: vec![AgentMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
                tool_results: None,
            }],
            depth: 0,
            total_tools_executed: 0,
            should_continue: true,
        }
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: content.to_string(),
            tool_results: None,
        });
    }

    /// Add tool results
    pub fn add_tool_results(&mut self, results: ExecutionResult) {
        self.total_tools_executed += results.tool_results.len();
        self.messages.push(AgentMessage {
            role: "tool".to_string(),
            content: results.to_llm_context(),
            tool_results: Some(results.tool_results),
        });
        self.depth += 1;
    }

    /// Get the full conversation for LLM context
    #[allow(dead_code)] // Agentic loop utilities - for advanced multi-turn tool use
    pub fn to_conversation_context(&self) -> String {
        let mut context = String::new();

        for msg in &self.messages {
            match msg.role.as_str() {
                "user" => context.push_str(&format!("\nUSER: {}\n", msg.content)),
                "assistant" => context.push_str(&format!("\nASSISTANT: {}\n", msg.content)),
                "tool" => context.push_str(&msg.content),
                _ => {}
            }
        }

        context
    }

    /// Get the last assistant message
    #[allow(dead_code)] // Agentic loop utilities - for advanced multi-turn tool use
    pub fn last_assistant_message(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == "assistant")
            .map(|m| m.content.as_str())
    }

    /// Get all tool results from the conversation
    #[allow(dead_code)] // Agentic loop utilities - for advanced multi-turn tool use
    pub fn all_tool_results(&self) -> Vec<&ToolExecutionRecord> {
        self.messages
            .iter()
            .filter_map(|m| m.tool_results.as_ref())
            .flatten()
            .collect()
    }
}

/// Configuration for the agentic loop
#[derive(Debug, Clone)]
#[allow(dead_code)] // Agentic loop utilities - for advanced multi-turn tool use
pub struct AgentLoopConfig {
    /// Maximum number of LLM calls in a single loop
    pub max_iterations: usize,
    /// Maximum total tools to execute
    pub max_total_tools: usize,
    /// Whether to automatically continue after tool execution
    pub auto_continue: bool,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            max_total_tools: 10,
            auto_continue: true,
        }
    }
}

/// Run a single step of the agentic loop
///
/// Returns (should_continue, updated_state)
///
/// Agentic loop utilities - for advanced multi-turn tool use
#[allow(dead_code)]
pub async fn run_agent_step(
    executor: &ToolExecutor,
    state: &AppState,
    agent_state: &mut AgentState,
    llm_response: &str,
    config: &AgentLoopConfig,
) -> Result<bool> {
    // Check limits
    if agent_state.depth >= config.max_iterations {
        info!("Agent loop reached max iterations ({})", config.max_iterations);
        agent_state.should_continue = false;
        return Ok(false);
    }

    if agent_state.total_tools_executed >= config.max_total_tools {
        info!("Agent loop reached max total tools ({})", config.max_total_tools);
        agent_state.should_continue = false;
        return Ok(false);
    }

    // Add assistant message
    agent_state.add_assistant_message(llm_response);

    // Execute any tools
    let execution_result = executor.execute_all(llm_response, state).await?;

    if execution_result.tool_results.is_empty() {
        // No tools to execute, conversation complete
        agent_state.should_continue = false;
        return Ok(false);
    }

    // Add tool results to state
    agent_state.add_tool_results(execution_result);

    // Should continue if configured and limits not reached
    let should_continue = config.auto_continue
        && agent_state.depth < config.max_iterations
        && agent_state.total_tools_executed < config.max_total_tools;

    agent_state.should_continue = should_continue;
    Ok(should_continue)
}

// ============================================================================
// Response Formatting
// ============================================================================

/// Format execution results for display in the UI
pub fn format_for_ui(result: &ExecutionResult) -> UIResponse {
    let mut actions = Vec::new();
    let mut messages = Vec::new();

    for record in &result.tool_results {
        // Add action indicator
        actions.push(UIAction {
            tool_name: record.tool_name.clone(),
            success: record.result.success,
            icon: get_tool_icon(&record.tool_name),
        });

        // Add message
        messages.push(record.result.message.clone());
    }

    UIResponse {
        actions,
        message: messages.join("\n\n"),
        has_errors: !result.all_succeeded,
    }
}

/// UI-friendly response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIResponse {
    /// Actions that were taken
    pub actions: Vec<UIAction>,
    /// Combined message for display
    pub message: String,
    /// Whether any errors occurred
    pub has_errors: bool,
}

/// UI-friendly action indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // All fields used for UI display
pub struct UIAction {
    /// Name of the tool
    pub tool_name: String,
    /// Whether it succeeded
    pub success: bool,
    /// Icon name for the UI
    pub icon: String,
}

/// Get an icon name for a tool (for UI display)
fn get_tool_icon(tool_name: &str) -> String {
    match tool_name {
        "start_focus_session" => "play".to_string(),
        "end_focus_session" => "stop".to_string(),
        "get_active_session" => "clock".to_string(),
        "get_session_stats" => "chart".to_string(),
        "log_trigger" => "alert".to_string(),
        "get_trigger_patterns" => "trending".to_string(),
        "get_blocking_stats" => "shield".to_string(),
        "add_blocked_item" => "plus-circle".to_string(),
        "get_streak_info" => "flame".to_string(),
        "set_focus_goal" => "target".to_string(),
        _ => "tool".to_string(),
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Quick execution of a tool by name
///
/// Agentic loop utilities - for advanced multi-turn tool use
#[allow(dead_code)]
pub async fn quick_execute(
    tool_name: &str,
    params: serde_json::Value,
    state: &AppState,
) -> Result<ToolResult> {
    let executor = ToolExecutor::new();
    executor.execute_by_name(tool_name, params, state).await
}

/// Parse and execute tools from text
///
/// Agentic loop utilities - for advanced multi-turn tool use
#[allow(dead_code)]
pub async fn parse_and_execute(
    text: &str,
    state: &AppState,
) -> Result<ExecutionResult> {
    let executor = ToolExecutor::new();
    executor.execute_all(text, state).await
}

/// Check if text contains executable tools
///
/// Agentic loop utilities - for advanced multi-turn tool use
#[allow(dead_code)]
pub fn has_executable_tools(text: &str) -> bool {
    let executor = ToolExecutor::new();
    let parse_result = executor.parse_output(text);
    parse_result.has_tools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = ToolExecutor::new();
        assert!(executor.has_tool("start_focus_session"));
        assert!(executor.has_tool("get_session_stats"));
        assert!(!executor.has_tool("nonexistent_tool"));
    }

    #[test]
    fn test_available_tools() {
        let executor = ToolExecutor::new();
        let tools = executor.available_tools();
        assert!(tools.contains(&"start_focus_session"));
        assert!(tools.contains(&"end_focus_session"));
        assert!(tools.contains(&"log_trigger"));
    }

    #[test]
    fn test_tool_documentation() {
        let executor = ToolExecutor::new();
        let docs = executor.get_tool_documentation();

        assert!(docs.contains("Available Tools"));
        assert!(docs.contains("start_focus_session"));
        assert!(docs.contains("duration"));
    }

    #[test]
    fn test_agent_state() {
        let mut state = AgentState::new("Help me focus");

        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.depth, 0);

        state.add_assistant_message("I'll help you start a session");
        assert_eq!(state.messages.len(), 2);

        let context = state.to_conversation_context();
        assert!(context.contains("USER: Help me focus"));
        assert!(context.contains("ASSISTANT: I'll help you start a session"));
    }

    #[test]
    fn test_execution_result_llm_context() {
        let result = ExecutionResult {
            tool_results: vec![ToolExecutionRecord {
                tool_name: "get_session_stats".to_string(),
                params: serde_json::json!({}),
                result: ToolResult::success("Today: 2 hours focused"),
                execution_time_ms: 50,
            }],
            summary: "Executed 1 tool".to_string(),
            all_succeeded: true,
            execution_time_ms: 50,
        };

        let context = result.to_llm_context();
        assert!(context.contains("TOOL EXECUTION RESULTS"));
        assert!(context.contains("get_session_stats"));
        assert!(context.contains("2 hours"));
    }

    #[test]
    fn test_ui_response_formatting() {
        let result = ExecutionResult {
            tool_results: vec![
                ToolExecutionRecord {
                    tool_name: "start_focus_session".to_string(),
                    params: serde_json::json!({"duration": 25}),
                    result: ToolResult::success("Started 25-minute session"),
                    execution_time_ms: 100,
                },
            ],
            summary: "Executed 1 tool".to_string(),
            all_succeeded: true,
            execution_time_ms: 100,
        };

        let ui_response = format_for_ui(&result);
        assert_eq!(ui_response.actions.len(), 1);
        assert_eq!(ui_response.actions[0].tool_name, "start_focus_session");
        assert_eq!(ui_response.actions[0].icon, "play");
        assert!(!ui_response.has_errors);
    }

    #[test]
    fn test_has_executable_tools() {
        assert!(has_executable_tools(r#"<tool name="start_focus_session"/>"#));
        assert!(has_executable_tools(r#"[[get_session_stats()]]"#));
        assert!(!has_executable_tools("Just regular text"));
    }
}
