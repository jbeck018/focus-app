// commands/achievements.rs - Achievement system commands

use crate::{
    db::queries::{self, Achievement, AchievementWithStatus, UserAchievement},
    state::AppState,
    Result,
};
use chrono::Timelike;
use serde::Serialize;
use tauri::State;
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Serialize)]
pub struct AchievementStatsResponse {
    pub total_achievements: i64,
    pub unlocked_count: i64,
    pub total_points: i64,
    pub completion_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct AchievementCheckResult {
    pub newly_unlocked: Vec<Achievement>,
}

/// Get all achievements with unlock status
#[tauri::command]
pub async fn get_achievements(
    state: State<'_, AppState>,
) -> Result<Vec<AchievementWithStatus>> {
    let user_id = state.get_user_id().await;
    let achievements = queries::get_achievements_with_status(
        state.pool(),
        user_id.as_deref(),
    )
    .await?;

    Ok(achievements)
}

/// Get user achievement statistics
#[tauri::command]
pub async fn get_achievement_stats(
    state: State<'_, AppState>,
) -> Result<AchievementStatsResponse> {
    let user_id = state.get_user_id().await;
    let (total, unlocked, points) = queries::get_achievement_stats(
        state.pool(),
        user_id.as_deref(),
    )
    .await?;

    let completion_percentage = if total > 0 {
        (unlocked as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(AchievementStatsResponse {
        total_achievements: total,
        unlocked_count: unlocked,
        total_points: points,
        completion_percentage,
    })
}

/// Get recently unlocked achievements
#[tauri::command]
pub async fn get_recent_achievements(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<UserAchievement>> {
    let user_id = state.get_user_id().await;
    let achievements = queries::get_recent_achievements(
        state.pool(),
        user_id.as_deref(),
        limit.unwrap_or(10),
    )
    .await?;

    Ok(achievements)
}

/// Check for new achievements after session completion
///
/// This is called automatically after each session ends to check
/// if the user has unlocked any new achievements.
#[tauri::command]
pub async fn check_achievements(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<AchievementCheckResult> {
    let user_id = state.get_user_id().await;
    let user_id_ref = user_id.as_deref();

    let mut newly_unlocked = Vec::new();

    // Get the session to check special achievements
    let session = queries::get_session(state.pool(), &session_id).await?;

    // Check session-based achievements
    let sessions_count = queries::get_completed_sessions_count(state.pool(), user_id_ref).await?;
    newly_unlocked.extend(
        check_threshold_achievement(
            state.pool(),
            user_id_ref,
            "first_focus",
            sessions_count,
        )
        .await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "sessions_10", sessions_count)
            .await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "sessions_50", sessions_count)
            .await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "sessions_100", sessions_count)
            .await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "sessions_500", sessions_count)
            .await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "sessions_1000", sessions_count)
            .await?,
    );

    // Check streak-based achievements
    let streak = queries::get_current_streak(state.pool(), user_id_ref).await?;
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "streak_3", streak).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "streak_7", streak).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "streak_14", streak).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "streak_30", streak).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "streak_100", streak).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "streak_365", streak).await?,
    );

    // Check time-based achievements (in hours)
    let hours = queries::get_total_focus_hours(state.pool(), user_id_ref).await?;
    newly_unlocked
        .extend(check_threshold_achievement(state.pool(), user_id_ref, "time_1h", hours).await?);
    newly_unlocked
        .extend(check_threshold_achievement(state.pool(), user_id_ref, "time_10h", hours).await?);
    newly_unlocked
        .extend(check_threshold_achievement(state.pool(), user_id_ref, "time_50h", hours).await?);
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "time_100h", hours).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "time_500h", hours).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "time_1000h", hours).await?,
    );

    // Check blocking achievements
    let blocks = queries::get_total_blocks_count(state.pool(), user_id_ref).await?;
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "first_block", blocks).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "blocks_100", blocks).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "blocks_500", blocks).await?,
    );
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "blocks_1000", blocks).await?,
    );

    // Check special achievements based on session data
    if let Some(session) = session {
        let start_hour = session.start_time.hour();

        // Night Owl (10PM - 4AM)
        if start_hour >= 22 || start_hour < 4 {
            newly_unlocked.extend(
                check_special_achievement(state.pool(), user_id_ref, "night_owl").await?,
            );
        }

        // Early Bird (5AM - 7AM)
        if start_hour >= 5 && start_hour < 7 {
            newly_unlocked.extend(
                check_special_achievement(state.pool(), user_id_ref, "early_bird").await?,
            );
        }

        // Marathon (2+ hour session)
        if let Some(duration) = session.actual_duration_seconds {
            if duration >= 7200 {
                // 2 hours = 7200 seconds
                newly_unlocked.extend(
                    check_special_achievement(state.pool(), user_id_ref, "marathon").await?,
                );
            }
        }
    }

    // Weekend Warrior
    let weekend_sessions = queries::get_weekend_sessions_count(state.pool(), user_id_ref).await?;
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "weekend_warrior", weekend_sessions)
            .await?,
    );

    // Perfectionist
    let perfect_sessions = queries::get_perfect_sessions_count(state.pool(), user_id_ref).await?;
    newly_unlocked.extend(
        check_threshold_achievement(state.pool(), user_id_ref, "perfectionist", perfect_sessions)
            .await?,
    );

    // Consistency King (check if we have sessions every day for the past week)
    if streak >= 7 {
        newly_unlocked.extend(
            check_special_achievement(state.pool(), user_id_ref, "consistency_king").await?,
        );
    }

    // Zen Master (zero distractions)
    let zero_block_sessions =
        queries::get_zero_block_sessions_count(state.pool(), user_id_ref).await?;
    newly_unlocked.extend(
        check_threshold_achievement(
            state.pool(),
            user_id_ref,
            "zero_distractions",
            zero_block_sessions,
        )
        .await?,
    );

    // Send notifications for newly unlocked achievements
    for achievement in &newly_unlocked {
        send_achievement_notification(&state, achievement).await;
    }

    Ok(AchievementCheckResult { newly_unlocked })
}

/// Helper function to check threshold-based achievements
async fn check_threshold_achievement(
    pool: &sqlx::SqlitePool,
    user_id: Option<&str>,
    achievement_key: &str,
    current_value: i64,
) -> Result<Vec<Achievement>> {
    // Check if already unlocked
    if queries::is_achievement_unlocked(pool, user_id, achievement_key).await? {
        return Ok(Vec::new());
    }

    // Get achievement details
    let achievement = queries::get_achievement_by_key(pool, achievement_key).await?;

    if let Some(achievement) = achievement {
        if current_value >= achievement.threshold {
            // Unlock the achievement
            queries::unlock_achievement(pool, user_id, achievement.id).await?;
            return Ok(vec![achievement]);
        }
    }

    Ok(Vec::new())
}

/// Helper function to check special (non-threshold) achievements
async fn check_special_achievement(
    pool: &sqlx::SqlitePool,
    user_id: Option<&str>,
    achievement_key: &str,
) -> Result<Vec<Achievement>> {
    // Check if already unlocked
    if queries::is_achievement_unlocked(pool, user_id, achievement_key).await? {
        return Ok(Vec::new());
    }

    // Get achievement details
    let achievement = queries::get_achievement_by_key(pool, achievement_key).await?;

    if let Some(achievement) = achievement {
        // Unlock the achievement
        queries::unlock_achievement(pool, user_id, achievement.id).await?;
        return Ok(vec![achievement]);
    }

    Ok(Vec::new())
}

/// Send notification for newly unlocked achievement
async fn send_achievement_notification(state: &AppState, achievement: &Achievement) {
    let title = format!("Achievement Unlocked: {}", achievement.name);
    let body = format!("{} {} - {} points!", achievement.icon, achievement.description, achievement.points);

    if let Err(e) = state
        .app_handle
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
    {
        tracing::warn!("Failed to send achievement notification: {}", e);
    }
}
