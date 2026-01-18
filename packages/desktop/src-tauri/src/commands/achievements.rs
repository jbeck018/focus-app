// commands/achievements.rs - Achievement system commands
//
// Includes a 5-tier progressive celebration system based on gamification psychology:
// - Heavy celebrations for new users to establish habit (positive reinforcement)
// - Lighter celebrations as users advance (avoid overjustification effect)
// - Variable ratio reinforcement through rarity-based scaling

use crate::{
    db::queries::{self, Achievement, AchievementWithStatus, UserAchievement},
    state::AppState,
    Result,
};
use chrono::Timelike;
use serde::Serialize;
use tauri::{Emitter, State};
use tauri_plugin_notification::NotificationExt;

/// Celebration tier determines the intensity of the unlock animation
/// Tier 5 (Epic): Full-screen modal, fireworks, confetti 4s, fanfare, 6s duration
/// Tier 4 (Major): Large animated toast, confetti 2s, chime, sparkles, 5s duration
/// Tier 3 (Standard): Enhanced toast, subtle sparkle, soft ding, 4s duration
/// Tier 2 (Light): Simple toast, icon highlight, no sound, 3s duration
/// Tier 1 (Minimal): Badge indicator update only, no interruption
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CelebrationTier(pub u8);

#[allow(dead_code)]
impl CelebrationTier {
    pub const MINIMAL: Self = Self(1);
    pub const LIGHT: Self = Self(2);
    pub const STANDARD: Self = Self(3);
    pub const MAJOR: Self = Self(4);
    pub const EPIC: Self = Self(5);
}

/// User experience level based on total unlocked achievements
#[derive(Debug, Clone, Copy)]
enum UserLevel {
    New,          // 0-3 unlocks
    Beginner,     // 4-8 unlocks
    Intermediate, // 9-14 unlocks
    Advanced,     // 15-20 unlocks
    Master,       // 21+ unlocks
}

impl UserLevel {
    fn from_unlock_count(count: i64) -> Self {
        match count {
            0..=3 => Self::New,
            4..=8 => Self::Beginner,
            9..=14 => Self::Intermediate,
            15..=20 => Self::Advanced,
            _ => Self::Master,
        }
    }
}

/// Achievement rarity for celebration tier calculation
#[derive(Debug, Clone, Copy)]
enum AchievementRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl AchievementRarity {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "legendary" => Self::Legendary,
            "epic" => Self::Epic,
            "rare" => Self::Rare,
            _ => Self::Common,
        }
    }
}

/// Calculate celebration tier based on user level and achievement rarity
/// Matrix:
/// | User Level   | Common | Rare | Epic | Legendary |
/// |--------------|--------|------|------|-----------|
/// | New (0-3)    | Tier 3 | Tier 4 | Tier 5 | Tier 5 |
/// | Beginner     | Tier 2 | Tier 3 | Tier 4 | Tier 5 |
/// | Intermediate | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
/// | Advanced     | Tier 1 | Tier 1 | Tier 2 | Tier 3 |
/// | Master (21+) | Tier 1 | Tier 1 | Tier 1 | Tier 2 |
fn calculate_celebration_tier(rarity: &str, total_unlocked: i64) -> CelebrationTier {
    let user_level = UserLevel::from_unlock_count(total_unlocked);
    let achievement_rarity = AchievementRarity::from_str(rarity);

    let tier = match (user_level, achievement_rarity) {
        // New users (0-3 unlocks) - Heavy celebrations to establish habit
        (UserLevel::New, AchievementRarity::Common) => 3,
        (UserLevel::New, AchievementRarity::Rare) => 4,
        (UserLevel::New, AchievementRarity::Epic) => 5,
        (UserLevel::New, AchievementRarity::Legendary) => 5,

        // Beginner users (4-8 unlocks)
        (UserLevel::Beginner, AchievementRarity::Common) => 2,
        (UserLevel::Beginner, AchievementRarity::Rare) => 3,
        (UserLevel::Beginner, AchievementRarity::Epic) => 4,
        (UserLevel::Beginner, AchievementRarity::Legendary) => 5,

        // Intermediate users (9-14 unlocks)
        (UserLevel::Intermediate, AchievementRarity::Common) => 1,
        (UserLevel::Intermediate, AchievementRarity::Rare) => 2,
        (UserLevel::Intermediate, AchievementRarity::Epic) => 3,
        (UserLevel::Intermediate, AchievementRarity::Legendary) => 4,

        // Advanced users (15-20 unlocks)
        (UserLevel::Advanced, AchievementRarity::Common) => 1,
        (UserLevel::Advanced, AchievementRarity::Rare) => 1,
        (UserLevel::Advanced, AchievementRarity::Epic) => 2,
        (UserLevel::Advanced, AchievementRarity::Legendary) => 3,

        // Master users (21+ unlocks) - Minimal interruption, intrinsic motivation
        (UserLevel::Master, AchievementRarity::Common) => 1,
        (UserLevel::Master, AchievementRarity::Rare) => 1,
        (UserLevel::Master, AchievementRarity::Epic) => 1,
        (UserLevel::Master, AchievementRarity::Legendary) => 2,
    };

    CelebrationTier(tier)
}

/// Payload for achievement-unlocked event
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementUnlockPayload {
    pub achievement: Achievement,
    pub celebration_tier: u8,
    pub is_first_in_category: bool,
    pub total_unlocked: i64,
    pub user_level: String,
}

#[derive(Debug, Serialize)]
pub struct AchievementStatsResponse {
    pub total_achievements: i64,
    pub unlocked_count: i64,
    pub total_points: i64,
    pub completion_percentage: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementCheckResult {
    pub newly_unlocked: Vec<Achievement>,
    pub celebration_payloads: Vec<AchievementUnlockPayload>,
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

    // Get current unlock count before checking (for celebration tier calculation)
    let (_, initial_unlocked, _) = queries::get_achievement_stats(
        state.pool(),
        user_id_ref,
    ).await?;

    // Track categories that have been unlocked for "first in category" detection
    let existing_categories = queries::get_unlocked_categories(state.pool(), user_id_ref).await
        .unwrap_or_default();

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
        if !(4..22).contains(&start_hour) {
            newly_unlocked.extend(
                check_special_achievement(state.pool(), user_id_ref, "night_owl").await?,
            );
        }

        // Early Bird (5AM - 7AM)
        if (5..7).contains(&start_hour) {
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

    // Build celebration payloads and emit events
    let mut celebration_payloads = Vec::new();

    for (i, achievement) in newly_unlocked.iter().enumerate() {
        // Calculate tier based on unlock count at time of this achievement
        let unlocks_at_time = initial_unlocked + i as i64;
        let tier = calculate_celebration_tier(&achievement.rarity, unlocks_at_time);

        // Check if this is the first achievement in its category
        let is_first_in_category = !existing_categories.contains(&achievement.category);

        let user_level = match UserLevel::from_unlock_count(unlocks_at_time) {
            UserLevel::New => "new",
            UserLevel::Beginner => "beginner",
            UserLevel::Intermediate => "intermediate",
            UserLevel::Advanced => "advanced",
            UserLevel::Master => "master",
        };

        let payload = AchievementUnlockPayload {
            achievement: achievement.clone(),
            celebration_tier: tier.0,
            is_first_in_category,
            total_unlocked: unlocks_at_time + 1,
            user_level: user_level.to_string(),
        };

        celebration_payloads.push(payload.clone());

        // Emit event for each achievement (allows frontend to queue celebrations)
        if let Err(e) = state.app_handle.emit("achievement-unlocked", &payload) {
            tracing::warn!("Failed to emit achievement-unlocked event: {}", e);
        }

        // Send system notification (Tier 3+ gets notification)
        if tier.0 >= 3 {
            send_achievement_notification(&state, achievement, tier.0).await;
        }

        tracing::info!(
            "Achievement unlocked: {} (tier {}, category: {}, first_in_cat: {})",
            achievement.name,
            tier.0,
            achievement.category,
            is_first_in_category
        );
    }

    Ok(AchievementCheckResult {
        newly_unlocked,
        celebration_payloads,
    })
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
/// Tier 5: Extra celebratory language
/// Tier 4: Enthusiastic
/// Tier 3: Standard congratulations
async fn send_achievement_notification(state: &AppState, achievement: &Achievement, tier: u8) {
    let title = match tier {
        5 => format!("🎉 EPIC ACHIEVEMENT: {}", achievement.name),
        4 => format!("🌟 Achievement Unlocked: {}", achievement.name),
        _ => format!("Achievement Unlocked: {}", achievement.name),
    };

    let body = match tier {
        5 => format!(
            "{} {} - {} points! You're on fire!",
            achievement.icon, achievement.description, achievement.points
        ),
        4 => format!(
            "{} {} - {} points! Keep it up!",
            achievement.icon, achievement.description, achievement.points
        ),
        _ => format!(
            "{} {} - {} points",
            achievement.icon, achievement.description, achievement.points
        ),
    };

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
