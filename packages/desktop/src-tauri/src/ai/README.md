# AI Coach - Local LLM Integration

This module provides 100% local AI coaching using llama.cpp. No data leaves the user's device.

## Architecture

### Components

1. **model_manager.rs** - Model lifecycle management
   - Downloads models from HuggingFace
   - Validates file integrity
   - Manages cache directory
   - Provides progress tracking

2. **llm_engine.rs** - Core inference engine
   - Lazy model loading (only loads when needed)
   - Streaming and non-streaming generation
   - Memory-efficient context management
   - Thread-safe with Arc<RwLock<>>

3. **prompt_templates.rs** - Prompt engineering
   - System prompt with Indistractable framework principles
   - Context-aware user prompts
   - Template-based prompt construction
   - Suggestion generation based on user state

4. **commands/ai.rs** - Frontend API
   - Model management (download, load, unload)
   - Status queries
   - Settings control

## Supported Models

### Phi-3.5-mini (Recommended)
- **Size**: ~2.3GB (Q4_K_M quantization)
- **Parameters**: 3.8B
- **Context**: 4096 tokens
- **Quality**: Best balance of speed and coherence
- **Use case**: Default for most users

### TinyLlama
- **Size**: ~600MB (Q4_K_M quantization)
- **Parameters**: 1.1B
- **Context**: 2048 tokens
- **Quality**: Lower quality, faster inference
- **Use case**: Low-spec machines or testing

## Usage

### From Frontend (TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Check LLM status
const status = await invoke('get_llm_status');
// { enabled: true, model_loaded: false, current_model: null }

// Load model (first-time will be slow)
await invoke('load_model');

// Get coaching response (falls back to templates if model not loaded)
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
// { message: "...", suggestions: [...] }

// Unload model to free memory
await invoke('unload_model');
```

### From Rust Backend

```rust
use crate::ai::{LlmEngine, ModelConfig, build_coach_prompt, PromptTemplate};

// Initialize engine (in AppState)
let models_dir = app_data_dir.join("models");
let engine = LlmEngine::new(models_dir, ModelConfig::phi_3_5_mini())?;

// Load model
engine.load_model().await?;

// Generate response
let prompt = build_coach_prompt(
    PromptTemplate::DailyTip,
    "",
    &user_context
);

let response = engine.generate(&prompt, 200, 0.7).await?;
println!("Generated {} tokens in {}ms",
    response.tokens_generated,
    response.inference_time_ms
);
```

## Model Storage

Models are stored in the app data directory:
- **macOS**: `~/Library/Application Support/com.focusflow.app/models/`
- **Windows**: `%APPDATA%\com.focusflow.app\models\`
- **Linux**: `~/.local/share/com.focusflow.app/models/`

## Performance Characteristics

### Phi-3.5-mini (Q4_K_M)
- **Load time**: 2-5 seconds (depending on disk speed)
- **Memory usage**: ~3GB RAM
- **Generation speed**: 10-30 tokens/sec (CPU), 50-100 tokens/sec (Metal/CUDA)
- **First token latency**: 100-300ms

### TinyLlama (Q4_K_M)
- **Load time**: 1-2 seconds
- **Memory usage**: ~800MB RAM
- **Generation speed**: 20-50 tokens/sec (CPU), 100-200 tokens/sec (Metal/CUDA)
- **First token latency**: 50-150ms

## Fallback Strategy

The coach commands implement graceful degradation:

1. **Try LLM**: Attempt to use loaded model
2. **Fall back to templates**: If model not loaded or error occurs
3. **User experience**: No difference in API, responses are always fast

This ensures the app works even on low-spec machines or if the user disables AI.

## Privacy & Security

- **100% local inference**: No API calls, no data transmission
- **No telemetry**: Model doesn't log or report usage
- **User data stays local**: Context is built from local database only
- **Offline-first**: Works without internet after model download

## Adding New Models

To add a new model variant:

1. Add configuration in `model_manager.rs`:
```rust
impl ModelConfig {
    pub fn new_model() -> Self {
        Self {
            name: "Model Name".to_string(),
            url: "https://huggingface.co/.../model.gguf".to_string(),
            filename: "model-name-q4.gguf".to_string(),
            size_mb: 1500,
            context_size: 4096,
            description: "Description".to_string(),
        }
    }
}
```

2. Update `all_models()` to include it
3. Frontend can now select it via settings

## Troubleshooting

### "Model not loaded" errors
- Model files may not be downloaded yet
- Check `get_llm_status()` to verify model state
- Call `load_model()` explicitly if needed

### Slow first response
- First model load is expensive (2-5 seconds)
- Consider loading model at app startup for power users
- Show loading indicator in UI

### Out of memory crashes
- Reduce to TinyLlama on low-spec machines
- Call `unload_model()` when not in use
- Monitor memory usage via Activity Monitor/Task Manager

### Poor response quality
- Increase max_tokens (default 200)
- Adjust temperature (0.5-0.9 range)
- Improve prompt templates with more context
- Consider upgrading to larger model variant

## Future Enhancements

- [ ] Streaming responses for real-time UX
- [ ] Custom fine-tuned models for productivity coaching
- [ ] Multi-turn conversation support
- [ ] Model quantization options (Q5, Q8 for better quality)
- [ ] GPU acceleration detection and optimization
- [ ] Automatic model updates
- [ ] A/B testing different prompt strategies
