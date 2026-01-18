// commands/dimming.rs - Screen dimming overlay commands
//
// Provides commands to enable/disable screen dimming during focus mode.
// The dimming overlay dims everything except the focused application window.

use crate::system::dimming::DimmingState;
use crate::{AppState, Result};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

/// Response for dimming state queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimmingStateResponse {
    pub enabled: bool,
    pub opacity: f32,
    pub strict_mode: bool,
    pub session_id: Option<String>,
}

impl From<&DimmingState> for DimmingStateResponse {
    fn from(state: &DimmingState) -> Self {
        Self {
            enabled: state.enabled,
            opacity: state.opacity,
            strict_mode: state.strict_mode,
            session_id: state.session_id.clone(),
        }
    }
}

/// Enable screen dimming overlay
///
/// Creates overlay windows on all monitors that dim content except
/// the foreground window.
#[tauri::command]
pub async fn enable_screen_dimming(
    opacity: Option<f32>,
    strict: Option<bool>,
    session_id: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DimmingStateResponse> {
    let opacity = opacity.unwrap_or(0.7);
    let strict = strict.unwrap_or(true);

    // Update state
    {
        let mut dimming = state.dimming_state.write().await;
        dimming.enable(opacity, strict, session_id.clone());
    }

    // Start the dimming overlay
    if let Err(e) = start_dimming_overlay(&app_handle, opacity).await {
        tracing::warn!("Failed to start dimming overlay: {}", e);
        // Continue anyway - state is set, overlay might work partially
    }

    // Emit event to notify UI
    if let Err(e) = app_handle.emit("dimming-state-changed", serde_json::json!({
        "enabled": true,
        "opacity": opacity,
        "strictMode": strict,
    })) {
        tracing::warn!("Failed to emit dimming state event: {}", e);
    }

    let dimming = state.dimming_state.read().await;
    Ok(DimmingStateResponse::from(&*dimming))
}

/// Disable screen dimming overlay
#[tauri::command]
pub async fn disable_screen_dimming(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DimmingStateResponse> {
    // Check strict mode - if tied to a session, don't allow disable
    {
        let dimming = state.dimming_state.read().await;
        if dimming.strict_mode && dimming.session_id.is_some() {
            return Err(crate::Error::Validation(
                "Cannot disable dimming in strict mode while session is active".to_string(),
            ));
        }
    }

    // Update state
    {
        let mut dimming = state.dimming_state.write().await;
        dimming.disable();
    }

    // Stop the dimming overlay
    if let Err(e) = stop_dimming_overlay(&app_handle).await {
        tracing::warn!("Failed to stop dimming overlay: {}", e);
    }

    // Emit event to notify UI
    if let Err(e) = app_handle.emit("dimming-state-changed", serde_json::json!({
        "enabled": false,
    })) {
        tracing::warn!("Failed to emit dimming state event: {}", e);
    }

    let dimming = state.dimming_state.read().await;
    Ok(DimmingStateResponse::from(&*dimming))
}

/// Force disable dimming (bypasses strict mode check)
/// Used when ending a focus session
pub async fn force_disable_dimming(
    state: &AppState,
    app_handle: &tauri::AppHandle,
) -> Result<()> {
    {
        let mut dimming = state.dimming_state.write().await;
        dimming.disable();
    }

    if let Err(e) = stop_dimming_overlay(app_handle).await {
        tracing::warn!("Failed to stop dimming overlay: {}", e);
    }

    if let Err(e) = app_handle.emit("dimming-state-changed", serde_json::json!({
        "enabled": false,
    })) {
        tracing::warn!("Failed to emit dimming state event: {}", e);
    }

    Ok(())
}

/// Internal function to enable dimming without State wrapper
/// Used when calling from other commands like focus session start
pub async fn enable_dimming_internal(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    opacity: f32,
    strict: bool,
    session_id: Option<String>,
) -> Result<()> {
    // Update state
    {
        let mut dimming = state.dimming_state.write().await;
        dimming.enable(opacity, strict, session_id);
    }

    // Start the dimming overlay
    if let Err(e) = start_dimming_overlay(app_handle, opacity).await {
        tracing::warn!("Failed to start dimming overlay: {}", e);
    }

    // Emit event to notify UI
    if let Err(e) = app_handle.emit("dimming-state-changed", serde_json::json!({
        "enabled": true,
        "opacity": opacity,
        "strictMode": strict,
    })) {
        tracing::warn!("Failed to emit dimming state event: {}", e);
    }

    Ok(())
}

/// Get current dimming state
#[tauri::command]
pub async fn get_dimming_state(
    state: State<'_, AppState>,
) -> Result<DimmingStateResponse> {
    let dimming = state.dimming_state.read().await;
    Ok(DimmingStateResponse::from(&*dimming))
}

/// Set dimming opacity
#[tauri::command]
pub async fn set_dimming_opacity(
    opacity: f32,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DimmingStateResponse> {
    let was_enabled = {
        let mut dimming = state.dimming_state.write().await;
        let was_enabled = dimming.enabled;
        dimming.set_opacity(opacity);
        was_enabled
    };

    // Update overlay opacity if enabled
    if was_enabled {
        if let Err(e) = update_dimming_opacity(&app_handle, opacity).await {
            tracing::warn!("Failed to update dimming opacity: {}", e);
        }
    }

    let dimming = state.dimming_state.read().await;
    Ok(DimmingStateResponse::from(&*dimming))
}

// ============================================================================
// Platform-specific overlay implementation
// ============================================================================

/// Start the dimming overlay on all monitors
async fn start_dimming_overlay(app_handle: &tauri::AppHandle, opacity: f32) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        start_dimming_overlay_macos(app_handle, opacity).await
    }

    #[cfg(target_os = "windows")]
    {
        start_dimming_overlay_windows(app_handle, opacity).await
    }

    #[cfg(target_os = "linux")]
    {
        start_dimming_overlay_linux(app_handle, opacity).await
    }
}

/// Stop the dimming overlay
async fn stop_dimming_overlay(app_handle: &tauri::AppHandle) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        stop_dimming_overlay_macos(app_handle).await
    }

    #[cfg(target_os = "windows")]
    {
        stop_dimming_overlay_windows(app_handle).await
    }

    #[cfg(target_os = "linux")]
    {
        stop_dimming_overlay_linux(app_handle).await
    }
}

/// Update overlay opacity
async fn update_dimming_opacity(app_handle: &tauri::AppHandle, opacity: f32) -> Result<()> {
    // For now, restart the overlay with new opacity
    // A more sophisticated implementation would update in-place
    stop_dimming_overlay(app_handle).await?;
    start_dimming_overlay(app_handle, opacity).await
}

// ============================================================================
// macOS implementation
// ============================================================================

#[cfg(target_os = "macos")]
async fn start_dimming_overlay_macos(app_handle: &tauri::AppHandle, opacity: f32) -> Result<()> {
    use tauri::WebviewWindowBuilder;

    // Get all monitors
    let monitors = app_handle.available_monitors()
        .map_err(|e| crate::Error::System(format!("Failed to get monitors: {}", e)))?;

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("dimming-overlay-{}", i);
        let position = monitor.position();
        let size = monitor.size();

        // Create fullscreen overlay window
        let window = WebviewWindowBuilder::new(
            app_handle,
            &label,
            tauri::WebviewUrl::App("dimming-overlay.html".into()),
        )
        .title("")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .position(position.x as f64, position.y as f64)
        .inner_size(size.width as f64, size.height as f64)
        .build();

        match window {
            Ok(win) => {
                // Send opacity to the overlay window
                if let Err(e) = win.emit("set-opacity", opacity) {
                    tracing::warn!("Failed to set overlay opacity: {}", e);
                }
                tracing::info!("Created dimming overlay on monitor {}", i);
            }
            Err(e) => {
                tracing::warn!("Failed to create dimming overlay on monitor {}: {}", i, e);
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn stop_dimming_overlay_macos(app_handle: &tauri::AppHandle) -> Result<()> {
    // Close all dimming overlay windows
    for i in 0..10 {
        let label = format!("dimming-overlay-{}", i);
        if let Some(window) = app_handle.get_webview_window(&label) {
            if let Err(e) = window.close() {
                tracing::warn!("Failed to close overlay window {}: {}", label, e);
            }
        }
    }
    Ok(())
}

// ============================================================================
// Windows implementation
// ============================================================================

#[cfg(target_os = "windows")]
async fn start_dimming_overlay_windows(app_handle: &tauri::AppHandle, opacity: f32) -> Result<()> {
    use tauri::WebviewWindowBuilder;

    // Get all monitors
    let monitors = app_handle.available_monitors()
        .map_err(|e| crate::Error::System(format!("Failed to get monitors: {}", e)))?;

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("dimming-overlay-{}", i);
        let position = monitor.position();
        let size = monitor.size();

        // Create fullscreen overlay window
        let window = WebviewWindowBuilder::new(
            app_handle,
            &label,
            tauri::WebviewUrl::App("dimming-overlay.html".into()),
        )
        .title("")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .position(position.x as f64, position.y as f64)
        .inner_size(size.width as f64, size.height as f64)
        .build();

        match window {
            Ok(win) => {
                if let Err(e) = win.emit("set-opacity", opacity) {
                    tracing::warn!("Failed to set overlay opacity: {}", e);
                }
                tracing::info!("Created dimming overlay on monitor {}", i);
            }
            Err(e) => {
                tracing::warn!("Failed to create dimming overlay on monitor {}: {}", i, e);
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
async fn stop_dimming_overlay_windows(app_handle: &tauri::AppHandle) -> Result<()> {
    for i in 0..10 {
        let label = format!("dimming-overlay-{}", i);
        if let Some(window) = app_handle.get_webview_window(&label) {
            if let Err(e) = window.close() {
                tracing::warn!("Failed to close overlay window {}: {}", label, e);
            }
        }
    }
    Ok(())
}

// ============================================================================
// Linux implementation
// ============================================================================

#[cfg(target_os = "linux")]
async fn start_dimming_overlay_linux(app_handle: &tauri::AppHandle, opacity: f32) -> Result<()> {
    use tauri::WebviewWindowBuilder;

    // Note: On Wayland, overlay windows have restrictions
    // This implementation works best on X11
    let monitors = app_handle.available_monitors()
        .map_err(|e| crate::Error::System(format!("Failed to get monitors: {}", e)))?;

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("dimming-overlay-{}", i);
        let position = monitor.position();
        let size = monitor.size();

        let window = WebviewWindowBuilder::new(
            app_handle,
            &label,
            tauri::WebviewUrl::App("dimming-overlay.html".into()),
        )
        .title("")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .position(position.x as f64, position.y as f64)
        .inner_size(size.width as f64, size.height as f64)
        .build();

        match window {
            Ok(win) => {
                if let Err(e) = win.emit("set-opacity", opacity) {
                    tracing::warn!("Failed to set overlay opacity: {}", e);
                }
                tracing::info!("Created dimming overlay on monitor {}", i);
            }
            Err(e) => {
                // On Wayland this might fail - log but don't error
                tracing::warn!(
                    "Failed to create dimming overlay on monitor {} (may not work on Wayland): {}",
                    i, e
                );
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
async fn stop_dimming_overlay_linux(app_handle: &tauri::AppHandle) -> Result<()> {
    for i in 0..10 {
        let label = format!("dimming-overlay-{}", i);
        if let Some(window) = app_handle.get_webview_window(&label) {
            if let Err(e) = window.close() {
                tracing::warn!("Failed to close overlay window {}: {}", label, e);
            }
        }
    }
    Ok(())
}
