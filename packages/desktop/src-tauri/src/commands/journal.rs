// commands/journal.rs - Trigger journaling commands

use crate::{AppState, Result};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Trigger types based on Indistractable framework
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum TriggerType {
    // Internal triggers
    Boredom,
    Anxiety,
    Stress,
    Fatigue,
    // External triggers
    Notification,
    Person,
    Environment,
    Other,
}

/// Emotion types for journaling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Emotion {
    Frustrated,
    Anxious,
    Tired,
    Distracted,
    Curious,
    Bored,
    Overwhelmed,
    Neutral,
}

/// Journal entry response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub session_id: Option<String>,
    pub trigger_type: String,
    pub emotion: Option<String>,
    pub notes: Option<String>,
    pub intensity: Option<i32>,
    pub created_at: String,
}

/// Create journal entry request
#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub session_id: Option<String>,
    pub trigger_type: String,
    pub emotion: Option<String>,
    pub notes: Option<String>,
    pub intensity: Option<i32>,
}

/// Trigger pattern insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerInsight {
    pub trigger_type: String,
    pub frequency: i32,
    pub peak_hour: Option<i32>,
    pub peak_day: Option<i32>,
}

/// Create a new journal entry
#[tauri::command]
pub async fn create_journal_entry(
    state: State<'_, AppState>,
    request: CreateJournalEntryRequest,
) -> Result<JournalEntry> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Get device_id from settings
    let device_id: Option<String> = sqlx::query_scalar(
        "SELECT value FROM user_settings WHERE key = 'device_id'"
    )
    .fetch_optional(state.pool())
    .await?;

    sqlx::query(
        r#"
        INSERT INTO journal_entries (id, session_id, trigger_type, emotion, notes, intensity, created_at, device_id, last_modified)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&id)
    .bind(&request.session_id)
    .bind(&request.trigger_type)
    .bind(&request.emotion)
    .bind(&request.notes)
    .bind(request.intensity)
    .bind(&now)
    .bind(&device_id)
    .bind(&now)
    .execute(state.pool())
    .await?;

    // Update trigger patterns for analytics
    update_trigger_patterns(&state, &request.trigger_type).await?;

    Ok(JournalEntry {
        id,
        session_id: request.session_id,
        trigger_type: request.trigger_type,
        emotion: request.emotion,
        notes: request.notes,
        intensity: request.intensity,
        created_at: now,
    })
}

/// Get journal entries for a session
#[tauri::command]
pub async fn get_session_journal_entries(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<JournalEntry>> {
    let entries: Vec<JournalEntry> = sqlx::query_as::<_, (String, Option<String>, String, Option<String>, Option<String>, Option<i32>, String)>(
        r#"
        SELECT id, session_id, trigger_type, emotion, notes, intensity, created_at
        FROM journal_entries
        WHERE session_id = ? AND deleted = 0
        ORDER BY created_at DESC
        "#
    )
    .bind(&session_id)
    .fetch_all(state.pool())
    .await?
    .into_iter()
    .map(|(id, session_id, trigger_type, emotion, notes, intensity, created_at)| {
        JournalEntry {
            id,
            session_id,
            trigger_type,
            emotion,
            notes,
            intensity,
            created_at,
        }
    })
    .collect();

    Ok(entries)
}

/// Get recent journal entries
#[tauri::command]
pub async fn get_recent_journal_entries(
    state: State<'_, AppState>,
    limit: i32,
) -> Result<Vec<JournalEntry>> {
    let entries: Vec<JournalEntry> = sqlx::query_as::<_, (String, Option<String>, String, Option<String>, Option<String>, Option<i32>, String)>(
        r#"
        SELECT id, session_id, trigger_type, emotion, notes, intensity, created_at
        FROM journal_entries
        WHERE deleted = 0
        ORDER BY created_at DESC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(state.pool())
    .await?
    .into_iter()
    .map(|(id, session_id, trigger_type, emotion, notes, intensity, created_at)| {
        JournalEntry {
            id,
            session_id,
            trigger_type,
            emotion,
            notes,
            intensity,
            created_at,
        }
    })
    .collect();

    Ok(entries)
}

/// Get trigger pattern insights
#[tauri::command]
pub async fn get_trigger_insights(state: State<'_, AppState>) -> Result<Vec<TriggerInsight>> {
    // Get frequency by trigger type
    let insights: Vec<TriggerInsight> = sqlx::query_as::<_, (String, i32)>(
        r#"
        SELECT trigger_type, COUNT(*) as frequency
        FROM journal_entries
        WHERE deleted = 0 AND created_at >= datetime('now', '-30 days')
        GROUP BY trigger_type
        ORDER BY frequency DESC
        "#
    )
    .fetch_all(state.pool())
    .await?
    .into_iter()
    .map(|(trigger_type, frequency)| {
        TriggerInsight {
            trigger_type,
            frequency,
            peak_hour: None,
            peak_day: None,
        }
    })
    .collect();

    Ok(insights)
}

/// Get peak distraction times
#[tauri::command]
pub async fn get_peak_distraction_times(state: State<'_, AppState>) -> Result<PeakTimes> {
    // Get peak hour
    let peak_hour: Option<i32> = sqlx::query_scalar(
        r#"
        SELECT CAST(strftime('%H', created_at) AS INTEGER) as hour
        FROM journal_entries
        WHERE deleted = 0 AND created_at >= datetime('now', '-30 days')
        GROUP BY hour
        ORDER BY COUNT(*) DESC
        LIMIT 1
        "#
    )
    .fetch_optional(state.pool())
    .await?;

    // Get peak day (0 = Sunday, 6 = Saturday)
    let peak_day: Option<i32> = sqlx::query_scalar(
        r#"
        SELECT CAST(strftime('%w', created_at) AS INTEGER) as day
        FROM journal_entries
        WHERE deleted = 0 AND created_at >= datetime('now', '-30 days')
        GROUP BY day
        ORDER BY COUNT(*) DESC
        LIMIT 1
        "#
    )
    .fetch_optional(state.pool())
    .await?;

    Ok(PeakTimes {
        peak_hour,
        peak_day,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakTimes {
    pub peak_hour: Option<i32>,
    pub peak_day: Option<i32>,
}

/// Update trigger patterns for analytics
async fn update_trigger_patterns(state: &State<'_, AppState>, trigger_type: &str) -> Result<()> {
    let now = chrono::Utc::now();
    let hour = now.hour() as i32;
    let day = now.weekday().num_days_from_sunday() as i32;

    // Upsert pattern
    sqlx::query(
        r#"
        INSERT INTO trigger_patterns (user_id, trigger_type, hour_of_day, day_of_week, frequency, updated_at)
        VALUES ('local', ?, ?, ?, 1, CURRENT_TIMESTAMP)
        ON CONFLICT (user_id, trigger_type, hour_of_day, day_of_week)
        DO UPDATE SET frequency = frequency + 1, updated_at = CURRENT_TIMESTAMP
        "#
    )
    .bind(trigger_type)
    .bind(hour)
    .bind(day)
    .execute(state.pool())
    .await?;

    Ok(())
}

use chrono::{Datelike, Timelike};
