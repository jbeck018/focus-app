// oauth/provider.rs - OAuth provider trait and shared types

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Result;

/// OAuth token response from authorization server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64, // seconds
    pub token_type: String,
    pub scope: Option<String>,
}

impl TokenResponse {
    /// Calculate when this token will expire
    pub fn expires_at(&self) -> DateTime<Utc> {
        Utc::now() + chrono::Duration::seconds(self.expires_in)
    }

    /// Check if token is expired or will expire within buffer_seconds
    pub fn is_expired(&self, buffer_seconds: i64) -> bool {
        let expires_at = Utc::now() + chrono::Duration::seconds(self.expires_in);
        let threshold = Utc::now() + chrono::Duration::seconds(buffer_seconds);
        expires_at <= threshold
    }
}

/// Calendar event from external provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_all_day: bool,
    pub is_busy: bool,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub organizer: Option<String>,
    pub html_link: Option<String>,
}

impl CalendarEvent {
    /// Get duration of event in minutes
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    /// Check if event is happening now
    pub fn is_happening_now(&self) -> bool {
        let now = Utc::now();
        now >= self.start_time && now <= self.end_time
    }

    /// Check if event is in the future
    pub fn is_upcoming(&self) -> bool {
        self.start_time > Utc::now()
    }
}

/// OAuth provider trait for calendar integrations
///
/// Implementations must handle:
/// - PKCE flow (no client secret in desktop app)
/// - Token refresh logic
/// - API integration for fetching events
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Provider name (e.g., "google", "microsoft")
    fn provider_name(&self) -> &'static str;

    /// Generate authorization URL with PKCE challenge
    ///
    /// # Arguments
    /// * `state` - CSRF protection token
    /// * `code_challenge` - PKCE code challenge (S256 hashed verifier)
    ///
    /// # Returns
    /// Authorization URL to open in browser
    fn auth_url(&self, state: &str, code_challenge: &str) -> String;

    /// Exchange authorization code for tokens
    ///
    /// # Arguments
    /// * `code` - Authorization code from OAuth callback
    /// * `code_verifier` - PKCE code verifier (original unhashed value)
    ///
    /// # Returns
    /// Token response with access token, refresh token, and expiration
    async fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<TokenResponse>;

    /// Refresh an expired access token
    ///
    /// # Arguments
    /// * `refresh_token` - Refresh token from previous token response
    ///
    /// # Returns
    /// New token response with refreshed access token
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse>;

    /// Fetch calendar events for a date range
    ///
    /// # Arguments
    /// * `access_token` - Valid access token
    /// * `start` - Start of date range
    /// * `end` - End of date range
    ///
    /// # Returns
    /// List of calendar events in the specified range
    async fn fetch_events(
        &self,
        access_token: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>>;

    /// Get user's email/identifier
    ///
    /// # Arguments
    /// * `access_token` - Valid access token
    ///
    /// # Returns
    /// User's email address or identifier
    async fn get_user_email(&self, access_token: &str) -> Result<String>;
}
