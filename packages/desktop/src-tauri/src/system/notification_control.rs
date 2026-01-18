// system/notification_control.rs - System notification pausing for focus mode
//
// Integrates with platform-specific Do Not Disturb / Focus Assist APIs to
// temporarily pause notifications during focus sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Notification control state for pausing system notifications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationControlState {
    /// Whether notifications are currently paused
    pub paused: bool,
    /// Session ID this pause is tied to
    pub session_id: Option<String>,
    /// When notifications were paused
    pub paused_at: Option<DateTime<Utc>>,
    /// Previous DND state to restore when resuming
    pub previous_dnd_enabled: Option<bool>,
    /// Platform-specific restoration data
    pub restore_data: Option<String>,
}

impl NotificationControlState {
    /// Mark notifications as paused for a session
    pub fn pause(&mut self, session_id: String, previous_dnd: Option<bool>) {
        self.paused = true;
        self.session_id = Some(session_id);
        self.paused_at = Some(Utc::now());
        self.previous_dnd_enabled = previous_dnd;
    }

    /// Resume notifications and clear state
    pub fn resume(&mut self) {
        self.paused = false;
        self.session_id = None;
        self.paused_at = None;
        // Keep previous_dnd_enabled for reference until next pause
    }

    /// Store platform-specific restoration data
    pub fn set_restore_data(&mut self, data: String) {
        self.restore_data = Some(data);
    }

    /// Clear all state
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Get duration notifications have been paused (if paused)
    pub fn pause_duration_seconds(&self) -> Option<i64> {
        self.paused_at.map(|paused_at| {
            (Utc::now() - paused_at).num_seconds()
        })
    }
}

/// Platform notification permission status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPermissionStatus {
    /// Whether the app can control DND/Focus Assist
    pub can_control_dnd: bool,
    /// Human-readable permission state
    pub permission_state: PermissionState,
    /// Platform-specific notes about permissions
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
    Unavailable,
}

impl Default for NotificationPermissionStatus {
    fn default() -> Self {
        Self {
            can_control_dnd: false,
            permission_state: PermissionState::NotDetermined,
            notes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_state_default() {
        let state = NotificationControlState::default();
        assert!(!state.paused);
        assert!(state.session_id.is_none());
        assert!(state.paused_at.is_none());
    }

    #[test]
    fn test_pause_and_resume() {
        let mut state = NotificationControlState::default();

        state.pause("session-456".to_string(), Some(false));
        assert!(state.paused);
        assert_eq!(state.session_id, Some("session-456".to_string()));
        assert!(state.paused_at.is_some());
        assert_eq!(state.previous_dnd_enabled, Some(false));

        state.resume();
        assert!(!state.paused);
        assert!(state.session_id.is_none());
        assert!(state.paused_at.is_none());
        // previous_dnd_enabled preserved for reference
        assert_eq!(state.previous_dnd_enabled, Some(false));
    }

    #[test]
    fn test_pause_duration() {
        let mut state = NotificationControlState::default();
        assert!(state.pause_duration_seconds().is_none());

        state.pause("session".to_string(), None);
        // Should have some duration (at least 0)
        let duration = state.pause_duration_seconds();
        assert!(duration.is_some());
        assert!(duration.unwrap() >= 0);
    }

    #[test]
    fn test_restore_data() {
        let mut state = NotificationControlState::default();
        state.set_restore_data("{\"focus_mode\": \"Work\"}".to_string());
        assert_eq!(state.restore_data, Some("{\"focus_mode\": \"Work\"}".to_string()));
    }
}
