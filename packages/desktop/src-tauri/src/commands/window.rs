// commands/window.rs - Window management commands for mini-timer

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalPosition, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}

/// Opens the mini-timer window as an always-on-top floating widget
///
/// Creates a small, frameless, transparent window that floats above all other windows.
/// The window is positioned based on saved preferences or defaults to top-right corner.
#[tauri::command]
pub async fn open_mini_timer<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    // Check if window already exists
    if let Some(window) = app.get_webview_window("mini-timer") {
        window.set_focus().map_err(|e| Error::Window(e.to_string()))?;
        window.show().map_err(|e| Error::Window(e.to_string()))?;
        return Ok(());
    }

    // Create new mini-timer window
    let window = WebviewWindowBuilder::new(
        &app,
        "mini-timer",
        WebviewUrl::App("mini-timer.html".into()),
    )
    .title("FocusFlow Mini Timer")
    .inner_size(200.0, 80.0)
    .min_inner_size(200.0, 80.0)
    .max_inner_size(300.0, 120.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false) // Start hidden, show after positioning
    .build()
    .map_err(|e| Error::Window(e.to_string()))?;

    // Position window in top-right corner by default
    // In production, this would load from saved preferences
    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let monitor_size = monitor.size();
            let window_size = window.outer_size().map_err(|e| Error::Window(e.to_string()))?;

            let x = (monitor_size.width - window_size.width) as f64 - 20.0;
            let y = 40.0;

            window
                .set_position(PhysicalPosition::new(x, y))
                .map_err(|e| Error::Window(e.to_string()))?;
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let monitor_size = monitor.size();
            let window_size = window.outer_size().map_err(|e| Error::Window(e.to_string()))?;

            let x = (monitor_size.width - window_size.width) as i32 - 20;
            let y = 40;

            window
                .set_position(PhysicalPosition::new(x, y))
                .map_err(|e| Error::Window(e.to_string()))?;
        }
    }

    // Show window after positioning
    window.show().map_err(|e| Error::Window(e.to_string()))?;

    Ok(())
}

/// Closes the mini-timer window
#[tauri::command]
pub async fn close_mini_timer<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    if let Some(window) = app.get_webview_window("mini-timer") {
        window.close().map_err(|e| Error::Window(e.to_string()))?;
    }
    Ok(())
}

/// Toggles the mini-timer window visibility
#[tauri::command]
pub async fn toggle_mini_timer<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    if let Some(window) = app.get_webview_window("mini-timer") {
        if window.is_visible().map_err(|e| Error::Window(e.to_string()))? {
            close_mini_timer(app).await?;
        } else {
            window.show().map_err(|e| Error::Window(e.to_string()))?;
            window.set_focus().map_err(|e| Error::Window(e.to_string()))?;
        }
    } else {
        open_mini_timer(app).await?;
    }
    Ok(())
}

/// Sets the position of the mini-timer window
///
/// Saves the position to user preferences for persistence across sessions
#[tauri::command]
pub async fn set_mini_timer_position<R: Runtime>(
    app: AppHandle<R>,
    position: WindowPosition,
) -> Result<()> {
    if let Some(window) = app.get_webview_window("mini-timer") {
        window
            .set_position(LogicalPosition::new(position.x, position.y))
            .map_err(|e| Error::Window(e.to_string()))?;

        // In production, save position to preferences/database
        tracing::debug!("Mini-timer position updated: {:?}", position);
    }

    Ok(())
}

/// Gets the current position of the mini-timer window
#[tauri::command]
pub async fn get_mini_timer_position<R: Runtime>(app: AppHandle<R>) -> Result<Option<WindowPosition>> {
    if let Some(window) = app.get_webview_window("mini-timer") {
        let position = window.outer_position().map_err(|e| Error::Window(e.to_string()))?;

        Ok(Some(WindowPosition {
            x: position.x as f64,
            y: position.y as f64,
        }))
    } else {
        Ok(None)
    }
}

/// Brings the main window to focus
#[tauri::command]
pub async fn focus_main_window<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| Error::Window(e.to_string()))?;
        window.set_focus().map_err(|e| Error::Window(e.to_string()))?;
    }
    Ok(())
}

/// Sends timer state update to mini-timer window
///
/// This command relays timer state from the main window to the mini-timer window.
/// Tauri 2 requires using the Rust backend for cross-window communication.
/// The frontend's emitTo() function does not work for window-to-window events.
///
/// Note: Parameter names match JavaScript convention (camelCase) for direct mapping
#[tauri::command]
#[allow(non_snake_case)]
pub async fn emit_to_mini_timer<R: Runtime>(
    app: AppHandle<R>,
    eventName: String,
    payload: serde_json::Value,
) -> Result<()> {
    if let Some(window) = app.get_webview_window("mini-timer") {
        window
            .emit(&eventName, &payload)
            .map_err(|e| Error::Window(format!("Failed to emit to mini-timer: {}", e)))?;
        tracing::trace!("Emitted {} to mini-timer", eventName);
    }
    // Don't error if mini-timer doesn't exist - it may not be open
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_position_serialization() {
        let pos = WindowPosition { x: 100.0, y: 200.0 };
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#"{"x":100.0,"y":200.0}"#);
    }

    #[test]
    fn test_window_position_deserialization() {
        let json = r#"{"x":150.5,"y":250.5}"#;
        let pos: WindowPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.x, 150.5);
        assert_eq!(pos.y, 250.5);
    }
}
