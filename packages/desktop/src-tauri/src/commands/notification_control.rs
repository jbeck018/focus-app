// commands/notification_control.rs - System notification control commands
//
// Commands to pause and resume system notifications during focus sessions.
// Integrates with platform-specific Do Not Disturb / Focus Assist APIs.
//
// Error Handling Philosophy:
// - DND failures should NOT prevent focus sessions from starting
// - Users should always be informed when DND couldn't be enabled
// - The app tracks both app-level state and system-level state separately
// - Detailed logging helps debug platform-specific issues

use crate::system::notification_control::{
    DndMethod, DndOperationResult, NotificationControlState, NotificationPermissionStatus, PermissionState,
};
use crate::{AppState, Result};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

/// Response for notification control state queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationControlResponse {
    /// Whether the app considers notifications paused
    pub paused: bool,
    /// Session ID this pause is tied to
    pub session_id: Option<String>,
    /// How long notifications have been paused
    pub pause_duration_seconds: Option<i64>,
    /// Whether system DND is actually active (may differ from paused if system API failed)
    pub system_dnd_active: bool,
    /// User-facing status message
    pub status_message: String,
    /// Details about the last DND operation (for debugging/user feedback)
    pub last_operation: Option<DndOperationResult>,
}

impl From<&NotificationControlState> for NotificationControlResponse {
    fn from(state: &NotificationControlState) -> Self {
        Self {
            paused: state.paused,
            session_id: state.session_id.clone(),
            pause_duration_seconds: state.pause_duration_seconds(),
            system_dnd_active: state.is_system_dnd_active(),
            status_message: state.status_message(),
            last_operation: state.last_operation_result.clone(),
        }
    }
}

/// Pause system notifications
///
/// Enables Do Not Disturb / Focus Assist mode on the system level.
/// Returns detailed status including whether system DND was actually enabled.
#[tauri::command]
pub async fn pause_system_notifications(
    session_id: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<NotificationControlResponse> {
    let sid = session_id.clone().unwrap_or_else(|| "manual".to_string());
    tracing::info!(session_id = %sid, "Attempting to pause system notifications");

    // Get current DND state before pausing
    let previous_dnd = get_current_dnd_state().await;
    tracing::debug!(previous_dnd = ?previous_dnd, "Current DND state before pause");

    // Enable DND/Focus Assist and get detailed result
    let operation_result = enable_system_dnd().await;

    if operation_result.success {
        tracing::info!(
            method = ?operation_result.method,
            "System DND enabled successfully"
        );
    } else {
        tracing::warn!(
            method = ?operation_result.method,
            message = %operation_result.message,
            user_action = ?operation_result.user_action,
            "System DND could not be enabled - continuing with app-level tracking"
        );
    }

    // Update state with operation result
    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.pause(sid.clone(), previous_dnd, operation_result.clone());
    }

    // Emit event with detailed status
    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": true,
        "sessionId": session_id,
        "systemDndActive": operation_result.success,
        "message": operation_result.message,
        "userAction": operation_result.user_action,
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    let notification_state = state.notification_control_state.read().await;
    Ok(NotificationControlResponse::from(&*notification_state))
}

/// Internal function to pause notifications without State wrapper
/// Used when calling from other commands like focus session start
///
/// Returns the operation result for the caller to handle (e.g., notify user of failure)
pub async fn pause_notifications_internal(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    session_id: Option<String>,
) -> Result<DndOperationResult> {
    let sid = session_id.clone().unwrap_or_else(|| "manual".to_string());
    tracing::info!(session_id = %sid, "Attempting to pause system notifications (internal)");

    // Get current DND state before pausing
    let previous_dnd = get_current_dnd_state().await;
    tracing::debug!(previous_dnd = ?previous_dnd, "Current DND state before pause");

    // Enable DND/Focus Assist and get detailed result
    let operation_result = enable_system_dnd().await;

    if operation_result.success {
        tracing::info!(
            method = ?operation_result.method,
            "System DND enabled successfully"
        );
    } else {
        tracing::warn!(
            method = ?operation_result.method,
            message = %operation_result.message,
            user_action = ?operation_result.user_action,
            "System DND could not be enabled - continuing with app-level tracking"
        );
    }

    // Update state with operation result
    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.pause(sid, previous_dnd, operation_result.clone());
    }

    // Emit event with detailed status
    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": true,
        "sessionId": session_id,
        "systemDndActive": operation_result.success,
        "message": operation_result.message,
        "userAction": operation_result.user_action,
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    Ok(operation_result)
}

/// Resume system notifications
///
/// Disables Do Not Disturb / Focus Assist, restoring previous state.
#[tauri::command]
pub async fn resume_system_notifications(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<NotificationControlResponse> {
    tracing::info!("Attempting to resume system notifications");

    let (previous_dnd, was_system_dnd_active) = {
        let notification_state = state.notification_control_state.read().await;
        (notification_state.previous_dnd_enabled, notification_state.system_dnd_enabled)
    };

    tracing::debug!(
        previous_dnd = ?previous_dnd,
        was_system_dnd_active = was_system_dnd_active,
        "State before resume"
    );

    // Only disable DND if it wasn't enabled before we paused AND we actually enabled it
    let operation_result = if previous_dnd != Some(true) && was_system_dnd_active {
        let result = disable_system_dnd().await;
        if result.success {
            tracing::info!(method = ?result.method, "System DND disabled successfully");
        } else {
            tracing::warn!(
                method = ?result.method,
                message = %result.message,
                "Failed to disable system DND - user may need to disable manually"
            );
        }
        Some(result)
    } else if previous_dnd == Some(true) {
        tracing::info!("Preserving user's pre-existing DND state");
        Some(DndOperationResult::success(
            DndMethod::Manual,
            "DND was already enabled by user - leaving enabled",
        ))
    } else {
        tracing::debug!("System DND was not active, no action needed");
        None
    };

    // Update state
    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.resume(operation_result.clone());
    }

    // Emit event
    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": false,
        "message": operation_result.as_ref().map(|r| r.message.clone()),
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    let notification_state = state.notification_control_state.read().await;
    Ok(NotificationControlResponse::from(&*notification_state))
}

/// Force resume notifications (bypasses session check)
/// Used when ending a focus session
///
/// Returns the operation result for logging/debugging purposes
pub async fn force_resume_notifications(
    state: &AppState,
    app_handle: &tauri::AppHandle,
) -> Result<Option<DndOperationResult>> {
    tracing::info!("Force resuming notifications (session ended)");

    let (previous_dnd, was_system_dnd_active) = {
        let notification_state = state.notification_control_state.read().await;
        (notification_state.previous_dnd_enabled, notification_state.system_dnd_enabled)
    };

    // Only disable DND if it wasn't enabled before we paused AND we actually enabled it
    let operation_result = if previous_dnd != Some(true) && was_system_dnd_active {
        let result = disable_system_dnd().await;
        if result.success {
            tracing::info!(method = ?result.method, "System DND disabled on session end");
        } else {
            tracing::warn!(
                method = ?result.method,
                message = %result.message,
                "Failed to disable system DND on session end"
            );
        }
        Some(result)
    } else if previous_dnd == Some(true) {
        tracing::info!("Preserving user's pre-existing DND state after session");
        None
    } else {
        tracing::debug!("System DND was not active, no action needed on resume");
        None
    };

    {
        let mut notification_state = state.notification_control_state.write().await;
        notification_state.resume(operation_result.clone());
    }

    if let Err(e) = app_handle.emit("notification-control-changed", serde_json::json!({
        "paused": false,
        "message": operation_result.as_ref().map(|r| r.message.clone()),
    })) {
        tracing::warn!("Failed to emit notification control event: {}", e);
    }

    Ok(operation_result)
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
/// Returns detailed operation result instead of just success/failure
async fn enable_system_dnd() -> DndOperationResult {
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
/// Returns detailed operation result instead of just success/failure
async fn disable_system_dnd() -> DndOperationResult {
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
    use tokio::process::Command;

    tracing::debug!("Checking macOS DND state");

    // Try multiple methods to detect DND state

    // Method 1: Check Focus mode status via defaults (macOS 12+)
    let focus_output = Command::new("defaults")
        .args(["read", "com.apple.controlcenter", "NSStatusItem Visible FocusModes"])
        .output()
        .await;

    if let Ok(output) = focus_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim() == "1" {
            tracing::debug!("Focus mode indicator is visible (DND likely active)");
            return Some(true);
        }
    }

    // Method 2: Check notification center preferences
    let nc_output = Command::new("defaults")
        .args(["-currentHost", "read", "com.apple.notificationcenterui", "doNotDisturb"])
        .output()
        .await;

    if let Ok(output) = nc_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim() == "1" {
            tracing::debug!("Legacy DND flag is set");
            return Some(true);
        }
    }

    tracing::debug!("DND does not appear to be active");
    Some(false)
}

#[cfg(target_os = "macos")]
async fn enable_dnd_macos() -> DndOperationResult {
    use tokio::process::Command;

    tracing::info!("Attempting to enable macOS DND");

    // Method 1: Try Shortcuts app (macOS 12+)
    // This is the most reliable method but requires user setup
    let shortcuts_output = Command::new("shortcuts")
        .args(["run", "Turn On Do Not Disturb"])
        .output()
        .await;

    match shortcuts_output {
        Ok(o) if o.status.success() => {
            tracing::info!("macOS DND enabled via Shortcuts");
            return DndOperationResult::success(
                DndMethod::MacosShortcuts,
                "Do Not Disturb enabled via Shortcuts",
            );
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let exit_code = o.status.code().unwrap_or(-1);
            tracing::debug!(
                exit_code = exit_code,
                stderr = %stderr.trim(),
                "Shortcuts method failed, trying alternative"
            );
        }
        Err(e) => {
            tracing::debug!(error = %e, "shortcuts command not available");
        }
    }

    // Method 2: Try AppleScript as fallback
    // This may work on some macOS versions
    let applescript = r#"
        tell application "System Events"
            try
                -- Try to enable Focus mode via menu bar
                tell process "ControlCenter"
                    click menu bar item "Focus" of menu bar 1
                    delay 0.5
                    click checkbox "Do Not Disturb" of window 1
                end tell
                return "success"
            on error errMsg
                return "error: " & errMsg
            end try
        end tell
    "#;

    let applescript_output = Command::new("osascript")
        .args(["-e", applescript])
        .output()
        .await;

    match applescript_output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("success") {
                tracing::info!("macOS DND enabled via AppleScript");
                return DndOperationResult::success(
                    DndMethod::MacosApplescript,
                    "Do Not Disturb enabled via system automation",
                );
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            tracing::debug!(stderr = %stderr.trim(), "AppleScript method failed");
        }
        Err(e) => {
            tracing::debug!(error = %e, "osascript command failed");
        }
    }

    // All methods failed - provide helpful guidance
    tracing::warn!("Could not enable macOS DND automatically");
    DndOperationResult::partial(
        DndMethod::MacosShortcuts,
        "Could not enable Do Not Disturb automatically",
        "To enable automatic DND:\n\
         1. Open Shortcuts app\n\
         2. Create a shortcut named 'Turn On Do Not Disturb'\n\
         3. Add action: Set Focus > Turn Do Not Disturb On\n\
         Or enable DND manually from Control Center",
    )
}

#[cfg(target_os = "macos")]
async fn disable_dnd_macos() -> DndOperationResult {
    use tokio::process::Command;

    tracing::info!("Attempting to disable macOS DND");

    // Method 1: Try Shortcuts app (macOS 12+)
    let shortcuts_output = Command::new("shortcuts")
        .args(["run", "Turn Off Do Not Disturb"])
        .output()
        .await;

    match shortcuts_output {
        Ok(o) if o.status.success() => {
            tracing::info!("macOS DND disabled via Shortcuts");
            return DndOperationResult::success(
                DndMethod::MacosShortcuts,
                "Do Not Disturb disabled via Shortcuts",
            );
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            tracing::debug!(stderr = %stderr.trim(), "Shortcuts disable method failed");
        }
        Err(e) => {
            tracing::debug!(error = %e, "shortcuts command not available for disable");
        }
    }

    // Method 2: AppleScript fallback
    let applescript = r#"
        tell application "System Events"
            try
                tell process "ControlCenter"
                    click menu bar item "Focus" of menu bar 1
                    delay 0.5
                    -- Click to disable if currently enabled
                    set focusCheckbox to checkbox "Do Not Disturb" of window 1
                    if value of focusCheckbox is 1 then
                        click focusCheckbox
                    end if
                end tell
                return "success"
            on error errMsg
                return "error: " & errMsg
            end try
        end tell
    "#;

    let applescript_output = Command::new("osascript")
        .args(["-e", applescript])
        .output()
        .await;

    match applescript_output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("success") {
                tracing::info!("macOS DND disabled via AppleScript");
                return DndOperationResult::success(
                    DndMethod::MacosApplescript,
                    "Do Not Disturb disabled via system automation",
                );
            }
        }
        _ => {}
    }

    tracing::warn!("Could not disable macOS DND automatically");
    DndOperationResult::partial(
        DndMethod::MacosShortcuts,
        "Could not disable Do Not Disturb automatically",
        "Please disable Do Not Disturb manually from Control Center, \
         or create a 'Turn Off Do Not Disturb' shortcut",
    )
}

#[cfg(target_os = "macos")]
async fn check_permission_macos() -> Result<NotificationPermissionStatus> {
    use tokio::process::Command;

    // Check if shortcuts command exists
    let shortcuts_available = Command::new("which")
        .arg("shortcuts")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if the required shortcuts exist
    let has_on_shortcut = Command::new("shortcuts")
        .args(["list"])
        .output()
        .await
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("Turn On Do Not Disturb")
        })
        .unwrap_or(false);

    let has_off_shortcut = Command::new("shortcuts")
        .args(["list"])
        .output()
        .await
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("Turn Off Do Not Disturb")
        })
        .unwrap_or(false);

    let can_control = shortcuts_available && has_on_shortcut && has_off_shortcut;

    let notes = if !shortcuts_available {
        Some("macOS 12 (Monterey) or later required for DND control".to_string())
    } else if !has_on_shortcut || !has_off_shortcut {
        Some(
            "Create shortcuts named 'Turn On Do Not Disturb' and 'Turn Off Do Not Disturb' \
             in the Shortcuts app for automatic DND control"
                .to_string(),
        )
    } else {
        Some("DND control available via Shortcuts".to_string())
    };

    Ok(NotificationPermissionStatus {
        can_control_dnd: can_control,
        permission_state: if can_control {
            PermissionState::Granted
        } else {
            PermissionState::NotDetermined
        },
        notes,
    })
}

// ============================================================================
// Windows implementation
// ============================================================================

#[cfg(target_os = "windows")]
async fn get_dnd_state_windows() -> Option<bool> {
    use tokio::process::Command;

    tracing::debug!("Checking Windows Focus Assist state");

    // Try reading the priority only mode state (Windows 10+)
    let result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"
            $ErrorActionPreference = 'SilentlyContinue'

            # Method 1: Check QuietHoursProfile (0 = Off, 1 = Priority Only, 2 = Alarms Only)
            $regPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\QuietHours'
            if (Test-Path $regPath) {
                $profile = Get-ItemProperty -Path $regPath -Name 'Profile' -ErrorAction SilentlyContinue
                if ($profile -and $profile.Profile -gt 0) {
                    Write-Output 'true:quiethours'
                    exit 0
                }
            }

            # Method 2: Check CloudStore for Focus Assist state (Windows 10 1809+)
            $cloudPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\DefaultAccount\Current\default$windows.data.notifications.quiethourssettings\windows.data.notifications.quiethourssettings'
            if (Test-Path $cloudPath) {
                # If this path exists and has data, Focus Assist is configured
                $data = Get-ItemProperty -Path $cloudPath -Name 'Data' -ErrorAction SilentlyContinue
                if ($data) {
                    Write-Output 'true:cloudstore'
                    exit 0
                }
            }

            # Method 3: Check notification settings registry
            $notifPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings'
            if (Test-Path $notifPath) {
                $allowToasts = Get-ItemProperty -Path $notifPath -Name 'NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK' -ErrorAction SilentlyContinue
                if ($allowToasts -and $allowToasts.NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK -eq 0) {
                    Write-Output 'true:registry'
                    exit 0
                }
            }

            Write-Output 'false'
            "#,
        ])
        .output()
        .await
        .ok()?;

    if result.status.success() {
        let stdout = String::from_utf8_lossy(&result.stdout);
        let trimmed = stdout.trim();
        let is_enabled = trimmed.starts_with("true");
        if is_enabled {
            let method = trimmed.split(':').nth(1).unwrap_or("unknown");
            tracing::debug!(method = method, "Windows Focus Assist is enabled");
        } else {
            tracing::debug!("Windows Focus Assist is not enabled");
        }
        Some(is_enabled)
    } else {
        tracing::debug!("Could not read Windows Focus Assist state");
        None
    }
}

#[cfg(target_os = "windows")]
async fn enable_dnd_windows() -> DndOperationResult {
    use tokio::process::Command;

    tracing::info!("Attempting to enable Windows Focus Assist");

    // Method 1: Try registry-based notification suppression
    // This is the most reliable cross-version approach
    let registry_result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"
            $ErrorActionPreference = 'Stop'
            try {
                # Suppress toast notifications
                $notifPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings'
                if (!(Test-Path $notifPath)) { New-Item -Path $notifPath -Force | Out-Null }
                Set-ItemProperty -Path $notifPath -Name 'NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK' -Value 0 -Type DWord
                Set-ItemProperty -Path $notifPath -Name 'NOC_GLOBAL_SETTING_ALLOW_CRITICAL_TOASTS_ABOVE_LOCK' -Value 0 -Type DWord

                # Also try to set QuietHours profile to Priority Only (1)
                $quietPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\QuietHours'
                if (!(Test-Path $quietPath)) { New-Item -Path $quietPath -Force | Out-Null }
                Set-ItemProperty -Path $quietPath -Name 'Profile' -Value 1 -Type DWord

                Write-Output 'success'
            } catch {
                Write-Output "error:$($_.Exception.Message)"
            }
            "#,
        ])
        .output()
        .await;

    match registry_result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim() == "success" {
                tracing::info!("Windows Focus Assist enabled via registry");
                return DndOperationResult::success(
                    DndMethod::WindowsRegistry,
                    "Focus Assist enabled - notifications suppressed",
                );
            } else if stdout.starts_with("error:") {
                let error_msg = stdout.trim_start_matches("error:");
                tracing::warn!(error = error_msg, "Registry method failed");
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::debug!(stderr = %stderr.trim(), "Registry PowerShell command failed");
        }
        Err(e) => {
            tracing::debug!(error = %e, "PowerShell command failed");
        }
    }

    // Method 2: Try using WNF (Windows Notification Facility)
    // This is undocumented but can directly trigger Focus Assist
    let wnf_result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"
            # Note: WNF requires specific binary manipulation which isn't easily done in PowerShell
            # This is a placeholder for future implementation with native code
            Write-Output 'unavailable'
            "#,
        ])
        .output()
        .await;

    if let Ok(output) = wnf_result {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim() == "success" {
            tracing::info!("Windows Focus Assist enabled via WNF");
            return DndOperationResult::success(
                DndMethod::WindowsWnf,
                "Focus Assist enabled via Windows Notification Facility",
            );
        }
    }

    // All methods failed
    tracing::warn!("Could not reliably enable Windows Focus Assist");
    DndOperationResult::partial(
        DndMethod::WindowsRegistry,
        "Notification suppression partially enabled",
        "For full Focus Assist:\n\
         1. Click the Focus Assist icon in the system tray\n\
         2. Select 'Priority only' or 'Alarms only'\n\
         Or use Windows Settings > System > Focus assist",
    )
}

#[cfg(target_os = "windows")]
async fn disable_dnd_windows() -> DndOperationResult {
    use tokio::process::Command;

    tracing::info!("Attempting to disable Windows Focus Assist");

    let result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"
            $ErrorActionPreference = 'SilentlyContinue'
            try {
                # Restore toast notifications
                $notifPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings'
                if (Test-Path $notifPath) {
                    Set-ItemProperty -Path $notifPath -Name 'NOC_GLOBAL_SETTING_ALLOW_TOASTS_ABOVE_LOCK' -Value 1 -Type DWord
                    Set-ItemProperty -Path $notifPath -Name 'NOC_GLOBAL_SETTING_ALLOW_CRITICAL_TOASTS_ABOVE_LOCK' -Value 1 -Type DWord
                }

                # Reset QuietHours profile to Off (0)
                $quietPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\QuietHours'
                if (Test-Path $quietPath) {
                    Set-ItemProperty -Path $quietPath -Name 'Profile' -Value 0 -Type DWord
                }

                Write-Output 'success'
            } catch {
                Write-Output "error:$($_.Exception.Message)"
            }
            "#,
        ])
        .output()
        .await;

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim() == "success" {
                tracing::info!("Windows Focus Assist disabled via registry");
                return DndOperationResult::success(
                    DndMethod::WindowsRegistry,
                    "Focus Assist disabled - notifications restored",
                );
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::debug!(stderr = %stderr.trim(), "Registry disable failed");
        }
        Err(e) => {
            tracing::debug!(error = %e, "PowerShell disable command failed");
        }
    }

    tracing::warn!("Could not automatically disable Windows Focus Assist");
    DndOperationResult::partial(
        DndMethod::WindowsRegistry,
        "Could not fully disable Focus Assist",
        "Please disable Focus Assist manually from the system tray or Windows Settings",
    )
}

#[cfg(target_os = "windows")]
async fn check_permission_windows() -> Result<NotificationPermissionStatus> {
    use tokio::process::Command;

    // Check Windows version and Focus Assist availability
    let version_check = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"
            $os = Get-CimInstance Win32_OperatingSystem
            $build = [int]$os.BuildNumber
            # Focus Assist was introduced in Windows 10 1709 (build 16299)
            if ($build -ge 16299) {
                Write-Output "available:$build"
            } else {
                Write-Output "unavailable:$build"
            }
            "#,
        ])
        .output()
        .await;

    let (can_control, notes) = match version_check {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.starts_with("available") {
                (true, Some("Windows Focus Assist control available".to_string()))
            } else {
                (false, Some("Focus Assist requires Windows 10 version 1709 or later".to_string()))
            }
        }
        _ => (true, Some("Focus Assist control available (version check failed)".to_string())),
    };

    Ok(NotificationPermissionStatus {
        can_control_dnd: can_control,
        permission_state: if can_control {
            PermissionState::Granted
        } else {
            PermissionState::Unavailable
        },
        notes,
    })
}

// ============================================================================
// Linux implementation
// ============================================================================

/// Detected Linux desktop environment for DND control
#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
enum LinuxDesktop {
    Gnome,
    Kde,
    Dunst,
    Sway,
    Unknown,
}

#[cfg(target_os = "linux")]
async fn detect_linux_desktop() -> LinuxDesktop {
    use tokio::process::Command;

    // Check for running processes/services to detect DE
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    let session = std::env::var("DESKTOP_SESSION").unwrap_or_default().to_lowercase();

    tracing::debug!(desktop = %desktop, session = %session, "Detecting Linux desktop environment");

    // Check for GNOME
    if desktop.contains("gnome") || session.contains("gnome") {
        return LinuxDesktop::Gnome;
    }

    // Check for KDE
    if desktop.contains("kde") || desktop.contains("plasma") || session.contains("plasma") {
        return LinuxDesktop::Kde;
    }

    // Check for Sway/wlroots
    if desktop.contains("sway") || std::env::var("SWAYSOCK").is_ok() {
        return LinuxDesktop::Sway;
    }

    // Check if dunst is running (common standalone notification daemon)
    let dunst_running = Command::new("pgrep")
        .arg("dunst")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if dunst_running {
        return LinuxDesktop::Dunst;
    }

    // Check if dunstctl is available
    let has_dunstctl = Command::new("which")
        .arg("dunstctl")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_dunstctl {
        return LinuxDesktop::Dunst;
    }

    // Fallback: check for gsettings (GNOME-based)
    let gsettings_works = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.notifications", "show-banners"])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if gsettings_works {
        return LinuxDesktop::Gnome;
    }

    LinuxDesktop::Unknown
}

#[cfg(target_os = "linux")]
async fn get_dnd_state_linux() -> Option<bool> {
    let desktop = detect_linux_desktop().await;
    tracing::debug!(desktop = ?desktop, "Checking Linux DND state");

    match desktop {
        LinuxDesktop::Dunst => get_dunst_paused().await,
        LinuxDesktop::Gnome => get_gnome_dnd().await,
        LinuxDesktop::Kde => get_kde_dnd().await,
        LinuxDesktop::Sway => get_sway_dnd().await,
        LinuxDesktop::Unknown => {
            // Try all methods
            if let Some(state) = get_dunst_paused().await {
                return Some(state);
            }
            if let Some(state) = get_gnome_dnd().await {
                return Some(state);
            }
            None
        }
    }
}

#[cfg(target_os = "linux")]
async fn get_dunst_paused() -> Option<bool> {
    use tokio::process::Command;

    let output = Command::new("dunstctl")
        .arg("is-paused")
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let paused = stdout.trim() == "true";
        tracing::debug!(paused = paused, "dunst pause state");
        Some(paused)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
async fn get_gnome_dnd() -> Option<bool> {
    use tokio::process::Command;

    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.notifications", "show-banners"])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // DND is enabled when banners are NOT shown
        let dnd_enabled = stdout.trim() == "false";
        tracing::debug!(dnd_enabled = dnd_enabled, "GNOME DND state");
        Some(dnd_enabled)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
async fn get_kde_dnd() -> Option<bool> {
    use tokio::process::Command;

    // KDE uses dbus for notification control
    let output = Command::new("qdbus")
        .args([
            "org.kde.plasmashell",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications.Inhibited",
        ])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let inhibited = stdout.trim() == "true";
        tracing::debug!(inhibited = inhibited, "KDE notification inhibited state");
        Some(inhibited)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
async fn get_sway_dnd() -> Option<bool> {
    use tokio::process::Command;

    // Sway typically uses mako or dunst
    // Check mako first
    let mako_output = Command::new("makoctl")
        .arg("mode")
        .output()
        .await;

    if let Ok(output) = mako_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let dnd_enabled = stdout.contains("do-not-disturb");
            tracing::debug!(dnd_enabled = dnd_enabled, "mako DND state");
            return Some(dnd_enabled);
        }
    }

    // Fall back to dunst
    get_dunst_paused().await
}

#[cfg(target_os = "linux")]
async fn enable_dnd_linux() -> DndOperationResult {
    use tokio::process::Command;

    let desktop = detect_linux_desktop().await;
    tracing::info!(desktop = ?desktop, "Attempting to enable Linux DND");

    match desktop {
        LinuxDesktop::Dunst => {
            let result = Command::new("dunstctl")
                .args(["set-paused", "true"])
                .output()
                .await;

            match result {
                Ok(output) if output.status.success() => {
                    tracing::info!("DND enabled via dunst");
                    DndOperationResult::success(DndMethod::LinuxDunst, "Notifications paused via dunst")
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!(stderr = %stderr.trim(), "dunstctl failed");
                    DndOperationResult::partial(
                        DndMethod::LinuxDunst,
                        format!("dunst command failed: {}", stderr.trim()),
                        "Try running 'dunstctl set-paused true' manually",
                    )
                }
                Err(e) => {
                    tracing::warn!(error = %e, "dunstctl not available");
                    DndOperationResult::partial(
                        DndMethod::LinuxDunst,
                        "dunstctl command not found",
                        "Install dunst or use your desktop environment's DND settings",
                    )
                }
            }
        }
        LinuxDesktop::Gnome => {
            let result = Command::new("gsettings")
                .args(["set", "org.gnome.desktop.notifications", "show-banners", "false"])
                .output()
                .await;

            match result {
                Ok(output) if output.status.success() => {
                    tracing::info!("DND enabled via GNOME settings");
                    DndOperationResult::success(DndMethod::LinuxGnome, "Do Not Disturb enabled in GNOME")
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!(stderr = %stderr.trim(), "gsettings failed");
                    DndOperationResult::partial(
                        DndMethod::LinuxGnome,
                        format!("gsettings command failed: {}", stderr.trim()),
                        "Enable Do Not Disturb from GNOME Settings > Notifications",
                    )
                }
                Err(e) => {
                    tracing::warn!(error = %e, "gsettings not available");
                    DndOperationResult::partial(
                        DndMethod::LinuxGnome,
                        "gsettings command not found",
                        "Enable Do Not Disturb from GNOME Settings",
                    )
                }
            }
        }
        LinuxDesktop::Kde => {
            let result = Command::new("qdbus")
                .args([
                    "org.kde.plasmashell",
                    "/org/freedesktop/Notifications",
                    "org.freedesktop.Notifications.Inhibit",
                    "FocusFlow",
                    "Focus session active",
                ])
                .output()
                .await;

            match result {
                Ok(output) if output.status.success() => {
                    // Store the inhibition cookie for later release
                    let cookie = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    tracing::info!(cookie = %cookie, "DND enabled via KDE");
                    DndOperationResult::success(DndMethod::LinuxKde, "Notifications inhibited in KDE Plasma")
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!(stderr = %stderr.trim(), "qdbus failed");
                    DndOperationResult::partial(
                        DndMethod::LinuxKde,
                        "Could not inhibit KDE notifications",
                        "Enable Do Not Disturb from KDE System Settings or click the notification icon",
                    )
                }
                Err(e) => {
                    tracing::warn!(error = %e, "qdbus not available");
                    DndOperationResult::partial(
                        DndMethod::LinuxKde,
                        "qdbus command not found",
                        "Enable Do Not Disturb from KDE System Settings",
                    )
                }
            }
        }
        LinuxDesktop::Sway => {
            // Try mako first
            let mako_result = Command::new("makoctl")
                .args(["mode", "-a", "do-not-disturb"])
                .output()
                .await;

            if let Ok(output) = mako_result {
                if output.status.success() {
                    tracing::info!("DND enabled via mako");
                    return DndOperationResult::success(DndMethod::LinuxDunst, "Do Not Disturb mode enabled via mako");
                }
            }

            // Fall back to dunst
            let dunst_result = Command::new("dunstctl")
                .args(["set-paused", "true"])
                .output()
                .await;

            match dunst_result {
                Ok(output) if output.status.success() => {
                    tracing::info!("DND enabled via dunst (Sway)");
                    DndOperationResult::success(DndMethod::LinuxDunst, "Notifications paused via dunst")
                }
                _ => {
                    DndOperationResult::partial(
                        DndMethod::Unavailable,
                        "Could not enable DND automatically",
                        "Use 'makoctl mode -a do-not-disturb' or 'dunstctl set-paused true'",
                    )
                }
            }
        }
        LinuxDesktop::Unknown => {
            tracing::warn!("Unknown Linux desktop - trying all methods");

            // Try dunst first
            if let Ok(output) = Command::new("dunstctl").args(["set-paused", "true"]).output().await {
                if output.status.success() {
                    return DndOperationResult::success(DndMethod::LinuxDunst, "Notifications paused via dunst");
                }
            }

            // Try GNOME
            if let Ok(output) = Command::new("gsettings")
                .args(["set", "org.gnome.desktop.notifications", "show-banners", "false"])
                .output()
                .await
            {
                if output.status.success() {
                    return DndOperationResult::success(DndMethod::LinuxGnome, "Do Not Disturb enabled via gsettings");
                }
            }

            DndOperationResult::partial(
                DndMethod::Unavailable,
                "No supported notification daemon found",
                "Install dunst, or enable DND from your desktop environment's settings",
            )
        }
    }
}

#[cfg(target_os = "linux")]
async fn disable_dnd_linux() -> DndOperationResult {
    use tokio::process::Command;

    let desktop = detect_linux_desktop().await;
    tracing::info!(desktop = ?desktop, "Attempting to disable Linux DND");

    let mut success_count = 0;
    let mut last_error = String::new();

    // Try dunst
    if let Ok(output) = Command::new("dunstctl").args(["set-paused", "false"]).output().await {
        if output.status.success() {
            tracing::debug!("dunst notifications resumed");
            success_count += 1;
        }
    }

    // Try GNOME
    if let Ok(output) = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.notifications", "show-banners", "true"])
        .output()
        .await
    {
        if output.status.success() {
            tracing::debug!("GNOME notifications resumed");
            success_count += 1;
        } else {
            last_error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        }
    }

    // Note: KDE inhibition should auto-expire when uninhibited or when the calling process exits
    // We could store and release the cookie, but for now we rely on the automatic timeout

    if success_count > 0 {
        let method = match desktop {
            LinuxDesktop::Dunst => DndMethod::LinuxDunst,
            LinuxDesktop::Gnome => DndMethod::LinuxGnome,
            LinuxDesktop::Kde => DndMethod::LinuxKde,
            _ => DndMethod::Manual,
        };
        DndOperationResult::success(method, "Notifications resumed")
    } else {
        DndOperationResult::partial(
            DndMethod::Unavailable,
            format!("Could not resume notifications: {}", last_error),
            "Disable Do Not Disturb from your desktop settings manually",
        )
    }
}

#[cfg(target_os = "linux")]
async fn check_permission_linux() -> Result<NotificationPermissionStatus> {
    let desktop = detect_linux_desktop().await;

    let (can_control, notes) = match desktop {
        LinuxDesktop::Dunst => (true, "Notification control available via dunst".to_string()),
        LinuxDesktop::Gnome => (true, "Notification control available via GNOME settings".to_string()),
        LinuxDesktop::Kde => (true, "Notification control available via KDE Plasma".to_string()),
        LinuxDesktop::Sway => (true, "Notification control available via mako/dunst".to_string()),
        LinuxDesktop::Unknown => {
            // Check if any method is available
            use tokio::process::Command;

            let has_dunst = Command::new("which").arg("dunstctl").output().await
                .map(|o| o.status.success()).unwrap_or(false);
            let has_gsettings = Command::new("which").arg("gsettings").output().await
                .map(|o| o.status.success()).unwrap_or(false);

            if has_dunst || has_gsettings {
                (true, "Notification control may be available".to_string())
            } else {
                (false, "No supported notification daemon found. Install dunst or use a supported desktop environment.".to_string())
            }
        }
    };

    Ok(NotificationPermissionStatus {
        can_control_dnd: can_control,
        permission_state: if can_control {
            PermissionState::Granted
        } else {
            PermissionState::Unavailable
        },
        notes: Some(notes),
    })
}
