// ai/mod.rs - Local AI module for privacy-first coaching
//
// # Overview
//
// This module provides 100% local LLM inference using llama.cpp with no network calls.
// Designed for privacy-conscious users who want AI coaching without data leaving their device.
//
// # Feature Flag: `local-ai`
//
// The local AI functionality is controlled by the `local-ai` Cargo feature flag.
// When enabled, the application can:
// - Download and run local models like Phi-3.5-mini or TinyLlama
// - Perform AI inference entirely on the user's device
// - Provide privacy-first AI coaching with zero data leaving the machine
//
// When disabled:
// - Local AI provider is marked as unavailable
// - Users are directed to use cloud providers instead
// - The application still functions with cloud-based AI providers
//
// ## Enabling Local AI
//
// In Cargo.toml, local-ai is included in default features:
// ```toml
// [features]
// default = ["custom-protocol", "macos-private-api", "local-ai"]
// local-ai = ["llama-cpp-2"]
// ```
//
// To build without local AI (smaller binary, faster compilation):
// ```bash
// cargo build --release --no-default-features --features custom-protocol
// ```
//
// ## Supported Models
//
// When local-ai is enabled, the following models are supported:
// - **phi-3.5-mini** (Recommended): Microsoft's Phi-3.5-mini (3.8B params, ~2.3 GB)
//   - Best balance of quality and speed
//   - Context size: 16,384 tokens
// - **tinyllama**: TinyLlama (1.1B params, ~670 MB)
//   - Fastest option, lower quality
//   - Context size: 4,096 tokens
//
// # Architecture
//
// - system_prompts: Modular prompt components (identity, framework, guidelines)
// - prompt_templates: High-level prompt building interface
// - llm_engine: Local LLM inference (llama.cpp binding) [requires local-ai feature]
// - model_manager: Model download and lifecycle management [requires local-ai feature]
// - guidelines: Parlant-style guideline types and matching logic
// - guideline_registry: Registry of all available coaching guidelines
// - orchestrator: Dynamic guideline evaluation and prompt building
// - tools: Tool definitions and handlers for agent actions
// - tool_parser: Parse tool calls from LLM output
// - providers: Multi-provider abstraction (cloud + local)

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
