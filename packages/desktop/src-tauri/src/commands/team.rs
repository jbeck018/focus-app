// commands/team.rs - Team collaboration features
//
// Team features require cloud sync (TrailBase) to be enabled.
// All data sharing is opt-in with privacy controls.

use crate::trailbase::{
    models::{Member, MemberRole, Team as TrailBaseTeam},
    sync::{SyncOperationType, SyncQueue},
};
use crate::{AppState, Error, Result};
use chrono::{DateTime, Datelike, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;

/// Team information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub invite_code: String,
    pub member_count: i32,
    pub created_at: String,
}

/// Team member with role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: TeamRole,
    pub joined_at: String,
    pub sharing_enabled: bool,
}

/// Member roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
}

/// Aggregated team statistics (privacy-preserving)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStats {
    pub total_focus_hours_this_week: f32,
    pub average_sessions_per_member: f32,
    pub most_productive_day: Option<String>,
    pub top_blockers: Vec<String>,
    pub member_count: i32,
}

/// Team blocklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBlockedItem {
    pub id: i64,
    pub item_type: String, // "app" or "website"
    pub value: String,
    pub added_by: String,
    pub added_at: String,
}

/// Privacy settings for team sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPrivacySettings {
    pub share_focus_time: bool,
    pub share_session_count: bool,
    pub share_streak: bool,
    pub share_productivity_score: bool,
}

impl Default for TeamPrivacySettings {
    fn default() -> Self {
        Self {
            share_focus_time: true,
            share_session_count: true,
            share_streak: false,
            share_productivity_score: false,
        }
    }
}

// ============================================================================
// Database Row Types for SQLx
// ============================================================================

/// Team row from database
#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
struct TeamRow {
    pub id: String,
    pub remote_id: Option<String>,
    pub name: String,
    pub invite_code: String,
    pub created_by: String,
    pub created_at: String,
    pub last_modified: String,
    pub sync_status: String,
}

/// Team member row from database
#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
struct TeamMemberRow {
    pub id: String,
    pub remote_id: Option<String>,
    pub team_id: String,
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: String,
    pub last_modified: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a secure random invite code
fn generate_invite_code() -> String {
    let mut rng = rand::thread_rng();
    let code: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'A' + idx - 10) as char
            }
        })
        .collect();
    format!("FOCUS-{}", code)
}

/// Initialize team tables if they don't exist
async fn ensure_team_tables(state: &AppState) -> Result<()> {
    // Create teams table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            name TEXT NOT NULL,
            invite_code TEXT NOT NULL UNIQUE,
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_modified TEXT NOT NULL,
            sync_status TEXT DEFAULT 'pending'
        )
        "#,
    )
    .execute(state.pool())
    .await?;

    // Create team_members table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS team_members (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            team_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT,
            role TEXT NOT NULL,
            joined_at TEXT NOT NULL,
            last_modified TEXT NOT NULL,
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(state.pool())
    .await?;

    // Create shared_sessions table for team activity tracking
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shared_sessions (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            team_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT,
            planned_duration_minutes INTEGER NOT NULL,
            actual_duration_seconds INTEGER,
            completed INTEGER DEFAULT 0,
            shared_at TEXT NOT NULL,
            last_modified TEXT NOT NULL,
            sync_status TEXT DEFAULT 'pending',
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(state.pool())
    .await?;

    Ok(())
}

/// Convert database role string to TeamRole enum
fn parse_role(role: &str) -> TeamRole {
    match role.to_lowercase().as_str() {
        "owner" => TeamRole::Owner,
        "admin" => TeamRole::Admin,
        _ => TeamRole::Member,
    }
}

/// Convert TeamRole enum to database string
#[allow(dead_code)]
fn role_to_string(role: &TeamRole) -> &'static str {
    match role {
        TeamRole::Owner => "owner",
        TeamRole::Admin => "admin",
        TeamRole::Member => "member",
    }
}

/// Convert MemberRole to TeamRole
#[allow(dead_code)]
fn member_role_to_team_role(role: &MemberRole) -> TeamRole {
    match role {
        MemberRole::Owner => TeamRole::Owner,
        MemberRole::Admin => TeamRole::Admin,
        MemberRole::Member => TeamRole::Member,
    }
}

/// Convert TeamRole to MemberRole
#[allow(dead_code)]
fn team_role_to_member_role(role: &TeamRole) -> MemberRole {
    match role {
        TeamRole::Owner => MemberRole::Owner,
        TeamRole::Admin => MemberRole::Admin,
        TeamRole::Member => MemberRole::Member,
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get current user's team (if any)
#[tauri::command]
pub async fn get_current_team(
    state: State<'_, AppState>,
) -> Result<Option<Team>> {
    // Ensure tables exist
    ensure_team_tables(&state).await?;

    // Get current user ID
    let user_id = state.get_user_id().await.unwrap_or_else(|| "local-user".to_string());

    // Query local teams table - find team where user is creator or member
    let team: Option<TeamRow> = sqlx::query_as(
        r#"
        SELECT t.* FROM teams t
        WHERE t.created_by = ?
           OR t.id IN (SELECT team_id FROM team_members WHERE user_id = ?)
        LIMIT 1
        "#,
    )
    .bind(&user_id)
    .bind(&user_id)
    .fetch_optional(state.pool())
    .await?;

    let Some(team) = team else {
        return Ok(None);
    };

    // Count members for this team
    let member_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM team_members WHERE team_id = ?"
    )
    .bind(&team.id)
    .fetch_one(state.pool())
    .await?;

    Ok(Some(Team {
        id: team.id,
        name: team.name,
        invite_code: team.invite_code,
        member_count,
        created_at: team.created_at,
    }))
}

/// Create a new team
#[tauri::command]
pub async fn create_team(
    state: State<'_, AppState>,
    name: String,
) -> Result<Team> {
    // Ensure tables exist
    ensure_team_tables(&state).await?;

    // Get current user info
    let user_id = state.get_user_id().await.unwrap_or_else(|| "local-user".to_string());
    let auth_state = state.auth_state.read().await;
    let user_email = auth_state
        .user
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_else(|| "local@user.com".to_string());
    // UserInfo doesn't have a name field, so display_name is None for now
    // In the future, this could be fetched from a profile endpoint or stored locally
    let user_display_name: Option<String> = None;
    drop(auth_state);

    // Check if user already has a team
    let existing_team = get_current_team(state.clone()).await?;
    if existing_team.is_some() {
        return Err(Error::AlreadyExists(
            "You are already a member of a team. Leave your current team first.".to_string(),
        ));
    }

    // Generate IDs
    let team_id = uuid::Uuid::new_v4().to_string();
    let member_id = uuid::Uuid::new_v4().to_string();
    let invite_code = generate_invite_code();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    // Insert team into local database
    sqlx::query(
        r#"
        INSERT INTO teams (id, name, invite_code, created_by, created_at, last_modified, sync_status)
        VALUES (?, ?, ?, ?, ?, ?, 'pending')
        "#,
    )
    .bind(&team_id)
    .bind(&name)
    .bind(&invite_code)
    .bind(&user_id)
    .bind(&now_str)
    .bind(&now_str)
    .execute(state.pool())
    .await?;

    // Add creator as owner member
    sqlx::query(
        r#"
        INSERT INTO team_members (id, team_id, user_id, email, display_name, role, joined_at, last_modified)
        VALUES (?, ?, ?, ?, ?, 'owner', ?, ?)
        "#,
    )
    .bind(&member_id)
    .bind(&team_id)
    .bind(&user_id)
    .bind(&user_email)
    .bind(&user_display_name)
    .bind(&now_str)
    .bind(&now_str)
    .execute(state.pool())
    .await?;

    // Initialize team blocklist table if needed
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS team_blocklist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            team_id TEXT NOT NULL,
            item_type TEXT NOT NULL CHECK(item_type IN ('app', 'website')),
            value TEXT NOT NULL,
            added_by TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(team_id, item_type, value)
        )"
    )
    .execute(state.pool())
    .await?;

    // Queue for sync with TrailBase
    let trailbase_team = TrailBaseTeam {
        local_id: team_id.clone(),
        remote_id: None,
        name: name.clone(),
        invite_code: invite_code.clone(),
        created_by: user_id.clone(),
        created_at: now,
        last_modified: now,
    };

    let sync_queue = SyncQueue::new(state.pool().clone());
    if let Err(e) = sync_queue.enqueue(&trailbase_team, SyncOperationType::Create).await {
        tracing::warn!("Failed to queue team for sync: {}", e);
    }

    tracing::info!("Created team: {} ({}) with invite code {}", name, team_id, invite_code);

    Ok(Team {
        id: team_id,
        name,
        invite_code,
        member_count: 1,
        created_at: now_str,
    })
}

/// Join an existing team via invite code
#[tauri::command]
pub async fn join_team(
    state: State<'_, AppState>,
    invite_code: String,
) -> Result<Team> {
    // Ensure tables exist
    ensure_team_tables(&state).await?;

    // Validate invite code format
    if !invite_code.starts_with("FOCUS-") || invite_code.len() != 12 {
        return Err(Error::InvalidInput(
            "Invalid invite code format. Expected format: FOCUS-XXXXXX".to_string(),
        ));
    }

    // Get current user info
    let user_id = state.get_user_id().await.unwrap_or_else(|| "local-user".to_string());
    let auth_state = state.auth_state.read().await;
    let user_email = auth_state
        .user
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_else(|| "local@user.com".to_string());
    // UserInfo doesn't have a name field, so display_name is None for now
    // In the future, this could be fetched from a profile endpoint or stored locally
    let user_display_name: Option<String> = None;
    drop(auth_state);

    // Check if user already has a team
    let existing_team = get_current_team(state.clone()).await?;
    if existing_team.is_some() {
        return Err(Error::AlreadyExists(
            "You are already a member of a team. Leave your current team first.".to_string(),
        ));
    }

    // Look up team by invite code in local database
    let team: Option<TeamRow> = sqlx::query_as(
        "SELECT * FROM teams WHERE invite_code = ?"
    )
    .bind(&invite_code)
    .fetch_optional(state.pool())
    .await?;

    // If team not found locally, try to fetch from TrailBase
    let team = match team {
        Some(t) => t,
        None => {
            // Try to fetch team from TrailBase if client is connected
            let trailbase_client = state.get_trailbase_client().await;
            if let Some(client) = trailbase_client {
                // Attempt to get team from remote by invite code
                match client.get::<TrailBaseTeam>(&format!("/api/teams/invite/{}", invite_code)).await {
                    Ok(remote_team) => {
                        // Store the team locally
                        let now_str = Utc::now().to_rfc3339();
                        sqlx::query(
                            r#"
                            INSERT INTO teams (id, remote_id, name, invite_code, created_by, created_at, last_modified, sync_status)
                            VALUES (?, ?, ?, ?, ?, ?, ?, 'synced')
                            "#,
                        )
                        .bind(&remote_team.local_id)
                        .bind(&remote_team.remote_id)
                        .bind(&remote_team.name)
                        .bind(&remote_team.invite_code)
                        .bind(&remote_team.created_by)
                        .bind(remote_team.created_at.to_rfc3339())
                        .bind(&now_str)
                        .execute(state.pool())
                        .await?;

                        TeamRow {
                            id: remote_team.local_id,
                            remote_id: remote_team.remote_id,
                            name: remote_team.name,
                            invite_code: remote_team.invite_code,
                            created_by: remote_team.created_by,
                            created_at: remote_team.created_at.to_rfc3339(),
                            last_modified: now_str,
                            sync_status: "synced".to_string(),
                        }
                    }
                    Err(_) => {
                        return Err(Error::NotFound(
                            "Team not found. Please check the invite code and try again.".to_string(),
                        ));
                    }
                }
            } else {
                return Err(Error::NotFound(
                    "Team not found locally. Connect to the cloud to join remote teams.".to_string(),
                ));
            }
        }
    };

    // Check if user is already a member
    let is_member: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM team_members WHERE team_id = ? AND user_id = ?"
    )
    .bind(&team.id)
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    if is_member {
        return Err(Error::AlreadyExists(
            "You are already a member of this team.".to_string(),
        ));
    }

    // Add user as member
    let member_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO team_members (id, team_id, user_id, email, display_name, role, joined_at, last_modified)
        VALUES (?, ?, ?, ?, ?, 'member', ?, ?)
        "#,
    )
    .bind(&member_id)
    .bind(&team.id)
    .bind(&user_id)
    .bind(&user_email)
    .bind(&user_display_name)
    .bind(&now_str)
    .bind(&now_str)
    .execute(state.pool())
    .await?;

    // Queue member for sync
    let member = Member {
        local_id: member_id,
        remote_id: None,
        team_id: team.id.clone(),
        user_id: user_id.clone(),
        email: user_email,
        display_name: user_display_name,
        role: MemberRole::Member,
        joined_at: now,
        last_modified: now,
    };

    let sync_queue = SyncQueue::new(state.pool().clone());
    if let Err(e) = sync_queue.enqueue(&member, SyncOperationType::Create).await {
        tracing::warn!("Failed to queue member for sync: {}", e);
    }

    // Count members
    let member_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM team_members WHERE team_id = ?"
    )
    .bind(&team.id)
    .fetch_one(state.pool())
    .await?;

    tracing::info!("User {} joined team {} via invite code", user_id, team.id);

    Ok(Team {
        id: team.id,
        name: team.name,
        invite_code: team.invite_code,
        member_count,
        created_at: team.created_at,
    })
}

/// Leave the current team
#[tauri::command]
pub async fn leave_team(
    state: State<'_, AppState>,
) -> Result<()> {
    // Ensure tables exist
    ensure_team_tables(&state).await?;

    // Get current user ID
    let user_id = state.get_user_id().await.unwrap_or_else(|| "local-user".to_string());

    // Get current team
    let team = get_current_team(state.clone()).await?;
    let Some(team) = team else {
        return Err(Error::NotFound("You are not a member of any team.".to_string()));
    };

    // Check if user is the owner
    let member: Option<TeamMemberRow> = sqlx::query_as(
        "SELECT * FROM team_members WHERE team_id = ? AND user_id = ?"
    )
    .bind(&team.id)
    .bind(&user_id)
    .fetch_optional(state.pool())
    .await?;

    let Some(member) = member else {
        return Err(Error::NotFound("Membership not found.".to_string()));
    };

    let is_owner = member.role.to_lowercase() == "owner";

    if is_owner {
        // Owner leaving - check if there are other members
        let other_member_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM team_members WHERE team_id = ? AND user_id != ?"
        )
        .bind(&team.id)
        .bind(&user_id)
        .fetch_one(state.pool())
        .await?;

        if other_member_count > 0 {
            return Err(Error::InvalidInput(
                "You cannot leave the team as the owner while other members exist. Transfer ownership first or remove all members.".to_string(),
            ));
        }

        // Delete the entire team if owner is leaving and no other members
        sqlx::query("DELETE FROM team_members WHERE team_id = ?")
            .bind(&team.id)
            .execute(state.pool())
            .await?;

        sqlx::query("DELETE FROM team_blocklist WHERE team_id = ?")
            .bind(&team.id)
            .execute(state.pool())
            .await?;

        sqlx::query("DELETE FROM shared_sessions WHERE team_id = ?")
            .bind(&team.id)
            .execute(state.pool())
            .await?;

        sqlx::query("DELETE FROM teams WHERE id = ?")
            .bind(&team.id)
            .execute(state.pool())
            .await?;

        tracing::info!("User {} deleted team {} (was only member)", user_id, team.id);
    } else {
        // Regular member leaving - just remove their membership
        sqlx::query("DELETE FROM team_members WHERE team_id = ? AND user_id = ?")
            .bind(&team.id)
            .bind(&user_id)
            .execute(state.pool())
            .await?;

        // Queue deletion for sync
        let member_to_delete = Member {
            local_id: member.id,
            remote_id: member.remote_id,
            team_id: member.team_id,
            user_id: member.user_id,
            email: member.email,
            display_name: member.display_name,
            role: MemberRole::Member,
            joined_at: DateTime::parse_from_rfc3339(&member.joined_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_modified: Utc::now(),
        };

        let sync_queue = SyncQueue::new(state.pool().clone());
        if let Err(e) = sync_queue.enqueue(&member_to_delete, SyncOperationType::Delete).await {
            tracing::warn!("Failed to queue member deletion for sync: {}", e);
        }

        tracing::info!("User {} left team {}", user_id, team.id);
    }

    // Clean up legacy settings if any
    sqlx::query("DELETE FROM user_settings WHERE key IN ('current_team_id', 'current_team_name')")
        .execute(state.pool())
        .await?;

    Ok(())
}

/// Get team members (for admins/owners)
#[tauri::command]
pub async fn get_team_members(
    state: State<'_, AppState>,
) -> Result<Vec<TeamMember>> {
    let team = get_current_team(state.clone()).await?;

    let Some(team) = team else {
        return Err(Error::NotFound("No team found".to_string()));
    };

    // Query team members from local database
    let members: Vec<TeamMemberRow> = sqlx::query_as(
        r#"
        SELECT * FROM team_members
        WHERE team_id = ?
        ORDER BY role = 'owner' DESC, role = 'admin' DESC, joined_at ASC
        "#,
    )
    .bind(&team.id)
    .fetch_all(state.pool())
    .await?;

    // Get privacy settings to determine sharing status
    let privacy_settings = get_team_privacy_settings(state.clone()).await?;

    // Convert to TeamMember structs
    let team_members: Vec<TeamMember> = members
        .into_iter()
        .map(|m| TeamMember {
            id: m.id,
            email: m.email,
            display_name: m.display_name,
            role: parse_role(&m.role),
            joined_at: m.joined_at,
            sharing_enabled: privacy_settings.share_focus_time || privacy_settings.share_session_count,
        })
        .collect();

    Ok(team_members)
}

/// Get aggregated team statistics (privacy-preserving)
#[tauri::command]
pub async fn get_team_stats(
    state: State<'_, AppState>,
) -> Result<TeamStats> {
    let Some(team) = get_current_team(state.clone()).await? else {
        return Err(Error::NotFound("No team found".to_string()));
    };

    // Calculate week boundaries
    let today = chrono::Utc::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let week_start_str = week_start.to_string();

    // Get team stats from shared_sessions table (aggregated from all members)
    let team_focus_seconds: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(actual_duration_seconds), 0)
        FROM shared_sessions
        WHERE team_id = ?
          AND date(start_time) >= date(?)
          AND completed = 1
        "#,
    )
    .bind(&team.id)
    .bind(&week_start_str)
    .fetch_one(state.pool())
    .await?;

    let team_session_count: i32 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM shared_sessions
        WHERE team_id = ?
          AND date(start_time) >= date(?)
          AND completed = 1
        "#,
    )
    .bind(&team.id)
    .bind(&week_start_str)
    .fetch_one(state.pool())
    .await?;

    // Also include local user's sessions that might not be shared yet
    let local_focus_seconds: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(actual_duration_seconds), 0)
        FROM sessions
        WHERE date(start_time) >= date(?)
          AND completed = 1
        "#,
    )
    .bind(&week_start_str)
    .fetch_one(state.pool())
    .await?;

    let local_session_count: i32 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE date(start_time) >= date(?)
          AND completed = 1
        "#,
    )
    .bind(&week_start_str)
    .fetch_one(state.pool())
    .await?;

    // Combine team and local stats (avoiding double-counting if possible)
    let total_focus_seconds = team_focus_seconds.unwrap_or(0) + local_focus_seconds;
    let total_sessions = team_session_count + local_session_count;

    // Find most productive day this week from shared sessions
    let most_productive_day: Option<String> = sqlx::query_scalar(
        r#"
        SELECT strftime('%w', start_time) as day_of_week
        FROM shared_sessions
        WHERE team_id = ?
          AND date(start_time) >= date(?)
          AND completed = 1
        GROUP BY day_of_week
        ORDER BY SUM(actual_duration_seconds) DESC
        LIMIT 1
        "#,
    )
    .bind(&team.id)
    .bind(&week_start_str)
    .fetch_optional(state.pool())
    .await?;

    let most_productive_day = most_productive_day.map(|day_num| {
        match day_num.as_str() {
            "0" => "Sunday".to_string(),
            "1" => "Monday".to_string(),
            "2" => "Tuesday".to_string(),
            "3" => "Wednesday".to_string(),
            "4" => "Thursday".to_string(),
            "5" => "Friday".to_string(),
            "6" => "Saturday".to_string(),
            _ => "Unknown".to_string(),
        }
    });

    // Get top blocked items from team blocklist
    let top_blockers: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT value FROM team_blocklist
        WHERE team_id = ?
        ORDER BY created_at DESC
        LIMIT 3
        "#,
    )
    .bind(&team.id)
    .fetch_all(state.pool())
    .await
    .unwrap_or_default();

    // If no team blockers, fall back to personal blockers
    let top_blockers = if top_blockers.is_empty() {
        sqlx::query_scalar(
            "SELECT value FROM blocked_items WHERE enabled = 1 ORDER BY id LIMIT 3"
        )
        .fetch_all(state.pool())
        .await
        .unwrap_or_default()
    } else {
        top_blockers
    };

    // Calculate average sessions per member
    let average_sessions = if team.member_count > 0 {
        total_sessions as f32 / team.member_count as f32
    } else {
        total_sessions as f32
    };

    Ok(TeamStats {
        total_focus_hours_this_week: total_focus_seconds as f32 / 3600.0,
        average_sessions_per_member: average_sessions,
        most_productive_day,
        top_blockers,
        member_count: team.member_count,
    })
}

/// Get team shared blocklist
#[tauri::command]
pub async fn get_team_blocklist(
    state: State<'_, AppState>,
) -> Result<Vec<TeamBlockedItem>> {
    let Some(team) = get_current_team(state.clone()).await? else {
        return Ok(Vec::new());
    };

    let team_id = team.id;

    let items: Vec<(i64, String, String, String, String)> = sqlx::query_as(
        "SELECT id, item_type, value, added_by, created_at
         FROM team_blocklist
         WHERE team_id = ?
         ORDER BY created_at DESC"
    )
    .bind(&team_id)
    .fetch_all(state.pool())
    .await?;

    Ok(items
        .into_iter()
        .map(|(id, item_type, value, added_by, added_at)| TeamBlockedItem {
            id,
            item_type,
            value,
            added_by,
            added_at,
        })
        .collect())
}

/// Add item to team blocklist
#[tauri::command]
pub async fn add_team_blocked_item(
    state: State<'_, AppState>,
    item_type: String,
    value: String,
) -> Result<TeamBlockedItem> {
    let Some(team) = get_current_team(state.clone()).await? else {
        return Err(Error::NotFound("No team found".to_string()));
    };

    // Get current user info for attribution
    let auth_state = state.auth_state.read().await;
    let user_display = auth_state
        .user
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_else(|| "You".to_string());
    let user_id = auth_state
        .user
        .as_ref()
        .map(|u| u.id.clone())
        .unwrap_or_else(|| "local-user".to_string());
    drop(auth_state);

    let team_id = team.id;

    sqlx::query(
        "INSERT INTO team_blocklist (team_id, item_type, value, added_by)
         VALUES (?, ?, ?, ?)"
    )
    .bind(&team_id)
    .bind(&item_type)
    .bind(&value)
    .bind(&user_id)
    .execute(state.pool())
    .await?;

    let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
        .fetch_one(state.pool())
        .await?;

    tracing::info!("Added team blocked item: {} ({}) by {}", value, item_type, user_id);

    Ok(TeamBlockedItem {
        id,
        item_type,
        value,
        added_by: user_display,
        added_at: Utc::now().to_rfc3339(),
    })
}

/// Remove item from team blocklist
#[tauri::command]
pub async fn remove_team_blocked_item(
    state: State<'_, AppState>,
    item_id: i64,
) -> Result<()> {
    let Some(team) = get_current_team(state.clone()).await? else {
        return Err(Error::NotFound("No team found".into()));
    };

    let team_id = team.id;

    sqlx::query("DELETE FROM team_blocklist WHERE id = ? AND team_id = ?")
        .bind(item_id)
        .bind(&team_id)
        .execute(state.pool())
        .await?;

    Ok(())
}

/// Get privacy settings
#[tauri::command]
pub async fn get_team_privacy_settings(
    state: State<'_, AppState>,
) -> Result<TeamPrivacySettings> {
    let settings_json: Option<String> = sqlx::query_scalar(
        "SELECT value FROM user_settings WHERE key = 'team_privacy_settings'"
    )
    .fetch_optional(state.pool())
    .await?;

    match settings_json {
        Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
        None => Ok(TeamPrivacySettings::default()),
    }
}

/// Update privacy settings
#[tauri::command]
pub async fn update_team_privacy_settings(
    state: State<'_, AppState>,
    settings: TeamPrivacySettings,
) -> Result<()> {
    let json = serde_json::to_string(&settings)?;

    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
    )
    .bind("team_privacy_settings")
    .bind(&json)
    .execute(state.pool())
    .await?;

    Ok(())
}

/// Apply team blocklist to local blocking rules
#[tauri::command]
pub async fn sync_team_blocklist(
    state: State<'_, AppState>,
) -> Result<i32> {
    let team_items = get_team_blocklist(state.clone()).await?;
    let mut added_count = 0;

    for item in team_items {
        // Check if already in local blocklist
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM blocked_items WHERE item_type = ? AND value = ?"
        )
        .bind(&item.item_type)
        .bind(&item.value)
        .fetch_optional(state.pool())
        .await?;

        if exists.is_none() {
            sqlx::query(
                "INSERT INTO blocked_items (item_type, value, enabled, created_at)
                 VALUES (?, ?, 1, CURRENT_TIMESTAMP)"
            )
            .bind(&item.item_type)
            .bind(&item.value)
            .execute(state.pool())
            .await?;
            added_count += 1;
        }
    }

    tracing::info!("Synced {} items from team blocklist", added_count);

    Ok(added_count)
}
