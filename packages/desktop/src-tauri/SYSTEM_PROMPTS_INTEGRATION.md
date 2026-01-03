# System Prompts Integration with Guideline Orchestrator

## Overview

The system prompts module has been successfully integrated with the guideline orchestrator, creating a modular and maintainable prompt architecture.

## Architecture

### Component Responsibilities

1. **`system_prompts.rs`** - Core Prompt Components
   - `CORE_IDENTITY`: AI personality, communication style, boundaries
   - `FRAMEWORK_KNOWLEDGE`: Indistractable principles reference
   - `RESPONSE_GUIDELINES`: Response structure and tone guidelines
   - `TOOL_USAGE_INSTRUCTIONS`: Tool usage patterns and formatting
   - `PromptContext`: Structured user context data type

2. **`guideline_registry.rs`** - Scenario-Specific Guidelines
   - Detailed guidelines for specific scenarios (focus, distraction, motivation, etc.)
   - Keyword matching and condition-based triggering
   - Priority-based guideline selection
   - Tools associated with each guideline

3. **`orchestrator.rs`** - Dynamic Prompt Building
   - Imports base components from `system_prompts.rs`
   - Evaluates guidelines from `guideline_registry.rs`
   - Combines base prompt + matched guidelines + user context
   - Supports optional tool instruction inclusion

### How It Works

```
User Message + Context
         ↓
   Orchestrator
         ↓
   ┌─────────────────────────────────────┐
   │  1. Build Base System Prompt        │
   │     - CORE_IDENTITY                 │
   │     - FRAMEWORK_KNOWLEDGE           │
   │     - RESPONSE_GUIDELINES           │
   │     - User Context (from            │
   │       PromptContext)                │
   └─────────────────────────────────────┘
         ↓
   ┌─────────────────────────────────────┐
   │  2. Evaluate Guidelines             │
   │     - Match keywords                │
   │     - Check conditions              │
   │     - Sort by priority              │
   └─────────────────────────────────────┘
         ↓
   ┌─────────────────────────────────────┐
   │  3. Build Dynamic Prompt            │
   │     - Base system prompt            │
   │     - Tool instructions (optional)  │
   │     - Matched guidelines            │
   │     - User message                  │
   └─────────────────────────────────────┘
         ↓
   Complete Dynamic Prompt
```

## Key Changes

### orchestrator.rs Updates

1. **Added imports from system_prompts:**
   ```rust
   use crate::ai::system_prompts::{
       CORE_IDENTITY, FRAMEWORK_KNOWLEDGE, RESPONSE_GUIDELINES,
       TOOL_USAGE_INSTRUCTIONS, PromptContext,
   };
   ```

2. **Refactored `build_system_prompt()`:**
   - Now uses constants from `system_prompts.rs` instead of hardcoded strings
   - Creates `PromptContext` from `UserContext`
   - Builds user context section using shared logic

3. **Added `build_dynamic_prompt_with_options()`:**
   - Allows optional inclusion/exclusion of tool instructions
   - Default behavior includes tools via `build_dynamic_prompt()`

4. **Added helper methods:**
   - `convert_user_context_to_prompt_context()`: Convert between context types
   - `build_user_context_section()`: Build formatted user context string

### system_prompts.rs Updates

1. **Deprecated duplicate scenario guidelines:**
   - `GUIDELINE_FOCUS_INQUIRY` and other scenario constants marked as deprecated
   - These are now managed by `guideline_registry.rs`
   - Kept for backward compatibility

2. **Kept essential components:**
   - Core prompt constants (CORE_IDENTITY, etc.) - **actively used**
   - `PromptContext` struct - **actively used**
   - `UserIntent` enum - available but not currently used
   - `ScenarioPromptBuilder` - available for simple cases

3. **Updated documentation:**
   - Clear explanation of orchestrator integration
   - Deprecation notices with migration guidance
   - Usage recommendations

## Usage Examples

### Production Usage (Recommended)

```rust
use crate::ai::orchestrator::GuidelineOrchestrator;
use crate::commands::coach::UserContext;

let orchestrator = GuidelineOrchestrator::new();
let context = UserContext { /* ... */ };

// Full orchestration with guideline matching
let result = orchestrator.orchestrate("How can I focus better?", &context);

// Access the built prompt
let prompt = result.dynamic_prompt;

// Access matched guidelines
for guideline in &result.matched_guidelines {
    println!("Matched: {}", guideline.name);
}

// Access suggested tools
for tool in &result.suggested_tools {
    println!("Tool: {}", tool);
}
```

### Custom Prompt Building

```rust
// Build prompt without tool instructions
let prompt = orchestrator.build_dynamic_prompt_with_options(
    "Help me stay focused",
    &context,
    &matched_guidelines,
    false  // exclude tools
);
```

### Type-Specific Prompts

```rust
use crate::ai::orchestrator::PromptType;

// Get prompt for daily tip
let result = orchestrator.orchestrate_for_type(
    PromptType::DailyTip,
    &context,
    None
);
```

## Benefits

1. **Modularity**: Core prompt components are reusable across the system
2. **Maintainability**: Update CORE_IDENTITY in one place, affects all prompts
3. **Flexibility**: Easy to add/remove guideline sections
4. **Type Safety**: `PromptContext` provides structured context data
5. **Testability**: Clear separation of concerns makes testing easier
6. **Dynamic**: Guidelines are matched based on user intent and context

## Migration Notes

### If you were using `build_system_prompt()` directly:

**Before:**
```rust
use crate::ai::system_prompts::{build_system_prompt, PromptContext, UserIntent};

let context = PromptContext { /* ... */ };
let prompt = build_system_prompt(&context, UserIntent::FocusInquiry, true);
```

**After (Recommended):**
```rust
use crate::ai::orchestrator::GuidelineOrchestrator;
use crate::commands::coach::UserContext;

let orchestrator = GuidelineOrchestrator::new();
let context = UserContext { /* ... */ };
let result = orchestrator.orchestrate("How can I focus better?", &context);
let prompt = result.dynamic_prompt;
```

## Testing

All integration is covered by tests in `orchestrator.rs`:

- `test_system_prompts_integration()`: Verifies all core components are included
- `test_tool_instructions_inclusion()`: Verifies optional tool instruction handling
- Plus 7 other tests covering guideline matching and prompt building

Run tests with:
```bash
cargo test --features local-ai orchestrator::tests
```

## Future Improvements

1. **PromptContext Enhancement**: Consider adding more context fields as needed
2. **UserIntent Integration**: Could optionally use `UserIntent::detect()` as a fallback or complement to guideline matching
3. **ScenarioPromptBuilder**: Evaluate if this pattern is useful for simple scenarios or should be fully replaced
4. **Tool Registry Integration**: Further integrate with the tool system for dynamic tool documentation

## Files Modified

- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/ai/orchestrator.rs`
  - Added imports from system_prompts
  - Refactored prompt building to use modular components
  - Added helper methods for context conversion
  - Added integration tests

- `/Users/jacob/projects/focus-app/packages/desktop/src-tauri/src/ai/system_prompts.rs`
  - Deprecated duplicate scenario guidelines
  - Updated module documentation
  - Added deprecation notices and migration guidance
  - Kept core components actively used by orchestrator

## Verification

✅ All tests pass (9/9 orchestrator tests)
✅ `cargo check --features local-ai` succeeds
✅ No breaking changes to public API
✅ Backward compatibility maintained
✅ Documentation updated
