// commands/coach.rs - Local AI Coach commands
//
// This module provides a privacy-first AI coaching experience using
// local LLM inference (llama.cpp) with Parlant-style guideline orchestration.
// 100% local - no data leaves the user's device.
//
// The orchestrator dynamically evaluates ALL guidelines against each user
// turn and builds context-aware prompts. Template responses are only used
// as fallbacks when the LLM is unavailable.

use crate::ai::{
    build_suggestions_from_guidelines, GuidelineOrchestrator, PromptType,
    tool_executor::{ToolExecutor, format_for_ui},
};
use crate::commands::chat_context;
use crate::commands::chat_history::{self, MessageRole};
use crate::{AppState, Error, Result};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tauri::State;
use tracing::{debug, info, warn};

/// Global orchestrator instance (initialized once, reused for all requests)
static ORCHESTRATOR: OnceLock<GuidelineOrchestrator> = OnceLock::new();

/// Global tool executor instance (initialized once, reused for all requests)
static TOOL_EXECUTOR: OnceLock<ToolExecutor> = OnceLock::new();

/// Get or initialize the global orchestrator
fn get_orchestrator() -> &'static GuidelineOrchestrator {
    ORCHESTRATOR.get_or_init(|| {
        info!("Initializing guideline orchestrator");
        GuidelineOrchestrator::new()
    })
}

/// Get or initialize the global tool executor
fn get_tool_executor() -> &'static ToolExecutor {
    TOOL_EXECUTOR.get_or_init(|| {
        info!("Initializing tool executor");
        ToolExecutor::new()
    })
}

/// Chat message in the coach conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: String,
}

/// Coach response with optional suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachResponse {
    pub message: String,
    pub suggestions: Vec<String>,
}

/// User context for personalized coaching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub total_focus_hours_today: f32,
    pub sessions_completed_today: i32,
    pub current_streak_days: i32,
    pub top_trigger: Option<String>,
    pub average_session_minutes: i32,
}

/// Get a coaching response based on user input and context
///
/// Uses Parlant-style guideline orchestration to dynamically match relevant
/// guidelines and build a context-aware prompt for the LLM.
#[tauri::command]
pub async fn get_coach_response(
    state: State<'_, AppState>,
    message: String,
    context: Option<UserContext>,
) -> Result<CoachResponse> {
    // Fetch user stats for context if not provided
    let ctx = match context {
        Some(c) => c,
        None => get_user_context(&state).await?,
    };

    // Use orchestrator to evaluate guidelines and build dynamic prompt
    let orchestrator = get_orchestrator();
    let orchestration = orchestrator.orchestrate(&message, &ctx);

    debug!(
        "Orchestrated {} guidelines for message: {:?}",
        orchestration.matched_guidelines.len(),
        orchestration.matched_guidelines.iter().map(|g| &g.id).collect::<Vec<_>>()
    );

    // Try LLM with orchestrated prompt, fall back to templates
    match try_orchestrated_llm_response(&state, &orchestration.dynamic_prompt, &ctx, &orchestration).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("LLM generation failed, using template fallback: {}", e);
            Ok(generate_response(&message, &ctx))
        }
    }
}

/// Get coaching tips based on current user stats
///
/// Uses orchestrator with PromptType::DailyTip to match contextual guidelines.
#[tauri::command]
pub async fn get_daily_tip(
    state: State<'_, AppState>,
) -> Result<CoachResponse> {
    let ctx = get_user_context(&state).await?;

    // Use orchestrator for daily tip type
    let orchestrator = get_orchestrator();
    let orchestration = orchestrator.orchestrate_for_type(PromptType::DailyTip, &ctx, None);

    debug!(
        "Daily tip orchestration matched {} guidelines",
        orchestration.matched_guidelines.len()
    );

    // Try LLM with orchestrated prompt, fall back to template
    match try_orchestrated_llm_response(&state, &orchestration.dynamic_prompt, &ctx, &orchestration).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("LLM generation failed for daily tip: {}", e);
            Ok(generate_daily_tip(&ctx))
        }
    }
}

/// Get focus session planning advice
///
/// Uses orchestrator with PromptType::SessionAdvice for pre-session guidance.
#[tauri::command]
pub async fn get_session_advice(
    state: State<'_, AppState>,
    planned_duration_minutes: i32,
) -> Result<CoachResponse> {
    let ctx = get_user_context(&state).await?;

    let message = format!("Planning a {}-minute focus session", planned_duration_minutes);

    // Use orchestrator for session advice
    let orchestrator = get_orchestrator();
    let orchestration = orchestrator.orchestrate_for_type(
        PromptType::SessionAdvice,
        &ctx,
        Some(&message),
    );

    debug!(
        "Session advice orchestration matched {} guidelines",
        orchestration.matched_guidelines.len()
    );

    // Try LLM with orchestrated prompt, fall back to template
    match try_orchestrated_llm_response(&state, &orchestration.dynamic_prompt, &ctx, &orchestration).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("LLM generation failed for session advice: {}", e);
            Ok(generate_session_advice(&ctx, planned_duration_minutes))
        }
    }
}

/// Get post-session reflection prompt
///
/// Uses orchestrator with PromptType::Reflection for post-session guidance.
#[tauri::command]
pub async fn get_reflection_prompt(
    state: State<'_, AppState>,
    session_completed: bool,
    actual_duration_minutes: i32,
) -> Result<CoachResponse> {
    let ctx = get_user_context(&state).await?;

    let message = if session_completed {
        format!("Completed a {}-minute session successfully", actual_duration_minutes)
    } else {
        format!("Ended session early after {} minutes", actual_duration_minutes)
    };

    // Use orchestrator for reflection
    let orchestrator = get_orchestrator();
    let orchestration = orchestrator.orchestrate_for_type(
        PromptType::Reflection,
        &ctx,
        Some(&message),
    );

    debug!(
        "Reflection orchestration matched {} guidelines",
        orchestration.matched_guidelines.len()
    );

    // Try LLM with orchestrated prompt, fall back to template
    match try_orchestrated_llm_response(&state, &orchestration.dynamic_prompt, &ctx, &orchestration).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("LLM generation failed for reflection: {}", e);
            Ok(generate_reflection_prompt(&ctx, session_completed, actual_duration_minutes))
        }
    }
}

/// Analyze user's distraction patterns and provide insights
///
/// Uses orchestrator with PromptType::PatternAnalysis for deep pattern insights.
#[tauri::command]
pub async fn analyze_patterns(
    state: State<'_, AppState>,
) -> Result<CoachResponse> {
    let ctx = get_user_context(&state).await?;

    // Use orchestrator for pattern analysis
    let orchestrator = get_orchestrator();
    let orchestration = orchestrator.orchestrate_for_type(PromptType::PatternAnalysis, &ctx, None);

    debug!(
        "Pattern analysis orchestration matched {} guidelines",
        orchestration.matched_guidelines.len()
    );

    // Try LLM with orchestrated prompt, fall back to template
    match try_orchestrated_llm_response(&state, &orchestration.dynamic_prompt, &ctx, &orchestration).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("LLM generation failed for pattern analysis: {}", e);
            Ok(generate_pattern_analysis(&ctx))
        }
    }
}

// Helper: Clean up response text
//
// Removes common formatting issues that occur with small LLMs like Phi-3.5-mini:
// - Strips "Response:", "Assistant:", "A:" prefixes
// - Removes malformed/empty bracket patterns [ ] that failed to parse as tools
// - Cleans up stray/malformed backticks
// - Normalizes whitespace
fn clean_response_text(text: &str) -> String {
    let mut result = text.trim().to_string();

    // Strip markdown header prefixes like "## Response:" first
    let header_prefixes = [
        "### response:", "## response:", "# response:",
        "### Response:", "## Response:", "# Response:",
    ];
    for prefix in header_prefixes {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].trim_start().to_string();
            break;
        }
    }

    // Strip common LLM prefixes (case-insensitive)
    let prefixes_to_strip = ["response:", "assistant:", "a:"];
    for prefix in prefixes_to_strip {
        if result.to_lowercase().starts_with(prefix) {
            result = result[prefix.len()..].trim_start().to_string();
            break;
        }
    }

    // Strip code block wrappers around tool calls
    if result.contains("```") && result.contains("<tool ") {
        // Find and extract the tool call without the code block
        if let Some(tool_start) = result.find("<tool ") {
            if let Some(tool_end) = result[tool_start..].find("/>") {
                let tool_call = result[tool_start..tool_start + tool_end + 2].to_string();
                // Get text before the code block
                let before = if let Some(block_start) = result.find("```") {
                    result[..block_start].trim().to_string()
                } else {
                    String::new()
                };
                result = if before.is_empty() {
                    tool_call
                } else {
                    format!("{}\n\n{}", before, tool_call)
                };
            }
        }
    }

    // Remove malformed bracket patterns that the LLM generates
    // These are empty or partial brackets like [ ], [  ]
    // Simple string replacement is more efficient than regex for this simple case
    while result.contains("[ ]") || result.contains("[  ]") || result.contains("[   ]") {
        result = result.replace("[ ]", " ").replace("[  ]", " ").replace("[   ]", " ");
    }

    // Remove incomplete bracket patterns like [: or [:value]
    // Only remove if they don't have a valid tool name (start with letter)
    let mut cleaned = String::new();
    let mut chars = result.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            // Look ahead to see what's in the bracket
            let mut bracket_content = String::new();
            let mut found_closing = false;

            while let Some(&next_ch) = chars.peek() {
                if next_ch == ']' {
                    chars.next(); // consume the ]
                    found_closing = true;
                    break;
                }
                bracket_content.push(chars.next().unwrap());
            }

            // Only keep the bracket if it looks like a valid tool pattern:
            // [tool_name] or [tool_name:params] where tool_name starts with a letter
            let looks_valid = bracket_content.split(':').next()
                .map(|name| name.trim().chars().next().map(|c| c.is_alphabetic()).unwrap_or(false))
                .unwrap_or(false);

            if looks_valid && found_closing {
                cleaned.push('[');
                cleaned.push_str(&bracket_content);
                cleaned.push(']');
            } else if !found_closing {
                // Unclosed bracket - skip it
                cleaned.push_str(&bracket_content);
            }
        } else {
            cleaned.push(ch);
        }
    }

    result = cleaned;

    // Clean up stray backticks that aren't part of code blocks
    let mut cleaned = String::new();
    let mut in_code = false;
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '`' {
            // Check if this is a triple backtick (code block)
            if i + 2 < chars.len() && chars[i + 1] == '`' && chars[i + 2] == '`' {
                in_code = !in_code;
                cleaned.push('`');
                cleaned.push('`');
                cleaned.push('`');
                i += 3;
            } else if !in_code {
                // Single backtick outside code block - check if it has a closing pair
                let remaining = &result[i + 1..];
                if remaining.contains('`') {
                    // Has a closing backtick, keep it
                    cleaned.push('`');
                } else {
                    // No closing backtick, skip it (stray backtick)
                    // Only skip if not inside a code block
                }
                i += 1;
            } else {
                cleaned.push('`');
                i += 1;
            }
        } else {
            cleaned.push(chars[i]);
            i += 1;
        }
    }

    result = cleaned;

    // Normalize whitespace (collapse multiple spaces, but preserve newlines)
    result = result
        .split('\n')
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n");

    // Final pass: remove lines that are just whitespace after cleaning
    result = result
        .split('\n')
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    // Remove any unwanted error or execution messages that the LLM might include
    // These often appear as standalone lines or phrases
    let to_remove = [
        "Execution failed",
        "execution failed",
        "execution error",
        "Unable to",
        "unable to",
    ];

    for phrase in &to_remove {
        // Only remove if it's on its own line or a short standalone phrase
        let lines: Vec<&str> = result.split('\n').collect();
        let filtered: Vec<&str> = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                // Keep the line unless it's just the error phrase
                !(trimmed == *phrase || trimmed.starts_with(&format!("{} ", phrase)))
            })
            .copied()
            .collect();
        result = filtered.join("\n");
    }

    result.trim().to_string()
}

// Helper: Try to generate response using LLM with orchestrated prompt
//
// This function uses the dynamic prompt built by the orchestrator, which includes
// all matched guidelines injected into context. It also handles tool orchestration:
// 1. Builds prompt with tool documentation
// 2. Gets LLM response
// 3. Parses for tool calls
// 4. Executes any tools found
// 5. Formats response for UI
async fn try_orchestrated_llm_response(
    state: &State<'_, AppState>,
    dynamic_prompt: &str,
    _ctx: &UserContext,
    orchestration: &crate::ai::OrchestrationResult,
) -> Result<CoachResponse> {
    #[cfg(not(feature = "local-ai"))]
    {
        // Suppress unused variable warnings when local-ai is disabled
        let _ = (dynamic_prompt, _ctx, orchestration);
        // When local-ai is disabled, always return error to trigger fallback
        return Err(Error::NotFound("LLM engine not available in this build".into()));
    }

    #[cfg(feature = "local-ai")]
    {
        // Check if LLM engine is available
        let engine = state.llm_engine.read().await;
        let engine = engine.as_ref().ok_or_else(|| {
            Error::NotFound("LLM engine not initialized".into())
        })?;

        // Try to load model if not already loaded
        if !engine.is_loaded().await {
            debug!("Model not loaded, attempting to load...");
            match engine.load_model().await {
                Ok(()) => {
                    info!("LLM model loaded successfully");
                }
                Err(e) => {
                    // Provide helpful error message based on the error type
                    let msg = if e.to_string().contains("not downloaded") {
                        "LLM model not downloaded. Please download a model first using the AI Settings."
                    } else {
                        "Failed to load LLM model. Check the AI Settings for model status."
                    };
                    return Err(Error::NotFound(format!("{} ({})", msg, e)));
                }
            }
        }

        // Get tool executor and add tool documentation to prompt
        let tool_executor = get_tool_executor();
        let tool_docs = tool_executor.get_tool_documentation();

        // Combine dynamic prompt with tool documentation
        let full_prompt = format!(
            "{}\n\n{}\n\n{}",
            tool_docs,
            dynamic_prompt,
            "If you need to take an action (like starting a session or logging a trigger), use the appropriate tool. Otherwise, just respond conversationally."
        );

        debug!(
            "Generating LLM response with orchestrated prompt ({} chars, {} guidelines) + tool docs ({} chars)",
            dynamic_prompt.len(),
            orchestration.matched_guidelines.len(),
            tool_docs.len()
        );

        // Generate response with generous token budget
        // Phi-3.5-mini has 16K context window, so we can afford 800+ tokens for responses
        // This allows for comprehensive coaching responses without truncation
        let max_response_tokens = 800;
        let temperature = 0.7;
        let llm_response = engine.generate(&full_prompt, max_response_tokens, temperature).await?;

        debug!(
            "LLM generated {} tokens in {}ms",
            llm_response.tokens_generated,
            llm_response.inference_time_ms
        );

        // Log raw LLM output for debugging
        debug!("RAW LLM OUTPUT:\n{}\n--- END RAW OUTPUT ---", llm_response.text);

        // Parse for tool calls
        let parse_result = tool_executor.parse_output(&llm_response.text);

        debug!(
            "Tool parsing result: has_tools={}, tool_count={}, text_segments={}",
            parse_result.has_tools,
            parse_result.tool_calls.len(),
            parse_result.text_segments.len()
        );

        if parse_result.has_tools {
            debug!(
                "Found tool calls: {:?}",
                parse_result.tool_calls.iter().map(|t| &t.name).collect::<Vec<_>>()
            );
        }

        // Extract text without tool calls for the response message
        let mut response_text = if parse_result.has_tools {
            parse_result.text_without_tools()
        } else {
            llm_response.text.clone()
        };

        debug!("Text before cleaning: '{}' ({} chars)", response_text, response_text.len());

        // Clean up response text: strip "Response:" prefix and clean stray backticks
        response_text = clean_response_text(&response_text);

        debug!("Text after cleaning: '{}' ({} chars)", response_text, response_text.len());

        // Execute any tools found
        let mut final_message = response_text.clone();
        if parse_result.has_tools {
            debug!(
                "Found {} tool call(s) in LLM response, executing...",
                parse_result.tool_calls.len()
            );

            match tool_executor.execute_all(&llm_response.text, state.inner()).await {
                Ok(execution_result) => {
                    debug!(
                        "Tool execution completed: {} tool(s) in {}ms, all_succeeded={}",
                        execution_result.tool_results.len(),
                        execution_result.execution_time_ms,
                        execution_result.all_succeeded
                    );

                    // Format tool results for user
                    let ui_response = format_for_ui(&execution_result);

                    debug!("UI response message: '{}'", ui_response.message);

                    // Determine if we should use the LLM message or tool message
                    // Prefer tool message if it exists and the LLM response is minimal
                    let llm_response_minimal = response_text.len() < 50 || response_text.trim().is_empty();

                    if !ui_response.message.is_empty() {
                        if llm_response_minimal {
                            // Tool message is sufficient, use it alone
                            final_message = ui_response.message;
                        } else if !response_text.is_empty() {
                            // Include both messages if LLM provided substantial context
                            final_message = format!("{}\n\n{}", response_text, ui_response.message);
                        }
                    }
                    // If no tool message, keep the LLM response
                }
                Err(e) => {
                    warn!("Tool execution failed: {}", e);
                    debug!("Continuing with LLM response despite tool execution error");
                    // Continue with the LLM's response even if tools failed
                }
            }
        }

        // Build suggestions from matched guidelines
        let orchestrator = get_orchestrator();
        let matches = orchestrator.evaluate_guidelines("", _ctx);
        let suggestions = build_suggestions_from_guidelines(&matches, _ctx);

        Ok(CoachResponse {
            message: final_message,
            suggestions,
        })
    }
}

// Helper: Get user context from database
async fn get_user_context(state: &State<'_, AppState>) -> Result<UserContext> {
    // Get today's stats
    let today = chrono::Utc::now().date_naive().to_string();

    let daily_stats: Option<(i64, i32)> = sqlx::query_as(
        "SELECT COALESCE(SUM(actual_duration_seconds), 0), COUNT(*)
         FROM sessions
         WHERE date(start_time) = date(?) AND completed = 1"
    )
    .bind(&today)
    .fetch_optional(state.pool())
    .await?;

    let (focus_seconds, sessions) = daily_stats.unwrap_or((0, 0));

    // Get top trigger from journal
    let top_trigger: Option<String> = sqlx::query_scalar(
        "SELECT trigger_type FROM journal_entries
         WHERE created_at >= datetime('now', '-30 days')
         GROUP BY trigger_type
         ORDER BY COUNT(*) DESC
         LIMIT 1"
    )
    .fetch_optional(state.pool())
    .await?;

    // Get average session duration
    let avg_duration: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(actual_duration_seconds / 60.0) FROM sessions WHERE completed = 1"
    )
    .fetch_optional(state.pool())
    .await?;

    // Calculate streak (simplified)
    let streak: i32 = sqlx::query_scalar(
        "WITH RECURSIVE streak_days AS (
            SELECT date('now') as day, 1 as streak
            UNION ALL
            SELECT date(day, '-1 day'), streak + 1
            FROM streak_days
            WHERE EXISTS (
                SELECT 1 FROM sessions
                WHERE date(start_time) = date(day, '-1 day') AND completed = 1
            )
            AND streak < 365
        )
        SELECT MAX(streak) FROM streak_days"
    )
    .fetch_optional(state.pool())
    .await?
    .unwrap_or(0);

    Ok(UserContext {
        total_focus_hours_today: focus_seconds as f32 / 3600.0,
        sessions_completed_today: sessions,
        current_streak_days: streak,
        top_trigger,
        average_session_minutes: avg_duration.unwrap_or(25.0) as i32,
    })
}

// Response generation functions
// These use templated responses but are designed to be replaced with LLM calls

fn generate_response(message: &str, ctx: &UserContext) -> CoachResponse {
    let msg_lower = message.to_lowercase();

    // Intent detection (simple keyword matching, can be upgraded to NLU)
    if msg_lower.contains("focus") || msg_lower.contains("concentrate") {
        return CoachResponse {
            message: format!(
                "Great that you're thinking about focus! Based on your patterns, \
                 you typically do well with {}-minute sessions. Remember to remove \
                 distractions before you begin, and set a clear intention for what \
                 you want to accomplish.",
                ctx.average_session_minutes
            ),
            suggestions: vec![
                "Start a focus session now".into(),
                "Review your blocked apps".into(),
                "Check your trigger journal".into(),
            ],
        };
    }

    if msg_lower.contains("distract") || msg_lower.contains("interrupt") {
        let trigger_advice = match &ctx.top_trigger {
            Some(trigger) => format!(
                "I notice '{}' is your most common trigger. Consider creating \
                 a pre-commitment: before your next session, write down what \
                 you'll do when this urge arises.",
                trigger
            ),
            None => "Start logging your distractions in the journal to identify patterns.".into(),
        };

        return CoachResponse {
            message: trigger_advice,
            suggestions: vec![
                "Log a trigger now".into(),
                "View trigger insights".into(),
                "Update blocking rules".into(),
            ],
        };
    }

    if msg_lower.contains("motivation") || msg_lower.contains("tired") || msg_lower.contains("exhausted") {
        return CoachResponse {
            message: format!(
                "It's normal to feel drained sometimes. You've completed {} sessions today \
                 and built a {}-day streak - that's real progress! Consider taking a proper \
                 break: step away from screens, move your body, or practice deep breathing. \
                 Your focus will return stronger.",
                ctx.sessions_completed_today, ctx.current_streak_days
            ),
            suggestions: vec![
                "Take a 10-minute break".into(),
                "Start a shorter session".into(),
                "Review your achievements".into(),
            ],
        };
    }

    if msg_lower.contains("help") || msg_lower.contains("how") || msg_lower.contains("what") {
        return CoachResponse {
            message: "I can help you with:\n\
                      - Planning focus sessions\n\
                      - Understanding your distraction patterns\n\
                      - Building better focus habits\n\
                      - Reflecting on your progress\n\n\
                      What would you like to work on?".into(),
            suggestions: vec![
                "Plan my next session".into(),
                "Analyze my patterns".into(),
                "Give me today's tip".into(),
            ],
        };
    }

    // Default response
    CoachResponse {
        message: "I'm here to help you build better focus habits. You can ask me about \
                  planning sessions, understanding your distraction patterns, or getting \
                  personalized productivity advice.".into(),
        suggestions: vec![
            "How can I focus better?".into(),
            "Analyze my distraction patterns".into(),
            "What's my progress?".into(),
        ],
    }
}

fn generate_daily_tip(ctx: &UserContext) -> CoachResponse {
    let tips = vec![
        (
            "Start with your hardest task. Your willpower is highest in the morning.",
            vec!["Start a morning session".into(), "Plan today's priorities".into()],
        ),
        (
            "Single-tasking beats multitasking. Focus on one thing at a time for deeper work.",
            vec!["Start a focus session".into(), "Review blocked distractions".into()],
        ),
        (
            "Take regular breaks. The Pomodoro technique suggests 5 minutes every 25 minutes.",
            vec!["Start a 25-min session".into(), "Set break reminders".into()],
        ),
        (
            "Your environment shapes your behavior. Remove temptations before you need willpower.",
            vec!["Update blocking rules".into(), "Check your setup".into()],
        ),
        (
            "Track your triggers. Understanding what distracts you is the first step to fixing it.",
            vec!["View trigger insights".into(), "Log a trigger".into()],
        ),
    ];

    // Select tip based on day
    let day_of_year = chrono::Utc::now().ordinal() as usize;
    let (tip, suggestions) = &tips[day_of_year % tips.len()];

    let personalized = if ctx.current_streak_days > 0 {
        format!("{} (You're on a {}-day streak - keep it up!)", tip, ctx.current_streak_days)
    } else {
        (*tip).to_string()
    };

    CoachResponse {
        message: personalized,
        suggestions: suggestions.clone(),
    }
}

fn generate_session_advice(ctx: &UserContext, planned_duration: i32) -> CoachResponse {
    let mut advice = Vec::new();

    if planned_duration > ctx.average_session_minutes + 15 {
        advice.push(format!(
            "This session is longer than your usual {} minutes. Consider breaking \
             it into smaller chunks with short breaks.",
            ctx.average_session_minutes
        ));
    }

    if ctx.sessions_completed_today >= 4 {
        advice.push(
            "You've already done great work today! Make sure you're taking \
             proper breaks between sessions."
                .into(),
        );
    }

    if let Some(ref trigger) = ctx.top_trigger {
        advice.push(format!(
            "Watch out for '{}' - it's your most common distraction. \
             Have a plan for when it comes up.",
            trigger
        ));
    }

    advice.push("Set a clear intention: What specific outcome do you want from this session?".into());

    CoachResponse {
        message: advice.join("\n\n"),
        suggestions: vec![
            "Start the session".into(),
            "Adjust duration".into(),
            "Review blocked apps".into(),
        ],
    }
}

fn generate_reflection_prompt(
    _ctx: &UserContext,
    completed: bool,
    actual_duration: i32,
) -> CoachResponse {
    if completed {
        CoachResponse {
            message: format!(
                "Great job completing your {} minute session! Take a moment to reflect:\n\
                 - What went well during this session?\n\
                 - Did you stay focused on your intention?\n\
                 - What would you do differently next time?",
                actual_duration
            ),
            suggestions: vec![
                "Log any triggers".into(),
                "Start another session".into(),
                "Take a break".into(),
            ],
        }
    } else {
        CoachResponse {
            message: format!(
                "You worked for {} minutes before ending early. That's okay - \
                 every attempt builds your focus muscle.\n\n\
                 What happened? Logging your triggers helps identify patterns \
                 so you can plan better next time.",
                actual_duration
            ),
            suggestions: vec![
                "Log what distracted me".into(),
                "Try a shorter session".into(),
                "View my patterns".into(),
            ],
        }
    }
}

fn generate_pattern_analysis(ctx: &UserContext) -> CoachResponse {
    let mut insights = Vec::new();

    // Focus time analysis
    if ctx.total_focus_hours_today > 4.0 {
        insights.push("You've done significant deep work today. Make sure to rest!".into());
    } else if ctx.total_focus_hours_today > 2.0 {
        insights.push("Good progress on focus time today. You're building momentum.".into());
    } else if ctx.sessions_completed_today > 0 {
        insights.push("You've started focusing today. Can you fit in one more session?".into());
    } else {
        insights.push("No focus sessions yet today. Even a 15-minute session helps!".into());
    }

    // Trigger analysis
    if let Some(ref trigger) = ctx.top_trigger {
        insights.push(format!(
            "Your most common distraction trigger is '{}'. Consider: What need \
             is this fulfilling? Can you address that need in a healthier way?",
            trigger
        ));
    }

    // Streak analysis
    if ctx.current_streak_days > 7 {
        insights.push(format!(
            "Amazing! You're on a {}-day streak. Consistency is building real habits.",
            ctx.current_streak_days
        ));
    } else if ctx.current_streak_days > 0 {
        insights.push(format!(
            "You're on a {}-day streak. Keep going to build the habit!",
            ctx.current_streak_days
        ));
    }

    CoachResponse {
        message: insights.join("\n\n"),
        suggestions: vec![
            "Start a session".into(),
            "View full analytics".into(),
            "Set a focus goal".into(),
        ],
    }
}

/// Send a message with conversation history and memory context
///
/// Enhanced version of get_coach_response that:
/// - Saves messages to conversation history
/// - Loads recent conversation context
/// - Incorporates relevant memories
/// - Auto-generates conversation title from first message
#[tauri::command]
pub async fn send_coach_message(
    state: State<'_, AppState>,
    conversation_id: Option<String>,
    message: String,
    context: Option<UserContext>,
) -> Result<CoachResponse> {
    // Fetch user stats for context if not provided
    let ctx = match context {
        Some(c) => c,
        None => get_user_context(&state).await?,
    };

    // Get or create conversation
    let conv_id = match conversation_id {
        Some(id) => id,
        None => {
            // Create new conversation
            let id = chat_history::create_conversation(state.clone(), None).await?;
            id
        }
    };

    // Build conversation context
    let conv_context = chat_history::build_conversation_context(
        state.clone(),
        conv_id.clone(),
        Some(message.clone()),
        Some(10),
    )
    .await?;

    // Build enhanced prompt with conversation history and memories
    let context_prompt = chat_context::build_context_prompt(
        &ctx,
        &conv_context.recent_messages,
        &conv_context.relevant_memories,
        conv_context.conversation_summary.as_deref(),
    );

    // Use orchestrator to evaluate guidelines and build dynamic prompt
    let orchestrator = get_orchestrator();
    let mut orchestration = orchestrator.orchestrate(&message, &ctx);

    // Enhance the dynamic prompt with conversation context
    orchestration.dynamic_prompt = format!(
        "{}\n\n{}",
        chat_context::format_as_system_message(&context_prompt),
        orchestration.dynamic_prompt
    );

    debug!(
        "Orchestrated {} guidelines with conversation context ({} messages, {} memories)",
        orchestration.matched_guidelines.len(),
        conv_context.recent_messages.len(),
        conv_context.relevant_memories.len()
    );

    // Save user message
    let token_count = chat_context::estimate_token_count(&message);
    chat_history::add_message(
        state.clone(),
        conv_id.clone(),
        "user".to_string(),
        message.clone(),
        Some(token_count),
        None,
    )
    .await?;

    // Try LLM with orchestrated prompt, fall back to templates
    let response = match try_orchestrated_llm_response(&state, &orchestration.dynamic_prompt, &ctx, &orchestration).await {
        Ok(r) => r,
        Err(e) => {
            warn!("LLM generation failed, using template fallback: {}", e);
            generate_response(&message, &ctx)
        }
    };

    // Save assistant response
    let response_token_count = chat_context::estimate_token_count(&response.message);
    chat_history::add_message(
        state.clone(),
        conv_id.clone(),
        "assistant".to_string(),
        response.message.clone(),
        Some(response_token_count),
        None,
    )
    .await?;

    // Extract and save potential memories from the conversation
    let user_memories = chat_context::extract_potential_memories(&message, &MessageRole::User);
    for (key, value, category) in user_memories {
        if let Err(e) = chat_history::save_memory(
            state.clone(),
            key,
            value,
            category,
            Some(0.8), // Medium confidence for auto-extracted memories
            Some(conv_id.clone()),
        )
        .await
        {
            warn!("Failed to save memory: {}", e);
        }
    }

    // Update conversation title if this is the first message
    if conv_context.recent_messages.is_empty() {
        let title = chat_context::generate_conversation_title(&message);
        if let Err(e) = chat_history::update_conversation_title(state.clone(), conv_id.clone(), title).await {
            warn!("Failed to update conversation title: {}", e);
        }
    }

    Ok(response)
}

/// Get active conversation or create a new one
///
/// Helper command to get the most recent active conversation,
/// or create a new one if none exists.
#[tauri::command]
pub async fn get_or_create_active_conversation(state: State<'_, AppState>) -> Result<String> {
    // Try to get the most recent conversation
    let recent = chat_history::list_conversations(state.clone(), Some(1), Some(0), Some(false)).await?;

    if let Some(conversation) = recent.first() {
        Ok(conversation.id.clone())
    } else {
        // Create new conversation
        chat_history::create_conversation(state, None).await
    }
}
