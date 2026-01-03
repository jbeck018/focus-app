// commands/blocking-advanced.rs - Advanced blocking features
//
// Implements schedule-based blocking, categories, strict mode, nuclear option, and statistics

use crate::{db::queries, state::AppState, Error, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

// ============================================================================
// Blocking Schedules
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockingSchedule {
    pub id: i64,
    pub user_id: Option<String>,
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub id: i64,
    pub enabled: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// Create a new blocking schedule
#[tauri::command]
pub async fn create_blocking_schedule(
    request: CreateScheduleRequest,
    state: State<'_, AppState>,
) -> Result<BlockingSchedule> {
    // Validate day of week
    if !(0..=6).contains(&request.day_of_week) {
        return Err(Error::InvalidInput(
            "Day of week must be between 0 (Sunday) and 6 (Saturday)".to_string(),
        ));
    }

    // Validate time format (HH:MM)
    if !is_valid_time(&request.start_time) || !is_valid_time(&request.end_time) {
        return Err(Error::InvalidInput(
            "Time must be in HH:MM format (24-hour)".to_string(),
        ));
    }

    let result = sqlx::query(
        r#"
        INSERT INTO blocking_schedules (day_of_week, start_time, end_time)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(request.day_of_week)
    .bind(&request.start_time)
    .bind(&request.end_time)
    .execute(state.pool())
    .await?;

    let id = result.last_insert_rowid();

    // Fetch the created schedule
    let schedule = sqlx::query_as::<_, BlockingSchedule>(
        r#"
        SELECT * FROM blocking_schedules WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(state.pool())
    .await?;

    tracing::info!("Created blocking schedule: {:?}", schedule);

    Ok(schedule)
}

/// Get all blocking schedules
#[tauri::command]
pub async fn get_blocking_schedules(
    state: State<'_, AppState>,
) -> Result<Vec<BlockingSchedule>> {
    let schedules = sqlx::query_as::<_, BlockingSchedule>(
        r#"
        SELECT * FROM blocking_schedules
        ORDER BY day_of_week, start_time
        "#,
    )
    .fetch_all(state.pool())
    .await?;

    Ok(schedules)
}

/// Update a blocking schedule
#[tauri::command]
pub async fn update_blocking_schedule(
    request: UpdateScheduleRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    if let Some(start_time) = &request.start_time {
        if !is_valid_time(start_time) {
            return Err(Error::InvalidInput("Invalid start time format".to_string()));
        }
    }

    if let Some(end_time) = &request.end_time {
        if !is_valid_time(end_time) {
            return Err(Error::InvalidInput("Invalid end time format".to_string()));
        }
    }

    let mut query = String::from("UPDATE blocking_schedules SET updated_at = CURRENT_TIMESTAMP");
    let mut bindings: Vec<String> = Vec::new();

    if let Some(enabled) = request.enabled {
        query.push_str(", enabled = ?");
        bindings.push(if enabled { "1" } else { "0" }.to_string());
    }

    if let Some(start_time) = request.start_time {
        query.push_str(", start_time = ?");
        bindings.push(start_time);
    }

    if let Some(end_time) = request.end_time {
        query.push_str(", end_time = ?");
        bindings.push(end_time);
    }

    query.push_str(" WHERE id = ?");

    let mut sql_query = sqlx::query(&query);
    for binding in bindings {
        sql_query = sql_query.bind(binding);
    }
    sql_query = sql_query.bind(request.id);

    sql_query.execute(state.pool()).await?;

    tracing::info!("Updated blocking schedule: {}", request.id);

    Ok(())
}

/// Delete a blocking schedule
#[tauri::command]
pub async fn delete_blocking_schedule(id: i64, state: State<'_, AppState>) -> Result<()> {
    sqlx::query("DELETE FROM blocking_schedules WHERE id = ?")
        .bind(id)
        .execute(state.pool())
        .await?;

    tracing::info!("Deleted blocking schedule: {}", id);

    Ok(())
}

// ============================================================================
// Blocking Categories
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockingCategory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub items: String, // JSON array stored as string
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub items: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

/// Get all blocking categories
#[tauri::command]
pub async fn get_blocking_categories(
    state: State<'_, AppState>,
) -> Result<Vec<CategoryResponse>> {
    let categories = sqlx::query_as::<_, BlockingCategory>(
        r#"
        SELECT * FROM blocking_categories
        ORDER BY name
        "#,
    )
    .fetch_all(state.pool())
    .await?;

    let response: Vec<CategoryResponse> = categories
        .into_iter()
        .map(|cat| {
            let items: Vec<String> = serde_json::from_str(&cat.items).unwrap_or_default();
            CategoryResponse {
                id: cat.id,
                name: cat.name,
                description: cat.description,
                items,
                enabled: cat.enabled,
                created_at: cat.created_at,
                updated_at: cat.updated_at,
            }
        })
        .collect();

    Ok(response)
}

/// Create a custom blocking category
#[tauri::command]
pub async fn create_blocking_category(
    request: CreateCategoryRequest,
    state: State<'_, AppState>,
) -> Result<CategoryResponse> {
    let items_json = serde_json::to_string(&request.items)?;

    let result = sqlx::query(
        r#"
        INSERT INTO blocking_categories (name, description, items)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&items_json)
    .execute(state.pool())
    .await?;

    let id = result.last_insert_rowid();

    // Fetch the created category
    let category = sqlx::query_as::<_, BlockingCategory>(
        r#"
        SELECT * FROM blocking_categories WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(state.pool())
    .await?;

    let items: Vec<String> = serde_json::from_str(&category.items)?;

    tracing::info!("Created blocking category: {}", request.name);

    Ok(CategoryResponse {
        id: category.id,
        name: category.name,
        description: category.description,
        items,
        enabled: category.enabled,
        created_at: category.created_at,
        updated_at: category.updated_at,
    })
}

/// Update a blocking category
#[tauri::command]
pub async fn update_blocking_category(
    request: UpdateCategoryRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    let mut query = String::from("UPDATE blocking_categories SET updated_at = CURRENT_TIMESTAMP");
    let mut bindings: Vec<(String, String)> = Vec::new();

    if let Some(name) = request.name {
        query.push_str(", name = ?");
        bindings.push(("name".to_string(), name));
    }

    if let Some(description) = request.description {
        query.push_str(", description = ?");
        bindings.push(("description".to_string(), description));
    }

    if let Some(items) = request.items {
        let items_json = serde_json::to_string(&items)?;
        query.push_str(", items = ?");
        bindings.push(("items".to_string(), items_json));
    }

    if let Some(enabled) = request.enabled {
        query.push_str(", enabled = ?");
        bindings.push(("enabled".to_string(), if enabled { "1" } else { "0" }.to_string()));
    }

    query.push_str(" WHERE id = ?");

    let mut sql_query = sqlx::query(&query);
    for (_, value) in bindings {
        sql_query = sql_query.bind(value);
    }
    sql_query = sql_query.bind(request.id);

    sql_query.execute(state.pool()).await?;

    tracing::info!("Updated blocking category: {}", request.id);

    Ok(())
}

/// Toggle category enabled state
#[tauri::command]
pub async fn toggle_blocking_category(
    id: i64,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE blocking_categories
        SET enabled = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(enabled)
    .bind(id)
    .execute(state.pool())
    .await?;

    tracing::info!("Toggled category {} to {}", id, enabled);

    Ok(())
}

// ============================================================================
// Strict Mode
// ============================================================================

#[derive(Debug, Serialize)]
pub struct StrictModeState {
    pub enabled: bool,
    pub session_id: Option<String>,
    pub started_at: Option<String>,
    pub can_disable: bool,
}

/// Enable strict mode (cannot disable until session ends)
#[tauri::command]
pub async fn enable_strict_mode(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<()> {
    let mut strict_mode = state.strict_mode_state.write().await;
    strict_mode.enabled = true;
    strict_mode.session_id = Some(session_id.clone());
    strict_mode.started_at = Some(Utc::now().to_rfc3339());

    tracing::info!("Strict mode enabled for session: {}", session_id);

    Ok(())
}

/// Disable strict mode (only if session has ended)
#[tauri::command]
pub async fn disable_strict_mode(state: State<'_, AppState>) -> Result<()> {
    let strict_mode = state.strict_mode_state.read().await;

    // Check if there's an active session
    if let Some(session_id) = &strict_mode.session_id {
        let active_session = queries::get_session(state.pool(), session_id).await?;

        if let Some(session) = active_session {
            if session.end_time.is_none() {
                return Err(Error::InvalidInput(
                    "Cannot disable strict mode while session is active".to_string(),
                ));
            }
        }
    }

    drop(strict_mode); // Release read lock

    let mut strict_mode = state.strict_mode_state.write().await;
    strict_mode.enabled = false;
    strict_mode.session_id = None;
    strict_mode.started_at = None;

    tracing::info!("Strict mode disabled");

    Ok(())
}

/// Get strict mode state
#[tauri::command]
pub async fn get_strict_mode_state(state: State<'_, AppState>) -> Result<StrictModeState> {
    let strict_mode = state.strict_mode_state.read().await;

    let can_disable = if let Some(session_id) = &strict_mode.session_id {
        let active_session = queries::get_session(state.pool(), session_id).await?;
        active_session.map(|s| s.end_time.is_some()).unwrap_or(true)
    } else {
        true
    };

    Ok(StrictModeState {
        enabled: strict_mode.enabled,
        session_id: strict_mode.session_id.clone(),
        started_at: strict_mode.started_at.clone(),
        can_disable,
    })
}

// ============================================================================
// Nuclear Option
// ============================================================================

#[derive(Debug, Serialize)]
pub struct NuclearOption {
    pub active: bool,
    pub duration_minutes: i32,
    pub started_at: Option<String>,
    pub ends_at: Option<String>,
    pub remaining_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ActivateNuclearOptionRequest {
    pub duration_minutes: i32,
}

/// Activate nuclear option (irreversible lockdown)
#[tauri::command]
pub async fn activate_nuclear_option(
    request: ActivateNuclearOptionRequest,
    state: State<'_, AppState>,
) -> Result<NuclearOption> {
    // Validate duration
    if ![5, 10, 15, 30, 60].contains(&request.duration_minutes) {
        return Err(Error::InvalidInput(
            "Duration must be 5, 10, 15, 30, or 60 minutes".to_string(),
        ));
    }

    let started_at = Utc::now();
    let ends_at = started_at + chrono::Duration::minutes(request.duration_minutes as i64);

    let mut nuclear_state = state.nuclear_option_state.write().await;
    nuclear_state.active = true;
    nuclear_state.duration_minutes = request.duration_minutes;
    nuclear_state.started_at = Some(started_at);
    nuclear_state.ends_at = Some(ends_at);

    tracing::warn!(
        "Nuclear option activated for {} minutes",
        request.duration_minutes
    );

    let remaining = (ends_at - Utc::now()).num_seconds();

    Ok(NuclearOption {
        active: true,
        duration_minutes: request.duration_minutes,
        started_at: Some(started_at.to_rfc3339()),
        ends_at: Some(ends_at.to_rfc3339()),
        remaining_seconds: Some(remaining),
    })
}

/// Get nuclear option state
#[tauri::command]
pub async fn get_nuclear_option_state(state: State<'_, AppState>) -> Result<NuclearOption> {
    let nuclear_state = state.nuclear_option_state.read().await;

    if nuclear_state.active {
        if let Some(ends_at) = nuclear_state.ends_at {
            let now = Utc::now();

            // Check if nuclear option has expired
            if now >= ends_at {
                drop(nuclear_state); // Release read lock
                let mut nuclear_state = state.nuclear_option_state.write().await;
                nuclear_state.active = false;
                nuclear_state.started_at = None;
                nuclear_state.ends_at = None;

                return Ok(NuclearOption {
                    active: false,
                    duration_minutes: 0,
                    started_at: None,
                    ends_at: None,
                    remaining_seconds: None,
                });
            }

            let remaining = (ends_at - now).num_seconds();

            return Ok(NuclearOption {
                active: true,
                duration_minutes: nuclear_state.duration_minutes,
                started_at: nuclear_state.started_at.map(|dt| dt.to_rfc3339()),
                ends_at: Some(ends_at.to_rfc3339()),
                remaining_seconds: Some(remaining),
            });
        }
    }

    Ok(NuclearOption {
        active: false,
        duration_minutes: 0,
        started_at: None,
        ends_at: None,
        remaining_seconds: None,
    })
}

// ============================================================================
// Block Statistics
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockAttempt {
    pub id: i64,
    pub user_id: Option<String>,
    pub item_type: String,
    pub item_value: String,
    pub blocked_at: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockStatistics {
    pub total_attempts: i64,
    pub attempts_today: i64,
    pub attempts_this_week: i64,
    pub attempts_this_month: i64,
    pub top_blocked_items: Vec<BlockedItemStats>,
    pub attempts_by_hour: Vec<HourlyStats>,
    pub attempts_by_type: BlocksByType,
    pub recent_blocks: Vec<RecentBlock>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BlockedItemStats {
    pub item_type: String,
    pub item_value: String,
    pub count: i64,
    pub last_attempt: String,
}

#[derive(Debug, Serialize)]
pub struct HourlyStats {
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct BlocksByType {
    pub apps: i64,
    pub websites: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RecentBlock {
    pub item_type: String,
    pub item_value: String,
    pub blocked_at: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordAttemptRequest {
    pub item_type: String,
    pub item_value: String,
    pub session_id: Option<String>,
}

/// Record a block attempt
#[tauri::command]
pub async fn record_block_attempt(
    request: RecordAttemptRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO block_attempts (item_type, item_value, session_id)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&request.item_type)
    .bind(&request.item_value)
    .bind(&request.session_id)
    .execute(state.pool())
    .await?;

    tracing::debug!("Recorded block attempt: {}", request.item_value);

    Ok(())
}

/// Get block statistics
#[tauri::command]
pub async fn get_block_statistics(
    days: Option<i32>,
    state: State<'_, AppState>,
) -> Result<BlockStatistics> {
    let _days = days.unwrap_or(7);
    let user_id = state.get_user_id().await;

    // Total attempts
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM block_attempts WHERE user_id IS NULL OR user_id = ?"
    )
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    // Attempts today
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let attempts_today: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM block_attempts WHERE DATE(blocked_at) = ? AND (user_id IS NULL OR user_id = ?)",
    )
    .bind(&today)
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    // Attempts this week
    let week_ago = (Utc::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let attempts_week: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM block_attempts WHERE DATE(blocked_at) >= ? AND (user_id IS NULL OR user_id = ?)",
    )
    .bind(&week_ago)
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    // Attempts this month
    let month_ago = (Utc::now() - chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();
    let attempts_month: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM block_attempts WHERE DATE(blocked_at) >= ? AND (user_id IS NULL OR user_id = ?)",
    )
    .bind(&month_ago)
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    // Attempts by type
    let apps_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM block_attempts WHERE item_type = 'app' AND DATE(blocked_at) >= ? AND (user_id IS NULL OR user_id = ?)",
    )
    .bind(&week_ago)
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    let websites_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM block_attempts WHERE item_type = 'website' AND DATE(blocked_at) >= ? AND (user_id IS NULL OR user_id = ?)",
    )
    .bind(&week_ago)
    .bind(&user_id)
    .fetch_one(state.pool())
    .await?;

    // Top blocked items
    let top_items = sqlx::query_as::<_, BlockedItemStats>(
        r#"
        SELECT
            item_type,
            item_value,
            COUNT(*) as count,
            MAX(blocked_at) as last_attempt
        FROM block_attempts
        WHERE DATE(blocked_at) >= ?
            AND (user_id IS NULL OR user_id = ?)
        GROUP BY item_type, item_value
        ORDER BY count DESC
        LIMIT 10
        "#,
    )
    .bind(&week_ago)
    .bind(&user_id)
    .fetch_all(state.pool())
    .await?;

    // Attempts by hour (for the selected time period)
    let hourly_raw: Vec<(i32, i64)> = sqlx::query_as(
        r#"
        SELECT
            CAST(strftime('%H', blocked_at) AS INTEGER) as hour,
            COUNT(*) as count
        FROM block_attempts
        WHERE DATE(blocked_at) >= ?
            AND (user_id IS NULL OR user_id = ?)
        GROUP BY hour
        ORDER BY hour
        "#,
    )
    .bind(&week_ago)
    .bind(&user_id)
    .fetch_all(state.pool())
    .await?;

    // Create a map for quick lookup
    let hourly_map: std::collections::HashMap<i32, i64> = hourly_raw.into_iter().collect();

    // Fill all 24 hours
    let attempts_by_hour: Vec<HourlyStats> = (0..24)
        .map(|hour| HourlyStats {
            hour,
            count: *hourly_map.get(&hour).unwrap_or(&0),
        })
        .collect();

    // Recent blocks
    let recent_blocks = sqlx::query_as::<_, RecentBlock>(
        r#"
        SELECT
            item_type,
            item_value,
            blocked_at,
            session_id
        FROM block_attempts
        WHERE user_id IS NULL OR user_id = ?
        ORDER BY blocked_at DESC
        LIMIT 20
        "#,
    )
    .bind(&user_id)
    .fetch_all(state.pool())
    .await?;

    Ok(BlockStatistics {
        total_attempts: total.0,
        attempts_today: attempts_today.0,
        attempts_this_week: attempts_week.0,
        attempts_this_month: attempts_month.0,
        top_blocked_items: top_items,
        attempts_by_hour,
        attempts_by_type: BlocksByType {
            apps: apps_count.0,
            websites: websites_count.0,
        },
        recent_blocks,
    })
}

/// Get block attempts for a specific session
#[tauri::command]
pub async fn get_session_blocks(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<BlockAttempt>> {
    let user_id = state.get_user_id().await;

    let blocks = sqlx::query_as::<_, BlockAttempt>(
        r#"
        SELECT
            id,
            user_id,
            item_type,
            item_value,
            blocked_at,
            session_id
        FROM block_attempts
        WHERE session_id = ?
            AND (user_id IS NULL OR user_id = ?)
        ORDER BY blocked_at DESC
        "#,
    )
    .bind(&session_id)
    .bind(&user_id)
    .fetch_all(state.pool())
    .await?;

    Ok(blocks)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn is_valid_time(time: &str) -> bool {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    let hours = parts[0].parse::<u32>();
    let minutes = parts[1].parse::<u32>();

    matches!((hours, minutes), (Ok(h), Ok(m)) if h < 24 && m < 60)
}
