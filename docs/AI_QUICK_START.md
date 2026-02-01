# AI Coach Quick Start Guide

## Build Prerequisites

```bash
# Install CMake (required by llama-cpp-2)
brew install cmake  # macOS
```

## First Build

```bash
cd /Users/jacob/projects/focus-app/packages/desktop/src-tauri
cargo build
```

This will:
1. Download llama-cpp-2 and dependencies
2. Compile llama.cpp from source (takes 5-10 minutes first time)
3. Link everything into your Tauri app

## Model Setup (First Run)

The app initializes the LLM engine but doesn't download models automatically yet.

### Option 1: Manual Download
```bash
# Create models directory
mkdir -p ~/Library/Application\ Support/com.focusflow.app/models

# Download Phi-3.5-mini (recommended)
cd ~/Library/Application\ Support/com.focusflow.app/models
curl -L -O https://huggingface.co/microsoft/Phi-3.5-mini-instruct-gguf/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf

# Rename to expected filename
mv Phi-3.5-mini-instruct-Q4_K_M.gguf phi-3.5-mini-instruct-q4.gguf
```

### Option 2: Via Frontend (Future)
Once the download UI is implemented:
1. Go to Settings → AI Coach
2. Click "Download Model"
3. Select Phi-3.5-mini or TinyLlama
4. Wait for download to complete

## Usage in Code

### Check Status
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const status = await invoke('get_llm_status');
console.log(status);
// { enabled: true, model_loaded: false, current_model: null }
```

### Load Model
```typescript
// Load into memory (takes 2-5 seconds)
await invoke('load_model');

// Check again
const status = await invoke('get_llm_status');
// { enabled: true, model_loaded: true, current_model: "Phi-3.5-mini" }
```

### Get Coaching Response
```typescript
const response = await invoke('get_coach_response', {
  message: "How can I focus better?",
  context: {
    total_focus_hours_today: 2.5,
    sessions_completed_today: 3,
    current_streak_days: 5,
    top_trigger: "social media",
    average_session_minutes: 25
  }
});

console.log(response.message);
// "Based on your patterns, you're doing well with 25-minute sessions.
//  Your main trigger is social media - consider blocking it before your
//  next session, not during. What specific outcome do you want from
//  your next session?"

console.log(response.suggestions);
// ["Start a focus session", "Update blocked apps", "View my progress"]
```

### All Available Commands
```typescript
// AI Coach (automatically uses LLM or falls back to templates)
await invoke('get_coach_response', { message, context });
await invoke('get_daily_tip');
await invoke('get_session_advice', { planned_duration_minutes: 25 });
await invoke('get_reflection_prompt', { session_completed: true, actual_duration_minutes: 25 });
await invoke('analyze_patterns');

// LLM Management
await invoke('get_llm_status');
await invoke('load_model');
await invoke('unload_model');
await invoke('toggle_ai_coach', { enabled: true });
await invoke('get_available_models');
await invoke('is_model_downloaded', { model_name: "Phi-3.5-mini" });
```

## Key Files Reference

```
packages/desktop/src-tauri/src/
├── ai/
│   ├── mod.rs                    # Module exports
│   ├── model_manager.rs          # Download & cache management
│   ├── llm_engine.rs             # Core inference engine
│   ├── prompt_templates.rs       # Prompt engineering
│   └── README.md                 # Detailed documentation
│
├── commands/
│   ├── ai.rs                     # Model management commands
│   └── coach.rs                  # Coaching commands (LLM + fallback)
│
└── state.rs                      # AppState with llm_engine field
```

## Performance Tips

1. **Preload on startup** (for power users):
   ```typescript
   // In your main app initialization
   invoke('load_model').catch(err => {
     console.log('Model preload failed, will load on first use');
   });
   ```

2. **Unload when not needed**:
   ```typescript
   // When user closes AI coach panel
   await invoke('unload_model'); // Frees 2-3GB RAM
   ```

3. **Use TinyLlama for low-spec machines**:
   - Detect available memory in settings
   - Auto-select TinyLlama if RAM < 8GB
   - User can override in preferences

## Debugging

### Enable detailed logging
```bash
RUST_LOG=focusflow=debug cargo run
```

### Check model loading
```typescript
const status = await invoke('get_llm_status');
if (!status.model_loaded) {
  console.error('Model not loaded');
  // Show loading UI or fallback message
}
```

### Monitor performance
```rust
// In llm_engine.rs, logging is already built-in:
info!("Generated {} tokens in {:.2}s ({:.1} tok/s)",
    n_generated,
    duration.as_secs_f64(),
    tokens_per_sec
);
```

## Common Issues

| Issue | Solution |
|-------|----------|
| "cmake not found" | Install CMake: `brew install cmake` |
| "Model not loaded" | Call `load_model()` or download model file |
| Slow first response | Normal - model load takes 2-5 seconds |
| Out of memory | Use TinyLlama or unload when not needed |
| Template responses | Model not loaded - check `get_llm_status()` |

## What Works Now

✅ LLM engine initialization
✅ Model loading/unloading
✅ Response generation with context
✅ Fallback to templates
✅ All coaching commands enhanced
✅ Frontend API complete
✅ Error handling
✅ Memory management

## What's Pending

⏳ Automatic model download (manual works)
⏳ Progress tracking for downloads
⏳ Streaming responses UI
⏳ Multi-turn conversations
⏳ GPU acceleration detection

## Next Steps

1. **Install CMake** (if not done)
2. **Run `cargo build`** to compile
3. **Download a model** (Phi-3.5-mini recommended)
4. **Test with frontend** using the commands above
5. **Add loading UI** for model operations
6. **Implement settings page** for model selection

## Support

See full documentation in:
- `/packages/desktop/INTEGRATION_SUMMARY.md` - Complete implementation details
- `/packages/desktop/src-tauri/src/ai/README.md` - AI module documentation
