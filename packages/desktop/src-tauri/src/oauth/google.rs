// oauth/google.rs - Google Calendar OAuth provider

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::oauth::provider::{CalendarEvent, OAuthProvider, TokenResponse};
use crate::{Error, Result};

/// Google Calendar OAuth provider
///
/// Uses PKCE flow for authorization code exchange.
/// Redirect URI should be registered as: focusflow://oauth/callback
pub struct GoogleCalendar {
    client_id: String,
    redirect_uri: String,
    http_client: reqwest::Client,
}

impl GoogleCalendar {
    /// Create a new Google Calendar provider
    ///
    /// # Arguments
    /// * `client_id` - Google OAuth client ID (from Google Cloud Console)
    /// * `redirect_uri` - Redirect URI (should be focusflow://oauth/callback)
    pub fn new(client_id: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            redirect_uri,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get default Google Calendar provider with standard redirect URI
    pub fn default(client_id: String) -> Self {
        Self::new(client_id, "focusflow://oauth/callback".to_string())
    }
}

#[async_trait]
impl OAuthProvider for GoogleCalendar {
    fn provider_name(&self) -> &'static str {
        "google"
    }

    fn auth_url(&self, state: &str, code_challenge: &str) -> String {
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?\
            client_id={}&\
            redirect_uri={}&\
            response_type=code&\
            scope={}&\
            state={}&\
            access_type=offline&\
            prompt=consent&\
            code_challenge={}&\
            code_challenge_method=S256",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode("https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/userinfo.email"),
            urlencoding::encode(state),
            urlencoding::encode(code_challenge)
        )
    }

    async fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<TokenResponse> {
        #[derive(Serialize)]
        struct TokenRequest {
            client_id: String,
            code: String,
            code_verifier: String,
            grant_type: String,
            redirect_uri: String,
        }

        #[derive(Deserialize)]
        struct GoogleTokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            token_type: String,
            scope: Option<String>,
        }

        let request = TokenRequest {
            client_id: self.client_id.clone(),
            code: code.to_string(),
            code_verifier: code_verifier.to_string(),
            grant_type: "authorization_code".to_string(),
            redirect_uri: self.redirect_uri.clone(),
        };

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to exchange code: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Auth(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let google_response: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse token response: {}", e)))?;

        Ok(TokenResponse {
            access_token: google_response.access_token,
            refresh_token: google_response.refresh_token,
            expires_in: google_response.expires_in,
            token_type: google_response.token_type,
            scope: google_response.scope,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        #[derive(Serialize)]
        struct RefreshRequest {
            client_id: String,
            refresh_token: String,
            grant_type: String,
        }

        #[derive(Deserialize)]
        struct GoogleRefreshResponse {
            access_token: String,
            expires_in: i64,
            token_type: String,
            scope: Option<String>,
        }

        let request = RefreshRequest {
            client_id: self.client_id.clone(),
            refresh_token: refresh_token.to_string(),
            grant_type: "refresh_token".to_string(),
        };

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to refresh token: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Auth(format!(
                "Token refresh failed: {}",
                error_text
            )));
        }

        let google_response: GoogleRefreshResponse = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse refresh response: {}", e)))?;

        Ok(TokenResponse {
            access_token: google_response.access_token,
            refresh_token: Some(refresh_token.to_string()), // Preserve original refresh token
            expires_in: google_response.expires_in,
            token_type: google_response.token_type,
            scope: google_response.scope,
        })
    }

    async fn fetch_events(
        &self,
        access_token: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        #[derive(Deserialize)]
        struct GoogleCalendarList {
            items: Vec<GoogleEvent>,
        }

        #[derive(Deserialize)]
        struct GoogleEvent {
            id: String,
            summary: Option<String>,
            description: Option<String>,
            location: Option<String>,
            start: GoogleDateTime,
            end: GoogleDateTime,
            #[serde(rename = "htmlLink")]
            html_link: Option<String>,
            organizer: Option<GoogleOrganizer>,
            attendees: Option<Vec<GoogleAttendee>>,
        }

        #[derive(Deserialize)]
        struct GoogleDateTime {
            #[serde(rename = "dateTime")]
            date_time: Option<String>,
            date: Option<String>,
        }

        #[derive(Deserialize)]
        struct GoogleOrganizer {
            email: Option<String>,
        }

        #[derive(Deserialize)]
        struct GoogleAttendee {
            email: Option<String>,
        }

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/primary/events?\
            timeMin={}&\
            timeMax={}&\
            singleEvents=true&\
            orderBy=startTime",
            urlencoding::encode(&start.to_rfc3339()),
            urlencoding::encode(&end.to_rfc3339())
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch events: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Network(format!(
                "Failed to fetch calendar events: {}",
                error_text
            )));
        }

        let calendar_list: GoogleCalendarList = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse events response: {}", e)))?;

        let events = calendar_list
            .items
            .into_iter()
            .filter_map(|event| {
                let is_all_day = event.start.date.is_some();

                let start_time = if let Some(date_time) = &event.start.date_time {
                    DateTime::parse_from_rfc3339(date_time).ok()?.with_timezone(&Utc)
                } else if let Some(date) = &event.start.date {
                    // All-day events - use date at midnight UTC
                    let naive_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
                    DateTime::from_naive_utc_and_offset(
                        naive_date.and_hms_opt(0, 0, 0)?,
                        Utc,
                    )
                } else {
                    return None;
                };

                let end_time = if let Some(date_time) = &event.end.date_time {
                    DateTime::parse_from_rfc3339(date_time).ok()?.with_timezone(&Utc)
                } else if let Some(date) = &event.end.date {
                    let naive_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
                    DateTime::from_naive_utc_and_offset(
                        naive_date.and_hms_opt(0, 0, 0)?,
                        Utc,
                    )
                } else {
                    return None;
                };

                Some(CalendarEvent {
                    id: event.id,
                    title: event.summary.unwrap_or_else(|| "(No title)".to_string()),
                    description: event.description,
                    start_time,
                    end_time,
                    is_all_day,
                    is_busy: true, // Google doesn't expose transparency directly in list
                    location: event.location,
                    attendees: event
                        .attendees
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|a| a.email)
                        .collect(),
                    organizer: event.organizer.and_then(|o| o.email),
                    html_link: event.html_link,
                })
            })
            .collect();

        Ok(events)
    }

    async fn get_user_email(&self, access_token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct UserInfo {
            email: String,
        }

        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch user info: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Auth(format!(
                "Failed to get user email: {}",
                error_text
            )));
        }

        let user_info: UserInfo = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse user info: {}", e)))?;

        Ok(user_info.email)
    }
}
