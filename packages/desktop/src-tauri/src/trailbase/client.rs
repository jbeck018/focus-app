// trailbase/client.rs - HTTP client for TrailBase API
//
// Provides authentication, request handling, and entity sync capabilities.

use crate::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

use super::sync::{SyncResult, SyncableEntity};

/// TrailBase API client
#[derive(Clone)]
pub struct TrailBaseClient {
    base_url: String,
    api_key: Option<String>,
    http_client: reqwest::Client,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

/// Authentication response from TrailBase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Team connection info stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConnection {
    pub id: String,
    pub server_url: String,
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub api_key: Option<String>,
    pub connected_at: i64,
}

impl TrailBaseClient {
    /// Create a new TrailBase client
    pub fn new(base_url: String) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| Error::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: None,
            http_client,
        })
    }

    /// Set the API key for authenticated requests
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    /// Clear the API key (logout)
    pub fn clear_api_key(&mut self) {
        self.api_key = None;
    }

    /// Check if client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Authenticate with TrailBase
    pub async fn authenticate(&mut self, credentials: Credentials) -> Result<AuthResponse> {
        let url = format!("{}/auth/login", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .json(&credentials)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Authentication request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Auth(format!(
                "Authentication failed with status {}: {}",
                status, error_text
            )));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| Error::Auth(format!("Failed to parse auth response: {}", e)))?;

        // Store the access token
        self.api_key = Some(auth_response.access_token.clone());

        tracing::info!("Successfully authenticated user: {}", auth_response.email);

        Ok(auth_response)
    }

    /// Register a new user
    pub async fn register(&mut self, credentials: Credentials) -> Result<AuthResponse> {
        let url = format!("{}/auth/register", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .json(&credentials)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Registration request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Auth(format!(
                "Registration failed with status {}: {}",
                status, error_text
            )));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| Error::Auth(format!("Failed to parse registration response: {}", e)))?;

        // Store the access token
        self.api_key = Some(auth_response.access_token.clone());

        tracing::info!("Successfully registered user: {}", auth_response.email);

        Ok(auth_response)
    }

    /// Refresh the access token
    pub async fn refresh_token(&mut self, refresh_token: String) -> Result<AuthResponse> {
        #[derive(Serialize)]
        struct RefreshRequest {
            refresh_token: String,
        }

        let url = format!("{}/auth/refresh", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .json(&RefreshRequest { refresh_token })
            .send()
            .await
            .map_err(|e| Error::Network(format!("Token refresh request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Auth("Token refresh failed".to_string()));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| Error::Auth(format!("Failed to parse refresh response: {}", e)))?;

        // Update the access token
        self.api_key = Some(auth_response.access_token.clone());

        Ok(auth_response)
    }

    /// Perform a GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let headers = self.build_headers()?;

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::Network(format!("GET request failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Perform a POST request
    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, endpoint);
        let headers = self.build_headers()?;

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("POST request failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Perform a PUT request
    pub async fn put<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, endpoint);
        let headers = self.build_headers()?;

        let response = self
            .http_client
            .put(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("PUT request failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Perform a DELETE request
    pub async fn delete(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, endpoint);
        let headers = self.build_headers()?;

        let response = self
            .http_client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::Network(format!("DELETE request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Network(format!(
                "DELETE request failed with status {}: {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Sync an entity with the backend
    pub async fn sync_entity<E: SyncableEntity>(&self, entity: &E) -> Result<SyncResult> {
        let entity_type = E::entity_type();
        let local_id = entity.local_id();
        let remote_id = entity.remote_id();

        // Determine if this is a create or update
        let endpoint = if let Some(remote_id) = remote_id {
            // Update existing entity
            format!("/api/{}/{}", entity_type, remote_id)
        } else {
            // Create new entity
            format!("/api/{}", entity_type)
        };

        let response = if remote_id.is_some() {
            // PUT for update
            self.put::<E, E>(&endpoint, entity).await
        } else {
            // POST for create
            self.post::<E, E>(&endpoint, entity).await
        };

        match response {
            Ok(remote_entity) => {
                let new_remote_id = remote_entity
                    .remote_id()
                    .ok_or_else(|| Error::Sync("Remote entity missing ID".to_string()))?
                    .to_string();

                tracing::info!(
                    "Successfully synced {} entity: {} -> {}",
                    entity_type,
                    local_id,
                    new_remote_id
                );

                Ok(SyncResult::Success {
                    local_id: local_id.to_string(),
                    remote_id: new_remote_id,
                })
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to sync {} entity {}: {}",
                    entity_type,
                    local_id,
                    e
                );

                Ok(SyncResult::Failed {
                    local_id: local_id.to_string(),
                    error: e.to_string(),
                })
            }
        }
    }

    /// Fetch remote changes for an entity type
    pub async fn fetch_remote_changes<E: SyncableEntity>(
        &self,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<E>> {
        let entity_type = E::entity_type();
        let mut endpoint = format!("/api/{}", entity_type);

        if let Some(timestamp) = since {
            endpoint.push_str(&format!("?since={}", timestamp.to_rfc3339()));
        }

        let response: Vec<E> = self.get(&endpoint).await?;

        tracing::info!(
            "Fetched {} remote {} entities",
            response.len(),
            entity_type
        );

        Ok(response)
    }

    /// Build headers for authenticated requests
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(ref api_key) = self.api_key {
            let auth_value = HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| Error::Auth(format!("Invalid API key: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_value);
        }

        Ok(headers)
    }

    /// Handle HTTP response and extract JSON body
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                401 => Error::Auth(format!("Unauthorized: {}", error_text)),
                403 => Error::Auth(format!("Forbidden: {}", error_text)),
                404 => Error::NotFound(error_text),
                _ => Error::Network(format!("Request failed with status {}: {}", status, error_text)),
            });
        }

        response
            .json::<T>()
            .await
            .map_err(|e| Error::Network(format!("Failed to parse response: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TrailBaseClient::new("https://api.example.com".to_string());
        assert!(client.is_ok());

        let client = client.unwrap();
        assert!(!client.is_authenticated());
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_api_key_management() {
        let mut client = TrailBaseClient::new("https://api.example.com".to_string()).unwrap();

        assert!(!client.is_authenticated());

        client.set_api_key("test-key".to_string());
        assert!(client.is_authenticated());

        client.clear_api_key();
        assert!(!client.is_authenticated());
    }
}
