// ai/mod.rs - Local AI module for privacy-first coaching
//
// This module provides 100% local LLM inference using llama.cpp with no network calls.
// Designed for privacy-conscious users who want AI coaching without data leaving their device.
//
// Architecture:
// - system_prompts: Modular prompt components (identity, framework, guidelines)
// - prompt_templates: High-level prompt building interface
// - llm_engine: Local LLM inference (llama.cpp binding)
// - model_manager: Model download and lifecycle management
// - guidelines: Parlant-style guideline types and matching logic
// - guideline_registry: Registry of all available coaching guidelines
// - orchestrator: Dynamic guideline evaluation and prompt building
// - tools: Tool definitions and handlers for agent actions
// - tool_parser: Parse tool calls from LLM output

#[cfg(feature = "local-ai")]
pub mod llm_engine;
#[cfg(feature = "local-ai")]
pub mod model_manager;
pub mod prompt_templates;
pub mod system_prompts;

// Parlant-style guideline orchestration system
pub mod guidelines;
pub mod guideline_registry;
pub mod orchestrator;

// Tool system for agent actions (LangGraph-inspired)
pub mod tools;
pub mod tool_parser;
pub mod tool_executor;

// Multi-provider LLM abstraction
pub mod providers;

#[cfg(feature = "local-ai")]
#[allow(unused_imports)]
pub use llm_engine::{LlmEngine, LlmResponse, StreamChunk};
#[cfg(feature = "local-ai")]
#[allow(unused_imports)]
pub use model_manager::{ModelConfig, ModelManager, ModelStatus};
#[allow(unused_imports)]
pub use prompt_templates::{
    build_chat_prompt, build_coach_prompt, build_lightweight_prompt,
    build_reflection_prompt_detailed, build_session_prompt, build_suggestions,
    build_agentic_prompt, build_agentic_chat_prompt, build_action_prompt,
    PromptTemplate, UserIntent,
};
#[allow(unused_imports)]
pub use system_prompts::{PromptContext, ScenarioPromptBuilder};

// Re-export guideline orchestration types
#[allow(unused_imports)]
pub use guidelines::{Guideline, GuidelineCategory, GuidelineCondition, GuidelineMatch};
#[allow(unused_imports)]
pub use guideline_registry::GuidelineRegistry;
#[allow(unused_imports)]
pub use orchestrator::{
    build_suggestions_from_guidelines, GuidelineOrchestrator, OrchestrationResult,
    OrchestratorConfig, PromptType,
};

// Re-export tool types
#[allow(unused_imports)]
pub use tools::{
    Tool, ToolCategory, ToolExample, ToolHandler, ToolParameter, ToolRegistry, ToolResult,
    ParameterType,
};
#[allow(unused_imports)]
pub use tool_parser::{
    ParsedToolCall, ParseResult, TextSegment, ToolParser,
    contains_tool_calls, extract_first_tool, validate_tool_call,
};
#[allow(unused_imports)]
pub use tool_executor::{
    ToolExecutor, ExecutorConfig, ExecutionResult, ToolExecutionRecord,
    AgentState, AgentMessage, AgentLoopConfig,
    run_agent_step, format_for_ui, UIResponse, UIAction,
    quick_execute, parse_and_execute, has_executable_tools,
};

// Stub types when local-ai is disabled
#[cfg(not(feature = "local-ai"))]
pub struct LlmEngine;

#[cfg(not(feature = "local-ai"))]
pub struct ModelConfig;

#[cfg(not(feature = "local-ai"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
    Loaded,
    Error,
}

#[cfg(not(feature = "local-ai"))]
impl ModelConfig {
    pub fn phi_3_5_mini() -> Self {
        Self
    }
}
