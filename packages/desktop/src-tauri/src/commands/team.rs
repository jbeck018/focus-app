// commands/team.rs - Team collaboration features
//
// Team features require cloud sync (TrailBase) to be enabled.
// All data sharing is opt-in with privacy controls.

use crate::{AppState, Error, Result};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
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

/// Get current user's team (if any)
#[tauri::command]
pub async fn get_current_team(
    state: State<'_, AppState>,
) -> Result<Option<Team>> {
    // Check if user has a team stored locally
    let team_id: Option<String> = sqlx::query_scalar(
        "SELECT value FROM user_settings WHERE key = 'current_team_id'"
    )
    .fetch_optional(state.pool())
    .await?;

    if team_id.is_none() {
        return Ok(None);
    }

    // In production, this would fetch from TrailBase
    // For now, return mock data
    let team_name: Option<String> = sqlx::query_scalar(
        "SELECT value FROM user_settings WHERE key = 'current_team_name'"
    )
    .fetch_optional(state.pool())
    .await?;

    Ok(Some(Team {
        id: team_id.unwrap(),
        name: team_name.unwrap_or_else(|| "My Team".into()),
        invite_code: "FOCUS-XXXX".into(),
        member_count: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Create a new team
#[tauri::command]
pub async fn create_team(
    state: State<'_, AppState>,
    name: String,
) -> Result<Team> {
    // Generate team ID and invite code
    let team_id = uuid::Uuid::new_v4().to_string();
    let invite_code = format!("FOCUS-{}", &team_id[..4].to_uppercase());

    // Store team locally
    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
    )
    .bind("current_team_id")
    .bind(&team_id)
    .execute(state.pool())
    .await?;

    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
    )
    .bind("current_team_name")
    .bind(&name)
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

    tracing::info!("Created team: {} ({})", name, team_id);

    Ok(Team {
        id: team_id,
        name,
        invite_code,
        member_count: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Join an existing team via invite code
#[tauri::command]
pub async fn join_team(
    state: State<'_, AppState>,
    invite_code: String,
) -> Result<Team> {
    // In production, this would validate the invite code with TrailBase
    // For now, simulate joining

    if !invite_code.starts_with("FOCUS-") {
        return Err(Error::InvalidInput("Invalid invite code format".into()));
    }

    let team_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
    )
    .bind("current_team_id")
    .bind(&team_id)
    .execute(state.pool())
    .await?;

    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
    )
    .bind("current_team_name")
    .bind("Joined Team")
    .execute(state.pool())
    .await?;

    Ok(Team {
        id: team_id,
        name: "Joined Team".into(),
        invite_code,
        member_count: 2, // Mock: at least one other member
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Leave the current team
#[tauri::command]
pub async fn leave_team(
    state: State<'_, AppState>,
) -> Result<()> {
    sqlx::query("DELETE FROM user_settings WHERE key IN ('current_team_id', 'current_team_name')")
        .execute(state.pool())
        .await?;

    tracing::info!("Left team");

    Ok(())
}

/// Get team members (for admins/owners)
#[tauri::command]
pub async fn get_team_members(
    state: State<'_, AppState>,
) -> Result<Vec<TeamMember>> {
    let team = get_current_team(state.clone()).await?;

    if team.is_none() {
        return Err(Error::NotFound("No team found".into()));
    }

    // In production, fetch from TrailBase
    // For now, return mock data with current user
    Ok(vec![TeamMember {
        id: "current-user".into(),
        email: "you@example.com".into(),
        display_name: Some("You".into()),
        role: TeamRole::Owner,
        joined_at: chrono::Utc::now().to_rfc3339(),
        sharing_enabled: true,
    }])
}

/// Get aggregated team statistics (privacy-preserving)
#[tauri::command]
pub async fn get_team_stats(
    state: State<'_, AppState>,
) -> Result<TeamStats> {
    let team = get_current_team(state.clone()).await?;

    if team.is_none() {
        return Err(Error::NotFound("No team found".into()));
    }

    // Get local user's stats to contribute to team totals
    let today = chrono::Utc::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

    let focus_seconds: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(actual_duration_seconds), 0)
         FROM sessions
         WHERE date(start_time) >= date(?) AND completed = 1"
    )
    .bind(week_start.to_string())
    .fetch_one(state.pool())
    .await?;

    let session_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions
         WHERE date(start_time) >= date(?) AND completed = 1"
    )
    .bind(week_start.to_string())
    .fetch_one(state.pool())
    .await?;

    // Get top blocked items
    let top_blockers: Vec<String> = sqlx::query_scalar(
        "SELECT value FROM blocked_items WHERE enabled = 1 ORDER BY id LIMIT 3"
    )
    .fetch_all(state.pool())
    .await?;

    // In production, this would aggregate data from all team members via TrailBase
    // Privacy note: Only shares aggregated data, not individual stats

    Ok(TeamStats {
        total_focus_hours_this_week: focus_seconds as f32 / 3600.0,
        average_sessions_per_member: session_count as f32,
        most_productive_day: Some("Tuesday".into()), // Mock
        top_blockers,
        member_count: team.unwrap().member_count,
    })
}

/// Get team shared blocklist
#[tauri::command]
pub async fn get_team_blocklist(
    state: State<'_, AppState>,
) -> Result<Vec<TeamBlockedItem>> {
    let team = get_current_team(state.clone()).await?;

    if team.is_none() {
        return Ok(Vec::new());
    }

    let team_id = team.unwrap().id;

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
    let team = get_current_team(state.clone()).await?;

    if team.is_none() {
        return Err(Error::NotFound("No team found".into()));
    }

    let team_id = team.unwrap().id;

    sqlx::query(
        "INSERT INTO team_blocklist (team_id, item_type, value, added_by)
         VALUES (?, ?, ?, 'current-user')"
    )
    .bind(&team_id)
    .bind(&item_type)
    .bind(&value)
    .execute(state.pool())
    .await?;

    let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
        .fetch_one(state.pool())
        .await?;

    tracing::info!("Added team blocked item: {} ({})", value, item_type);

    Ok(TeamBlockedItem {
        id,
        item_type,
        value,
        added_by: "You".into(),
        added_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Remove item from team blocklist
#[tauri::command]
pub async fn remove_team_blocked_item(
    state: State<'_, AppState>,
    item_id: i64,
) -> Result<()> {
    let team = get_current_team(state.clone()).await?;

    if team.is_none() {
        return Err(Error::NotFound("No team found".into()));
    }

    let team_id = team.unwrap().id;

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
