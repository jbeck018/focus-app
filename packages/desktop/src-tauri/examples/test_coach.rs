// examples/test_coach.rs - Test the AI coach response generation
//
// Run with: cargo run --example test_coach --features local-ai
//
// This tests the LLM directly with a "What should I focus on?" query

use focusflow_lib::ai::{LlmEngine, ToolRegistry, ModelConfig};
use focusflow_lib::ai::system_prompts::TOOL_USAGE_INSTRUCTIONS;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("=== AI Coach Test ===\n");

    // Get models directory from environment or use default
    let models_dir = std::env::var("MODELS_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/Library/Application Support/com.focusflow.app/models", home)
    });

    println!("Models directory: {}\n", models_dir);

    // Check if models dir exists
    let models_path = PathBuf::from(&models_dir);
    if !models_path.exists() {
        eprintln!("ERROR: Models directory not found at {}", models_dir);
        eprintln!("Please run the app first to download the model");
        std::process::exit(1);
    }

    // Create LLM engine with Phi-3.5 config
    println!("Loading LLM engine...");
    let model_config = ModelConfig::phi_3_5_mini();
    let engine = LlmEngine::new(models_path, model_config)?;
    engine.load_model().await?;
    println!("Model loaded successfully!\n");

    // Build the test prompt (simulating what coach.rs does)
    let tool_registry = ToolRegistry::with_default_tools();
    let tool_docs = tool_registry.generate_documentation();

    let user_message = "What should I focus on?";

    let system_prompt = r#"You are Focus, a supportive AI coach for FocusFlow.
Your role is to help users stay focused and build better habits.

IMPORTANT RULES:
1. Keep responses SHORT (2-3 sentences max)
2. Be warm and encouraging, not preachy
3. Only use tools when the user asks for an action
4. Never repeat yourself or escalate into long explanations"#;

    let full_prompt = format!(
        "{}\n\n{}\n\n{}\n\nUser: {}\n\nAssistant:",
        system_prompt,
        tool_docs,
        TOOL_USAGE_INSTRUCTIONS,
        user_message
    );

    println!("=== Prompt ({} chars) ===", full_prompt.len());
    println!("{}", full_prompt);
    println!("\n=== Generating Response ===\n");

    // Generate with same settings as coach.rs
    let response = engine.generate(&full_prompt, 800, 0.7).await?;

    println!("=== Response ({} tokens, {}ms) ===", response.tokens_generated, response.inference_time_ms);
    println!("{}", response.text);
    println!("\n=== End Response ===");

    // Check for issues
    let mut issues = Vec::new();

    if response.text.contains("```") {
        issues.push("Contains code blocks (should be plain XML)");
    }
    if response.text.matches("<tool ").count() > 1 {
        issues.push("Multiple tool calls (should be max 1)");
    }
    if response.text.to_lowercase().contains("and also") {
        issues.push("Contains escalation pattern 'and also'");
    }
    if response.text.contains("[SYS]") {
        issues.push("Contains hallucinated [SYS] format");
    }
    if response.text.len() > 1500 {
        issues.push("Response too long (>1500 chars)");
    }

    if issues.is_empty() {
        println!("\n✅ Response looks good!");
    } else {
        println!("\n⚠️  Issues found:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }

    Ok(())
}
