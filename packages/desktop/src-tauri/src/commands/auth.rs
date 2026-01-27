// commands/auth.rs - Authentication commands for TrailBase integration
//
// Supports email/password authentication and Google OAuth sign-in.

use crate::{AppState, Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

/// TrailBase API configuration
const DEFAULT_TRAILBASE_URL: &str = "http://localhost:4000";

/// User authentication response from TrailBase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub created_at: String,
    #[serde(default)]
    pub subscription_tier: String,
}

/// Login request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Register request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// Current auth state for frontend
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user: Option<UserInfo>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub trailbase_url: String,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            user: None,
            access_token: None,
            refresh_token: None,
            trailbase_url: DEFAULT_TRAILBASE_URL.to_string(),
        }
    }

    /// Check if tokens need refresh
    pub fn needs_refresh(&self) -> bool {
        if let Some(ref token) = self.access_token {
            // Decode JWT and check expiration
            if let Some(exp) = decode_jwt_expiration(token) {
                let now = chrono::Utc::now().timestamp();
                // Refresh if less than 5 minutes remaining
                return exp - now < 300;
            }
        }
        true
    }
}

/// Decode JWT expiration time
fn decode_jwt_expiration(token: &str) -> Option<i64> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    claims.get("exp")?.as_i64()
}

/// Login with email and password
#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<AuthResponse> {
    let auth_state = state.auth_state.read().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/api/auth/login", auth_state.trailbase_url))
        .json(&LoginRequest { email, password })
        .send()
        .await
        .map_err(|e| Error::Network(format!("Login request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(Error::Auth(format!(
            "Login failed ({}): {}",
            status, error_text
        )));
    }

    let auth_response: AuthResponse = response
        .json()
        .await
        .map_err(|e| Error::Auth(format!("Invalid response: {}", e)))?;

    drop(auth_state);

    // Store tokens in state
    let mut auth_state = state.auth_state.write().await;
    auth_state.is_authenticated = true;
    auth_state.user = Some(auth_response.user.clone());
    auth_state.access_token = Some(auth_response.access_token.clone());
    auth_state.refresh_token = Some(auth_response.refresh_token.clone());

    // Also store tokens securely using tauri-plugin-store
    store_auth_tokens(&state, &auth_response).await?;

    // Update local user_id in database for sync
    update_local_user_id(&state, &auth_response.user.id).await?;

    tracing::info!("User logged in: {}", auth_response.user.email);

    Ok(auth_response)
}

/// Register new user
#[tauri::command]
pub async fn register(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<AuthResponse> {
    let auth_state = state.auth_state.read().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/api/auth/register", auth_state.trailbase_url))
        .json(&RegisterRequest { email, password })
        .send()
        .await
        .map_err(|e| Error::Network(format!("Registration request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(Error::Auth(format!(
            "Registration failed ({}): {}",
            status, error_text
        )));
    }

    let auth_response: AuthResponse = response
        .json()
        .await
        .map_err(|e| Error::Auth(format!("Invalid response: {}", e)))?;

    drop(auth_state);

    // Store tokens in state
    let mut auth_state = state.auth_state.write().await;
    auth_state.is_authenticated = true;
    auth_state.user = Some(auth_response.user.clone());
    auth_state.access_token = Some(auth_response.access_token.clone());
    auth_state.refresh_token = Some(auth_response.refresh_token.clone());

    // Store tokens securely
    store_auth_tokens(&state, &auth_response).await?;

    // Update local user_id
    update_local_user_id(&state, &auth_response.user.id).await?;

    tracing::info!("User registered: {}", auth_response.user.email);

    Ok(auth_response)
}

/// Logout current user
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<()> {
    let mut auth_state = state.auth_state.write().await;

    // Call TrailBase logout endpoint if we have a token
    if let Some(ref token) = auth_state.access_token {
        let client = reqwest::Client::new();
        let _ = client
            .post(format!("{}/api/auth/logout", auth_state.trailbase_url))
            .bearer_auth(token)
            .send()
            .await;
    }

    // Clear local state
    auth_state.is_authenticated = false;
    auth_state.user = None;
    auth_state.access_token = None;
    auth_state.refresh_token = None;

    // Clear stored tokens
    clear_auth_tokens(&state).await?;

    tracing::info!("User logged out");

    Ok(())
}

/// Get current auth state
#[tauri::command]
pub async fn get_auth_state(state: State<'_, AppState>) -> Result<AuthState> {
    let auth_state = state.auth_state.read().await;
    Ok(auth_state.clone())
}

/// Refresh access token
#[tauri::command]
pub async fn refresh_token(state: State<'_, AppState>) -> Result<AuthResponse> {
    let auth_state = state.auth_state.read().await;

    let refresh_token = auth_state
        .refresh_token
        .clone()
        .ok_or_else(|| Error::Auth("No refresh token available".into()))?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/refresh", auth_state.trailbase_url))
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .map_err(|e| Error::Network(format!("Token refresh failed: {}", e)))?;

    if !response.status().is_success() {
        drop(auth_state);
        // Token refresh failed, logout user
        let mut auth_state = state.auth_state.write().await;
        auth_state.is_authenticated = false;
        auth_state.user = None;
        auth_state.access_token = None;
        auth_state.refresh_token = None;
        return Err(Error::Auth("Session expired, please login again".into()));
    }

    let auth_response: AuthResponse = response
        .json()
        .await
        .map_err(|e| Error::Auth(format!("Invalid response: {}", e)))?;

    drop(auth_state);

    // Update tokens
    let mut auth_state = state.auth_state.write().await;
    auth_state.access_token = Some(auth_response.access_token.clone());
    auth_state.refresh_token = Some(auth_response.refresh_token.clone());

    // Store updated tokens
    store_auth_tokens(&state, &auth_response).await?;

    Ok(auth_response)
}

/// Set TrailBase server URL
#[tauri::command]
pub async fn set_trailbase_url(state: State<'_, AppState>, url: String) -> Result<()> {
    let mut auth_state = state.auth_state.write().await;
    auth_state.trailbase_url = url;
    Ok(())
}

/// Check if user is authenticated
#[tauri::command]
pub async fn is_authenticated(state: State<'_, AppState>) -> Result<bool> {
    let auth_state = state.auth_state.read().await;
    Ok(auth_state.is_authenticated)
}

/// Get current user info
#[tauri::command]
pub async fn get_current_user(state: State<'_, AppState>) -> Result<Option<UserInfo>> {
    let auth_state = state.auth_state.read().await;
    Ok(auth_state.user.clone())
}

/// Set subscription tier (DEV ONLY - for local testing)
/// This allows testing Pro/Team features without actual payment
#[cfg(debug_assertions)]
#[tauri::command]
pub async fn dev_set_subscription_tier(
    state: State<'_, AppState>,
    tier: String,
) -> Result<()> {
    let mut auth_state = state.auth_state.write().await;

    // Create a mock user if none exists
    if auth_state.user.is_none() {
        auth_state.user = Some(UserInfo {
            id: "dev-user".to_string(),
            email: "dev@localhost".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            subscription_tier: tier.clone(),
        });
        auth_state.is_authenticated = true;
    } else if let Some(ref mut user) = auth_state.user {
        user.subscription_tier = tier.clone();
    }

    tracing::info!("[Dev] Subscription tier set to: {}", tier);
    Ok(())
}

// Helper functions

async fn store_auth_tokens(state: &State<'_, AppState>, auth: &AuthResponse) -> Result<()> {
    // Store tokens in user_settings table
    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES
         ('auth_access_token', ?, CURRENT_TIMESTAMP),
         ('auth_refresh_token', ?, CURRENT_TIMESTAMP),
         ('auth_user_id', ?, CURRENT_TIMESTAMP),
         ('auth_user_email', ?, CURRENT_TIMESTAMP)"
    )
    .bind(&auth.access_token)
    .bind(&auth.refresh_token)
    .bind(&auth.user.id)
    .bind(&auth.user.email)
    .execute(state.pool())
    .await?;

    Ok(())
}

async fn clear_auth_tokens(state: &State<'_, AppState>) -> Result<()> {
    sqlx::query(
        "DELETE FROM user_settings WHERE key IN
         ('auth_access_token', 'auth_refresh_token', 'auth_user_id', 'auth_user_email')"
    )
    .execute(state.pool())
    .await?;

    Ok(())
}

async fn update_local_user_id(state: &State<'_, AppState>, user_id: &str) -> Result<()> {
    // Update all local data with user_id for sync
    sqlx::query("UPDATE sessions SET user_id = ? WHERE user_id IS NULL")
        .bind(user_id)
        .execute(state.pool())
        .await?;

    sqlx::query("UPDATE blocked_items SET user_id = ? WHERE user_id IS NULL")
        .bind(user_id)
        .execute(state.pool())
        .await?;

    sqlx::query("UPDATE daily_analytics SET user_id = ? WHERE user_id IS NULL")
        .bind(user_id)
        .execute(state.pool())
        .await?;

    Ok(())
}

/// Restore auth state from stored tokens on app startup
pub async fn restore_auth_state(state: &AppState) -> Result<()> {
    // Check for stored tokens
    let result: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT
            (SELECT value FROM user_settings WHERE key = 'auth_access_token'),
            (SELECT value FROM user_settings WHERE key = 'auth_refresh_token'),
            (SELECT value FROM user_settings WHERE key = 'auth_user_id'),
            (SELECT value FROM user_settings WHERE key = 'auth_user_email')"
    )
    .fetch_optional(state.pool())
    .await?;

    if let Some((access, refresh, user_id, email)) = result {
        if !access.is_empty() && !refresh.is_empty() {
            let mut auth_state = state.auth_state.write().await;
            auth_state.is_authenticated = true;
            auth_state.access_token = Some(access);
            auth_state.refresh_token = Some(refresh);
            auth_state.user = Some(UserInfo {
                id: user_id,
                email,
                created_at: String::new(),
                subscription_tier: "free".to_string(),
            });
            tracing::info!("Restored auth state from stored tokens");
        }
    }

    Ok(())
}

// ============================================================================
// Google OAuth Sign-In
// ============================================================================

/// Google OAuth configuration
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const GOOGLE_REDIRECT_URI: &str = "focusflow://oauth/auth-callback";

/// Pending OAuth state for PKCE verification
#[derive(Debug, Clone)]
pub struct PendingOAuthState {
    pub state: String,
    pub code_verifier: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Google OAuth response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleOAuthResponse {
    pub auth_url: String,
    pub state: String,
}

/// Google user info
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// Generate PKCE code verifier (random 43-128 char string)
fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate PKCE code challenge from verifier (SHA256 + base64url)
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate random state parameter
fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Start Google OAuth flow - returns auth URL to open in browser
///
/// Uses PKCE (Proof Key for Code Exchange) for security in desktop apps.
/// Returns an error if OAuth is not properly configured.
#[tauri::command]
pub async fn start_google_oauth(
    state: State<'_, AppState>,
) -> Result<GoogleOAuthResponse> {
    // Get and validate client ID from credentials store
    let client_id_opt = get_google_client_id(&state).await;
    let client_id = validate_google_client_id(&client_id_opt)?;

    // Generate PKCE values
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let oauth_state = generate_state();

    // Build auth URL
    let auth_url = format!(
        "{}?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope={}&\
        state={}&\
        access_type=offline&\
        prompt=consent&\
        code_challenge={}&\
        code_challenge_method=S256",
        GOOGLE_AUTH_URL,
        urlencoding::encode(&client_id),
        urlencoding::encode(GOOGLE_REDIRECT_URI),
        urlencoding::encode("openid email profile"),
        urlencoding::encode(&oauth_state),
        urlencoding::encode(&code_challenge)
    );

    // Store pending OAuth state for verification
    {
        let mut pending = state.pending_oauth.write().await;
        *pending = Some(PendingOAuthState {
            state: oauth_state.clone(),
            code_verifier,
            created_at: chrono::Utc::now(),
        });
    }

    tracing::info!("Started Google OAuth flow");

    Ok(GoogleOAuthResponse {
        auth_url,
        state: oauth_state,
    })
}

/// Complete Google OAuth flow - exchange code for tokens
///
/// Called after user is redirected back from Google with auth code.
#[tauri::command]
pub async fn complete_google_oauth(
    code: String,
    received_state: String,
    state: State<'_, AppState>,
) -> Result<AuthResponse> {
    // Verify state and get code verifier
    let code_verifier = {
        let pending = state.pending_oauth.read().await;
        let pending_state = pending.as_ref().ok_or_else(|| {
            Error::Auth("No pending OAuth state found".into())
        })?;

        // Verify state matches
        if pending_state.state != received_state {
            return Err(Error::Auth("OAuth state mismatch - possible CSRF attack".into()));
        }

        // Check if state hasn't expired (10 minute limit)
        let elapsed = chrono::Utc::now() - pending_state.created_at;
        if elapsed.num_minutes() > 10 {
            return Err(Error::Auth("OAuth state expired".into()));
        }

        pending_state.code_verifier.clone()
    };

    // Clear pending state
    {
        let mut pending = state.pending_oauth.write().await;
        *pending = None;
    }

    // Exchange code for Google tokens - validate client ID is configured
    let client_id_opt = get_google_client_id(&state).await;
    let client_id = validate_google_client_id(&client_id_opt)?;
    let client = reqwest::Client::new();

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
        id_token: String,
        #[allow(dead_code)]
        refresh_token: Option<String>,
        #[allow(dead_code)]
        expires_in: i64,
    }

    let token_response = client
        .post(GOOGLE_TOKEN_URL)
        .json(&TokenRequest {
            client_id: client_id.clone(),
            code,
            code_verifier,
            grant_type: "authorization_code".to_string(),
            redirect_uri: GOOGLE_REDIRECT_URI.to_string(),
        })
        .send()
        .await
        .map_err(|e| Error::Network(format!("Token exchange failed: {}", e)))?;

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_default();
        return Err(Error::Auth(format!("Google token exchange failed: {}", error_text)));
    }

    let google_tokens: GoogleTokenResponse = token_response
        .json()
        .await
        .map_err(|e| Error::Auth(format!("Invalid Google token response: {}", e)))?;

    // Get user info from Google
    let userinfo_response = client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(&google_tokens.access_token)
        .send()
        .await
        .map_err(|e| Error::Network(format!("Failed to get user info: {}", e)))?;

    if !userinfo_response.status().is_success() {
        let error_text = userinfo_response.text().await.unwrap_or_default();
        return Err(Error::Auth(format!("Failed to get Google user info: {}", error_text)));
    }

    let google_user: GoogleUserInfo = userinfo_response
        .json()
        .await
        .map_err(|e| Error::Auth(format!("Invalid Google user info: {}", e)))?;

    tracing::info!("Google OAuth successful for: {}", google_user.email);

    // Exchange Google ID token with TrailBase for our JWT tokens
    let auth_state = state.auth_state.read().await;
    let trailbase_url = auth_state.trailbase_url.clone();
    drop(auth_state);

    #[derive(Serialize)]
    struct TrailBaseOAuthRequest {
        provider: String,
        id_token: String,
        email: String,
        provider_user_id: String,
    }

    let trailbase_response = client
        .post(format!("{}/api/auth/oauth/google", trailbase_url))
        .json(&TrailBaseOAuthRequest {
            provider: "google".to_string(),
            id_token: google_tokens.id_token,
            email: google_user.email.clone(),
            provider_user_id: google_user.id,
        })
        .send()
        .await
        .map_err(|e| Error::Network(format!("TrailBase OAuth request failed: {}", e)))?;

    if !trailbase_response.status().is_success() {
        let status = trailbase_response.status();
        let error_text = trailbase_response.text().await.unwrap_or_default();
        return Err(Error::Auth(format!(
            "TrailBase OAuth failed ({}): {}",
            status, error_text
        )));
    }

    let auth_response: AuthResponse = trailbase_response
        .json()
        .await
        .map_err(|e| Error::Auth(format!("Invalid TrailBase response: {}", e)))?;

    // Store tokens in state
    {
        let mut auth_state = state.auth_state.write().await;
        auth_state.is_authenticated = true;
        auth_state.user = Some(auth_response.user.clone());
        auth_state.access_token = Some(auth_response.access_token.clone());
        auth_state.refresh_token = Some(auth_response.refresh_token.clone());
    }

    // Store tokens securely
    let state_ref = (*state).clone();
    store_auth_tokens_internal(&state_ref, &auth_response).await?;

    // Update local user_id
    update_local_user_id_internal(&state_ref, &auth_response.user.id).await?;

    tracing::info!("User authenticated via Google: {}", auth_response.user.email);

    Ok(auth_response)
}

/// Get Google client ID from credentials or fallback
async fn get_google_client_id(_state: &State<'_, AppState>) -> Option<String> {
    // Try environment variable first
    if let Ok(client_id) = std::env::var("GOOGLE_CLIENT_ID") {
        if !client_id.is_empty() && client_id != "YOUR_GOOGLE_CLIENT_ID" {
            return Some(client_id);
        }
    }

    // Try to get from credentials store
    if let Ok(Some(client_id)) = crate::commands::credentials::get_api_key_internal(
        "google_oauth_client_id",
    ).await {
        if !client_id.is_empty() {
            return Some(client_id);
        }
    }

    // Return None to indicate OAuth is not configured
    None
}

/// Validate that Google OAuth client ID is configured
fn validate_google_client_id(client_id: &Option<String>) -> Result<String> {
    match client_id {
        Some(id) if !id.is_empty() && !id.starts_with("MISSING_") => Ok(id.clone()),
        _ => Err(Error::OAuthNotConfigured(
            "Google OAuth is not configured. To enable Google sign-in:\n\
            1. Create a Google Cloud project at https://console.cloud.google.com\n\
            2. Enable the Google+ API and create OAuth 2.0 credentials\n\
            3. Set the GOOGLE_CLIENT_ID environment variable, or\n\
            4. Store the client ID in the app's credential store".to_string()
        ))
    }
}

// Internal helpers that don't require State wrapper
async fn store_auth_tokens_internal(state: &AppState, auth: &AuthResponse) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO user_settings (key, value, updated_at) VALUES
         ('auth_access_token', ?, CURRENT_TIMESTAMP),
         ('auth_refresh_token', ?, CURRENT_TIMESTAMP),
         ('auth_user_id', ?, CURRENT_TIMESTAMP),
         ('auth_user_email', ?, CURRENT_TIMESTAMP)"
    )
    .bind(&auth.access_token)
    .bind(&auth.refresh_token)
    .bind(&auth.user.id)
    .bind(&auth.user.email)
    .execute(state.pool())
    .await?;

    Ok(())
}

async fn update_local_user_id_internal(state: &AppState, user_id: &str) -> Result<()> {
    sqlx::query("UPDATE sessions SET user_id = ? WHERE user_id IS NULL")
        .bind(user_id)
        .execute(state.pool())
        .await?;

    sqlx::query("UPDATE blocked_items SET user_id = ? WHERE user_id IS NULL")
        .bind(user_id)
        .execute(state.pool())
        .await?;

    sqlx::query("UPDATE daily_analytics SET user_id = ? WHERE user_id IS NULL")
        .bind(user_id)
        .execute(state.pool())
        .await?;

    Ok(())
}
