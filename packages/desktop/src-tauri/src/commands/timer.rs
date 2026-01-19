// commands/timer.rs - Backend timer management with broadcast to all windows
//
// This module implements a backend-owned timer that broadcasts state to all windows,
// ensuring perfect synchronization between main timer and mini-timer windows.

use crate::{
    state::AppState,
    Error, Result,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, trace};

/// Static flag to ensure only one timer loop runs at a time
static TIMER_LOOP_RUNNING: AtomicBool = AtomicBool::new(false);

/// Payload broadcast to all windows on every timer tick
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerTickPayload {
    pub session_id: String,
    pub elapsed_seconds: i64,
    pub remaining_seconds: i64,
    pub planned_duration_minutes: i32,
    pub session_type: String,
    pub is_running: bool,
    pub is_paused: bool,
}

/// Start the backend timer broadcast loop
///
/// This spawns a background task that ticks every second and broadcasts
/// timer state to ALL windows. Only one loop runs at a time.
/// Returns a cancellation sender that can be used to stop the loop.
pub fn start_timer_loop(app_handle: AppHandle, state: AppState) -> oneshot::Sender<()> {
    // Ensure only one timer loop runs
    if TIMER_LOOP_RUNNING.swap(true, Ordering::SeqCst) {
        debug!("Timer loop already running, cancelling and starting new one");
        // Cancel any existing timer
        if let Ok(mut cancel_guard) = state.timer_cancellation.try_write() {
            if let Some(sender) = cancel_guard.take() {
                let _ = sender.send(());
            }
        }
    }

    info!("Starting backend timer broadcast loop");

    let (cancel_tx, mut cancel_rx) = oneshot::channel();

    tokio::spawn(async move {
        let mut tick_interval = interval(Duration::from_secs(1));
        let mut idle_ticks = 0u32;

        loop {
            tokio::select! {
                _ = &mut cancel_rx => {
                    info!("Timer loop cancelled by session end");
                    TIMER_LOOP_RUNNING.store(false, Ordering::SeqCst);
                    break;
                }
                _ = tick_interval.tick() => {
                    // Get current session and timer state
                    let (session, timer_state) = {
                        let session_guard = state.active_session.read().await;
                        let timer_guard = state.timer_state.read().await;
                        (session_guard.clone(), timer_guard.clone())
                    };

                    // Check if we have an active session
                    let Some(session) = session else {
                        // No active session - increment idle counter
                        idle_ticks += 1;

                        // After 60 seconds of no session, stop the loop to save resources
                        // It will restart when a new session begins
                        if idle_ticks >= 60 {
                            debug!("Timer loop idle for 60s, stopping");
                            TIMER_LOOP_RUNNING.store(false, Ordering::SeqCst);
                            break;
                        }
                        continue;
                    };

                    // Reset idle counter when we have a session
                    idle_ticks = 0;

                    // Only broadcast if timer is running
                    if !timer_state.is_running {
                        trace!("Timer state is_running=false, skipping broadcast");
                        continue;
                    }

                    // Calculate elapsed time accounting for pauses
                    let elapsed = timer_state.calculate_elapsed(session.start_time);
                    let planned_seconds = session.planned_duration_minutes as i64 * 60;
                    let remaining = (planned_seconds - elapsed).max(0);

                    let payload = TimerTickPayload {
                        session_id: session.id.clone(),
                        elapsed_seconds: elapsed,
                        remaining_seconds: remaining,
                        planned_duration_minutes: session.planned_duration_minutes,
                        session_type: format!("{:?}", session.session_type).to_lowercase(),
                        is_running: !timer_state.is_paused,
                        is_paused: timer_state.is_paused,
                    };

                    // Broadcast to ALL windows using app-level emit
                    if let Err(e) = app_handle.emit("timer-tick", &payload) {
                        error!("Failed to broadcast timer tick: {}", e);
                    } else {
                        // Log every 5 seconds at debug level, every second at trace level
                        if elapsed % 5 == 0 {
                            debug!(
                                "Timer tick broadcast: elapsed={}s remaining={}s paused={}",
                                elapsed,
                                remaining,
                                timer_state.is_paused
                            );
                        } else {
                            trace!(
                                "Timer tick: session={} elapsed={}s remaining={}s paused={}",
                                session.id,
                                elapsed,
                                remaining,
                                timer_state.is_paused
                            );
                        }
                    }

                    // Auto-complete notification when time is up
                    if remaining == 0 && elapsed >= planned_seconds && !timer_state.is_paused {
                        debug!("Session {} timer completed", session.id);
                        if let Err(e) = app_handle.emit("timer-completed", &payload) {
                            error!("Failed to emit timer-completed: {}", e);
                        }
                    }
                }
            }
        }

        info!("Timer loop ended");
    });

    cancel_tx
}

/// Ensure timer loop is running (call when session starts or mini-timer opens)
pub async fn ensure_timer_loop_running(app_handle: AppHandle, state: AppState) {
    if !TIMER_LOOP_RUNNING.load(Ordering::SeqCst) {
        debug!("Timer loop not running, starting it");
        let cancel_tx = start_timer_loop(app_handle, state.clone());
        // Store the cancellation sender in state
        let mut cancellation = state.timer_cancellation.write().await;
        *cancellation = Some(cancel_tx);
    }
}

/// Stop the timer loop (call when session ends)
pub async fn stop_timer_loop(state: &AppState) {
    let mut cancellation = state.timer_cancellation.write().await;
    if let Some(sender) = cancellation.take() {
        debug!("Sending timer loop cancellation signal");
        let _ = sender.send(());
    }
}

/// Toggle pause/resume for the current session
#[tauri::command]
pub async fn toggle_timer_pause(state: State<'_, AppState>) -> Result<bool> {
    // Verify there's an active session
    {
        let active = state.active_session.read().await;
        if active.is_none() {
            return Err(Error::InvalidSession(
                "No active session to toggle".to_string(),
            ));
        }
    }

    let is_paused = {
        let mut timer_state = state.timer_state.write().await;
        timer_state.toggle();
        timer_state.is_paused
    };

    debug!("Timer pause toggled: is_paused={}", is_paused);
    Ok(is_paused)
}

/// Get current timer state (for initial sync when window opens)
#[tauri::command]
pub async fn get_timer_state(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Option<TimerTickPayload>> {
    let session = {
        let guard = state.active_session.read().await;
        guard.clone()
    };

    let Some(session) = session else {
        return Ok(None);
    };

    let timer_state = state.timer_state.read().await;

    // If timer isn't running, return None
    if !timer_state.is_running {
        return Ok(None);
    }

    // Ensure timer loop is running when a window requests state
    ensure_timer_loop_running(app_handle, (*state).clone()).await;

    let elapsed = timer_state.calculate_elapsed(session.start_time);
    let planned_seconds = session.planned_duration_minutes as i64 * 60;
    let remaining = (planned_seconds - elapsed).max(0);

    Ok(Some(TimerTickPayload {
        session_id: session.id.clone(),
        elapsed_seconds: elapsed,
        remaining_seconds: remaining,
        planned_duration_minutes: session.planned_duration_minutes,
        session_type: format!("{:?}", session.session_type).to_lowercase(),
        is_running: !timer_state.is_paused,
        is_paused: timer_state.is_paused,
    }))
}
