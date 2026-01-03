// commands/onboarding.rs - Tauri commands for onboarding flow

use crate::{Result, AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct OnboardingData {
    user_name: String,
    selected_apps: Vec<String>,
    selected_websites: Vec<String>,
    default_focus_duration: i32,
    default_break_duration: i32,
    enable_notifications: bool,
    auto_start_breaks: bool,
}

#[derive(Debug, Serialize)]
pub struct OnboardingCompleteResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Complete onboarding and save user preferences
///
/// This command:
/// 1. Saves user settings to the database
/// 2. Creates blocked items for selected apps/websites
/// 3. Marks onboarding as complete
#[tauri::command]
pub async fn complete_onboarding(
    state: tauri::State<'_, AppState>,
    data: OnboardingData,
) -> Result<OnboardingCompleteResponse> {
    let pool = state.pool();

    // Start a transaction for atomic operations
    let mut tx = pool.begin().await?;

    // 1. Save user name
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('user_name', ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&data.user_name)
    .execute(&mut *tx)
    .await?;

    // 2. Save default focus duration
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('default_focus_minutes', ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(data.default_focus_duration.to_string())
    .execute(&mut *tx)
    .await?;

    // 3. Save default break duration
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('default_break_minutes', ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(data.default_break_duration.to_string())
    .execute(&mut *tx)
    .await?;

    // 4. Save notification preference
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('notification_sound', ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(data.enable_notifications.to_string())
    .execute(&mut *tx)
    .await?;

    // 5. Save auto-start breaks preference
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('auto_start_breaks', ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(data.auto_start_breaks.to_string())
    .execute(&mut *tx)
    .await?;

    // 6. Insert blocked apps
    for app in &data.selected_apps {
        sqlx::query(
            r#"
            INSERT INTO blocked_items (item_type, value, enabled)
            VALUES ('app', ?, 1)
            ON CONFLICT(item_type, value) DO UPDATE SET enabled = 1
            "#,
        )
        .bind(app)
        .execute(&mut *tx)
        .await?;
    }

    // 7. Insert blocked websites
    for website in &data.selected_websites {
        sqlx::query(
            r#"
            INSERT INTO blocked_items (item_type, value, enabled)
            VALUES ('website', ?, 1)
            ON CONFLICT(item_type, value) DO UPDATE SET enabled = 1
            "#,
        )
        .bind(website)
        .execute(&mut *tx)
        .await?;
    }

    // 8. Mark onboarding as complete
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('onboarding_completed', 'true')
        ON CONFLICT(key) DO UPDATE SET value = 'true', updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // 9. Save completion timestamp
    sqlx::query(
        r#"
        INSERT INTO user_settings (key, value)
        VALUES ('onboarding_completed_at', datetime('now'))
        ON CONFLICT(key) DO UPDATE SET value = datetime('now'), updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    tracing::info!(
        "Onboarding completed for user: {}, apps: {}, websites: {}",
        data.user_name,
        data.selected_apps.len(),
        data.selected_websites.len()
    );

    Ok(OnboardingCompleteResponse {
        success: true,
        user_id: None,
        error: None,
    })
}

/// Check if onboarding has been completed
#[tauri::command]
pub async fn is_onboarding_complete(
    state: tauri::State<'_, AppState>,
) -> Result<bool> {
    let pool = state.pool();

    let result: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT value FROM user_settings
        WHERE key = 'onboarding_completed'
        "#,
    )
    .fetch_optional(pool)
    .await?;

    match result {
        Some((value,)) => Ok(value == "true"),
        None => Ok(false),
    }
}

/// Get onboarding data (for resuming interrupted onboarding)
#[tauri::command]
pub async fn get_onboarding_data(
    state: tauri::State<'_, AppState>,
) -> Result<Option<OnboardingDataResponse>> {
    let pool = state.pool();

    // Check if onboarding is complete
    let is_complete = is_onboarding_complete(state.clone()).await?;
    if is_complete {
        return Ok(None);
    }

    // Fetch existing settings if any
    let user_name: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM user_settings WHERE key = 'user_name'",
    )
    .fetch_optional(pool)
    .await?;

    let default_focus: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM user_settings WHERE key = 'default_focus_minutes'",
    )
    .fetch_optional(pool)
    .await?;

    let default_break: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM user_settings WHERE key = 'default_break_minutes'",
    )
    .fetch_optional(pool)
    .await?;

    Ok(Some(OnboardingDataResponse {
        user_name: user_name.map(|(v,)| v),
        default_focus_duration: default_focus
            .and_then(|(v,)| v.parse().ok())
            .unwrap_or(25),
        default_break_duration: default_break
            .and_then(|(v,)| v.parse().ok())
            .unwrap_or(5),
    }))
}

#[derive(Debug, Serialize)]
pub struct OnboardingDataResponse {
    user_name: Option<String>,
    default_focus_duration: i32,
    default_break_duration: i32,
}

/// Reset onboarding (for testing or re-onboarding)
#[tauri::command]
pub async fn reset_onboarding(
    state: tauri::State<'_, AppState>,
) -> Result<()> {
    let pool = state.pool();

    sqlx::query(
        r#"
        DELETE FROM user_settings
        WHERE key IN (
            'onboarding_completed',
            'onboarding_completed_at',
            'user_name'
        )
        "#,
    )
    .execute(pool)
    .await?;

    tracing::info!("Onboarding reset");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_data_serialization() {
        let data = OnboardingData {
            user_name: "Test User".to_string(),
            selected_apps: vec!["Chrome".to_string()],
            selected_websites: vec!["twitter.com".to_string()],
            default_focus_duration: 25,
            default_break_duration: 5,
            enable_notifications: true,
            auto_start_breaks: false,
        };

        assert_eq!(data.user_name, "Test User");
        assert_eq!(data.selected_apps.len(), 1);
        assert_eq!(data.default_focus_duration, 25);
    }

    #[test]
    fn test_onboarding_response_serialization() {
        let response = OnboardingCompleteResponse {
            success: true,
            user_id: Some("123".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("user_id"));
    }
}
