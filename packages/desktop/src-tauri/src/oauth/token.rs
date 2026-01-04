// oauth/token.rs - Token manager for secure storage and automatic refresh

#![allow(clippy::type_complexity)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::oauth::provider::{OAuthProvider, TokenResponse};
use crate::{Error, Result};

/// Stored OAuth token with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub id: String,
    pub provider: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64, // Unix timestamp
    pub scopes: Option<String>,
    pub user_email: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl StoredToken {
    /// Check if token is expired or will expire within buffer_seconds
    pub fn is_expired(&self, buffer_seconds: i64) -> bool {
        let now = Utc::now().timestamp();
        self.expires_at <= (now + buffer_seconds)
    }

    /// Check if token needs refresh (5 minutes buffer)
    pub fn needs_refresh(&self) -> bool {
        self.is_expired(300) // 5 minutes
    }
}

/// Token manager for secure storage and automatic refresh
///
/// Handles:
/// - Storing tokens in SQLite database
/// - Automatic token refresh before expiry
/// - In-memory cache for performance
/// - Thread-safe access with RwLock
pub struct TokenManager {
    pool: SqlitePool,
    /// In-memory cache: provider_name -> StoredToken
    cache: Arc<RwLock<HashMap<String, StoredToken>>>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store or update a token
    ///
    /// # Arguments
    /// * `provider_name` - Provider identifier (e.g., "google", "microsoft")
    /// * `token_response` - Token response from OAuth provider
    /// * `user_email` - User's email address (optional)
    pub async fn store_token(
        &self,
        provider_name: &str,
        token_response: &TokenResponse,
        user_email: Option<String>,
    ) -> Result<()> {
        let now = Utc::now().timestamp();
        let expires_at = now + token_response.expires_in;
        let id = uuid::Uuid::new_v4().to_string();

        // Delete any existing tokens for this provider
        sqlx::query("DELETE FROM oauth_tokens WHERE provider = ?")
            .bind(provider_name)
            .execute(&self.pool)
            .await?;

        // Insert new token
        sqlx::query(
            r#"
            INSERT INTO oauth_tokens (
                id, provider, access_token, refresh_token,
                expires_at, scopes, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(provider_name)
        .bind(&token_response.access_token)
        .bind(&token_response.refresh_token)
        .bind(expires_at)
        .bind(&token_response.scope)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Update cache
        let stored_token = StoredToken {
            id,
            provider: provider_name.to_string(),
            access_token: token_response.access_token.clone(),
            refresh_token: token_response.refresh_token.clone(),
            expires_at,
            scopes: token_response.scope.clone(),
            user_email,
            created_at: now,
            updated_at: now,
        };

        let mut cache = self.cache.write().await;
        cache.insert(provider_name.to_string(), stored_token);

        tracing::info!("Stored token for provider: {}", provider_name);

        Ok(())
    }

    /// Get a valid access token, refreshing if necessary
    ///
    /// # Arguments
    /// * `provider_name` - Provider identifier
    /// * `provider` - OAuth provider implementation for token refresh
    ///
    /// # Returns
    /// Valid access token, or error if not available or refresh failed
    pub async fn get_valid_token(
        &self,
        provider_name: &str,
        provider: &dyn OAuthProvider,
    ) -> Result<String> {
        // Check cache first
        let cached_token = {
            let cache = self.cache.read().await;
            cache.get(provider_name).cloned()
        };

        let mut token = if let Some(cached) = cached_token {
            cached
        } else {
            // Load from database
            self.load_token(provider_name).await?
        };

        // Refresh if needed
        if token.needs_refresh() {
            tracing::info!("Token needs refresh for provider: {}", provider_name);
            token = self.refresh_token_internal(provider_name, provider, &token).await?;
        }

        Ok(token.access_token)
    }

    /// Get stored token without automatic refresh
    pub async fn get_token(&self, provider_name: &str) -> Result<StoredToken> {
        // Check cache first
        let cached_token = {
            let cache = self.cache.read().await;
            cache.get(provider_name).cloned()
        };

        if let Some(token) = cached_token {
            Ok(token)
        } else {
            // Load from database
            self.load_token(provider_name).await
        }
    }

    /// Load token from database
    async fn load_token(&self, provider_name: &str) -> Result<StoredToken> {
        let row: Option<(String, String, String, Option<String>, i64, Option<String>, i64, i64)> =
            sqlx::query_as(
                r#"
                SELECT id, provider, access_token, refresh_token,
                       expires_at, scopes, created_at, updated_at
                FROM oauth_tokens
                WHERE provider = ?
                "#,
            )
            .bind(provider_name)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some((
                id,
                provider,
                access_token,
                refresh_token,
                expires_at,
                scopes,
                created_at,
                updated_at,
            )) => {
                let token = StoredToken {
                    id,
                    provider,
                    access_token,
                    refresh_token,
                    expires_at,
                    scopes,
                    user_email: None,
                    created_at,
                    updated_at,
                };

                // Update cache
                let mut cache = self.cache.write().await;
                cache.insert(provider_name.to_string(), token.clone());

                Ok(token)
            }
            None => Err(Error::Auth(format!(
                "No token found for provider: {}",
                provider_name
            ))),
        }
    }

    /// Refresh a token using the provider
    async fn refresh_token_internal(
        &self,
        provider_name: &str,
        provider: &dyn OAuthProvider,
        token: &StoredToken,
    ) -> Result<StoredToken> {
        let refresh_token = token
            .refresh_token
            .as_ref()
            .ok_or_else(|| Error::Auth("No refresh token available".to_string()))?;

        // Call provider to refresh
        let token_response = provider.refresh_token(refresh_token).await?;

        // Store refreshed token
        let now = Utc::now().timestamp();
        let expires_at = now + token_response.expires_in;

        sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET access_token = ?, refresh_token = ?, expires_at = ?,
                scopes = ?, updated_at = ?
            WHERE provider = ?
            "#,
        )
        .bind(&token_response.access_token)
        .bind(&token_response.refresh_token)
        .bind(expires_at)
        .bind(&token_response.scope)
        .bind(now)
        .bind(provider_name)
        .execute(&self.pool)
        .await?;

        let refreshed_token = StoredToken {
            id: token.id.clone(),
            provider: provider_name.to_string(),
            access_token: token_response.access_token.clone(),
            refresh_token: token_response.refresh_token.clone(),
            expires_at,
            scopes: token_response.scope.clone(),
            user_email: token.user_email.clone(),
            created_at: token.created_at,
            updated_at: now,
        };

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(provider_name.to_string(), refreshed_token.clone());

        tracing::info!("Refreshed token for provider: {}", provider_name);

        Ok(refreshed_token)
    }

    /// Delete a stored token
    pub async fn delete_token(&self, provider_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM oauth_tokens WHERE provider = ?")
            .bind(provider_name)
            .execute(&self.pool)
            .await?;

        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(provider_name);

        tracing::info!("Deleted token for provider: {}", provider_name);

        Ok(())
    }

    /// Check if a token exists for a provider
    pub async fn has_token(&self, provider_name: &str) -> bool {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if cache.contains_key(provider_name) {
                return true;
            }
        }

        // Check database
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM oauth_tokens WHERE provider = ?")
            .bind(provider_name)
            .fetch_one(&self.pool)
            .await
            .unwrap_or((0,));

        count.0 > 0
    }

    /// List all connected providers
    pub async fn list_connected_providers(&self) -> Result<Vec<String>> {
        let providers: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT provider FROM oauth_tokens")
                .fetch_all(&self.pool)
                .await?;

        Ok(providers.into_iter().map(|(p,)| p).collect())
    }

    /// Clear all cached tokens (force reload from database)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::debug!("Cleared token cache");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_token_expiry() {
        let now = Utc::now().timestamp();

        // Token that expires in 10 minutes
        let token = StoredToken {
            id: "test".to_string(),
            provider: "test".to_string(),
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: now + 600, // 10 minutes
            scopes: None,
            user_email: None,
            created_at: now,
            updated_at: now,
        };

        // Should not be expired with 5 minute buffer
        assert!(!token.is_expired(300));

        // Should be expired with 15 minute buffer
        assert!(token.is_expired(900));
    }

    #[test]
    fn test_stored_token_needs_refresh() {
        let now = Utc::now().timestamp();

        // Token that expires in 10 minutes
        let token = StoredToken {
            id: "test".to_string(),
            provider: "test".to_string(),
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: now + 600,
            scopes: None,
            user_email: None,
            created_at: now,
            updated_at: now,
        };

        assert!(!token.needs_refresh());

        // Token that expires in 2 minutes
        let token_soon = StoredToken {
            expires_at: now + 120,
            ..token
        };

        assert!(token_soon.needs_refresh());
    }
}
