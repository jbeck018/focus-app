// system/notification_control.rs - System notification pausing for focus mode
//
// Integrates with platform-specific Do Not Disturb / Focus Assist APIs to
// temporarily pause notifications during focus sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of a DND operation with detailed status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DndOperationResult {
    /// Whether the operation succeeded at the system level
    pub success: bool,
    /// Human-readable status message
    pub message: String,
    /// The method that was used (or attempted)
    pub method: DndMethod,
    /// Whether the app's internal state was updated regardless of system success
    pub state_updated: bool,
    /// Suggested user action if the operation failed
    pub user_action: Option<String>,
}

impl DndOperationResult {
    pub fn success(method: DndMethod, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            method,
            state_updated: true,
            user_action: None,
        }
    }

    pub fn partial(method: DndMethod, message: impl Into<String>, user_action: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            method,
            state_updated: true,
            user_action: Some(user_action.into()),
        }
    }

    pub fn failure(method: DndMethod, message: impl Into<String>, user_action: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            method,
            state_updated: false,
            user_action: Some(user_action.into()),
        }
    }
}

/// Method used for DND control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DndMethod {
    /// macOS Shortcuts app
    MacosShortcuts,
    /// macOS AppleScript (legacy)
    MacosApplescript,
    /// Windows Focus Assist via registry
    WindowsRegistry,
    /// Windows Focus Assist via WNF
    WindowsWnf,
    /// Linux dunst notification daemon
    LinuxDunst,
    /// Linux GNOME desktop
    LinuxGnome,
    /// Linux KDE Plasma
    LinuxKde,
    /// No method available
    Unavailable,
    /// Manual/fallback (user must enable manually)
    Manual,
}

/// Notification control state for pausing system notifications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationControlState {
    /// Whether notifications are currently paused (app state)
    pub paused: bool,
    /// Session ID this pause is tied to
    pub session_id: Option<String>,
    /// When notifications were paused
    pub paused_at: Option<DateTime<Utc>>,
    /// Previous DND state to restore when resuming
    pub previous_dnd_enabled: Option<bool>,
    /// Platform-specific restoration data
    pub restore_data: Option<String>,
    /// Last DND operation result (for user feedback)
    pub last_operation_result: Option<DndOperationResult>,
    /// Whether system DND was actually enabled (may differ from paused if system API failed)
    pub system_dnd_enabled: bool,
    /// Count of failed DND operations in this session
    pub failure_count: u32,
}

impl NotificationControlState {
    /// Mark notifications as paused for a session
    pub fn pause(&mut self, session_id: String, previous_dnd: Option<bool>, operation_result: DndOperationResult) {
        self.paused = true;
        self.session_id = Some(session_id);
        self.paused_at = Some(Utc::now());
        self.previous_dnd_enabled = previous_dnd;
        self.system_dnd_enabled = operation_result.success;
        if !operation_result.success {
            self.failure_count += 1;
        }
        self.last_operation_result = Some(operation_result);
    }

    /// Resume notifications and clear state
    pub fn resume(&mut self, operation_result: Option<DndOperationResult>) {
        self.paused = false;
        self.session_id = None;
        self.paused_at = None;
        self.system_dnd_enabled = false;
        if let Some(result) = operation_result {
            if !result.success {
                self.failure_count += 1;
            }
            self.last_operation_result = Some(result);
        }
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

    /// Check if DND is actually working at the system level
    pub fn is_system_dnd_active(&self) -> bool {
        self.paused && self.system_dnd_enabled
    }

    /// Get user-facing status message
    pub fn status_message(&self) -> String {
        if !self.paused {
            "Notifications are enabled".to_string()
        } else if self.system_dnd_enabled {
            "Do Not Disturb is active".to_string()
        } else {
            "Focus mode active (DND could not be enabled automatically - consider enabling manually)".to_string()
        }
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

    fn success_result() -> DndOperationResult {
        DndOperationResult::success(DndMethod::Manual, "Test success")
    }

    fn failure_result() -> DndOperationResult {
        DndOperationResult::partial(DndMethod::Manual, "Test failure", "Enable DND manually")
    }

    #[test]
    fn test_notification_state_default() {
        let state = NotificationControlState::default();
        assert!(!state.paused);
        assert!(state.session_id.is_none());
        assert!(state.paused_at.is_none());
        assert!(!state.system_dnd_enabled);
        assert_eq!(state.failure_count, 0);
    }

    #[test]
    fn test_pause_and_resume_success() {
        let mut state = NotificationControlState::default();

        state.pause("session-456".to_string(), Some(false), success_result());
        assert!(state.paused);
        assert!(state.system_dnd_enabled);
        assert_eq!(state.session_id, Some("session-456".to_string()));
        assert!(state.paused_at.is_some());
        assert_eq!(state.previous_dnd_enabled, Some(false));
        assert_eq!(state.failure_count, 0);

        state.resume(Some(success_result()));
        assert!(!state.paused);
        assert!(!state.system_dnd_enabled);
        assert!(state.session_id.is_none());
        assert!(state.paused_at.is_none());
        // previous_dnd_enabled preserved for reference
        assert_eq!(state.previous_dnd_enabled, Some(false));
    }

    #[test]
    fn test_pause_failure_tracking() {
        let mut state = NotificationControlState::default();

        state.pause("session-789".to_string(), None, failure_result());
        assert!(state.paused);
        assert!(!state.system_dnd_enabled);
        assert_eq!(state.failure_count, 1);
        assert!(state.last_operation_result.is_some());
        assert!(!state.last_operation_result.as_ref().unwrap().success);
    }

    #[test]
    fn test_is_system_dnd_active() {
        let mut state = NotificationControlState::default();

        // Not paused = not active
        assert!(!state.is_system_dnd_active());

        // Paused but DND failed = not active at system level
        state.pause("session".to_string(), None, failure_result());
        assert!(!state.is_system_dnd_active());

        // Clear and try with success
        state.clear();
        state.pause("session".to_string(), None, success_result());
        assert!(state.is_system_dnd_active());
    }

    #[test]
    fn test_status_message() {
        let mut state = NotificationControlState::default();
        assert_eq!(state.status_message(), "Notifications are enabled");

        state.pause("session".to_string(), None, success_result());
        assert_eq!(state.status_message(), "Do Not Disturb is active");

        state.clear();
        state.pause("session".to_string(), None, failure_result());
        assert!(state.status_message().contains("could not be enabled"));
    }

    #[test]
    fn test_pause_duration() {
        let mut state = NotificationControlState::default();
        assert!(state.pause_duration_seconds().is_none());

        state.pause("session".to_string(), None, success_result());
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
