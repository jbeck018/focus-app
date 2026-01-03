// commands/auth.rs - Authentication commands for TrailBase integration

use crate::{AppState, Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
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
