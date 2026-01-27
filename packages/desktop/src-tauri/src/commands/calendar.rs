// commands/calendar.rs - Calendar integration commands with OAuth

use crate::oauth::provider::OAuthProvider;
use crate::oauth::{generate_state, Pkce};
use crate::{AppState, Error, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Supported calendar providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarProvider {
    Google,
    Microsoft,
}

impl CalendarProvider {
    fn as_str(&self) -> &'static str {
        match self {
            CalendarProvider::Google => "google",
            CalendarProvider::Microsoft => "microsoft",
        }
    }
}

/// Calendar connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConnection {
    pub provider: CalendarProvider,
    pub connected: bool,
    pub email: Option<String>,
    pub last_sync: Option<String>,
}

/// Calendar event from external calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub is_all_day: bool,
    pub is_busy: bool,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub html_link: Option<String>,
}

/// Focus block suggestion based on free time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusBlockSuggestion {
    pub start_time: String,
    pub end_time: String,
    pub duration_minutes: i32,
    pub reason: String,
}

/// Meeting load statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingLoad {
    pub total_meeting_hours_this_week: f32,
    pub average_daily_meetings: f32,
    pub busiest_day: Option<String>,
    pub longest_free_block_minutes: i32,
}

/// OAuth authorization URL response
#[derive(Debug, Serialize)]
pub struct AuthorizationUrl {
    pub url: String,
    pub state: String,
}

/// Get calendar connection status for all providers
#[tauri::command]
pub async fn get_calendar_connections(
    state: State<'_, AppState>,
) -> Result<Vec<CalendarConnection>> {
    let mut connections = Vec::new();

    // Check Google connection
    let google_connected = state.token_manager.has_token("google").await;
    let google_email = if google_connected {
        match state.token_manager.get_token("google").await {
            Ok(token) => token.user_email,
            Err(_) => None,
        }
    } else {
        None
    };

    connections.push(CalendarConnection {
        provider: CalendarProvider::Google,
        connected: google_connected,
        email: google_email,
        last_sync: None,
    });

    // Check Microsoft connection
    let microsoft_connected = state.token_manager.has_token("microsoft").await;
    let microsoft_email = if microsoft_connected {
        match state.token_manager.get_token("microsoft").await {
            Ok(token) => token.user_email,
            Err(_) => None,
        }
    } else {
        None
    };

    connections.push(CalendarConnection {
        provider: CalendarProvider::Microsoft,
        connected: microsoft_connected,
        email: microsoft_email,
        last_sync: None,
    });

    Ok(connections)
}

/// Check if a provider's OAuth is properly configured
fn is_oauth_configured(client_id: &str) -> bool {
    !client_id.is_empty()
        && !client_id.starts_with("MISSING_")
        && client_id != "YOUR_GOOGLE_CLIENT_ID"
        && client_id != "YOUR_MICROSOFT_CLIENT_ID"
}

/// Get OAuth configuration status for all providers
#[tauri::command]
pub async fn get_oauth_config_status(
    _state: State<'_, AppState>,
) -> Result<OAuthConfigStatus> {
    // Check Google OAuth config by examining the client_id used in the provider
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let microsoft_client_id = std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default();

    Ok(OAuthConfigStatus {
        google_configured: is_oauth_configured(&google_client_id),
        microsoft_configured: is_oauth_configured(&microsoft_client_id),
        google_setup_url: "https://console.cloud.google.com/apis/credentials".to_string(),
        microsoft_setup_url: "https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps".to_string(),
    })
}

/// OAuth configuration status response
#[derive(Debug, Serialize)]
pub struct OAuthConfigStatus {
    pub google_configured: bool,
    pub microsoft_configured: bool,
    pub google_setup_url: String,
    pub microsoft_setup_url: String,
}

/// Start OAuth flow for a calendar provider
/// Returns an error with setup instructions if OAuth is not configured.
#[tauri::command]
pub async fn start_calendar_oauth(
    state: State<'_, AppState>,
    provider: CalendarProvider,
) -> Result<AuthorizationUrl> {
    // Validate OAuth configuration before starting the flow
    let client_id_env = match provider {
        CalendarProvider::Google => std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
        CalendarProvider::Microsoft => std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
    };

    if !is_oauth_configured(&client_id_env) {
        let (provider_name, setup_url, env_var) = match provider {
            CalendarProvider::Google => (
                "Google Calendar",
                "https://console.cloud.google.com/apis/credentials",
                "GOOGLE_CLIENT_ID",
            ),
            CalendarProvider::Microsoft => (
                "Microsoft Outlook",
                "https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps",
                "MICROSOFT_CLIENT_ID",
            ),
        };

        return Err(Error::OAuthNotConfigured(format!(
            "{} integration is not configured.\n\n\
            To enable {} calendar sync:\n\
            1. Visit {} to create OAuth credentials\n\
            2. Configure the redirect URI as: focusflow://oauth/callback\n\
            3. Set the {} environment variable with your client ID\n\n\
            See the OAuth setup documentation for detailed instructions.",
            provider_name, provider_name, setup_url, env_var
        )));
    }

    // Generate PKCE code verifier and challenge
    let pkce = Pkce::generate()?;
    let csrf_state = generate_state();

    // Get authorization URL from provider
    let auth_url = match provider {
        CalendarProvider::Google => {
            state.google_calendar.auth_url(&csrf_state, &pkce.code_challenge)
        }
        CalendarProvider::Microsoft => {
            state.microsoft_calendar.auth_url(&csrf_state, &pkce.code_challenge)
        }
    };

    // Store PKCE verifier for later use in code exchange
    let mut flow_state = state.oauth_flow_state.write().await;
    flow_state.insert(csrf_state.clone(), pkce);

    tracing::info!("Starting OAuth flow for provider: {:?}", provider);

    Ok(AuthorizationUrl {
        url: auth_url,
        state: csrf_state,
    })
}

/// Handle OAuth callback and exchange code for tokens
#[tauri::command]
pub async fn complete_calendar_oauth(
    state: State<'_, AppState>,
    provider: CalendarProvider,
    code: String,
    received_state: String,
) -> Result<CalendarConnection> {
    // Retrieve and remove PKCE verifier
    let pkce = {
        let mut flow_state = state.oauth_flow_state.write().await;
        flow_state.remove(&received_state)
            .ok_or_else(|| Error::Auth("Invalid OAuth state parameter - no matching flow found".into()))?
    };

    tracing::info!("Completing OAuth flow for provider: {:?}", provider);

    // Exchange authorization code for tokens
    let (token_response, user_email) = match provider {
        CalendarProvider::Google => {
            let token = state.google_calendar.exchange_code(&code, &pkce.code_verifier).await?;
            let email = state.google_calendar.get_user_email(&token.access_token).await?;
            (token, email)
        }
        CalendarProvider::Microsoft => {
            let token = state.microsoft_calendar.exchange_code(&code, &pkce.code_verifier).await?;
            let email = state.microsoft_calendar.get_user_email(&token.access_token).await?;
            (token, email)
        }
    };

    // Store tokens securely
    state.token_manager.store_token(
        provider.as_str(),
        &token_response,
        Some(user_email.clone()),
    ).await?;

    tracing::info!("Successfully connected calendar provider: {:?}", provider);

    Ok(CalendarConnection {
        provider,
        connected: true,
        email: Some(user_email),
        last_sync: Some(Utc::now().to_rfc3339()),
    })
}

/// Disconnect a calendar provider
#[tauri::command]
pub async fn disconnect_calendar(
    state: State<'_, AppState>,
    provider: CalendarProvider,
) -> Result<()> {
    state.token_manager.delete_token(provider.as_str()).await?;
    tracing::info!("Disconnected calendar provider: {:?}", provider);
    Ok(())
}

/// Get calendar events for a date range
#[tauri::command]
pub async fn get_calendar_events(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<CalendarEvent>> {
    let start_dt = DateTime::parse_from_rfc3339(&start_date)
        .map_err(|e| Error::Config(format!("Invalid start_date format: {}", e)))?
        .with_timezone(&Utc);

    let end_dt = DateTime::parse_from_rfc3339(&end_date)
        .map_err(|e| Error::Config(format!("Invalid end_date format: {}", e)))?
        .with_timezone(&Utc);

    let mut all_events = Vec::new();

    // Fetch from Google Calendar if connected
    if state.token_manager.has_token("google").await {
        match fetch_google_events(&state, start_dt, end_dt).await {
            Ok(mut events) => {
                all_events.append(&mut events);
            }
            Err(e) => {
                tracing::warn!("Failed to fetch Google Calendar events: {}", e);
            }
        }
    }

    // Fetch from Microsoft Calendar if connected
    if state.token_manager.has_token("microsoft").await {
        match fetch_microsoft_events(&state, start_dt, end_dt).await {
            Ok(mut events) => {
                all_events.append(&mut events);
            }
            Err(e) => {
                tracing::warn!("Failed to fetch Microsoft Calendar events: {}", e);
            }
        }
    }

    // Sort events by start time
    all_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    Ok(all_events)
}

/// Fetch events from Google Calendar
async fn fetch_google_events(
    state: &AppState,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<CalendarEvent>> {
    let access_token = state.token_manager.get_valid_token(
        "google",
        state.google_calendar.as_ref(),
    ).await?;

    let events = state.google_calendar.fetch_events(
        &access_token,
        start,
        end,
    ).await?;

    // Convert to command CalendarEvent format
    Ok(events.into_iter().map(|e| CalendarEvent {
        id: e.id,
        title: e.title,
        description: e.description,
        start_time: e.start_time.to_rfc3339(),
        end_time: e.end_time.to_rfc3339(),
        is_all_day: e.is_all_day,
        is_busy: e.is_busy,
        location: e.location,
        attendees: e.attendees,
        html_link: e.html_link,
    }).collect())
}

/// Fetch events from Microsoft Calendar
async fn fetch_microsoft_events(
    state: &AppState,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<CalendarEvent>> {
    let access_token = state.token_manager.get_valid_token(
        "microsoft",
        state.microsoft_calendar.as_ref(),
    ).await?;

    let events = state.microsoft_calendar.fetch_events(
        &access_token,
        start,
        end,
    ).await?;

    // Convert to command CalendarEvent format
    Ok(events.into_iter().map(|e| CalendarEvent {
        id: e.id,
        title: e.title,
        description: e.description,
        start_time: e.start_time.to_rfc3339(),
        end_time: e.end_time.to_rfc3339(),
        is_all_day: e.is_all_day,
        is_busy: e.is_busy,
        location: e.location,
        attendees: e.attendees,
        html_link: e.html_link,
    }).collect())
}

/// Get suggested focus blocks based on calendar gaps
#[tauri::command]
pub async fn get_focus_suggestions(
    state: State<'_, AppState>,
) -> Result<Vec<FocusBlockSuggestion>> {
    let now = Utc::now();
    let end_of_day = (now + Duration::hours(12)).date_naive()
        .and_hms_opt(23, 59, 59)
        .ok_or_else(|| Error::Config("Invalid time calculation".into()))?
        .and_utc();

    let events = get_calendar_events(
        state.clone(),
        now.to_rfc3339(),
        end_of_day.to_rfc3339(),
    ).await?;

    if events.is_empty() {
        return Ok(vec![FocusBlockSuggestion {
            start_time: now.to_rfc3339(),
            end_time: (now + Duration::hours(2)).to_rfc3339(),
            duration_minutes: 120,
            reason: "No meetings scheduled - great time for deep work!".into(),
        }]);
    }

    let mut suggestions = Vec::new();

    // Parse event times
    let mut busy_events: Vec<(DateTime<Utc>, DateTime<Utc>)> = events
        .iter()
        .filter(|e| e.is_busy)
        .filter_map(|e| {
            let start = DateTime::parse_from_rfc3339(&e.start_time).ok()?.with_timezone(&Utc);
            let end = DateTime::parse_from_rfc3339(&e.end_time).ok()?.with_timezone(&Utc);
            Some((start, end))
        })
        .collect();

    busy_events.sort_by(|a, b| a.0.cmp(&b.0));

    // Find gaps between meetings (minimum 30 minutes)
    let min_gap_minutes = 30;

    // Check gap from now to first event
    if let Some((first_start, _)) = busy_events.first() {
        let gap_minutes = (*first_start - now).num_minutes();
        if gap_minutes >= min_gap_minutes as i64 {
            suggestions.push(FocusBlockSuggestion {
                start_time: now.to_rfc3339(),
                end_time: (*first_start - Duration::minutes(5)).to_rfc3339(),
                duration_minutes: (gap_minutes - 5) as i32,
                reason: "Free time before your next meeting".to_string(),
            });
        }
    }

    // Find gaps between consecutive events
    for window in busy_events.windows(2) {
        let (_, prev_end) = window[0];
        let (next_start, _) = window[1];

        let gap_minutes = (next_start - prev_end).num_minutes();
        if gap_minutes >= min_gap_minutes as i64 {
            suggestions.push(FocusBlockSuggestion {
                start_time: (prev_end + Duration::minutes(5)).to_rfc3339(),
                end_time: (next_start - Duration::minutes(5)).to_rfc3339(),
                duration_minutes: (gap_minutes - 10) as i32,
                reason: format!("{}min gap between meetings", gap_minutes),
            });
        }
    }

    // Sort by duration (longest first)
    suggestions.sort_by(|a, b| b.duration_minutes.cmp(&a.duration_minutes));

    // Return top 3 suggestions
    Ok(suggestions.into_iter().take(3).collect())
}

/// Get meeting load analysis
#[tauri::command]
pub async fn get_meeting_load(
    state: State<'_, AppState>,
) -> Result<MeetingLoad> {
    let now = Utc::now();
    let start_of_week = now.date_naive()
        .week(chrono::Weekday::Mon)
        .first_day()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| Error::Config("Invalid time calculation".into()))?
        .and_utc();
    let end_of_week = start_of_week + Duration::days(7);

    let events = get_calendar_events(
        state.clone(),
        start_of_week.to_rfc3339(),
        end_of_week.to_rfc3339(),
    ).await?;

    let mut total_meeting_minutes = 0;
    let mut meeting_count = 0;
    let mut daily_meetings: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

    for event in events.iter().filter(|e| e.is_busy) {
        if let (Ok(start), Ok(end)) = (
            DateTime::parse_from_rfc3339(&event.start_time),
            DateTime::parse_from_rfc3339(&event.end_time),
        ) {
            let duration_minutes = (end - start).num_minutes();
            total_meeting_minutes += duration_minutes;
            meeting_count += 1;

            let day = start.format("%A").to_string();
            *daily_meetings.entry(day).or_insert(0) += 1;
        }
    }

    let total_meeting_hours = total_meeting_minutes as f32 / 60.0;
    let average_daily_meetings = if meeting_count > 0 {
        meeting_count as f32 / 7.0
    } else {
        0.0
    };

    let busiest_day = daily_meetings
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(day, _)| day);

    // Calculate longest free block by analyzing gaps between events
    let longest_free_block_minutes = if events.is_empty() {
        480 // 8 hours (full work day)
    } else {
        // Parse and sort events by start time
        let mut parsed_events: Vec<(DateTime<Utc>, DateTime<Utc>)> = events
            .iter()
            .filter(|e| e.is_busy)
            .filter_map(|e| {
                let start = DateTime::parse_from_rfc3339(&e.start_time).ok()?.with_timezone(&Utc);
                let end = DateTime::parse_from_rfc3339(&e.end_time).ok()?.with_timezone(&Utc);
                Some((start, end))
            })
            .collect();

        parsed_events.sort_by_key(|(start, _)| *start);

        if parsed_events.is_empty() {
            480 // No busy events, full 8 hour day free
        } else {
            let mut max_gap: i64 = 0;

            // Check gap from start of work day (9am) to first meeting
            let work_day_start = now.date_naive()
                .and_hms_opt(9, 0, 0)
                .map(|t| t.and_utc());

            if let Some(day_start) = work_day_start {
                if let Some((first_start, _)) = parsed_events.first() {
                    if *first_start > day_start {
                        let gap = (*first_start - day_start).num_minutes();
                        max_gap = max_gap.max(gap);
                    }
                }
            }

            // Check gaps between consecutive events
            for window in parsed_events.windows(2) {
                if let [(_, prev_end), (next_start, _)] = window {
                    if next_start > prev_end {
                        let gap = (*next_start - *prev_end).num_minutes();
                        max_gap = max_gap.max(gap);
                    }
                }
            }

            // Check gap from last meeting to end of work day (6pm)
            let work_day_end = now.date_naive()
                .and_hms_opt(18, 0, 0)
                .map(|t| t.and_utc());

            if let Some(day_end) = work_day_end {
                if let Some((_, last_end)) = parsed_events.last() {
                    if day_end > *last_end {
                        let gap = (day_end - *last_end).num_minutes();
                        max_gap = max_gap.max(gap);
                    }
                }
            }

            // Return at least 0, cap at 480 (8 hours)
            max_gap.clamp(0, 480) as i32
        }
    };

    Ok(MeetingLoad {
        total_meeting_hours_this_week: total_meeting_hours,
        average_daily_meetings,
        busiest_day,
        longest_free_block_minutes,
    })
}
