// commands/focus.rs - Focus session management commands

use crate::{
    blocking::hosts,
    commands::timer,
    db::queries::{self, Session},
    state::{ActiveSession, AppState, SessionType, TimerState},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionRequest {
    pub planned_duration_minutes: i32,
    pub session_type: SessionType,
    pub blocked_apps: Vec<String>,
    pub blocked_websites: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub planned_duration_minutes: i32,
    pub session_type: String,
}

/// Free tier daily session limit
const FREE_TIER_DAILY_LIMIT: i64 = 3;

/// Start a new focus session
///
/// This command:
/// 1. Enforces subscription-based session limits
/// 2. Creates session record in database
/// 3. Updates active session state
/// 4. Enables blocking for specified apps/websites
/// 5. Broadcasts session-count-changed event
#[tauri::command]
pub async fn start_focus_session(
    request: StartSessionRequest,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<SessionResponse> {
    // Check if there's already an active session
    {
        let active = state.active_session.read().await;
        if active.is_some() {
            return Err(Error::InvalidSession(
                "A session is already active".to_string(),
            ));
        }
    }

    // Enforce session limits for free tier users
    {
        let auth_state = state.auth_state.read().await;
        let subscription_tier = auth_state
            .user
            .as_ref()
            .map(|u| u.subscription_tier.as_str())
            .unwrap_or("free");

        // Free tier users have a daily limit
        if subscription_tier == "free" || subscription_tier.is_empty() {
            let sessions_today = queries::count_todays_sessions(state.pool()).await?;
            if sessions_today >= FREE_TIER_DAILY_LIMIT {
                return Err(Error::SessionLimitReached(format!(
                    "Daily session limit reached ({}/{}). Upgrade to Pro for unlimited sessions.",
                    sessions_today, FREE_TIER_DAILY_LIMIT
                )));
            }
        }
    }

    // Create new session
    let session = ActiveSession::new(
        request.planned_duration_minutes,
        request.session_type.clone(),
        request.blocked_apps.clone(),
        request.blocked_websites.clone(),
    );

    // Insert into database
    queries::insert_session(
        state.pool(),
        &session.id,
        session.start_time,
        session.planned_duration_minutes,
        match &session.session_type {
            SessionType::Focus => "focus",
            SessionType::Break => "break",
            SessionType::Custom => "custom",
        },
    )
    .await?;

    // Add blocked items to database
    for app in &request.blocked_apps {
        queries::insert_blocked_item(state.pool(), "app", app).await?;
    }

    for website in &request.blocked_websites {
        queries::insert_blocked_item(state.pool(), "website", website).await?;
    }

    // Enable blocking and update state
    {
        let mut blocking = state.blocking_state.write().await;
        blocking.enable();
        blocking.update_blocked_websites(request.blocked_websites.clone());
    }

    // Update hosts file with blocked websites (may fail without privileges)
    if !request.blocked_websites.is_empty() {
        if let Err(e) = hosts::update_hosts_file(&request.blocked_websites).await {
            tracing::warn!("Failed to update hosts file: {}, DNS fallback active", e);
            // Don't fail the session start if hosts file update fails
            // DNS fallback will still work for frontend-based blocking
        }
    }

    let response = SessionResponse {
        id: session.id.clone(),
        start_time: session.start_time,
        planned_duration_minutes: session.planned_duration_minutes,
        session_type: format!("{:?}", session.session_type).to_lowercase(),
    };

    // Set active session
    {
        let mut active = state.active_session.write().await;
        *active = Some(session);
    }

    // Initialize and start timer state
    {
        let mut timer_state = state.timer_state.write().await;
        *timer_state = TimerState::new_running();
    }

    // Start the backend timer broadcast loop
    timer::start_timer_loop(state.app_handle.clone(), (*state).clone());

    // Broadcast session count changed event to all windows
    let sessions_today = queries::count_todays_sessions(state.pool()).await.unwrap_or(1);
    if let Err(e) = app_handle.emit(
        "session-count-changed",
        serde_json::json!({
            "sessionsToday": sessions_today,
            "dailyLimit": FREE_TIER_DAILY_LIMIT,
        }),
    ) {
        tracing::warn!("Failed to emit session-count-changed: {}", e);
    }

    // Send notification
    if let Err(e) = state
        .app_handle
        .notification()
        .builder()
        .title("Focus Session Started")
        .body(format!(
            "Stay focused for {} minutes!",
            request.planned_duration_minutes
        ))
        .show()
    {
        tracing::warn!("Failed to send notification: {}", e);
    }

    Ok(response)
}

/// End the current focus session
#[tauri::command]
pub async fn end_focus_session(
    completed: bool,
    state: State<'_, AppState>,
) -> Result<SessionResponse> {
    let session = {
        let mut active = state.active_session.write().await;
        active.take().ok_or_else(|| {
            Error::InvalidSession("No active session to end".to_string())
        })?
    };

    // Stop the timer and reset timer state
    {
        let mut timer_state = state.timer_state.write().await;
        timer_state.stop();
        *timer_state = TimerState::default();
    }

    let end_time = chrono::Utc::now();

    // Update database
    queries::end_session(state.pool(), &session.id, end_time, completed).await?;

    // Disable blocking and clear state
    {
        let mut blocking = state.blocking_state.write().await;
        blocking.disable();
        blocking.update_blocked_websites(Vec::new());
    }

    // Clear hosts file to remove website blocking
    if let Err(e) = hosts::clear_hosts_file().await {
        tracing::warn!("Failed to clear hosts file: {}", e);
        // Don't fail the session end if hosts file clearing fails
    }

    // Update analytics
    let date = end_time.format("%Y-%m-%d").to_string();
    let duration = session.elapsed_seconds();

    let (focus_seconds, break_seconds) = match session.session_type {
        SessionType::Focus => (duration, 0),
        SessionType::Break => (0, duration),
        SessionType::Custom => (duration, 0),
    };

    queries::upsert_daily_analytics(
        state.pool(),
        &date,
        focus_seconds,
        break_seconds,
        if completed { 1 } else { 0 },
        if completed { 0 } else { 1 },
    )
    .await?;

    // Check achievements if session was completed
    // Note: Running inline rather than spawned to avoid state lifetime issues
    if completed {
        if let Err(e) = super::achievements::check_achievements(session.id.clone(), state.clone()).await {
            tracing::warn!("Failed to check achievements: {}", e);
        }
    }

    // Send completion notification
    let notification_body = if completed {
        format!("Great job! You focused for {} seconds", duration)
    } else {
        "Session ended early".to_string()
    };

    if let Err(e) = state
        .app_handle
        .notification()
        .builder()
        .title("Focus Session Ended")
        .body(notification_body)
        .show()
    {
        tracing::warn!("Failed to send notification: {}", e);
    }

    Ok(SessionResponse {
        id: session.id,
        start_time: session.start_time,
        planned_duration_minutes: session.planned_duration_minutes,
        session_type: format!("{:?}", session.session_type).to_lowercase(),
    })
}

/// Get the currently active session if any
#[tauri::command]
pub async fn get_active_session(
    state: State<'_, AppState>,
) -> Result<Option<ActiveSession>> {
    let active = state.active_session.read().await;
    Ok(active.clone())
}

/// Get session history for a date range
#[tauri::command]
pub async fn get_session_history(
    days: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Session>> {
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(days);

    let sessions = queries::get_sessions_in_range(state.pool(), start, end).await?;

    Ok(sessions)
}

/// Emit toggle pause/resume event to main window
/// The main window's timer component manages the actual pause/resume logic
#[tauri::command]
pub async fn toggle_session_pause(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    // Verify there's an active session
    {
        let active = state.active_session.read().await;
        if active.is_none() {
            return Err(Error::InvalidSession(
                "No active session to toggle".to_string(),
            ));
        }
    }

    // Emit event to main window to toggle the timer
    if let Some(window) = app_handle.get_webview_window("main") {
        if let Err(e) = window.emit("mini-timer-toggle", ()) {
            tracing::warn!("Failed to emit toggle event: {}", e);
        }
    }

    Ok(())
}

/// Extend the current session by additional minutes
/// Updates the planned duration and emits "session-extended" event
#[tauri::command]
pub async fn extend_session(
    additional_minutes: i32,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<i32> {
    let new_duration = {
        let mut active = state.active_session.write().await;
        let session = active.as_mut().ok_or_else(|| {
            Error::InvalidSession("No active session to extend".to_string())
        })?;

        session.planned_duration_minutes += additional_minutes;
        session.planned_duration_minutes
    };

    // Emit event to all windows about the extension
    if let Some(window) = app_handle.get_webview_window("main") {
        if let Err(e) = window.emit(
            "session-extended",
            serde_json::json!({
                "plannedDurationMinutes": new_duration,
                "additionalMinutes": additional_minutes,
            }),
        ) {
            tracing::warn!("Failed to emit extension event: {}", e);
        }
    }
    // Also emit to mini-timer window if it exists
    if let Some(window) = app_handle.get_webview_window("mini-timer") {
        let _ = window.emit(
            "session-extended",
            serde_json::json!({
                "plannedDurationMinutes": new_duration,
                "additionalMinutes": additional_minutes,
            }),
        );
    }

    Ok(new_duration)
}

/// Session count response with limit info
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCountResponse {
    pub sessions_today: i64,
    pub daily_limit: i64,
    pub is_unlimited: bool,
}

/// Get today's session count and limit info
/// Used by frontend to display remaining sessions for free tier
#[tauri::command]
pub async fn get_todays_session_count(
    state: State<'_, AppState>,
) -> Result<SessionCountResponse> {
    let sessions_today = queries::count_todays_sessions(state.pool()).await?;

    // Check subscription tier
    let auth_state = state.auth_state.read().await;
    let subscription_tier = auth_state
        .user
        .as_ref()
        .map(|u| u.subscription_tier.as_str())
        .unwrap_or("free");

    let is_unlimited = subscription_tier == "pro" || subscription_tier == "team";

    Ok(SessionCountResponse {
        sessions_today,
        daily_limit: FREE_TIER_DAILY_LIMIT,
        is_unlimited,
    })
}
