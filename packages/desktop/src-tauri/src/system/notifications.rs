// system/notifications.rs - Cross-platform notification helpers

use tauri::AppHandle;
use tauri_plugin_notification::{NotificationExt, Result as NotificationResult};

/// Notification helper for consistent formatting across the app
#[allow(dead_code)]
pub struct NotificationManager {
    app_handle: AppHandle,
}

#[allow(dead_code)]
impl NotificationManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Send focus session started notification
    pub fn session_started(&self, duration_minutes: i32) -> NotificationResult<()> {
        self.app_handle
            .notification()
            .builder()
            .title("Focus Session Started")
            .body(format!(
                "Stay focused for the next {} minutes. You've got this!",
                duration_minutes
            ))
            .show()
    }

    /// Send session completed notification
    pub fn session_completed(&self, duration_seconds: i64) -> NotificationResult<()> {
        let minutes = duration_seconds / 60;
        self.app_handle
            .notification()
            .builder()
            .title("Focus Session Completed!")
            .body(format!(
                "Great job! You stayed focused for {} minutes.",
                minutes
            ))
            .show()
    }

    /// Send session abandoned notification
    pub fn session_abandoned(&self) -> NotificationResult<()> {
        self.app_handle
            .notification()
            .builder()
            .title("Session Ended Early")
            .body("No worries! Try again when you're ready.")
            .show()
    }

    /// Send break reminder notification
    pub fn break_reminder(&self) -> NotificationResult<()> {
        self.app_handle
            .notification()
            .builder()
            .title("Time for a Break")
            .body("You've been focused for a while. Consider taking a short break.")
            .show()
    }

    /// Send blocked app warning
    pub fn blocked_app_warning(&self, app_name: &str) -> NotificationResult<()> {
        self.app_handle
            .notification()
            .builder()
            .title("Blocked Application")
            .body(format!(
                "{} is blocked during focus sessions. It will be closed.",
                app_name
            ))
            .show()
    }

    /// Send custom notification
    pub fn custom(&self, title: &str, body: &str) -> NotificationResult<()> {
        self.app_handle
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
    }
}

/// Schedule periodic notifications during long focus sessions
#[allow(dead_code)]
pub async fn schedule_break_reminders(
    app_handle: AppHandle,
    interval_minutes: u64,
) {
    let notifications = NotificationManager::new(app_handle);

    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(interval_minutes * 60)
    );

    loop {
        interval.tick().await;

        if let Err(e) = notifications.break_reminder() {
            tracing::warn!("Failed to send break reminder: {}", e);
        }
    }
}
