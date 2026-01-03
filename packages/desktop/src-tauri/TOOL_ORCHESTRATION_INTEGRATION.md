# Tool Orchestration Integration

## Overview

The tool orchestration system has been successfully integrated into the coach commands in `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/coach.rs`.

The AI coach can now take real actions in the app, such as:
- Starting and ending focus sessions
- Logging distraction triggers
- Retrieving statistics and analytics
- Managing blocked apps/websites
- Tracking streaks and goals

## Architecture

### Components

1. **Tool Registry** (`ai/tools.rs`)
   - Defines available tools with metadata
   - Registers tool handlers
   - Generates documentation for LLM system prompts

2. **Tool Parser** (`ai/tool_parser.rs`)
   - Parses tool calls from LLM output
   - Supports multiple formats: XML-style, function-style, JSON blocks
   - Extracts parameters and validates syntax

3. **Tool Executor** (`ai/tool_executor.rs`)
   - Executes parsed tool calls
   - Handles timeouts and errors
   - Formats results for both LLM and UI consumption
   - Supports agentic loops (LLM → tool → result → LLM)

### Integration Points in `coach.rs`

#### 1. Global Tool Executor Instance

```rust
static TOOL_EXECUTOR: OnceLock<ToolExecutor> = OnceLock::new();

fn get_tool_executor() -> &'static ToolExecutor {
    TOOL_EXECUTOR.get_or_init(|| {
        info!("Initializing tool executor");
        ToolExecutor::new()
    })
}
```

- Initialized once on first use
- Reused across all coach command invocations
- Contains the tool registry with all available tools

#### 2. Enhanced `try_orchestrated_llm_response` Function

The function now implements the complete tool orchestration flow:

**Step 1: Add Tool Documentation to Prompt**
```rust
let tool_executor = get_tool_executor();
let tool_docs = tool_executor.get_tool_documentation();

let full_prompt = format!(
    "{}\n\n{}\n\n{}",
    tool_docs,
    dynamic_prompt,
    "If you need to take an action (like starting a session or logging a trigger), use the appropriate tool. Otherwise, just respond conversationally."
);
```

**Step 2: Generate LLM Response**
```rust
let llm_response = engine.generate(&full_prompt, 300, 0.7).await?;
```

**Step 3: Parse for Tool Calls**
```rust
let parse_result = tool_executor.parse_output(&llm_response.text);

let response_text = if parse_result.has_tools {
    parse_result.text_without_tools()
} else {
    llm_response.text.clone()
};
```

**Step 4: Execute Tools**
```rust
if parse_result.has_tools {
    match tool_executor.execute_all(&llm_response.text, state.inner()).await {
        Ok(execution_result) => {
            let ui_response = format_for_ui(&execution_result);

            // Combine LLM message with tool execution results
            if !response_text.is_empty() && !ui_response.message.is_empty() {
                final_message = format!("{}\n\n{}", response_text, ui_response.message);
            } else if !ui_response.message.is_empty() {
                final_message = ui_response.message;
            }
        }
        Err(e) => {
            warn!("Tool execution failed: {}", e);
        }
    }
}
```

**Step 5: Return Combined Response**
```rust
Ok(CoachResponse {
    message: final_message,
    suggestions,
})
```

## Tool Call Formats Supported

The parser supports three formats that the LLM can use:

### 1. XML-style (Recommended)
```xml
<tool name="start_focus_session" duration="25"/>
```

### 2. Function-style
```
[[start_focus_session(duration=25)]]
```

### 3. JSON in markdown
```json
{
  "tool": "start_focus_session",
  "params": {"duration": 25}
}
```

## Available Tools

### Session Management
- `start_focus_session` - Start a new focus session
- `end_focus_session` - End the current session
- `get_active_session` - Check active session status

### Analytics
- `get_session_stats` - Get statistics (today or week)
- `get_trigger_patterns` - Analyze distraction patterns
- `get_streak_info` - Get current and longest streak

### Journal
- `log_trigger` - Log a distraction trigger
- `get_trigger_patterns` - View trigger patterns

### Blocking
- `get_blocking_stats` - View blocked apps/websites
- `add_blocked_item` - Add an app or website to block list

### Goals
- `set_focus_goal` - Set daily or weekly focus goal
- `get_streak_info` - Get streak information

## Example Interaction Flow

1. **User**: "I want to focus for 45 minutes"

2. **LLM with tool docs**: Generates response with tool call
   ```
   I'll start a 45-minute focus session for you.

   <tool name="start_focus_session" duration="45"/>

   Good luck staying focused!
   ```

3. **Parser**: Extracts tool call and text
   - Tool: `start_focus_session` with params `{duration: 45}`
   - Text: "I'll start a 45-minute focus session for you. Good luck staying focused!"

4. **Executor**: Runs the tool
   - Checks for active sessions
   - Validates session limits
   - Creates new session
   - Enables blocking
   - Returns: "Started a 45-minute focus session. Stay focused! Blocking 3 apps and 5 websites."

5. **Final Response**: Combined message to user
   ```
   I'll start a 45-minute focus session for you. Good luck staying focused!

   Started a 45-minute focus session. Stay focused! Blocking 3 apps and 5 websites.
   ```

## Benefits

1. **Action-Oriented AI**: The coach can now perform real actions, not just provide advice
2. **Privacy-First**: All execution happens locally, no data sent to external services
3. **Robust Error Handling**: Tools fail gracefully, and the conversation continues
4. **Flexible Parsing**: Supports multiple tool call formats from the LLM
5. **UI Integration**: Results are formatted for display with action indicators
6. **Extensible**: New tools can be easily added to the registry

## Configuration

The executor uses default configuration:
- Max tools per turn: 5
- Max loop depth: 3 (for agentic loops)
- Tool timeout: 30 seconds
- Continue on failure: true

These can be customized by creating an executor with `ExecutorConfig`.

## Testing

The integration has been tested with `cargo check --features local-ai` and compiles successfully.

### Manual Testing Checklist

To test the full integration:

1. Start the app with local-ai feature enabled
2. Ensure LLM model is loaded
3. Send coach messages that should trigger tool use:
   - "Start a 25 minute focus session"
   - "How much have I focused today?"
   - "Log that I felt distracted by social media"
   - "What's my current streak?"
   - "Block Twitter and Instagram"

4. Verify:
   - Tools are executed correctly
   - Results are displayed to user
   - App state is updated (sessions created, triggers logged, etc.)
   - Errors are handled gracefully

## Future Enhancements

1. **Agentic Loop**: Enable multi-turn tool orchestration where the LLM can see tool results and decide on next actions
2. **Tool Chaining**: Allow tools to depend on results from previous tools
3. **Conditional Tools**: Tools that are only available in certain contexts
4. **User Confirmation**: Optional confirmation dialog before executing sensitive tools
5. **Tool History**: Track and display tool execution history in the UI

## Implementation Status

✅ Tool registry with default tools
✅ Tool parser supporting multiple formats
✅ Tool executor with timeout handling
✅ Integration into coach.rs
✅ Prompt augmentation with tool docs
✅ Tool execution after LLM response
✅ Result formatting for UI
✅ Error handling and fallbacks
✅ Compilation verified with cargo check

## Files Modified

- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/commands/coach.rs`
  - Added tool executor initialization
  - Enhanced `try_orchestrated_llm_response` with tool orchestration
  - Added imports for tool system types

## Related Files

- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/ai/tools.rs`
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/ai/tool_parser.rs`
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/ai/tool_executor.rs`
- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/ai/mod.rs`
