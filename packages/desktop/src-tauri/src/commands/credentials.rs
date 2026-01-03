// commands/credentials.rs - Secure API key storage using OS keychain

use crate::{Error, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::command;
use tracing::{debug, info};

const SERVICE_NAME: &str = "com.focusflow.desktop";

/// Credential information (without the actual key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInfo {
    pub provider: String,
    pub has_api_key: bool,
}

/// Save an API key to the OS keychain
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "openai", "anthropic")
/// * `api_key` - The API key to store securely
#[command]
pub async fn save_api_key(provider: String, api_key: String) -> Result<()> {
    debug!("Saving API key for provider: {}", provider);

    let entry = Entry::new(SERVICE_NAME, &provider)
        .map_err(|e| Error::Config(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .set_password(&api_key)
        .map_err(|e| Error::Config(format!("Failed to save API key: {}", e)))?;

    info!("API key saved successfully for provider: {}", provider);
    Ok(())
}

/// Retrieve an API key from the OS keychain
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "openai", "anthropic")
#[command]
pub async fn get_api_key(provider: String) -> Result<String> {
    debug!("Retrieving API key for provider: {}", provider);

    let entry = Entry::new(SERVICE_NAME, &provider)
        .map_err(|e| Error::Config(format!("Failed to create keyring entry: {}", e)))?;

    let api_key = entry
        .get_password()
        .map_err(|e| Error::NotFound(format!("API key not found for provider {}: {}", provider, e)))?;

    debug!("API key retrieved successfully for provider: {}", provider);
    Ok(api_key)
}

/// Delete an API key from the OS keychain
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "openai", "anthropic")
#[command]
pub async fn delete_api_key(provider: String) -> Result<()> {
    debug!("Deleting API key for provider: {}", provider);

    let entry = Entry::new(SERVICE_NAME, &provider)
        .map_err(|e| Error::Config(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .delete_password()
        .map_err(|e| Error::Config(format!("Failed to delete API key: {}", e)))?;

    info!("API key deleted successfully for provider: {}", provider);
    Ok(())
}

/// Check if an API key exists for a provider
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "openai", "anthropic")
#[command]
pub async fn has_api_key(provider: String) -> Result<bool> {
    debug!("Checking if API key exists for provider: {}", provider);

    let entry = Entry::new(SERVICE_NAME, &provider)
        .map_err(|e| Error::Config(format!("Failed to create keyring entry: {}", e)))?;

    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// List all providers that have saved API keys
#[command]
pub async fn list_saved_providers() -> Result<Vec<CredentialInfo>> {
    debug!("Listing providers with saved API keys");

    // Check all known providers
    let providers = vec![
        "openai",
        "anthropic",
        "google",
        "openrouter",
    ];

    let mut credentials = Vec::new();

    for provider in providers {
        let entry = Entry::new(SERVICE_NAME, provider)
            .map_err(|e| Error::Config(format!("Failed to create keyring entry: {}", e)))?;

        let has_key = entry.get_password().is_ok();

        if has_key {
            credentials.push(CredentialInfo {
                provider: provider.to_string(),
                has_api_key: true,
            });
        }
    }

    debug!("Found {} providers with saved API keys", credentials.len());
    Ok(credentials)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_get_api_key() {
        let provider = "test_provider".to_string();
        let api_key = "test_key_123".to_string();

        // Save
        save_api_key(provider.clone(), api_key.clone())
            .await
            .unwrap();

        // Retrieve
        let retrieved = get_api_key(provider.clone()).await.unwrap();
        assert_eq!(retrieved, api_key);

        // Delete
        delete_api_key(provider.clone()).await.unwrap();

        // Verify deleted
        assert!(!has_api_key(provider).await.unwrap());
    }
}
