// commands/tray.rs - System tray icon state management commands

use crate::system::tray::{TrayIconState, update_tray_icon, get_current_tray_state};
use crate::Result;
use serde::{Deserialize, Serialize};

/// Tray state request from frontend
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrayStateRequest {
    Idle,
    Focus,
    Break,
    Paused,
}

impl From<TrayStateRequest> for TrayIconState {
    fn from(req: TrayStateRequest) -> Self {
        match req {
            TrayStateRequest::Idle => TrayIconState::Idle,
            TrayStateRequest::Focus => TrayIconState::Focus,
            TrayStateRequest::Break => TrayIconState::Break,
            TrayStateRequest::Paused => TrayIconState::Paused,
        }
    }
}

/// Tray state response to frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayStateResponse {
    pub state: String,
    pub tooltip: String,
}

impl From<TrayIconState> for TrayStateResponse {
    fn from(state: TrayIconState) -> Self {
        let (state_str, tooltip) = match state {
            TrayIconState::Idle => ("idle", "FocusFlow - Ready"),
            TrayIconState::Focus => ("focus", "FocusFlow - Focus Session Active"),
            TrayIconState::Break => ("break", "FocusFlow - Break Time"),
            TrayIconState::Paused => ("paused", "FocusFlow - Session Paused"),
        };
        TrayStateResponse {
            state: state_str.to_string(),
            tooltip: tooltip.to_string(),
        }
    }
}

/// Update the tray icon state
///
/// This command is called by the frontend when session state changes:
/// - When a focus session starts: set to "focus"
/// - When a break starts: set to "break"
/// - When session is paused: set to "paused"
/// - When session ends: set to "idle"
#[tauri::command]
pub fn set_tray_state(
    state: TrayStateRequest,
    app_handle: tauri::AppHandle,
) -> Result<TrayStateResponse> {
    let tray_state: TrayIconState = state.into();
    update_tray_icon(&app_handle, tray_state);
    Ok(tray_state.into())
}

/// Get the current tray icon state
#[tauri::command]
pub fn get_tray_state() -> Result<TrayStateResponse> {
    let state = get_current_tray_state();
    Ok(state.into())
}
