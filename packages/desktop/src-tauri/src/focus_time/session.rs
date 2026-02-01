// focus_time/session.rs - Focus Time session management
//
// This module handles the Focus Time session lifecycle:
// 1. Activation from calendar events
// 2. Override handling during sessions
// 3. Session end management

use crate::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::parser::FocusTimeConfig;

/// State of a Focus Time session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusTimeState {
    /// No Focus Time active
    Inactive,
    /// Focus Time active from calendar
    ActiveFromCalendar,
    /// Focus Time manually overridden (ended early)
    ManuallyEnded,
    /// Focus Time manually started (not from calendar)
    ManuallyStarted,
}

impl Default for FocusTimeState {
    fn default() -> Self {
        Self::Inactive
    }
}

/// A Focus Time session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTimeSession {
    /// Unique session identifier
    pub id: String,
    /// Current state
    pub state: FocusTimeState,
    /// When the session started
    pub start_time: DateTime<Utc>,
    /// When the session is scheduled to end
    pub end_time: DateTime<Utc>,
    /// Apps allowed during this session (inverse blocklist)
    pub allowed_apps: Vec<String>,
    /// Calendar event ID that triggered this session (if from calendar)
    pub source_event_id: Option<String>,
    /// Whether this session was manually overridden
    pub was_overridden: bool,
    /// Time of override (if overridden)
    pub override_time: Option<DateTime<Utc>>,
    /// Configuration used for this session
    pub config: FocusTimeConfig,
}

impl FocusTimeSession {
    /// Create a new Focus Time session from a calendar event
    pub fn from_calendar_event(
        event_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        config: FocusTimeConfig,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: FocusTimeState::ActiveFromCalendar,
            start_time,
            end_time,
            allowed_apps: config.allowed_apps.clone(),
            source_event_id: Some(event_id.to_string()),
            was_overridden: false,
            override_time: None,
            config,
        }
    }

    /// Create a new manually-started Focus Time session
    pub fn manual(duration_minutes: i32, allowed_apps: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: FocusTimeState::ManuallyStarted,
            start_time: now,
            end_time: now + chrono::Duration::minutes(duration_minutes as i64),
            allowed_apps: allowed_apps.clone(),
            source_event_id: None,
            was_overridden: false,
            override_time: None,
            config: FocusTimeConfig {
                is_focus_time: true,
                allowed_apps,
                allowed_categories: Vec::new(),
                raw_allowed_apps: Vec::new(),
                source_title: "Manual Focus Time".to_string(),
            },
        }
    }

    /// Check if the session has ended based on time
    pub fn has_ended(&self) -> bool {
        match self.state {
            FocusTimeState::Inactive | FocusTimeState::ManuallyEnded => true,
            FocusTimeState::ActiveFromCalendar | FocusTimeState::ManuallyStarted => {
                Utc::now() >= self.end_time
            }
        }
    }

    /// Check if the session is currently active
    pub fn is_active(&self) -> bool {
        match self.state {
            FocusTimeState::Inactive | FocusTimeState::ManuallyEnded => false,
            FocusTimeState::ActiveFromCalendar | FocusTimeState::ManuallyStarted => {
                !self.has_ended()
            }
        }
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> i64 {
        if !self.is_active() {
            return 0;
        }
        let remaining = self.end_time - Utc::now();
        remaining.num_seconds().max(0)
    }

    /// End the session early (override)
    pub fn end_early(&mut self) {
        self.was_overridden = true;
        self.override_time = Some(Utc::now());
        self.state = FocusTimeState::ManuallyEnded;
    }
}

/// Manager for Focus Time sessions
pub struct FocusTimeManager {
    current_session: Arc<RwLock<Option<FocusTimeSession>>>,
}

impl FocusTimeManager {
    /// Create a new Focus Time manager
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current session if any
    pub async fn get_session(&self) -> Option<FocusTimeSession> {
        let session = self.current_session.read().await;
        session.clone()
    }

    /// Start a Focus Time session from a calendar event
    pub async fn start_from_calendar(
        &self,
        event_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        config: FocusTimeConfig,
    ) -> Result<FocusTimeSession> {
        let session = FocusTimeSession::from_calendar_event(event_id, start_time, end_time, config);

        let mut current = self.current_session.write().await;
        *current = Some(session.clone());

        Ok(session)
    }

    /// Start a manual Focus Time session
    pub async fn start_manual(
        &self,
        duration_minutes: i32,
        allowed_apps: Vec<String>,
    ) -> Result<FocusTimeSession> {
        let session = FocusTimeSession::manual(duration_minutes, allowed_apps);

        let mut current = self.current_session.write().await;
        *current = Some(session.clone());

        Ok(session)
    }

    /// End the current session early (override)
    pub async fn end_early(&self) -> Result<Option<FocusTimeSession>> {
        let mut current = self.current_session.write().await;

        if let Some(ref mut session) = *current {
            session.end_early();
            let ended_session = session.clone();
            *current = None;
            return Ok(Some(ended_session));
        }

        Ok(None)
    }

    /// Check and clean up expired sessions
    pub async fn check_expiry(&self) -> Option<FocusTimeSession> {
        let mut current = self.current_session.write().await;

        if let Some(ref session) = *current {
            if session.has_ended() {
                let ended_session = session.clone();
                *current = None;
                return Some(ended_session);
            }
        }

        None
    }

    /// Check if there's an active Focus Time session
    pub async fn is_active(&self) -> bool {
        let current = self.current_session.read().await;
        current.as_ref().map(|s| s.is_active()).unwrap_or(false)
    }

    /// Get allowed apps for the current session
    pub async fn get_allowed_apps(&self) -> Vec<String> {
        let current = self.current_session.read().await;
        current
            .as_ref()
            .map(|s| s.allowed_apps.clone())
            .unwrap_or_default()
    }
}

impl Default for FocusTimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;

    #[test]
    fn test_session_from_calendar() {
        let config = FocusTimeConfig {
            is_focus_time: true,
            allowed_apps: vec!["Code".to_string(), "Terminal".to_string()],
            allowed_categories: Vec::new(),
            raw_allowed_apps: Vec::new(),
            source_title: "Focus Time".to_string(),
        };

        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);

        let session = FocusTimeSession::from_calendar_event("event-123", start, end, config);

        assert_eq!(session.state, FocusTimeState::ActiveFromCalendar);
        assert_eq!(session.source_event_id, Some("event-123".to_string()));
        assert!(!session.was_overridden);
        assert!(session.is_active());
    }

    #[test]
    fn test_session_manual() {
        let session = FocusTimeSession::manual(25, vec!["Code".to_string()]);

        assert_eq!(session.state, FocusTimeState::ManuallyStarted);
        assert!(session.source_event_id.is_none());
        assert!(session.is_active());
        assert!(session.remaining_seconds() > 0);
    }

    #[test]
    fn test_session_end_early() {
        let mut session = FocusTimeSession::manual(25, vec![]);

        assert!(session.is_active());

        session.end_early();

        assert!(!session.is_active());
        assert!(session.was_overridden);
        assert!(session.override_time.is_some());
        assert_eq!(session.state, FocusTimeState::ManuallyEnded);
    }

    #[test]
    fn test_session_expiry() {
        let config = FocusTimeConfig::default();
        let start = Utc::now() - chrono::Duration::hours(2);
        let end = Utc::now() - chrono::Duration::hours(1);

        let session = FocusTimeSession::from_calendar_event("event-123", start, end, config);

        assert!(session.has_ended());
        assert!(!session.is_active());
        assert_eq!(session.remaining_seconds(), 0);
    }

    #[tokio::test]
    async fn test_manager_lifecycle() {
        let manager = FocusTimeManager::new();

        // Initially no session
        assert!(!manager.is_active().await);
        assert!(manager.get_session().await.is_none());

        // Start a manual session
        let _session = manager
            .start_manual(25, vec!["Code".to_string()])
            .await
            .unwrap();

        assert!(manager.is_active().await);
        assert!(manager.get_session().await.is_some());

        let allowed = manager.get_allowed_apps().await;
        assert!(allowed.contains(&"Code".to_string()));

        // End early
        let ended = manager.end_early().await.unwrap();
        assert!(ended.is_some());
        assert!(ended.unwrap().was_overridden);

        // Session is now inactive
        assert!(!manager.is_active().await);
    }

    #[tokio::test]
    async fn test_manager_calendar_session() {
        let manager = FocusTimeManager::new();

        let config = FocusTimeConfig {
            is_focus_time: true,
            allowed_apps: vec!["Terminal".to_string()],
            allowed_categories: Vec::new(),
            raw_allowed_apps: Vec::new(),
            source_title: "Deep Work".to_string(),
        };

        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);

        let session = manager
            .start_from_calendar("cal-event-456", start, end, config)
            .await
            .unwrap();

        assert_eq!(session.state, FocusTimeState::ActiveFromCalendar);
        assert_eq!(session.source_event_id, Some("cal-event-456".to_string()));

        let current = manager.get_session().await.unwrap();
        assert_eq!(current.id, session.id);
    }
}
