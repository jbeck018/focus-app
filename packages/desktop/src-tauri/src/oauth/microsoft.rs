// oauth/microsoft.rs - Microsoft Outlook Calendar OAuth provider

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::oauth::provider::{CalendarEvent, OAuthProvider, TokenResponse};
use crate::{Error, Result};

/// Microsoft Outlook Calendar OAuth provider
///
/// Uses PKCE flow for authorization code exchange.
/// Redirect URI should be registered as: focusflow://oauth/callback
pub struct MicrosoftCalendar {
    client_id: String,
    redirect_uri: String,
    http_client: reqwest::Client,
}

impl MicrosoftCalendar {
    /// Create a new Microsoft Calendar provider
    ///
    /// # Arguments
    /// * `client_id` - Microsoft Azure AD application client ID
    /// * `redirect_uri` - Redirect URI (should be focusflow://oauth/callback)
    pub fn new(client_id: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            redirect_uri,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get default Microsoft Calendar provider with standard redirect URI
    pub fn default(client_id: String) -> Self {
        Self::new(client_id, "focusflow://oauth/callback".to_string())
    }
}

#[async_trait]
impl OAuthProvider for MicrosoftCalendar {
    fn provider_name(&self) -> &'static str {
        "microsoft"
    }

    fn auth_url(&self, state: &str, code_challenge: &str) -> String {
        format!(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?\
            client_id={}&\
            redirect_uri={}&\
            response_type=code&\
            scope={}&\
            state={}&\
            code_challenge={}&\
            code_challenge_method=S256",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode("offline_access Calendars.Read User.Read"),
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
        struct MicrosoftTokenResponse {
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
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&request)
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

        let ms_response: MicrosoftTokenResponse = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse token response: {}", e)))?;

        Ok(TokenResponse {
            access_token: ms_response.access_token,
            refresh_token: ms_response.refresh_token,
            expires_in: ms_response.expires_in,
            token_type: ms_response.token_type,
            scope: ms_response.scope,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        #[derive(Serialize)]
        struct RefreshRequest {
            client_id: String,
            refresh_token: String,
            grant_type: String,
            scope: String,
        }

        #[derive(Deserialize)]
        struct MicrosoftRefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            token_type: String,
            scope: Option<String>,
        }

        let request = RefreshRequest {
            client_id: self.client_id.clone(),
            refresh_token: refresh_token.to_string(),
            grant_type: "refresh_token".to_string(),
            scope: "offline_access Calendars.Read User.Read".to_string(),
        };

        let response = self
            .http_client
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&request)
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

        let ms_response: MicrosoftRefreshResponse = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse refresh response: {}", e)))?;

        Ok(TokenResponse {
            access_token: ms_response.access_token,
            refresh_token: ms_response
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())), // Preserve original if not returned
            expires_in: ms_response.expires_in,
            token_type: ms_response.token_type,
            scope: ms_response.scope,
        })
    }

    async fn fetch_events(
        &self,
        access_token: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        #[derive(Deserialize)]
        struct MicrosoftEventList {
            value: Vec<MicrosoftEvent>,
        }

        #[derive(Deserialize)]
        struct MicrosoftEvent {
            id: String,
            subject: String,
            #[serde(rename = "bodyPreview")]
            body_preview: Option<String>,
            location: Option<MicrosoftLocation>,
            start: MicrosoftDateTime,
            end: MicrosoftDateTime,
            #[serde(rename = "isAllDay")]
            is_all_day: bool,
            #[serde(rename = "showAs")]
            show_as: String,
            #[serde(rename = "webLink")]
            web_link: Option<String>,
            organizer: Option<MicrosoftOrganizer>,
            attendees: Option<Vec<MicrosoftAttendee>>,
        }

        #[derive(Deserialize)]
        struct MicrosoftDateTime {
            #[serde(rename = "dateTime")]
            date_time: String,
            #[serde(rename = "timeZone")]
            time_zone: String,
        }

        #[derive(Deserialize)]
        struct MicrosoftLocation {
            #[serde(rename = "displayName")]
            display_name: Option<String>,
        }

        #[derive(Deserialize)]
        struct MicrosoftOrganizer {
            #[serde(rename = "emailAddress")]
            email_address: Option<MicrosoftEmailAddress>,
        }

        #[derive(Deserialize)]
        struct MicrosoftAttendee {
            #[serde(rename = "emailAddress")]
            email_address: Option<MicrosoftEmailAddress>,
        }

        #[derive(Deserialize)]
        struct MicrosoftEmailAddress {
            address: String,
        }

        let url = format!(
            "https://graph.microsoft.com/v1.0/me/calendar/calendarView?\
            startDateTime={}&\
            endDateTime={}&\
            $orderby=start/dateTime",
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

        let event_list: MicrosoftEventList = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse events response: {}", e)))?;

        let events = event_list
            .value
            .into_iter()
            .filter_map(|event| {
                // Parse start time
                let start_time = if event.start.time_zone == "UTC" {
                    DateTime::parse_from_rfc3339(&event.start.date_time)
                        .ok()?
                        .with_timezone(&Utc)
                } else {
                    // Microsoft returns timezone-aware datetime strings
                    // Try parsing with timezone, fall back to treating as UTC
                    DateTime::parse_from_rfc3339(&event.start.date_time)
                        .ok()?
                        .with_timezone(&Utc)
                };

                // Parse end time
                let end_time = if event.end.time_zone == "UTC" {
                    DateTime::parse_from_rfc3339(&event.end.date_time)
                        .ok()?
                        .with_timezone(&Utc)
                } else {
                    DateTime::parse_from_rfc3339(&event.end.date_time)
                        .ok()?
                        .with_timezone(&Utc)
                };

                // showAs values: free, tentative, busy, oof (out of office), workingElsewhere, unknown
                let is_busy = matches!(
                    event.show_as.as_str(),
                    "busy" | "oof" | "workingElsewhere" | "tentative"
                );

                Some(CalendarEvent {
                    id: event.id,
                    title: event.subject,
                    description: event.body_preview,
                    start_time,
                    end_time,
                    is_all_day: event.is_all_day,
                    is_busy,
                    location: event
                        .location
                        .and_then(|l| l.display_name),
                    attendees: event
                        .attendees
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|a| a.email_address.map(|e| e.address))
                        .collect(),
                    organizer: event
                        .organizer
                        .and_then(|o| o.email_address.map(|e| e.address)),
                    html_link: event.web_link,
                })
            })
            .collect();

        Ok(events)
    }

    async fn get_user_email(&self, access_token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct UserProfile {
            #[serde(rename = "userPrincipalName")]
            user_principal_name: String,
        }

        let response = self
            .http_client
            .get("https://graph.microsoft.com/v1.0/me")
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

        let user_profile: UserProfile = response
            .json()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse user info: {}", e)))?;

        Ok(user_profile.user_principal_name)
    }
}
