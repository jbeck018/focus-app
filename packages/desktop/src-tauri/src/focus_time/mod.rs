// focus_time/mod.rs - Calendar-based Focus Time module
//
// This module handles automatic Focus Time activation based on calendar events.
// It provides:
// 1. Detection of Focus Time events from calendar data
// 2. Parsing of allowed apps from event descriptions
// 3. "Inverse blocking" - blocking all apps EXCEPT those explicitly allowed
// 4. App category expansion (e.g., @coding -> vscode, terminal, etc.)
// 5. Scheduler for automatic activation/deactivation

pub mod app_registry;
pub mod parser;
pub mod session;

#[cfg(test)]
mod focus_time_tests;

use crate::{commands::calendar::CalendarEvent, Error, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export commonly used types
pub use parser::{parse_focus_time_event, FocusTimeConfig, AppCategory, normalize_app_name, is_app_allowed};
pub use app_registry::AppRegistry;

/// Focus Time state managed separately from regular blocking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusTimeState {
    /// Whether a Focus Time session is currently active
    pub active: bool,
    /// The calendar event ID that triggered this Focus Time
    pub event_id: Option<String>,
    /// Event title for display purposes
    pub event_title: Option<String>,
    /// When the Focus Time started
    pub started_at: Option<DateTime<Utc>>,
    /// When the Focus Time is scheduled to end
    pub ends_at: Option<DateTime<Utc>>,
    /// Apps allowed during this Focus Time (inverse of blocklist)
    pub allowed_apps: Vec<String>,
    /// Original allowed apps from calendar event (before overrides)
    pub original_allowed_apps: Vec<String>,
    /// Apps added during the session (overrides)
    pub added_apps: Vec<String>,
    /// Apps removed during the session (overrides)
    pub removed_apps: Vec<String>,
    /// Whether this was manually started (vs auto-triggered by scheduler)
    pub manually_started: bool,
    /// Whether this was ended early by user
    pub ended_early: bool,
}

impl FocusTimeState {
    /// Create a new Focus Time state from a parsed event
    pub fn from_parsed_event(event: &FocusTimeEventParsed) -> Self {
        Self {
            active: true,
            event_id: Some(event.id.clone()),
            event_title: Some(event.clean_title.clone()),
            started_at: Some(Utc::now()),
            ends_at: Some(event.end_time),
            allowed_apps: event.allowed_apps.clone(),
            original_allowed_apps: event.allowed_apps.clone(),
            added_apps: Vec::new(),
            removed_apps: Vec::new(),
            manually_started: false,
            ended_early: false,
        }
    }

    /// Add an app to the allowed list during active session
    pub fn add_allowed_app(&mut self, app: &str) {
        let app_lower = app.to_lowercase();
        if !self.allowed_apps.iter().any(|a| a.to_lowercase() == app_lower) {
            self.allowed_apps.push(app.to_string());
            if !self.added_apps.iter().any(|a| a.to_lowercase() == app_lower) {
                self.added_apps.push(app.to_string());
            }
        }
    }

    /// Remove an app from the allowed list during active session
    pub fn remove_allowed_app(&mut self, app: &str) {
        let app_lower = app.to_lowercase();
        self.allowed_apps.retain(|a| a.to_lowercase() != app_lower);
        if !self.removed_apps.iter().any(|a| a.to_lowercase() == app_lower) {
            self.removed_apps.push(app.to_string());
        }
    }

    /// Reset overrides and restore original allowed apps
    pub fn reset_overrides(&mut self) {
        self.allowed_apps = self.original_allowed_apps.clone();
        self.added_apps.clear();
        self.removed_apps.clear();
    }

    /// End the Focus Time session
    pub fn end(&mut self, early: bool) {
        self.active = false;
        self.ended_early = early;
    }

    /// Check if a process/app is allowed during this Focus Time
    pub fn is_app_allowed(&self, app_name: &str) -> bool {
        if !self.active {
            return true; // If not active, all apps are allowed
        }

        // Use the parser's is_app_allowed function for consistent matching
        is_app_allowed(app_name, &self.allowed_apps)
    }
}

/// A calendar event parsed as a Focus Time block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusTimeEventParsed {
    /// Calendar event ID
    pub id: String,
    /// Event title (may include focus keywords)
    pub title: String,
    /// Clean title without Focus Time indicators
    pub clean_title: String,
    /// Event description
    pub description: Option<String>,
    /// Event start time
    pub start_time: DateTime<Utc>,
    /// Event end time
    pub end_time: DateTime<Utc>,
    /// Duration in minutes
    pub duration_minutes: i32,
    /// Apps allowed during this Focus Time
    pub allowed_apps: Vec<String>,
    /// Raw allowed apps string from description (for debugging)
    pub raw_allowed_apps: Option<String>,
    /// Categories detected in the event
    pub categories: Vec<String>,
    /// Whether this event is currently active
    pub is_active: bool,
    /// Whether this event is upcoming (starts within next hour)
    pub is_upcoming: bool,
    /// Source calendar provider
    pub source: String,
}

impl FocusTimeEventParsed {
    /// Create from a CalendarEvent
    pub fn from_calendar_event(event: &CalendarEvent, config: &FocusTimeConfig) -> Option<Self> {
        if !config.is_focus_time {
            return None;
        }

        let start_time = DateTime::parse_from_rfc3339(&event.start_time)
            .ok()?
            .with_timezone(&Utc);
        let end_time = DateTime::parse_from_rfc3339(&event.end_time)
            .ok()?
            .with_timezone(&Utc);

        let duration_minutes = (end_time - start_time).num_minutes() as i32;
        let now = Utc::now();
        let is_active = now >= start_time && now < end_time;
        let is_upcoming = start_time > now && start_time <= now + Duration::hours(1);

        // Clean the title (remove focus keywords prefix if present)
        let clean_title = clean_focus_title(&event.title);

        Some(Self {
            id: event.id.clone(),
            title: event.title.clone(),
            clean_title,
            description: event.description.clone(),
            start_time,
            end_time,
            duration_minutes,
            allowed_apps: config.allowed_apps.clone(),
            raw_allowed_apps: if config.raw_allowed_apps.is_empty() {
                None
            } else {
                Some(config.raw_allowed_apps.join(", "))
            },
            categories: config
                .allowed_categories
                .iter()
                .map(|c| format!("{:?}", c))
                .collect(),
            is_active,
            is_upcoming,
            source: "calendar".to_string(),
        })
    }

    /// Check if this event is currently active based on time
    pub fn check_is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.start_time && now < self.end_time
    }

    /// Check if this event is upcoming (within next 60 minutes)
    pub fn check_is_upcoming(&self) -> bool {
        let now = Utc::now();
        let one_hour_later = now + Duration::hours(1);
        self.start_time > now && self.start_time <= one_hour_later
    }
}

/// Clean a focus time event title by removing common prefixes
fn clean_focus_title(title: &str) -> String {
    let title = title.trim();

    // Remove common prefixes
    let prefixes = [
        "[Focus]", "[FOCUS]", "[focus]",
        "[Focus Time]", "[FOCUS TIME]",
        "[Deep Work]", "[DEEP WORK]",
        "Focus:", "Focus -", "Focus Time:",
        "Deep Work:", "Deep Work -",
    ];

    for prefix in prefixes {
        if let Some(stripped) = title.strip_prefix(prefix) {
            return stripped.trim().to_string();
        }
    }

    title.to_string()
}

/// Manager for Focus Time functionality
pub struct FocusTimeManager {
    state: Arc<RwLock<FocusTimeState>>,
}

impl FocusTimeManager {
    /// Create a new FocusTimeManager
    pub fn new(state: Arc<RwLock<FocusTimeState>>) -> Self {
        Self { state }
    }

    /// Get the current Focus Time state
    pub async fn get_state(&self) -> FocusTimeState {
        self.state.read().await.clone()
    }

    /// Check if Focus Time is currently active
    pub async fn is_active(&self) -> bool {
        self.state.read().await.active
    }

    /// Get the list of currently allowed apps
    pub async fn get_allowed_apps(&self) -> Vec<String> {
        self.state.read().await.allowed_apps.clone()
    }

    /// Start a Focus Time session from a parsed calendar event
    pub async fn start_from_event(&self, event: &FocusTimeEventParsed, manual: bool) -> Result<()> {
        let mut state = self.state.write().await;

        if state.active {
            return Err(Error::InvalidSession(
                "Focus Time is already active".to_string()
            ));
        }

        *state = FocusTimeState::from_parsed_event(event);
        state.manually_started = manual;

        tracing::info!(
            "Focus Time started from event '{}' with {} allowed apps (manual: {})",
            event.clean_title,
            state.allowed_apps.len(),
            manual
        );

        Ok(())
    }

    /// End the current Focus Time session
    pub async fn end(&self, early: bool) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.active {
            return Err(Error::InvalidSession(
                "No active Focus Time to end".to_string()
            ));
        }

        state.end(early);
        tracing::info!("Focus Time ended (early: {})", early);

        Ok(())
    }

    /// Add an app to the allowed list during active session
    pub async fn add_allowed_app(&self, app: &str) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.active {
            return Err(Error::InvalidSession(
                "No active Focus Time session".to_string()
            ));
        }

        state.add_allowed_app(app);
        tracing::debug!("Added '{}' to Focus Time allowed apps", app);

        Ok(())
    }

    /// Remove an app from the allowed list during active session
    pub async fn remove_allowed_app(&self, app: &str) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.active {
            return Err(Error::InvalidSession(
                "No active Focus Time session".to_string()
            ));
        }

        state.remove_allowed_app(app);
        tracing::debug!("Removed '{}' from Focus Time allowed apps", app);

        Ok(())
    }

    /// Check if a process is allowed during Focus Time
    pub async fn is_process_allowed(&self, process_name: &str) -> bool {
        let state = self.state.read().await;
        state.is_app_allowed(process_name)
    }

    /// Auto-deactivate if Focus Time has ended based on calendar end time
    pub async fn check_auto_deactivate(&self) -> bool {
        let mut state = self.state.write().await;

        if !state.active {
            return false;
        }

        if let Some(ends_at) = state.ends_at {
            if Utc::now() >= ends_at {
                state.end(false);
                tracing::info!("Focus Time auto-deactivated (scheduled end time reached)");
                return true;
            }
        }

        false
    }

    /// Reset to inactive state (used during cleanup)
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = FocusTimeState::default();
    }
}

/// Detect Focus Time events from a list of calendar events
pub fn detect_focus_time_events(events: &[CalendarEvent]) -> Vec<FocusTimeEventParsed> {
    events
        .iter()
        .filter_map(|event| {
            let config = parse_focus_time_event(&event.title, event.description.as_deref());
            FocusTimeEventParsed::from_calendar_event(event, &config)
        })
        .collect()
}

/// Find the currently active Focus Time event from a list
pub fn find_active_focus_time(events: &[FocusTimeEventParsed]) -> Option<FocusTimeEventParsed> {
    events.iter().find(|e| e.check_is_active()).cloned()
}

/// Find upcoming Focus Time events (within next hour)
pub fn find_upcoming_focus_times(events: &[FocusTimeEventParsed]) -> Vec<FocusTimeEventParsed> {
    events
        .iter()
        .filter(|e| e.check_is_upcoming())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_focus_title() {
        assert_eq!(clean_focus_title("[Focus] Deep Work"), "Deep Work");
        assert_eq!(clean_focus_title("[FOCUS] Coding Session"), "Coding Session");
        assert_eq!(clean_focus_title("[Deep Work] Writing"), "Writing");
        assert_eq!(clean_focus_title("Focus: Project Work"), "Project Work");
        assert_eq!(clean_focus_title("Regular Meeting"), "Regular Meeting");
    }

    #[test]
    fn test_focus_time_state_allowed_apps() {
        let mut state = FocusTimeState {
            active: true,
            allowed_apps: vec!["vscode".to_string(), "terminal".to_string()],
            ..Default::default()
        };

        assert!(state.is_app_allowed("vscode"));
        assert!(state.is_app_allowed("VSCode"));
        assert!(state.is_app_allowed("terminal"));
        assert!(!state.is_app_allowed("chrome"));

        // Test adding apps
        state.add_allowed_app("slack");
        assert!(state.is_app_allowed("slack"));

        // Test removing apps
        state.remove_allowed_app("vscode");
        assert!(!state.is_app_allowed("vscode"));
    }

    #[test]
    fn test_focus_time_state_inactive() {
        let state = FocusTimeState {
            active: false,
            allowed_apps: vec!["vscode".to_string()],
            ..Default::default()
        };

        // When inactive, all apps should be allowed
        assert!(state.is_app_allowed("chrome"));
        assert!(state.is_app_allowed("slack"));
    }
}
