// commands/focus.rs - Focus session management commands

use crate::{
    blocking::hosts,
    commands::timer,
    db::queries::{self, Session},
    state::{ActiveSession, AppState, SessionType, TimerState},
    system::notifications::NotificationManager,
    Error, Result,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionRequest {
    pub planned_duration_minutes: i32,
    pub session_type: SessionType,
    pub blocked_apps: Vec<String>,
    pub blocked_websites: Vec<String>,
    /// Enable screen dimming overlay during focus mode
    #[serde(default)]
    pub enable_dimming: bool,
    /// Dimming opacity (0.0-1.0)
    #[serde(default = "default_dimming_opacity")]
    pub dimming_opacity: f32,
    /// Pause system notifications during focus mode
    #[serde(default)]
    pub pause_notifications: bool,
}

fn default_dimming_opacity() -> f32 {
    0.7
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
/// 1. Validates input parameters
/// 2. Enforces subscription-based session limits
/// 3. Creates session record in database
/// 4. Updates active session state
/// 5. Enables blocking for specified apps/websites
/// 6. Broadcasts session-count-changed event
#[tauri::command]
pub async fn start_focus_session(
    request: StartSessionRequest,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<SessionResponse> {
    // Validate duration
    if request.planned_duration_minutes <= 0 {
        return Err(Error::Validation("Duration must be positive".into()));
    }
    if request.planned_duration_minutes > 480 {
        return Err(Error::Validation("Duration cannot exceed 8 hours".into()));
    }

    // Validate and sanitize blocked apps (prevent injection attacks)
    validate_blocked_apps(&request.blocked_apps)?;

    // Validate and sanitize blocked websites (prevent injection attacks)
    validate_blocked_websites(&request.blocked_websites)?;

    // SECURITY FIX: Hold write lock during entire check-and-set operation
    // This prevents TOCTOU race condition where multiple sessions could be started simultaneously
    let mut active = state.active_session.write().await;

    // Check if there's already an active session (while holding write lock)
    if active.is_some() {
        return Err(Error::InvalidSession(
            "A session is already active".to_string(),
        ));
    }

    // Enforce session limits for free tier users
    // Cache the session count to avoid duplicate queries
    let sessions_today = {
        let auth_state = state.auth_state.read().await;
        let subscription_tier = auth_state
            .user
            .as_ref()
            .map(|u| u.subscription_tier.as_str())
            .unwrap_or("free");

        // Free tier users have a daily limit
        if subscription_tier == "free" || subscription_tier.is_empty() {
            let sessions_count = queries::count_todays_sessions(state.pool()).await?;
            if sessions_count >= FREE_TIER_DAILY_LIMIT {
                return Err(Error::SessionLimitReached(format!(
                    "Daily session limit reached ({}/{}). Upgrade to Pro for unlimited sessions.",
                    sessions_count, FREE_TIER_DAILY_LIMIT
                )));
            }
            sessions_count
        } else {
            0
        }
    };

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

    // Batch insert all blocked items in a single query to avoid N+1 queries
    let mut blocked_items = Vec::new();
    for app in &request.blocked_apps {
        blocked_items.push(("app", app.as_str()));
    }
    for website in &request.blocked_websites {
        blocked_items.push(("website", website.as_str()));
    }
    if !blocked_items.is_empty() {
        queries::insert_blocked_items_batch(state.pool(), blocked_items).await?;
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

    // Set active session (still holding write lock from line 83)
    *active = Some(session);

    // Release write lock before proceeding with timer initialization
    drop(active);

    // Initialize and start timer state
    {
        let mut timer_state = state.timer_state.write().await;
        *timer_state = TimerState::new_running();
    }

    // Start the backend timer broadcast loop and store cancellation sender
    {
        let cancel_tx = timer::start_timer_loop(state.app_handle.clone(), (*state).clone());
        let mut timer_cancellation = state.timer_cancellation.write().await;
        *timer_cancellation = Some(cancel_tx);
    }

    // Enable screen dimming if requested
    if request.enable_dimming {
        let dimming_state = (*state).clone();
        let dimming_handle = app_handle.clone();
        let opacity = request.dimming_opacity;
        let session_id = response.id.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = super::dimming::enable_dimming_internal(
                &dimming_state,
                &dimming_handle,
                opacity,
                true,
                Some(session_id),
            ).await {
                tracing::warn!("Failed to enable screen dimming: {}", e);
            }
        });
    }

    // Pause notifications if requested
    if request.pause_notifications {
        let notification_state = (*state).clone();
        let notification_handle = app_handle.clone();
        let session_id = response.id.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = super::notification_control::pause_notifications_internal(
                &notification_state,
                &notification_handle,
                Some(session_id),
            ).await {
                tracing::warn!("Failed to pause notifications: {}", e);
            }
        });
    }

    // Broadcast session count changed event to all windows
    // Use cached sessions_today if available, otherwise query (for pro/team users)
    let sessions_count = if sessions_today > 0 {
        sessions_today + 1  // New session was just created
    } else {
        // For pro/team users or if not cached, query the count
        queries::count_todays_sessions(state.pool()).await.unwrap_or(1)
    };
    if let Err(e) = app_handle.emit(
        "session-count-changed",
        serde_json::json!({
            "sessionsToday": sessions_count,
            "dailyLimit": FREE_TIER_DAILY_LIMIT,
        }),
    ) {
        tracing::warn!("Failed to emit session-count-changed: {}", e);
    }

    // Send session started notification via NotificationManager
    let notification_manager = NotificationManager::new(app_handle.clone());
    if let Err(e) = notification_manager.session_started(request.planned_duration_minutes) {
        tracing::warn!("Failed to send session started notification: {}", e);
    }

    // Schedule break reminders for long sessions (>25 minutes)
    if request.planned_duration_minutes > 25 {
        let reminder_handle = app_handle.clone();
        let cancel_tx = crate::system::notifications::schedule_break_reminders(reminder_handle, 25);
        // Store the cancellation sender in state
        let mut break_cancellation = state.break_reminder_cancellation.write().await;
        *break_cancellation = Some(cancel_tx);
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

    // Stop the timer loop and reset timer state
    {
        let mut timer_state = state.timer_state.write().await;
        timer_state.stop();
        *timer_state = TimerState::default();
    }

    // Cancel the timer broadcast loop
    timer::stop_timer_loop(&state).await;

    // Cancel break reminder loop if it was started
    {
        let mut break_cancellation = state.break_reminder_cancellation.write().await;
        if let Some(sender) = break_cancellation.take() {
            let _ = sender.send(());
            tracing::debug!("Break reminder loop cancelled");
        }
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

    // Disable screen dimming if it was enabled
    {
        let dimming_state = state.dimming_state.read().await;
        if dimming_state.enabled {
            drop(dimming_state);
            if let Err(e) = super::dimming::force_disable_dimming(&state, &state.app_handle).await {
                tracing::warn!("Failed to disable screen dimming: {}", e);
            }
        }
    }

    // Resume notifications if they were paused
    {
        let notification_state = state.notification_control_state.read().await;
        if notification_state.paused {
            drop(notification_state);
            if let Err(e) = super::notification_control::force_resume_notifications(&state, &state.app_handle).await {
                tracing::warn!("Failed to resume notifications: {}", e);
            }
        }
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

    // Send completion notification via NotificationManager
    let notification_manager = NotificationManager::new(state.app_handle.clone());
    if completed {
        if let Err(e) = notification_manager.session_completed(duration) {
            tracing::warn!("Failed to send session completed notification: {}", e);
        }
    } else if let Err(e) = notification_manager.session_abandoned() {
        tracing::warn!("Failed to send session abandoned notification: {}", e);
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
/// Updates the planned duration, persists to database, and broadcasts to all windows
#[tauri::command]
pub async fn extend_session(
    additional_minutes: i32,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<i32> {
    // Validate input
    if additional_minutes <= 0 {
        return Err(Error::InvalidSession(
            "Extension must be at least 1 minute".to_string(),
        ));
    }
    if additional_minutes > 120 {
        return Err(Error::InvalidSession(
            "Extension cannot exceed 120 minutes".to_string(),
        ));
    }

    let (session_id, new_duration) = {
        let mut active = state.active_session.write().await;
        let session = active.as_mut().ok_or_else(|| {
            Error::InvalidSession("No active session to extend".to_string())
        })?;

        session.planned_duration_minutes += additional_minutes;
        (session.id.clone(), session.planned_duration_minutes)
    };

    // Persist to database
    let rows_affected = queries::update_session_duration(
        state.pool(),
        &session_id,
        new_duration,
    ).await?;

    if rows_affected == 0 {
        tracing::warn!(
            "No rows updated when extending session {} - session may have ended",
            session_id
        );
    }

    // Broadcast to ALL windows using app-level emit
    if let Err(e) = app_handle.emit(
        "session-extended",
        serde_json::json!({
            "sessionId": session_id,
            "plannedDurationMinutes": new_duration,
            "additionalMinutes": additional_minutes,
        }),
    ) {
        tracing::warn!("Failed to emit extension event: {}", e);
    }

    // Send notification via NotificationManager
    let notification_manager = NotificationManager::new(state.app_handle.clone());
    if let Err(e) = notification_manager.custom(
        "Session Extended",
        &format!("+{} minutes. New duration: {} minutes", additional_minutes, new_duration),
    ) {
        tracing::warn!("Failed to send extension notification: {}", e);
    }

    tracing::info!(
        "Session {} extended by {} minutes (new total: {})",
        session_id, additional_minutes, new_duration
    );

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

// ============================================================================
// Input Validation Helpers
// ============================================================================

/// Validate blocked apps list for security
///
/// Prevents:
/// - Empty strings
/// - Null bytes (injection attacks)
/// - Path traversal attempts
/// - Shell metacharacters
fn validate_blocked_apps(apps: &[String]) -> Result<()> {
    for app in apps {
        let trimmed = app.trim();

        // Reject empty strings
        if trimmed.is_empty() {
            return Err(Error::InvalidInput(
                "App name cannot be empty".to_string(),
            ));
        }

        // Reject null bytes (prevent injection)
        if trimmed.contains('\0') {
            return Err(Error::InvalidInput(
                "App name contains invalid characters".to_string(),
            ));
        }

        // Reject path traversal attempts
        if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
            return Err(Error::InvalidInput(
                "App name cannot contain path separators".to_string(),
            ));
        }

        // Reject shell metacharacters that could be used for injection
        let dangerous_chars = ['$', '`', '|', '&', ';', '<', '>', '(', ')', '{', '}', '[', ']', '!', '#'];
        if trimmed.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(Error::InvalidInput(
                "App name contains invalid shell metacharacters".to_string(),
            ));
        }

        // Check maximum length (reasonable limit for process names)
        if trimmed.len() > 255 {
            return Err(Error::InvalidInput(
                "App name is too long (max 255 characters)".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate blocked websites list for security
///
/// Prevents:
/// - Empty strings
/// - Null bytes (injection attacks)
/// - Invalid domain formats
/// - Path traversal attempts
fn validate_blocked_websites(websites: &[String]) -> Result<()> {
    for website in websites {
        let domain = website.trim().to_lowercase();

        // Reject empty strings
        if domain.is_empty() {
            return Err(Error::InvalidInput(
                "Domain cannot be empty".to_string(),
            ));
        }

        // Reject null bytes (prevent injection)
        if domain.contains('\0') {
            return Err(Error::InvalidInput(
                "Domain contains invalid characters".to_string(),
            ));
        }

        // Check domain length (max 253 chars per DNS spec)
        if domain.len() > 253 {
            return Err(Error::InvalidInput(
                "Domain name is too long (max 253 characters)".to_string(),
            ));
        }

        // Require at least one dot (e.g., "example.com")
        if !domain.contains('.') {
            return Err(Error::InvalidInput(
                "Invalid domain format. Domain must contain at least one dot (e.g., 'example.com')".to_string(),
            ));
        }

        // Reject protocol or path characters
        if domain.contains('/') || domain.contains(':') {
            return Err(Error::InvalidInput(
                "Invalid domain format. Use 'example.com' without protocol or path".to_string(),
            ));
        }

        // Only allow alphanumeric, hyphens, and dots
        if !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
            return Err(Error::InvalidInput(
                "Domain contains invalid characters. Only letters, numbers, hyphens, and dots are allowed".to_string(),
            ));
        }

        // Reject domains starting or ending with hyphen or dot
        if domain.starts_with('.') || domain.ends_with('.') || domain.starts_with('-') || domain.ends_with('-') {
            return Err(Error::InvalidInput(
                "Domain cannot start or end with a dot or hyphen".to_string(),
            ));
        }

        // Reject consecutive dots (invalid DNS)
        if domain.contains("..") {
            return Err(Error::InvalidInput(
                "Domain cannot contain consecutive dots".to_string(),
            ));
        }
    }

    Ok(())
}
