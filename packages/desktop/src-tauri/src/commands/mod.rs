// commands/mod.rs - Command modules aggregation

pub mod achievements;
pub mod ai;
pub mod ai_providers;
pub mod analytics;
pub mod auth;
pub mod blocking;
pub mod calendar;
pub mod chat_context;
pub mod chat_history;
pub mod coach;
pub mod credentials;
pub mod focus;
pub mod journal;
pub mod llm;
pub mod onboarding;
pub mod streaks;
pub mod sync;
pub mod team;
pub mod team_sync;
pub mod timer;
pub mod window;

// Rename blocking-advanced.rs to blocking_advanced for module import
#[path = "blocking-advanced.rs"]
pub mod blocking_advanced;
