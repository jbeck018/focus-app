// db/queries.rs - Type-safe database queries with SQLx macros

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::Result;

/// Session database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub planned_duration_minutes: i32,
    pub actual_duration_seconds: Option<i32>,
    pub session_type: String,
    pub completed: bool,
    pub notes: Option<String>,
}

/// Blocked item database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockedItem {
    pub id: i64,
    pub item_type: String,
    pub value: String,
    pub enabled: bool,
    #[serde(default = "default_match_type")]
    pub match_type: String,
}

/// Default match type for backward compatibility
fn default_match_type() -> String {
    "exact".to_string()
}

/// Daily analytics database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyAnalytics {
    pub date: String,
    pub total_focus_seconds: i64,
    pub total_break_seconds: i64,
    pub sessions_completed: i64,
    pub sessions_abandoned: i64,
    pub productivity_score: Option<f64>,
}

// ============================================================================
// Session Queries
// ============================================================================

/// Insert a new session record
pub async fn insert_session(
    pool: &SqlitePool,
    id: &str,
    start_time: DateTime<Utc>,
    planned_duration_minutes: i32,
    session_type: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sessions (id, start_time, planned_duration_minutes, session_type)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(start_time)
    .bind(planned_duration_minutes)
    .bind(session_type)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update session with end time and completion status
pub async fn end_session(
    pool: &SqlitePool,
    id: &str,
    end_time: DateTime<Utc>,
    completed: bool,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET end_time = ?,
            actual_duration_seconds = CAST((julianday(?) - julianday(start_time)) * 86400 AS INTEGER),
            completed = ?
        WHERE id = ?
        "#,
    )
    .bind(end_time)
    .bind(end_time)
    .bind(completed)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update planned duration for an active session (for session extension)
pub async fn update_session_duration(
    pool: &SqlitePool,
    session_id: &str,
    new_planned_duration_minutes: i32,
) -> Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET planned_duration_minutes = ?
        WHERE id = ? AND end_time IS NULL
        "#,
    )
    .bind(new_planned_duration_minutes)
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Get session by ID
pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Option<Session>> {
    let session = sqlx::query_as::<_, Session>(
        r#"
        SELECT * FROM sessions WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(session)
}

/// Get sessions within date range
pub async fn get_sessions_in_range(
    pool: &SqlitePool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Session>> {
    let sessions = sqlx::query_as::<_, Session>(
        r#"
        SELECT * FROM sessions
        WHERE start_time >= ? AND start_time <= ?
        ORDER BY start_time DESC
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    Ok(sessions)
}

/// Count sessions started today (for free tier limit enforcement)
/// Returns the number of sessions that have been started today, regardless of completion status
pub async fn count_todays_sessions(pool: &SqlitePool) -> Result<i64> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM sessions
        WHERE date(start_time) = ?
        "#,
    )
    .bind(&today)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

// ============================================================================
// Blocked Items Queries
// ============================================================================

/// Insert blocked item (app or website)
pub async fn insert_blocked_item(
    pool: &SqlitePool,
    item_type: &str,
    value: &str,
) -> Result<i64> {
    insert_blocked_item_with_match_type(pool, item_type, value, "exact").await
}

/// Insert blocked item with custom match type
pub async fn insert_blocked_item_with_match_type(
    pool: &SqlitePool,
    item_type: &str,
    value: &str,
    match_type: &str,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO blocked_items (item_type, value, match_type)
        VALUES (?, ?, ?)
        ON CONFLICT(item_type, value) DO UPDATE SET enabled = 1, match_type = ?
        "#,
    )
    .bind(item_type)
    .bind(value)
    .bind(match_type)
    .bind(match_type)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Remove blocked item by soft-delete (disable)
pub async fn remove_blocked_item(
    pool: &SqlitePool,
    item_type: &str,
    value: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE blocked_items
        SET enabled = 0
        WHERE item_type = ? AND value = ?
        "#,
    )
    .bind(item_type)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get all enabled blocked items of a specific type
pub async fn get_blocked_items(
    pool: &SqlitePool,
    item_type: Option<&str>,
) -> Result<Vec<BlockedItem>> {
    let items = if let Some(item_type) = item_type {
        sqlx::query_as::<_, BlockedItem>(
            r#"
            SELECT * FROM blocked_items
            WHERE item_type = ? AND enabled = 1
            ORDER BY created_at DESC
            "#,
        )
        .bind(item_type)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, BlockedItem>(
            r#"
            SELECT * FROM blocked_items
            WHERE enabled = 1
            ORDER BY item_type, created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?
    };

    Ok(items)
}

// ============================================================================
// Analytics Queries
// ============================================================================

/// Update or insert daily analytics
pub async fn upsert_daily_analytics(
    pool: &SqlitePool,
    date: &str,
    focus_seconds: i64,
    break_seconds: i64,
    completed: i64,
    abandoned: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO daily_analytics
            (date, total_focus_seconds, total_break_seconds, sessions_completed, sessions_abandoned)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(date) DO UPDATE SET
            total_focus_seconds = total_focus_seconds + excluded.total_focus_seconds,
            total_break_seconds = total_break_seconds + excluded.total_break_seconds,
            sessions_completed = sessions_completed + excluded.sessions_completed,
            sessions_abandoned = sessions_abandoned + excluded.sessions_abandoned,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(date)
    .bind(focus_seconds)
    .bind(break_seconds)
    .bind(completed)
    .bind(abandoned)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get daily analytics for a specific date
pub async fn get_daily_analytics(
    pool: &SqlitePool,
    date: &str,
) -> Result<Option<DailyAnalytics>> {
    let analytics = sqlx::query_as::<_, DailyAnalytics>(
        r#"
        SELECT * FROM daily_analytics WHERE date = ?
        "#,
    )
    .bind(date)
    .fetch_optional(pool)
    .await?;

    Ok(analytics)
}

/// Get analytics for date range (for weekly/monthly reports)
pub async fn get_analytics_range(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<DailyAnalytics>> {
    let analytics = sqlx::query_as::<_, DailyAnalytics>(
        r#"
        SELECT * FROM daily_analytics
        WHERE date >= ? AND date <= ?
        ORDER BY date DESC
        "#,
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    Ok(analytics)
}

// ============================================================================
// Achievement Queries
// ============================================================================

/// Achievement database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Achievement {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub rarity: String,
    pub threshold: i64,
    pub points: i64,
    pub hidden: bool,
    pub display_order: i64,
}

/// User achievement database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAchievement {
    pub id: i64,
    pub user_id: Option<String>,
    pub achievement_id: i64,
    pub unlocked_at: DateTime<Utc>,
    pub notification_sent: bool,
}

/// Achievement with unlock status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementWithStatus {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub rarity: String,
    pub threshold: i64,
    pub points: i64,
    pub hidden: bool,
    pub display_order: i64,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub progress: i64,
    #[serde(rename = "progressPercentage")]
    pub progress_percentage: f64,
}

/// Internal struct for database query result
#[derive(Debug, Clone, sqlx::FromRow)]
struct AchievementWithStatusRow {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub rarity: String,
    pub threshold: i64,
    pub points: i64,
    pub hidden: bool,
    pub display_order: i64,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
}

/// Get all achievements
///
/// Internal database query function. Achievement data is exposed via
/// commands::achievements::get_achievements which includes unlock status.
#[allow(dead_code)]
pub async fn get_all_achievements(pool: &SqlitePool) -> Result<Vec<Achievement>> {
    let achievements = sqlx::query_as::<_, Achievement>(
        r#"
        SELECT * FROM achievements
        ORDER BY category, display_order
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(achievements)
}

/// Get achievement by key
pub async fn get_achievement_by_key(
    pool: &SqlitePool,
    key: &str,
) -> Result<Option<Achievement>> {
    let achievement = sqlx::query_as::<_, Achievement>(
        r#"
        SELECT * FROM achievements WHERE key = ?
        "#,
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(achievement)
}

/// Get all achievements with unlock status for a user
pub async fn get_achievements_with_status(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<Vec<AchievementWithStatus>> {
    let rows = sqlx::query_as::<_, AchievementWithStatusRow>(
        r#"
        SELECT
            a.id,
            a.key,
            a.name,
            a.description,
            a.icon,
            a.category,
            a.rarity,
            a.threshold,
            a.points,
            a.hidden,
            a.display_order,
            CASE WHEN ua.id IS NOT NULL THEN 1 ELSE 0 END as unlocked,
            ua.unlocked_at
        FROM achievements a
        LEFT JOIN user_achievements ua ON a.id = ua.achievement_id
            AND (ua.user_id IS NULL OR ua.user_id = ?)
        ORDER BY a.category, a.display_order
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    // Calculate progress for each achievement
    let mut achievements = Vec::new();
    for row in rows {
        let progress = calculate_achievement_progress(pool, user_id, &row.key, &row.category).await?;
        let progress_percentage = if row.threshold > 0 {
            (progress as f64 / row.threshold as f64) * 100.0
        } else {
            0.0
        };

        achievements.push(AchievementWithStatus {
            id: row.id,
            key: row.key,
            name: row.name,
            description: row.description,
            icon: row.icon,
            category: row.category,
            rarity: row.rarity,
            threshold: row.threshold,
            points: row.points,
            hidden: row.hidden,
            display_order: row.display_order,
            unlocked: row.unlocked,
            unlocked_at: row.unlocked_at,
            progress,
            progress_percentage,
        });
    }

    Ok(achievements)
}

/// Calculate progress for an achievement based on its key and category
async fn calculate_achievement_progress(
    pool: &SqlitePool,
    user_id: Option<&str>,
    key: &str,
    category: &str,
) -> Result<i64> {
    match category {
        "session" => {
            // Count completed sessions
            get_completed_sessions_count(pool, user_id).await
        }
        "streak" => {
            // Get current streak
            get_current_streak(pool, user_id).await
        }
        "time" => {
            // Get total focus hours
            get_total_focus_hours(pool, user_id).await
        }
        "blocking" => {
            // Get total blocks
            get_total_blocks_count(pool, user_id).await
        }
        "special" => {
            // Special achievements need custom logic based on key
            match key {
                "weekend_warrior" => get_weekend_sessions_count(pool, user_id).await,
                "perfectionist" => get_perfect_sessions_count(pool, user_id).await,
                "zero_distractions" => get_zero_block_sessions_count(pool, user_id).await,
                _ => Ok(0), // Other special achievements are binary (0 or 1)
            }
        }
        _ => Ok(0),
    }
}

/// Check if achievement is unlocked for user
pub async fn is_achievement_unlocked(
    pool: &SqlitePool,
    user_id: Option<&str>,
    achievement_key: &str,
) -> Result<bool> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM user_achievements ua
        JOIN achievements a ON ua.achievement_id = a.id
        WHERE a.key = ?
            AND (ua.user_id IS NULL OR ua.user_id = ?)
        "#,
    )
    .bind(achievement_key)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0 > 0)
}

/// Unlock achievement for user
pub async fn unlock_achievement(
    pool: &SqlitePool,
    user_id: Option<&str>,
    achievement_id: i64,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT OR IGNORE INTO user_achievements (user_id, achievement_id)
        VALUES (?, ?)
        "#,
    )
    .bind(user_id)
    .bind(achievement_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get unlocked achievements for user
///
/// Internal database query function. User achievements are exposed via
/// commands::achievements::get_recent_achievements with proper formatting.
#[allow(dead_code)]
pub async fn get_user_achievements(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<Vec<UserAchievement>> {
    let achievements = sqlx::query_as::<_, UserAchievement>(
        r#"
        SELECT * FROM user_achievements
        WHERE user_id IS NULL OR user_id = ?
        ORDER BY unlocked_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(achievements)
}

/// Get achievement stats for user
pub async fn get_achievement_stats(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<(i64, i64, i64)> {
    // Returns (total_achievements, unlocked_count, total_points)
    let stats: (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(DISTINCT a.id) as total,
            COUNT(DISTINCT ua.id) as unlocked,
            COALESCE(SUM(CASE WHEN ua.id IS NOT NULL THEN a.points ELSE 0 END), 0) as points
        FROM achievements a
        LEFT JOIN user_achievements ua ON a.id = ua.achievement_id
            AND (ua.user_id IS NULL OR ua.user_id = ?)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(stats)
}

/// Get list of categories that have at least one unlocked achievement
pub async fn get_unlocked_categories(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<Vec<String>> {
    let categories: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT a.category
        FROM achievements a
        INNER JOIN user_achievements ua ON a.id = ua.achievement_id
        WHERE (ua.user_id IS NULL OR ua.user_id = ?)
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(categories.into_iter().map(|(c,)| c).collect())
}

/// Get recent unlocked achievements
pub async fn get_recent_achievements(
    pool: &SqlitePool,
    user_id: Option<&str>,
    limit: i64,
) -> Result<Vec<UserAchievement>> {
    let achievements = sqlx::query_as::<_, UserAchievement>(
        r#"
        SELECT * FROM user_achievements
        WHERE user_id IS NULL OR user_id = ?
        ORDER BY unlocked_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(achievements)
}

/// Get total completed sessions count
pub async fn get_completed_sessions_count(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE completed = 1
            AND (user_id IS NULL OR user_id = ?)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Get total focus time in hours
pub async fn get_total_focus_hours(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    let total: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT SUM(actual_duration_seconds) / 3600
        FROM sessions
        WHERE completed = 1
            AND session_type = 'focus'
            AND (user_id IS NULL OR user_id = ?)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(total.0.unwrap_or(0))
}

/// Get total blocks count
pub async fn get_total_blocks_count(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM block_attempts
        WHERE user_id IS NULL OR user_id = ?
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Get current streak
pub async fn get_current_streak(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    // Check if there's a session today
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    let has_session_today: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE date(start_time) = ?
            AND completed = 1
            AND (user_id IS NULL OR user_id = ?)
        "#,
    )
    .bind(&today)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if has_session_today.0 == 0 {
        return Ok(0);
    }

    // Calculate streak by counting consecutive days with completed sessions
    let streak: (Option<i64>,) = sqlx::query_as(
        r#"
        WITH RECURSIVE dates(date, streak) AS (
            SELECT date(start_time), 1
            FROM sessions
            WHERE completed = 1
                AND (user_id IS NULL OR user_id = ?)
            ORDER BY start_time DESC
            LIMIT 1
            
            UNION ALL
            
            SELECT date(s.start_time), d.streak + 1
            FROM sessions s
            JOIN dates d ON date(s.start_time) = date(d.date, '-1 day')
            WHERE s.completed = 1
                AND (s.user_id IS NULL OR s.user_id = ?)
        )
        SELECT MAX(streak) FROM dates
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(streak.0.unwrap_or(0))
}

/// Get weekend sessions count
pub async fn get_weekend_sessions_count(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE completed = 1
            AND CAST(strftime('%w', start_time) AS INTEGER) IN (0, 6)
            AND (user_id IS NULL OR user_id = ?)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Get perfect completion sessions count
pub async fn get_perfect_sessions_count(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE completed = 1
            AND actual_duration_seconds >= planned_duration_minutes * 60
            AND (user_id IS NULL OR user_id = ?)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Get sessions with zero blocks count
pub async fn get_zero_block_sessions_count(
    pool: &SqlitePool,
    user_id: Option<&str>,
) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT s.id)
        FROM sessions s
        WHERE s.completed = 1
            AND (s.user_id IS NULL OR s.user_id = ?)
            AND NOT EXISTS (
                SELECT 1 FROM block_attempts ba
                WHERE ba.session_id = s.id
            )
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

// ============================================================================
// Block Attempts Queries
// ============================================================================

/// Record a block attempt
pub async fn record_block_attempt(
    pool: &SqlitePool,
    item_type: &str,
    item_value: &str,
    session_id: Option<&str>,
    user_id: Option<&str>,
) -> Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO block_attempts (user_id, item_type, item_value, session_id)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(item_type)
    .bind(item_value)
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}
