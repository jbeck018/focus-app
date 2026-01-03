// commands/analytics.rs - Analytics and productivity metrics

use crate::{
    db::queries::{self, DailyAnalytics},
    state::AppState,
    Result,
};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DailyStatsResponse {
    pub date: String,
    pub total_focus_minutes: i64,
    pub total_break_minutes: i64,
    pub sessions_completed: i64,
    pub sessions_abandoned: i64,
    pub productivity_score: f64,
}

#[derive(Debug, Serialize)]
pub struct WeeklyStatsResponse {
    pub week_start: String,
    pub week_end: String,
    pub daily_stats: Vec<DailyStatsResponse>,
    pub weekly_totals: WeeklyTotals,
}

#[derive(Debug, Serialize)]
pub struct WeeklyTotals {
    pub total_focus_minutes: i64,
    pub total_sessions: i64,
    pub average_daily_focus: f64,
    pub best_day: Option<String>,
    pub productivity_trend: f64,
}

#[derive(Debug, Serialize)]
pub struct ProductivityScore {
    pub score: f64,
    pub grade: String,
    pub trend: String,
}

/// Get daily statistics for a specific date
#[tauri::command]
pub async fn get_daily_stats(
    date: Option<String>,
    state: State<'_, AppState>,
) -> Result<DailyStatsResponse> {
    let target_date = date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    let analytics = queries::get_daily_analytics(state.pool(), &target_date)
        .await?
        .unwrap_or_else(|| DailyAnalytics {
            date: target_date.clone(),
            total_focus_seconds: 0,
            total_break_seconds: 0,
            sessions_completed: 0,
            sessions_abandoned: 0,
            productivity_score: None,
        });

    let productivity_score = calculate_productivity_score(
        analytics.total_focus_seconds,
        analytics.sessions_completed,
        analytics.sessions_abandoned,
    );

    Ok(DailyStatsResponse {
        date: analytics.date,
        total_focus_minutes: analytics.total_focus_seconds / 60,
        total_break_minutes: analytics.total_break_seconds / 60,
        sessions_completed: analytics.sessions_completed,
        sessions_abandoned: analytics.sessions_abandoned,
        productivity_score,
    })
}

/// Get weekly statistics for the past 7 days
#[tauri::command]
pub async fn get_weekly_stats(
    state: State<'_, AppState>,
) -> Result<WeeklyStatsResponse> {
    let end_date = chrono::Utc::now();
    let start_date = end_date - chrono::Duration::days(6);

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();

    let analytics = queries::get_analytics_range(
        state.pool(),
        &start_str,
        &end_str,
    )
    .await?;

    let mut daily_stats = Vec::new();
    let mut total_focus_minutes = 0i64;
    let mut total_sessions = 0i64;
    let mut best_day: Option<(String, i64)> = None;

    for day_analytics in analytics {
        let focus_minutes = day_analytics.total_focus_seconds / 60;
        let productivity_score = calculate_productivity_score(
            day_analytics.total_focus_seconds,
            day_analytics.sessions_completed,
            day_analytics.sessions_abandoned,
        );

        // Track best day
        if let Some((_, best_focus)) = &best_day {
            if focus_minutes > *best_focus {
                best_day = Some((day_analytics.date.clone(), focus_minutes));
            }
        } else {
            best_day = Some((day_analytics.date.clone(), focus_minutes));
        }

        total_focus_minutes += focus_minutes;
        total_sessions += day_analytics.sessions_completed + day_analytics.sessions_abandoned;

        daily_stats.push(DailyStatsResponse {
            date: day_analytics.date,
            total_focus_minutes: focus_minutes,
            total_break_minutes: day_analytics.total_break_seconds / 60,
            sessions_completed: day_analytics.sessions_completed,
            sessions_abandoned: day_analytics.sessions_abandoned,
            productivity_score,
        });
    }

    let average_daily_focus = if !daily_stats.is_empty() {
        total_focus_minutes as f64 / daily_stats.len() as f64
    } else {
        0.0
    };

    // Calculate productivity trend (simple: compare first half to second half)
    let productivity_trend = calculate_trend(&daily_stats);

    Ok(WeeklyStatsResponse {
        week_start: start_str,
        week_end: end_str,
        daily_stats,
        weekly_totals: WeeklyTotals {
            total_focus_minutes,
            total_sessions,
            average_daily_focus,
            best_day: best_day.map(|(date, _)| date),
            productivity_trend,
        },
    })
}

/// Get analytics data for a date range
#[tauri::command]
pub async fn get_date_range_stats(
    start_date: String,
    end_date: String,
    state: State<'_, AppState>,
) -> Result<Vec<DailyStatsResponse>> {
    let analytics = queries::get_analytics_range(
        state.pool(),
        &start_date,
        &end_date,
    )
    .await?;

    let daily_stats: Vec<DailyStatsResponse> = analytics
        .into_iter()
        .map(|day_analytics| {
            let productivity_score = calculate_productivity_score(
                day_analytics.total_focus_seconds,
                day_analytics.sessions_completed,
                day_analytics.sessions_abandoned,
            );

            DailyStatsResponse {
                date: day_analytics.date,
                total_focus_minutes: day_analytics.total_focus_seconds / 60,
                total_break_minutes: day_analytics.total_break_seconds / 60,
                sessions_completed: day_analytics.sessions_completed,
                sessions_abandoned: day_analytics.sessions_abandoned,
                productivity_score,
            }
        })
        .collect();

    Ok(daily_stats)
}

/// Get current productivity score with grade
#[tauri::command]
pub async fn get_productivity_score(
    state: State<'_, AppState>,
) -> Result<ProductivityScore> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let analytics = queries::get_daily_analytics(state.pool(), &today)
        .await?
        .unwrap_or_else(|| DailyAnalytics {
            date: today,
            total_focus_seconds: 0,
            total_break_seconds: 0,
            sessions_completed: 0,
            sessions_abandoned: 0,
            productivity_score: None,
        });

    let score = calculate_productivity_score(
        analytics.total_focus_seconds,
        analytics.sessions_completed,
        analytics.sessions_abandoned,
    );

    let grade = match score {
        s if s >= 90.0 => "A+",
        s if s >= 80.0 => "A",
        s if s >= 70.0 => "B",
        s if s >= 60.0 => "C",
        s if s >= 50.0 => "D",
        _ => "F",
    };

    // Get yesterday's score for trend
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let yesterday_analytics = queries::get_daily_analytics(state.pool(), &yesterday).await?;

    let trend = if let Some(prev) = yesterday_analytics {
        let prev_score = calculate_productivity_score(
            prev.total_focus_seconds,
            prev.sessions_completed,
            prev.sessions_abandoned,
        );

        if score > prev_score + 5.0 {
            "improving"
        } else if score < prev_score - 5.0 {
            "declining"
        } else {
            "stable"
        }
    } else {
        "stable"
    };

    Ok(ProductivityScore {
        score,
        grade: grade.to_string(),
        trend: trend.to_string(),
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate productivity score (0-100) based on focus time and session completion
fn calculate_productivity_score(
    focus_seconds: i64,
    completed: i64,
    abandoned: i64,
) -> f64 {
    let total_sessions = completed + abandoned;
    if total_sessions == 0 {
        return 0.0;
    }

    // Component 1: Completion rate (0-50 points)
    let completion_rate = completed as f64 / total_sessions as f64;
    let completion_score = completion_rate * 50.0;

    // Component 2: Focus duration (0-50 points)
    // Target: 4 hours (14400 seconds) = 50 points
    let duration_score = (focus_seconds as f64 / 14400.0 * 50.0).min(50.0);

    (completion_score + duration_score).min(100.0)
}

/// Calculate productivity trend (positive = improving, negative = declining)
fn calculate_trend(daily_stats: &[DailyStatsResponse]) -> f64 {
    if daily_stats.len() < 2 {
        return 0.0;
    }

    let mid = daily_stats.len() / 2;
    let first_half: f64 = daily_stats[..mid]
        .iter()
        .map(|s| s.productivity_score)
        .sum::<f64>()
        / mid as f64;

    let second_half: f64 = daily_stats[mid..]
        .iter()
        .map(|s| s.productivity_score)
        .sum::<f64>()
        / (daily_stats.len() - mid) as f64;

    second_half - first_half
}
