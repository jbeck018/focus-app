// commands/focus_time.rs - Calendar-based Focus Time commands
//
// This module provides Tauri commands for:
// 1. Getting Focus Time events from calendar
// 2. Getting current active Focus Time
// 3. Managing allowed apps during Focus Time
// 4. Manual Focus Time control (start, end early)

use crate::{
    commands::calendar::get_calendar_events,
    focus_time::{
        app_registry::{get_app_categories, get_common_apps, AppEntry, AppRegistry, CategoryInfo},
        detect_focus_time_events, find_active_focus_time, find_upcoming_focus_times,
        FocusTimeEventParsed, FocusTimeState,
    },
    AppState, Error, Result,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

/// Response for Focus Time events query
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusTimeEventsResponse {
    /// All Focus Time events found in the calendar
    pub events: Vec<FocusTimeEventParsed>,
    /// The currently active Focus Time event (if any)
    pub active_event: Option<FocusTimeEventParsed>,
    /// Upcoming Focus Time events (within next hour)
    pub upcoming_events: Vec<FocusTimeEventParsed>,
}

/// Get all Focus Time events from connected calendars
///
/// Scans calendar events for the next 7 days and returns those
/// identified as Focus Time blocks.
#[tauri::command]
pub async fn get_focus_time_events(
    state: State<'_, AppState>,
) -> Result<FocusTimeEventsResponse> {
    // Get calendar events for the next 7 days
    let now = Utc::now();
    let end = now + Duration::days(7);

    let calendar_events = get_calendar_events(
        state.clone(),
        now.to_rfc3339(),
        end.to_rfc3339(),
    )
    .await?;

    // Detect Focus Time events
    let focus_events = detect_focus_time_events(&calendar_events);

    // Find active and upcoming events
    let active_event = find_active_focus_time(&focus_events);
    let upcoming_events = find_upcoming_focus_times(&focus_events);

    Ok(FocusTimeEventsResponse {
        events: focus_events,
        active_event,
        upcoming_events,
    })
}

/// Get the currently active Focus Time (if any)
#[tauri::command]
pub async fn get_active_focus_time(
    state: State<'_, AppState>,
) -> Result<Option<FocusTimeState>> {
    let focus_state = state.focus_time_state.read().await;

    if focus_state.active {
        Ok(Some(focus_state.clone()))
    } else {
        Ok(None)
    }
}

/// Get the list of apps allowed during the current Focus Time
#[tauri::command]
pub async fn get_allowed_apps(
    state: State<'_, AppState>,
) -> Result<Vec<String>> {
    let focus_state = state.focus_time_state.read().await;
    Ok(focus_state.allowed_apps.clone())
}

/// Request for overriding Focus Time apps
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideAppsRequest {
    /// Apps to add to the allowed list
    pub add: Option<Vec<String>>,
    /// Apps to remove from the allowed list
    pub remove: Option<Vec<String>>,
    /// Whether to reset to original allowed apps first
    pub reset: Option<bool>,
}

/// Override apps during an active Focus Time session
///
/// Allows dynamically adding or removing apps from the allowed list
/// during an active Focus Time session.
#[tauri::command]
pub async fn override_focus_time_apps(
    request: OverrideAppsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<String>> {
    let mut focus_state = state.focus_time_state.write().await;

    if !focus_state.active {
        return Err(Error::InvalidSession(
            "No active Focus Time session".to_string()
        ));
    }

    // Reset if requested
    if request.reset.unwrap_or(false) {
        focus_state.reset_overrides();
    }

    // Add apps
    if let Some(apps_to_add) = &request.add {
        for app in apps_to_add {
            focus_state.add_allowed_app(app);
        }
    }

    // Remove apps
    if let Some(apps_to_remove) = &request.remove {
        for app in apps_to_remove {
            focus_state.remove_allowed_app(app);
        }
    }

    tracing::info!(
        "Focus Time apps overridden: added {:?}, removed {:?}",
        request.add,
        request.remove
    );

    Ok(focus_state.allowed_apps.clone())
}

/// End the current Focus Time session early
#[tauri::command]
pub async fn end_focus_time_early(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    {
        let mut focus_state = state.focus_time_state.write().await;

        if !focus_state.active {
            return Err(Error::InvalidSession(
                "No active Focus Time to end".to_string()
            ));
        }

        focus_state.end(true);
    }

    // Notify the frontend
    if let Err(e) = app_handle.emit("focus-time-ended", serde_json::json!({
        "early": true,
        "timestamp": Utc::now().to_rfc3339(),
    })) {
        tracing::warn!("Failed to emit focus-time-ended event: {}", e);
    }

    tracing::info!("Focus Time ended early by user");

    Ok(())
}

/// Manually start a Focus Time from a calendar event
///
/// This allows users to start Focus Time manually from an upcoming
/// calendar event, rather than waiting for it to auto-activate.
#[tauri::command]
pub async fn start_focus_time_now(
    event_id: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<FocusTimeState> {
    // First check if Focus Time is already active
    {
        let focus_state = state.focus_time_state.read().await;
        if focus_state.active {
            return Err(Error::InvalidSession(
                "Focus Time is already active. End it first.".to_string()
            ));
        }
    }

    // Get the calendar event
    let now = Utc::now();
    let end = now + Duration::days(7);

    let calendar_events = get_calendar_events(
        state.clone(),
        now.to_rfc3339(),
        end.to_rfc3339(),
    )
    .await?;

    let focus_events = detect_focus_time_events(&calendar_events);

    // Find the requested event
    let event = focus_events
        .iter()
        .find(|e| e.id == event_id)
        .ok_or_else(|| Error::NotFound(format!("Focus Time event not found: {}", event_id)))?;

    // Start the Focus Time
    {
        let mut focus_state = state.focus_time_state.write().await;
        *focus_state = FocusTimeState::from_parsed_event(event);
        focus_state.manually_started = true;
    }

    // Notify the frontend
    if let Err(e) = app_handle.emit("focus-time-started", serde_json::json!({
        "eventId": event_id,
        "eventTitle": event.clean_title,
        "manual": true,
        "timestamp": Utc::now().to_rfc3339(),
    })) {
        tracing::warn!("Failed to emit focus-time-started event: {}", e);
    }

    tracing::info!("Focus Time manually started from event: {}", event.clean_title);

    let focus_state = state.focus_time_state.read().await;
    Ok(focus_state.clone())
}

/// Check if a specific process/app is allowed during current Focus Time
#[tauri::command]
pub async fn is_app_allowed_during_focus_time(
    app_name: String,
    state: State<'_, AppState>,
) -> Result<bool> {
    let focus_state = state.focus_time_state.read().await;
    Ok(focus_state.is_app_allowed(&app_name))
}

/// Get available app categories for Focus Time configuration
#[tauri::command]
pub async fn get_focus_time_categories() -> Result<Vec<CategoryInfo>> {
    Ok(get_app_categories())
}

/// Get common apps for Focus Time UI
#[tauri::command]
pub async fn get_focus_time_common_apps() -> Result<Vec<AppEntry>> {
    Ok(get_common_apps())
}

/// Expand app categories to individual apps
#[tauri::command]
pub async fn expand_focus_time_categories(
    items: Vec<String>,
) -> Result<Vec<String>> {
    let registry = AppRegistry::new();
    Ok(registry.expand_allowed_list(&items))
}

/// Response for Focus Time status
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusTimeStatusResponse {
    /// Whether Focus Time is active
    pub active: bool,
    /// Current Focus Time state (if active)
    pub state: Option<FocusTimeState>,
    /// Active calendar event (if from calendar)
    pub active_event: Option<FocusTimeEventParsed>,
    /// Upcoming Focus Time events
    pub upcoming_events: Vec<FocusTimeEventParsed>,
    /// Time until next Focus Time starts (in seconds, None if no upcoming)
    pub seconds_until_next: Option<i64>,
}

/// Get comprehensive Focus Time status
///
/// Returns the current Focus Time status including active state,
/// upcoming events, and time until next scheduled Focus Time.
#[tauri::command]
pub async fn get_focus_time_status(
    state: State<'_, AppState>,
) -> Result<FocusTimeStatusResponse> {
    let focus_state = state.focus_time_state.read().await;

    // Get calendar events for upcoming Focus Times
    let now = Utc::now();
    let end = now + Duration::days(1);

    let calendar_events = get_calendar_events(
        state.clone(),
        now.to_rfc3339(),
        end.to_rfc3339(),
    )
    .await
    .unwrap_or_default();

    let focus_events = detect_focus_time_events(&calendar_events);
    let active_event = find_active_focus_time(&focus_events);
    let upcoming_events = find_upcoming_focus_times(&focus_events);

    // Calculate time until next Focus Time
    let seconds_until_next = if !focus_state.active {
        upcoming_events
            .first()
            .map(|e| (e.start_time - now).num_seconds())
            .filter(|&s| s > 0)
    } else {
        None
    };

    Ok(FocusTimeStatusResponse {
        active: focus_state.active,
        state: if focus_state.active {
            Some(focus_state.clone())
        } else {
            None
        },
        active_event,
        upcoming_events,
        seconds_until_next,
    })
}

/// Sync Focus Time with calendar events
///
/// This should be called periodically to auto-activate Focus Time
/// when a calendar event starts and auto-deactivate when it ends.
#[tauri::command]
pub async fn sync_focus_time_with_calendar(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<bool> {
    let mut state_changed = false;

    // Check for auto-deactivation first
    {
        let mut focus_state = state.focus_time_state.write().await;

        if focus_state.active {
            if let Some(ends_at) = focus_state.ends_at {
                if Utc::now() >= ends_at {
                    focus_state.end(false);
                    state_changed = true;

                    if let Err(e) = app_handle.emit("focus-time-ended", serde_json::json!({
                        "early": false,
                        "scheduled": true,
                        "timestamp": Utc::now().to_rfc3339(),
                    })) {
                        tracing::warn!("Failed to emit focus-time-ended event: {}", e);
                    }

                    tracing::info!("Focus Time auto-deactivated (scheduled end time reached)");
                }
            }
        }
    }

    // Check for auto-activation
    let focus_state = state.focus_time_state.read().await;
    if !focus_state.active {
        drop(focus_state);

        // Get current calendar events
        let now = Utc::now();
        let end = now + Duration::hours(1);

        if let Ok(calendar_events) = get_calendar_events(
            state.clone(),
            now.to_rfc3339(),
            end.to_rfc3339(),
        )
        .await
        {
            let focus_events = detect_focus_time_events(&calendar_events);

            if let Some(active_event) = find_active_focus_time(&focus_events) {
                let mut focus_state = state.focus_time_state.write().await;

                // Double-check it's still inactive (race condition prevention)
                if !focus_state.active {
                    *focus_state = FocusTimeState::from_parsed_event(&active_event);
                    state_changed = true;

                    if let Err(e) = app_handle.emit("focus-time-started", serde_json::json!({
                        "eventId": active_event.id,
                        "eventTitle": active_event.clean_title,
                        "manual": false,
                        "scheduled": true,
                        "timestamp": Utc::now().to_rfc3339(),
                    })) {
                        tracing::warn!("Failed to emit focus-time-started event: {}", e);
                    }

                    tracing::info!(
                        "Focus Time auto-activated from calendar event: {}",
                        active_event.clean_title
                    );
                }
            }
        }
    }

    Ok(state_changed)
}
