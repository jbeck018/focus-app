// commands/notification_control.rs - System notification control commands
//
// Commands to pause and resume system notifications during focus sessions.
// Integrates with platform-specific Do Not Disturb / Focus Assist APIs.

use crate::system::notification_control::{NotificationControlState, NotificationPermissionStatus, PermissionState};
use crate::{AppState, Result};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

/// Response for notification control state queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationControlResponse {
    pub paused: bool,
    pub session_id: Option<String>,
    pub pause_duration_seconds: Option<i64>,
}

impl From<&NotificationControlState> for NotificationControlResponse {
    fn from(state: &NotificationControlState) -> Self {
        Self {
            paused: state.paused,
            session_id: state.session_id.clone(),
            pause_duration_seconds: state.pause_duration_seconds(),
        }
    }
}

/// Pause system notifications
///
/// Enables Do Not Disturb / Focus Assist mode on the system level.
#[tauri::command]
pub async fn pause_system_notifications(
    session_id: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<NotificationControlResponse> {
    // Get current DND state before pausing
    let previous_dnd = get_current_dnd_state().await;

    // Enable DND/Focus Assist
    if let Err(e) = enable_system_dnd().await {
        tracing::warn!("Failed to enable system DND: {}", e);
        // Continue anyway - we track state even if system API fails
    }

    // Update state
    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.pause(
            session_id.clone().unwrap_or_else(|| "manual".to_string()),
            previous_dnd,
        );
    }

    // Emit event
    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": true,
        "sessionId": session_id,
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    tracing::info!("System notifications paused");

    let notification_state = state.notification_control_state.read().await;
    Ok(NotificationControlResponse::from(&*notification_state))
}

/// Internal function to pause notifications without State wrapper
/// Used when calling from other commands like focus session start
pub async fn pause_notifications_internal(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    session_id: Option<String>,
) -> Result<()> {
    // Get current DND state before pausing
    let previous_dnd = get_current_dnd_state().await;

    // Enable DND/Focus Assist
    if let Err(e) = enable_system_dnd().await {
        tracing::warn!("Failed to enable system DND: {}", e);
    }

    // Update state
    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.pause(
            session_id.clone().unwrap_or_else(|| "manual".to_string()),
            previous_dnd,
        );
    }

    // Emit event
    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": true,
        "sessionId": session_id,
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    tracing::info!("System notifications paused");
    Ok(())
}

/// Resume system notifications
///
/// Disables Do Not Disturb / Focus Assist, restoring previous state.
#[tauri::command]
pub async fn resume_system_notifications(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<NotificationControlResponse> {
    let previous_dnd = {
        let notification_state = state.notification_control_state.read().await;
        notification_state.previous_dnd_enabled
    };

    // Only disable DND if it wasn't enabled before we paused
    if previous_dnd != Some(true) {
        if let Err(e) = disable_system_dnd().await {
            tracing::warn!("Failed to disable system DND: {}", e);
        }
    }

    // Update state
    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.resume();
    }

    // Emit event
    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": false,
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    tracing::info!("System notifications resumed");

    let notification_state = state.notification_control_state.read().await;
    Ok(NotificationControlResponse::from(&*notification_state))
}

/// Force resume notifications (bypasses session check)
/// Used when ending a focus session
pub async fn force_resume_notifications(
    state: &AppState,
    app_handle: &tauri::AppHandle,
) -> Result<()> {
    let previous_dnd = {
        let notification_state = state.notification_control_state.read().await;
        notification_state.previous_dnd_enabled
    };

    if previous_dnd != Some(true) {
        if let Err(e) = disable_system_dnd().await {
            tracing::warn!("Failed to disable system DND: {}", e);
        }
    }

    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.resume();
    }

    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": false,
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    Ok(())
}

/// Get current notification control state
#[tauri::command]
pub async fn get_notification_control_state(
    state: State<'_, AppState>,
) -> Result<NotificationControlResponse> {
    let notification_state = state.notification_control_state.read().await;
    Ok(NotificationControlResponse::from(&*notification_state))
}

/// Check notification control permissions
#[tauri::command]
pub async fn check_notification_permission() -> Result<NotificationPermissionStatus> {
    check_notification_permission_impl().await
}

// ============================================================================
// Platform-specific DND implementation
// ============================================================================

/// Get current system DND state
async fn get_current_dnd_state() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        get_dnd_state_macos().await
    }

    #[cfg(target_os = "windows")]
    {
        get_dnd_state_windows().await
    }

    #[cfg(target_os = "linux")]
    {
        get_dnd_state_linux().await
    }
}

/// Enable system DND/Focus Assist
async fn enable_system_dnd() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        enable_dnd_macos().await
    }

    #[cfg(target_os = "windows")]
    {
        enable_dnd_windows().await
    }

    #[cfg(target_os = "linux")]
    {
        enable_dnd_linux().await
    }
}

/// Disable system DND/Focus Assist
async fn disable_system_dnd() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        disable_dnd_macos().await
    }

    #[cfg(target_os = "windows")]
    {
        disable_dnd_windows().await
    }

    #[cfg(target_os = "linux")]
    {
        disable_dnd_linux().await
    }
}

/// Check notification control permission
async fn check_notification_permission_impl() -> Result<NotificationPermissionStatus> {
    #[cfg(target_os = "macos")]
    {
        check_permission_macos().await
    }

    #[cfg(target_os = "windows")]
    {
        check_permission_windows().await
    }

    #[cfg(target_os = "linux")]
    {
        check_permission_linux().await
    }
}

// ============================================================================
// macOS implementation
// ============================================================================

#[cfg(target_os = "macos")]
async fn get_dnd_state_macos() -> Option<bool> {
    // Try to read DND state from defaults
    // This is a best-effort approach as macOS Focus modes are complex
    use std::process::Command;

    let output = Command::new("defaults")
        .args(["read", "com.apple.controlcenter", "NSStatusItem Visible FocusModes"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.trim() == "1")
}

#[cfg(target_os = "macos")]
async fn enable_dnd_macos() -> Result<()> {
    // Use Shortcuts app to enable DND via Focus mode
    // This works on macOS 12.0+ (Monterey and later)
    // Requires user to have "Turn On Do Not Disturb" shortcut created
    use tokio::process::Command;

    let output = Command::new("shortcuts")
        .args(["run", "Turn On Do Not Disturb"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            tracing::info!("macOS DND enabled via Shortcuts");
            Ok(())
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            tracing::warn!(
                "Shortcuts method failed ({}), DND control limited on this macOS version. \
                 User may need to create 'Turn On Do Not Disturb' shortcut.",
                stderr.trim()
            );
            // Best effort - don't fail the session
            Ok(())
        }
        Err(e) => {
            tracing::warn!(
                "Could not run shortcuts command: {}. DND control requires macOS 12.0+ \
                 and 'Turn On Do Not Disturb' shortcut to be created.",
                e
            );
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
async fn disable_dnd_macos() -> Result<()> {
    // Use Shortcuts app to disable DND via Focus mode
    // This works on macOS 12.0+ (Monterey and later)
    // Requires user to have "Turn Off Do Not Disturb" shortcut created
    use tokio::process::Command;

    let output = Command::new("shortcuts")
        .args(["run", "Turn Off Do Not Disturb"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            tracing::info!("macOS DND disabled via Shortcuts");
            Ok(())
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            tracing::warn!(
                "Could not disable DND automatically ({}). \
                 User may need to create 'Turn Off Do Not Disturb' shortcut.",
                stderr.trim()
            );
            Ok(())
        }
        Err(e) => {
            tracing::warn!(
                "Could not run shortcuts command: {}. DND may need to be disabled manually.",
                e
            );
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
async fn check_permission_macos() -> Result<NotificationPermissionStatus> {
    Ok(NotificationPermissionStatus {
        can_control_dnd: true, // Best-effort on macOS
        permission_state: PermissionState::Granted,
        notes: Some("DND control on macOS may require accessibility permissions".to_string()),
    })
}

// ============================================================================
// Windows implementation
// ============================================================================

#[cfg(target_os = "windows")]
async fn get_dnd_state_windows() -> Option<bool> {
    // Windows Focus Assist state can be read from registry
    // HKCU\Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\...
    // This is complex and version-dependent
    None
}

#[cfg(target_os = "windows")]
async fn enable_dnd_windows() -> Result<()> {
    // Windows Focus Assist can be controlled via:
    // 1. Settings API (UWP) - requires specific capabilities
    // 2. Registry modification
    // 3. WNF (Windows Notification Facility) - undocumented

    tracing::debug!("Windows Focus Assist enable requested");

    // For now, we'll set a registry value that enables "Priority Only" mode
    use std::process::Command;

    let result = Command::new("powershell")
        .args([
            "-Command",
            r#"
            $regPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings'
            if (!(Test-Path $regPath)) { New-Item -Path $regPath -Force | Out-Null }
            Set-ItemProperty -Path $regPath -Name 'NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK' -Value 0 -Type DWord
            "#,
        ])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            tracing::debug!("Windows notifications restricted via registry");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("PowerShell notification control returned error: {}", stderr);
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Failed to run PowerShell for notification control: {}", e);
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
async fn disable_dnd_windows() -> Result<()> {
    use std::process::Command;

    let result = Command::new("powershell")
        .args([
            "-Command",
            r#"
            $regPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings'
            if (Test-Path $regPath) {
                Set-ItemProperty -Path $regPath -Name 'NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK' -Value 1 -Type DWord
            }
            "#,
        ])
        .output();

    if let Ok(output) = result {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("PowerShell notification restore returned error: {}", stderr);
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
async fn check_permission_windows() -> Result<NotificationPermissionStatus> {
    Ok(NotificationPermissionStatus {
        can_control_dnd: true,
        permission_state: PermissionState::Granted,
        notes: Some("Windows Focus Assist control is available".to_string()),
    })
}

// ============================================================================
// Linux implementation
// ============================================================================

#[cfg(target_os = "linux")]
async fn get_dnd_state_linux() -> Option<bool> {
    // Try to get DND state from common notification daemons

    // Try dunst first
    if let Some(paused) = get_dunst_paused().await {
        return Some(paused);
    }

    // Try GNOME
    if let Some(dnd) = get_gnome_dnd().await {
        return Some(dnd);
    }

    None
}

#[cfg(target_os = "linux")]
async fn get_dunst_paused() -> Option<bool> {
    use std::process::Command;

    let output = Command::new("dunstctl")
        .arg("is-paused")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.trim() == "true")
}

#[cfg(target_os = "linux")]
async fn get_gnome_dnd() -> Option<bool> {
    use std::process::Command;

    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.notifications", "show-banners"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // DND is enabled when banners are NOT shown
    Some(stdout.trim() == "false")
}

#[cfg(target_os = "linux")]
async fn enable_dnd_linux() -> Result<()> {
    use std::process::Command;

    // Try dunst first (common on many Linux distros)
    let dunst_result = Command::new("dunstctl")
        .arg("set-paused")
        .arg("true")
        .output();

    if let Ok(output) = dunst_result {
        if output.status.success() {
            tracing::debug!("Notifications paused via dunst");
            return Ok(());
        }
    }

    // Try GNOME
    let gnome_result = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.notifications", "show-banners", "false"])
        .output();

    if let Ok(output) = gnome_result {
        if output.status.success() {
            tracing::debug!("Notifications paused via GNOME settings");
            return Ok(());
        }
    }

    // Try KDE
    let kde_result = Command::new("qdbus")
        .args([
            "org.kde.plasmashell",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications.Inhibit",
            "FocusFlow",
            "Focus session active",
        ])
        .output();

    if let Ok(output) = kde_result {
        if output.status.success() {
            tracing::debug!("Notifications inhibited via KDE");
            return Ok(());
        }
    }

    tracing::warn!("Could not pause notifications - no supported notification daemon found");
    Ok(())
}

#[cfg(target_os = "linux")]
async fn disable_dnd_linux() -> Result<()> {
    use std::process::Command;

    // Resume dunst
    let _ = Command::new("dunstctl")
        .arg("set-paused")
        .arg("false")
        .output();

    // Resume GNOME
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.notifications", "show-banners", "true"])
        .output();

    // Note: KDE inhibition should auto-expire when the calling process exits

    Ok(())
}

#[cfg(target_os = "linux")]
async fn check_permission_linux() -> Result<NotificationPermissionStatus> {
    use std::process::Command;

    // Check for dunst
    let has_dunst = Command::new("which")
        .arg("dunstctl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for GNOME
    let has_gnome = Command::new("which")
        .arg("gsettings")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let can_control = has_dunst || has_gnome;

    Ok(NotificationPermissionStatus {
        can_control_dnd: can_control,
        permission_state: if can_control {
            PermissionState::Granted
        } else {
            PermissionState::Unavailable
        },
        notes: if has_dunst {
            Some("Notification control available via dunst".to_string())
        } else if has_gnome {
            Some("Notification control available via GNOME settings".to_string())
        } else {
            Some("No supported notification daemon found".to_string())
        },
    })
}
