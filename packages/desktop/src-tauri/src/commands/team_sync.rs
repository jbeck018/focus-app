// commands/team_sync.rs - Team collaboration commands with TrailBase integration
//
// Replaces the mock team commands with real backend integration.

#![allow(clippy::type_complexity)]

use crate::trailbase::{
    client::{AuthResponse, Credentials, TeamConnection, TrailBaseClient},
    models::{Member, SharedSession, SyncStatus},
    sync::{SyncOperation, SyncOperationType, SyncQueue, SyncResult},
};
use crate::{AppState, Error, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Team member info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberInfo {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

/// Team activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamActivitySummary {
    pub total_sessions: i32,
    pub total_focus_hours: f32,
    pub active_members: i32,
    pub recent_sessions: Vec<RecentSessionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSessionInfo {
    pub user_id: String,
    pub display_name: Option<String>,
    pub start_time: String,
    pub duration_minutes: i32,
    pub completed: bool,
}

/// Connect to a team (authenticate with TrailBase)
#[tauri::command]
pub async fn connect_team(
    state: State<'_, AppState>,
    server_url: String,
    credentials: Credentials,
) -> Result<AuthResponse> {
    tracing::info!("Connecting to team at: {}", server_url);

    // Create TrailBase client
    let mut client = TrailBaseClient::new(server_url.clone())?;

    // Authenticate
    let auth_response = client.authenticate(credentials).await?;

    // Store connection info
    let connection = TeamConnection {
        id: uuid::Uuid::new_v4().to_string(),
        server_url: server_url.clone(),
        team_id: None, // Will be set when joining/creating team
        user_id: Some(auth_response.user_id.clone()),
        api_key: Some(auth_response.access_token.clone()),
        connected_at: Utc::now().timestamp(),
    };

    sqlx::query(
        r#"
        INSERT INTO team_connection (id, server_url, team_id, user_id, api_key, connected_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&connection.id)
    .bind(&connection.server_url)
    .bind(&connection.team_id)
    .bind(&connection.user_id)
    .bind(&connection.api_key)
    .bind(connection.connected_at)
    .execute(state.pool())
    .await?;

    // Store client in app state
    state.set_trailbase_client(Some(client)).await;

    tracing::info!("Successfully connected to team: {}", auth_response.email);

    Ok(auth_response)
}

/// Disconnect from team
#[tauri::command]
pub async fn disconnect_team(state: State<'_, AppState>) -> Result<()> {
    tracing::info!("Disconnecting from team");

    // Clear connection from database
    sqlx::query("DELETE FROM team_connection")
        .execute(state.pool())
        .await?;

    // Clear client from app state
    state.set_trailbase_client(None).await;

    tracing::info!("Successfully disconnected from team");

    Ok(())
}

/// Get team members from TrailBase
#[tauri::command]
pub async fn get_team_members_sync(state: State<'_, AppState>) -> Result<Vec<TeamMemberInfo>> {
    let client = state
        .get_trailbase_client()
        .await
        .ok_or_else(|| Error::NotFound("Not connected to team".to_string()))?;

    // Fetch members from TrailBase
    let members: Vec<Member> = client.get("/api/team/members").await?;

    let member_infos = members
        .into_iter()
        .map(|m| TeamMemberInfo {
            id: m.user_id,
            email: m.email,
            display_name: m.display_name,
            role: format!("{:?}", m.role).to_lowercase(),
            joined_at: m.joined_at.to_rfc3339(),
        })
        .collect();

    Ok(member_infos)
}

/// Share a focus session with the team
#[tauri::command]
pub async fn share_session(
    state: State<'_, AppState>,
    session_id: String,
    team_id: String,
) -> Result<String> {
    tracing::info!("Sharing session {} with team {}", session_id, team_id);

    // Get session details
    let session: Option<(String, String, Option<String>, i32, Option<i32>, bool)> = sqlx::query_as(
        r#"
        SELECT id, start_time, end_time, planned_duration_minutes, actual_duration_seconds, completed
        FROM sessions
        WHERE id = ?
        "#,
    )
    .bind(&session_id)
    .fetch_optional(state.pool())
    .await?;

    let (id, start_time, end_time, planned_duration, actual_duration, completed) =
        session.ok_or_else(|| Error::NotFound(format!("Session not found: {}", session_id)))?;

    // Get connection info
    let connection: Option<(String, String)> = sqlx::query_as(
        "SELECT team_id, user_id FROM team_connection LIMIT 1"
    )
    .fetch_optional(state.pool())
    .await?;

    let (_, user_id) = connection.ok_or_else(|| Error::NotFound("Not connected to team".to_string()))?;

    // Create shared session
    let shared_session = SharedSession {
        local_id: uuid::Uuid::new_v4().to_string(),
        remote_id: None,
        team_id: team_id.clone(),
        user_id: user_id.clone(),
        session_id: id,
        start_time: chrono::DateTime::parse_from_rfc3339(&start_time)
            .map_err(|e| Error::Serialization(e.to_string()))?
            .with_timezone(&Utc),
        end_time: end_time
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
        planned_duration_minutes: planned_duration,
        actual_duration_seconds: actual_duration,
        completed,
        shared_at: Utc::now(),
        last_modified: Utc::now(),
        sync_status: SyncStatus::Pending,
    };

    // Store locally
    sqlx::query(
        r#"
        INSERT INTO shared_sessions
        (id, local_session_id, remote_session_id, team_id, user_id, start_time, end_time,
         planned_duration_minutes, actual_duration_seconds, completed, shared_at, last_modified, sync_status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&shared_session.local_id)
    .bind(&shared_session.session_id)
    .bind(&shared_session.remote_id)
    .bind(&shared_session.team_id)
    .bind(&shared_session.user_id)
    .bind(shared_session.start_time.to_rfc3339())
    .bind(shared_session.end_time.map(|t| t.to_rfc3339()))
    .bind(shared_session.planned_duration_minutes)
    .bind(shared_session.actual_duration_seconds)
    .bind(shared_session.completed)
    .bind(shared_session.shared_at.timestamp())
    .bind(shared_session.last_modified.to_rfc3339())
    .bind(serde_json::to_string(&shared_session.sync_status)?)
    .execute(state.pool())
    .await?;

    // Queue for sync
    let sync_queue = SyncQueue::new(state.pool().clone());
    sync_queue
        .enqueue(&shared_session, SyncOperationType::Create)
        .await?;

    tracing::info!("Queued session {} for sharing", session_id);

    Ok(shared_session.local_id)
}

/// Get team activity (recent shared sessions)
#[tauri::command]
pub async fn get_team_activity(state: State<'_, AppState>) -> Result<TeamActivitySummary> {
    let client = state
        .get_trailbase_client()
        .await
        .ok_or_else(|| Error::NotFound("Not connected to team".to_string()))?;

    // Fetch recent shared sessions from TrailBase
    let shared_sessions: Vec<SharedSession> = client
        .get("/api/team/activity?limit=20")
        .await
        .unwrap_or_default();

    let total_sessions = shared_sessions.len() as i32;

    let total_focus_hours: f32 = shared_sessions
        .iter()
        .filter_map(|s| s.actual_duration_seconds)
        .sum::<i32>() as f32
        / 3600.0;

    let active_members = shared_sessions
        .iter()
        .map(|s| &s.user_id)
        .collect::<std::collections::HashSet<_>>()
        .len() as i32;

    // Build a map of user_id -> display_name from team_members table
    let member_names: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT user_id, display_name, email FROM team_members"
    )
    .fetch_all(state.pool())
    .await
    .unwrap_or_default();

    let name_lookup: std::collections::HashMap<String, Option<String>> = member_names
        .into_iter()
        .map(|(user_id, display_name, email)| {
            // Use display_name if available, otherwise fall back to email prefix
            let name = display_name.or_else(|| {
                email.split('@').next().map(|s| s.to_string())
            });
            (user_id, name)
        })
        .collect();

    let recent_sessions = shared_sessions
        .into_iter()
        .take(10)
        .map(|s| {
            let display_name = name_lookup.get(&s.user_id).cloned().flatten();
            RecentSessionInfo {
                user_id: s.user_id.clone(),
                display_name,
                start_time: s.start_time.to_rfc3339(),
                duration_minutes: s.planned_duration_minutes,
                completed: s.completed,
            }
        })
        .collect();

    Ok(TeamActivitySummary {
        total_sessions,
        total_focus_hours,
        active_members,
        recent_sessions,
    })
}

/// Sync all pending changes with team
#[tauri::command]
pub async fn sync_with_team(state: State<'_, AppState>) -> Result<SyncStats> {
    tracing::info!("Starting team sync");

    let client = state
        .get_trailbase_client()
        .await
        .ok_or_else(|| Error::NotFound("Not connected to team".to_string()))?;

    let sync_queue = SyncQueue::new(state.pool().clone());

    // Get pending operations (increased from 50 to 500 to handle larger sync backlogs)
    let pending_ops = sync_queue.get_pending(500).await?;

    let mut success_count = 0;
    let mut failed_count = 0;

    for op in pending_ops {
        match sync_operation(&client, &sync_queue, op, state.pool()).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                tracing::warn!("Sync operation failed: {}", e);
                failed_count += 1;
            }
        }
    }

    tracing::info!(
        "Team sync completed: {} succeeded, {} failed",
        success_count,
        failed_count
    );

    Ok(SyncStats {
        synced: success_count,
        failed: failed_count,
        pending: sync_queue.get_stats().await?.pending_count,
    })
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub synced: i32,
    pub failed: i32,
    pub pending: i32,
}

/// Process a single sync operation
async fn sync_operation(
    client: &TrailBaseClient,
    sync_queue: &SyncQueue,
    op: SyncOperation,
    pool: &sqlx::SqlitePool,
) -> Result<()> {
    let result = match op.entity_type.as_str() {
        "shared_sessions" => {
            let session: SharedSession = serde_json::from_str(&op.payload)?;
            let sync_result = client.sync_entity(&session).await?;

            match sync_result {
                SyncResult::Success { remote_id, .. } => {
                    // Update local record with remote ID
                    sqlx::query(
                        r#"
                        UPDATE shared_sessions
                        SET remote_session_id = ?,
                            sync_status = 'synced',
                            last_modified = ?
                        WHERE id = ?
                        "#,
                    )
                    .bind(&remote_id)
                    .bind(Utc::now().to_rfc3339())
                    .bind(&session.local_id)
                    .execute(pool)
                    .await?;

                    sync_queue.mark_completed(&op.id).await?;
                    Ok(())
                }
                SyncResult::Failed { error, .. } => {
                    sync_queue.mark_failed(&op.id, &error).await?;
                    Err(Error::Sync(error))
                }
                SyncResult::Conflict { .. } => {
                    // Handle conflict (for now, mark as failed)
                    let error = "Conflict detected".to_string();
                    sync_queue.mark_failed(&op.id, &error).await?;
                    Err(Error::Sync(error))
                }
            }
        }
        _ => {
            tracing::warn!("Unknown entity type: {}", op.entity_type);
            sync_queue.mark_failed(&op.id, "Unknown entity type").await?;
            Err(Error::Sync("Unknown entity type".to_string()))
        }
    };

    result
}

/// Get sync queue status
#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStats> {
    let sync_queue = SyncQueue::new(state.pool().clone());
    let stats = sync_queue.get_stats().await?;

    Ok(SyncStats {
        synced: 0, // Would need to track this separately
        failed: stats.failed_count,
        pending: stats.pending_count,
    })
}

/// Retry failed sync operations
#[tauri::command]
pub async fn retry_failed_syncs(state: State<'_, AppState>) -> Result<i32> {
    let sync_queue = SyncQueue::new(state.pool().clone());
    let retried = sync_queue.retry_all_failed().await?;

    tracing::info!("Retrying {} failed sync operations", retried);

    Ok(retried)
}

/// Clear failed sync operations
#[tauri::command]
pub async fn clear_failed_syncs(state: State<'_, AppState>) -> Result<i32> {
    let sync_queue = SyncQueue::new(state.pool().clone());
    let cleared = sync_queue.clear_failed().await?;

    tracing::info!("Cleared {} failed sync operations", cleared);

    Ok(cleared)
}
