// commands/streaks.rs - Enhanced Streak System Commands
//
// Features:
// - GitHub-style heatmap data
// - Streak freeze management
// - Milestone tracking
// - Grace period handling
// - Weekly/monthly statistics

use crate::{AppState, Error, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use tauri::State;

// Constants
const GRACE_PERIOD_HOURS: i64 = 2;
const MIN_SESSIONS_FOR_STREAK: i32 = 1;
const PERFECT_DAY_SESSIONS: i32 = 4;

// Streak freeze source enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum FreezeSource {
    Weekly,
    Achievement,
    Purchase,
}

// Milestone tier enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum MilestoneTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl MilestoneTier {
    fn days_required(&self) -> i32 {
        match self {
            MilestoneTier::Bronze => 7,
            MilestoneTier::Silver => 30,
            MilestoneTier::Gold => 90,
            MilestoneTier::Platinum => 180,
            MilestoneTier::Diamond => 365,
        }
    }

    fn reward(&self) -> Option<String> {
        match self {
            MilestoneTier::Bronze => None,
            MilestoneTier::Silver => Some("streak_freeze".to_string()),
            MilestoneTier::Gold => Some("streak_freeze".to_string()),
            MilestoneTier::Platinum => Some("streak_freeze".to_string()),
            MilestoneTier::Diamond => Some("badge".to_string()),
        }
    }
}

// Database models
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StreakFreeze {
    pub id: String,
    pub user_id: Option<String>,
    pub used_at: Option<String>,
    pub source: FreezeSource,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StreakHistoryEntry {
    pub id: String,
    pub user_id: Option<String>,
    pub date: String,
    pub sessions_count: i32,
    pub focus_minutes: i32,
    pub was_frozen: bool,
    pub created_at: String,
}

// API models
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentStreak {
    pub current_count: i32,
    pub longest_count: i32,
    pub last_activity_date: Option<String>,
    pub is_in_grace_period: bool,
    pub grace_period_ends_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreakMilestone {
    pub tier: MilestoneTier,
    pub days_required: i32,
    pub achieved_at: Option<String>,
    pub is_achieved: bool,
    pub reward: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatmapCell {
    pub date: String,
    pub sessions_count: i32,
    pub focus_minutes: i32,
    pub intensity: i32,
    pub was_frozen: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreakHeatmapData {
    pub start_date: String,
    pub end_date: String,
    pub cells: Vec<HeatmapCell>,
    pub max_intensity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreakStats {
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub total_days: i32,
    pub active_days: i32,
    pub total_sessions: i32,
    pub total_focus_minutes: i32,
    pub average_sessions_per_day: f64,
    pub average_focus_minutes_per_day: f64,
    pub perfect_days: i32,
    pub current_streak: i32,
    pub longest_streak_in_period: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableFreezes {
    pub weekly_freeze: Option<StreakFreeze>,
    pub earned_freezes: Vec<StreakFreeze>,
    pub total_available: i32,
}

#[derive(Debug, Deserialize)]
pub struct UseStreakFreezeRequest {
    pub freeze_id: String,
    pub date: String,
}

// Tauri Commands

#[tauri::command]
pub async fn get_current_streak(state: State<'_, AppState>) -> Result<CurrentStreak> {
    let user_id = state.get_user_id().await;

    // Get streak history ordered by date descending
    let history: Vec<StreakHistoryEntry> = sqlx::query_as(
        r#"
        SELECT * FROM streak_history
        WHERE user_id IS NULL OR user_id = ?
        ORDER BY date DESC
        LIMIT 400
        "#,
    )
    .bind(&user_id)
    .fetch_all(state.pool())
    .await?;

    if history.is_empty() {
        return Ok(CurrentStreak {
            current_count: 0,
            longest_count: 0,
            last_activity_date: None,
            is_in_grace_period: false,
            grace_period_ends_at: None,
        });
    }

    let today = Local::now().date_naive();
    let _yesterday = today - Duration::days(1);

    // Calculate current streak
    let mut current_count = 0;
    let mut expected_date = today;

    for entry in &history {
        let entry_date = NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d")
            .map_err(|e| Error::Validation(e.to_string()))?;

        if entry_date == expected_date {
            if entry.sessions_count >= MIN_SESSIONS_FOR_STREAK || entry.was_frozen {
                current_count += 1;
                expected_date -= Duration::days(1);
            } else {
                break;
            }
        } else if entry_date < expected_date {
            break;
        }
    }

    // Calculate longest streak
    let longest_count = calculate_longest_streak(&history)?;

    // Check grace period
    let last_activity_date = history.first().map(|e| e.date.clone());
    let (is_in_grace_period, grace_period_ends_at) = check_grace_period(&last_activity_date)?;

    Ok(CurrentStreak {
        current_count,
        longest_count,
        last_activity_date,
        is_in_grace_period,
        grace_period_ends_at,
    })
}

#[tauri::command]
pub async fn get_streak_heatmap(
    state: State<'_, AppState>,
    months: Option<i32>,
) -> Result<StreakHeatmapData> {
    let user_id = state.get_user_id().await;

    let months = months.unwrap_or(12);
    let end_date = Local::now().date_naive();
    let start_date = end_date - Duration::days(months as i64 * 30);

    let history: Vec<StreakHistoryEntry> = sqlx::query_as(
        r#"
        SELECT * FROM streak_history
        WHERE (user_id IS NULL OR user_id = ?)
        AND date >= ? AND date <= ?
        ORDER BY date ASC
        "#,
    )
    .bind(&user_id)
    .bind(start_date.format("%Y-%m-%d").to_string())
    .bind(end_date.format("%Y-%m-%d").to_string())
    .fetch_all(state.pool())
    .await?;

    // Create map for quick lookup
    let history_map: HashMap<String, &StreakHistoryEntry> =
        history.iter().map(|e| (e.date.clone(), e)).collect();

    // Calculate max focus minutes for intensity scaling
    let max_focus_minutes = history.iter().map(|e| e.focus_minutes).max().unwrap_or(120);

    // Generate cells for each day
    let mut cells = Vec::new();
    let mut current = start_date;

    while current <= end_date {
        let date_str = current.format("%Y-%m-%d").to_string();

        if let Some(entry) = history_map.get(&date_str) {
            let intensity = calculate_intensity(entry.focus_minutes, max_focus_minutes);
            cells.push(HeatmapCell {
                date: date_str,
                sessions_count: entry.sessions_count,
                focus_minutes: entry.focus_minutes,
                intensity,
                was_frozen: entry.was_frozen,
            });
        } else {
            cells.push(HeatmapCell {
                date: date_str,
                sessions_count: 0,
                focus_minutes: 0,
                intensity: 0,
                was_frozen: false,
            });
        }

        current += Duration::days(1);
    }

    Ok(StreakHeatmapData {
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        cells,
        max_intensity: 4,
    })
}

#[tauri::command]
pub async fn get_streak_stats(
    state: State<'_, AppState>,
    period: String,
) -> Result<StreakStats> {
    let user_id = state.get_user_id().await;

    let (start_date, end_date) = match period.as_str() {
        "week" => {
            let end = Local::now().date_naive();
            let start = end - Duration::days(6);
            (start, end)
        }
        "month" => {
            let end = Local::now().date_naive();
            let start = end - Duration::days(29);
            (start, end)
        }
        _ => return Err(Error::Validation("Invalid period".to_string())),
    };

    let history: Vec<StreakHistoryEntry> = sqlx::query_as(
        r#"
        SELECT * FROM streak_history
        WHERE (user_id IS NULL OR user_id = ?)
        AND date >= ? AND date <= ?
        ORDER BY date ASC
        "#,
    )
    .bind(&user_id)
    .bind(start_date.format("%Y-%m-%d").to_string())
    .bind(end_date.format("%Y-%m-%d").to_string())
    .fetch_all(state.pool())
    .await?;

    let total_days = (end_date - start_date).num_days() as i32 + 1;
    let active_days = history
        .iter()
        .filter(|e| e.sessions_count >= MIN_SESSIONS_FOR_STREAK || e.was_frozen)
        .count() as i32;
    let total_sessions: i32 = history.iter().map(|e| e.sessions_count).sum();
    let total_focus_minutes: i32 = history.iter().map(|e| e.focus_minutes).sum();
    let perfect_days = history
        .iter()
        .filter(|e| e.sessions_count >= PERFECT_DAY_SESSIONS)
        .count() as i32;

    let average_sessions_per_day = if active_days > 0 {
        total_sessions as f64 / active_days as f64
    } else {
        0.0
    };

    let average_focus_minutes_per_day = if active_days > 0 {
        total_focus_minutes as f64 / active_days as f64
    } else {
        0.0
    };

    let current_streak = calculate_current_streak_in_period(&history)?;
    let longest_streak_in_period = calculate_longest_streak(&history)?;

    Ok(StreakStats {
        period,
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        total_days,
        active_days,
        total_sessions,
        total_focus_minutes,
        average_sessions_per_day,
        average_focus_minutes_per_day,
        perfect_days,
        current_streak,
        longest_streak_in_period,
    })
}

#[tauri::command]
pub async fn get_streak_milestones(state: State<'_, AppState>) -> Result<Vec<StreakMilestone>> {
    let current_streak = get_current_streak(state.clone()).await?;
    let current_count = current_streak.current_count;

    let tiers = vec![
        MilestoneTier::Bronze,
        MilestoneTier::Silver,
        MilestoneTier::Gold,
        MilestoneTier::Platinum,
        MilestoneTier::Diamond,
    ];

    let milestones = tiers
        .into_iter()
        .map(|tier| {
            let days_required = tier.days_required();
            let is_achieved = current_count >= days_required;
            let achieved_at = if is_achieved {
                Some(Utc::now().to_rfc3339())
            } else {
                None
            };

            StreakMilestone {
                reward: tier.reward(),
                tier,
                days_required,
                achieved_at,
                is_achieved,
            }
        })
        .collect();

    Ok(milestones)
}

#[tauri::command]
pub async fn get_available_freezes(state: State<'_, AppState>) -> Result<AvailableFreezes> {
    let user_id = state.get_user_id().await;

    let freezes: Vec<StreakFreeze> = sqlx::query_as(
        r#"
        SELECT * FROM streak_freezes
        WHERE (user_id IS NULL OR user_id = ?)
        AND used_at IS NULL
        AND (expires_at IS NULL OR expires_at > datetime('now'))
        ORDER BY created_at ASC
        "#,
    )
    .bind(&user_id)
    .fetch_all(state.pool())
    .await?;

    let mut weekly_freeze = None;
    let mut earned_freezes = Vec::new();

    for freeze in freezes {
        match freeze.source {
            FreezeSource::Weekly => weekly_freeze = Some(freeze),
            _ => earned_freezes.push(freeze),
        }
    }

    let total_available = weekly_freeze.iter().count() + earned_freezes.len();

    Ok(AvailableFreezes {
        weekly_freeze,
        earned_freezes,
        total_available: total_available as i32,
    })
}

#[tauri::command]
pub async fn use_streak_freeze(
    state: State<'_, AppState>,
    request: UseStreakFreezeRequest,
) -> Result<StreakHistoryEntry> {
    let user_id = state.get_user_id().await;

    // Verify freeze exists and is available
    let freeze: Option<StreakFreeze> = sqlx::query_as(
        r#"
        SELECT * FROM streak_freezes
        WHERE id = ? AND used_at IS NULL
        "#,
    )
    .bind(&request.freeze_id)
    .fetch_optional(state.pool())
    .await?;

    let _freeze = freeze.ok_or_else(|| Error::NotFound("Freeze not found or already used".to_string()))?;

    // Mark freeze as used
    sqlx::query(
        r#"
        UPDATE streak_freezes
        SET used_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(&request.freeze_id)
    .execute(state.pool())
    .await?;

    // Create or update streak history entry
    let entry_id = uuid::Uuid::new_v4().to_string();
    let entry: StreakHistoryEntry = sqlx::query_as(
        r#"
        INSERT INTO streak_history (id, user_id, date, sessions_count, focus_minutes, was_frozen)
        VALUES (?, ?, ?, 0, 0, 1)
        ON CONFLICT(user_id, date) DO UPDATE SET
            was_frozen = 1
        RETURNING *
        "#,
    )
    .bind(&entry_id)
    .bind(&user_id)
    .bind(&request.date)
    .fetch_one(state.pool())
    .await?;

    Ok(entry)
}

#[tauri::command]
pub async fn update_streak_history(state: State<'_, AppState>) -> Result<StreakHistoryEntry> {
    let user_id = state.get_user_id().await;

    let today = Local::now().format("%Y-%m-%d").to_string();

    // Count today's completed sessions
    let sessions_count: (i32,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM sessions
        WHERE (user_id IS NULL OR user_id = ?)
        AND DATE(start_time) = ?
        AND completed = 1
        "#,
    )
    .bind(&user_id)
    .bind(&today)
    .fetch_one(state.pool())
    .await?;

    // Sum today's focus minutes
    let focus_minutes: (i32,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(actual_duration_seconds / 60), 0) FROM sessions
        WHERE (user_id IS NULL OR user_id = ?)
        AND DATE(start_time) = ?
        AND completed = 1
        AND session_type = 'focus'
        "#,
    )
    .bind(&user_id)
    .bind(&today)
    .fetch_one(state.pool())
    .await?;

    // Update or create streak history
    let entry_id = uuid::Uuid::new_v4().to_string();
    let entry: StreakHistoryEntry = sqlx::query_as(
        r#"
        INSERT INTO streak_history (id, user_id, date, sessions_count, focus_minutes, was_frozen)
        VALUES (?, ?, ?, ?, ?, 0)
        ON CONFLICT(user_id, date) DO UPDATE SET
            sessions_count = excluded.sessions_count,
            focus_minutes = excluded.focus_minutes
        RETURNING *
        "#,
    )
    .bind(&entry_id)
    .bind(&user_id)
    .bind(&today)
    .bind(sessions_count.0)
    .bind(focus_minutes.0)
    .fetch_one(state.pool())
    .await?;

    Ok(entry)
}

#[tauri::command]
pub async fn create_weekly_freeze(state: State<'_, AppState>) -> Result<StreakFreeze> {
    let user_id = state.get_user_id().await;

    // Check if weekly freeze already exists for this week
    let existing: Option<StreakFreeze> = sqlx::query_as(
        r#"
        SELECT * FROM streak_freezes
        WHERE (user_id IS NULL OR user_id = ?)
        AND source = 'weekly'
        AND expires_at > datetime('now')
        LIMIT 1
        "#,
    )
    .bind(&user_id)
    .fetch_optional(state.pool())
    .await?;

    if let Some(freeze) = existing {
        return Ok(freeze);
    }

    // Create new weekly freeze (expires next Monday)
    let now = Local::now();
    let days_until_monday = (7 - now.weekday().num_days_from_monday()) % 7;
    let next_monday = now + Duration::days(days_until_monday as i64 + 7);

    let freeze_id = uuid::Uuid::new_v4().to_string();
    let freeze: StreakFreeze = sqlx::query_as(
        r#"
        INSERT INTO streak_freezes (id, user_id, source, expires_at)
        VALUES (?, ?, 'weekly', ?)
        RETURNING *
        "#,
    )
    .bind(&freeze_id)
    .bind(&user_id)
    .bind(next_monday.format("%Y-%m-%d %H:%M:%S").to_string())
    .fetch_one(state.pool())
    .await?;

    Ok(freeze)
}

// Helper functions

fn calculate_longest_streak(history: &[StreakHistoryEntry]) -> Result<i32> {
    if history.is_empty() {
        return Ok(0);
    }

    let mut longest = 0;
    let mut current = 0;
    let mut prev_date: Option<NaiveDate> = None;

    for entry in history.iter().rev() {
        let entry_date = NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d")
            .map_err(|e| Error::Validation(e.to_string()))?;

        if entry.sessions_count >= MIN_SESSIONS_FOR_STREAK || entry.was_frozen {
            if let Some(prev) = prev_date {
                if entry_date == prev + Duration::days(1) {
                    current += 1;
                } else {
                    longest = longest.max(current);
                    current = 1;
                }
            } else {
                current = 1;
            }
            prev_date = Some(entry_date);
        } else {
            longest = longest.max(current);
            current = 0;
            prev_date = None;
        }
    }

    Ok(longest.max(current))
}

fn calculate_current_streak_in_period(history: &[StreakHistoryEntry]) -> Result<i32> {
    if history.is_empty() {
        return Ok(0);
    }

    let mut current = 0;
    let mut prev_date: Option<NaiveDate> = None;

    for entry in history.iter().rev() {
        let entry_date = NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d")
            .map_err(|e| Error::Validation(e.to_string()))?;

        if entry.sessions_count >= MIN_SESSIONS_FOR_STREAK || entry.was_frozen {
            if let Some(prev) = prev_date {
                if entry_date == prev + Duration::days(1) {
                    current += 1;
                } else {
                    break;
                }
            } else {
                current = 1;
            }
            prev_date = Some(entry_date);
        } else if prev_date.is_some() {
            break;
        }
    }

    Ok(current)
}

fn check_grace_period(last_activity_date: &Option<String>) -> Result<(bool, Option<String>)> {
    let Some(last_date_str) = last_activity_date else {
        return Ok((false, None));
    };

    let last_date = NaiveDate::parse_from_str(last_date_str, "%Y-%m-%d")
        .map_err(|e| Error::Validation(e.to_string()))?;

    let now = Local::now();
    let today = now.date_naive();
    let yesterday = today - Duration::days(1);

    if last_date == yesterday {
        let midnight_time = today
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| Error::Validation("Invalid time: 00:00:00".to_string()))?;
        let midnight = Local
            .from_local_datetime(&midnight_time)
            .single()
            .ok_or_else(|| Error::Validation("Invalid datetime".to_string()))?;

        let grace_period_end = midnight + Duration::hours(GRACE_PERIOD_HOURS);

        if now < grace_period_end {
            return Ok((true, Some(grace_period_end.to_rfc3339())));
        }
    }

    Ok((false, None))
}

fn calculate_intensity(focus_minutes: i32, max_focus_minutes: i32) -> i32 {
    if focus_minutes == 0 {
        return 0;
    }

    let percentage = (focus_minutes as f64 / max_focus_minutes as f64 * 100.0) as i32;

    match percentage {
        0 => 0,
        1..=25 => 1,
        26..=50 => 2,
        51..=75 => 3,
        _ => 4,
    }
}
