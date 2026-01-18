// commands/blocking.rs - App and website blocking management

use crate::{
    blocking::{capabilities, dns, hosts},
    db::queries,
    state::AppState,
    Error, Result,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct AddBlockedItemRequest {
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct BlockedItemsResponse {
    pub apps: Vec<String>,
    pub websites: Vec<String>,
}

/// Add an application to the block list
///
/// Process name should be the executable name (e.g., "chrome", "slack")
#[tauri::command]
pub async fn add_blocked_app(
    request: AddBlockedItemRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    // Validate process name format
    if request.value.trim().is_empty() {
        return Err(Error::InvalidInput(
            "Process name cannot be empty".to_string(),
        ));
    }

    queries::insert_blocked_item(state.pool(), "app", &request.value).await?;

    tracing::info!("Added blocked app: {}", request.value);

    Ok(())
}

/// Remove an application from the block list
#[tauri::command]
pub async fn remove_blocked_app(
    request: AddBlockedItemRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    queries::remove_blocked_item(state.pool(), "app", &request.value).await?;

    tracing::info!("Removed blocked app: {}", request.value);

    Ok(())
}

/// Add a website to the block list
///
/// Domain should be in format: "example.com" or "www.example.com"
#[tauri::command]
pub async fn add_blocked_website(
    request: AddBlockedItemRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    // Validate domain format
    let domain = request.value.trim().to_lowercase();

    // Check for empty domain
    if domain.is_empty() {
        return Err(Error::InvalidInput(
            "Domain cannot be empty".to_string(),
        ));
    }

    // Check domain length (max 255 chars per DNS spec)
    if domain.len() > 253 {
        return Err(Error::InvalidInput(
            "Domain name is too long (max 253 characters)".to_string(),
        ));
    }

    // Check for at least one dot (e.g., "example.com")
    if !domain.contains('.') {
        return Err(Error::InvalidInput(
            "Invalid domain format. Domain must contain at least one dot (e.g., 'example.com')".to_string(),
        ));
    }

    // Check for protocol or path characters
    if domain.contains('/') || domain.contains(':') {
        return Err(Error::InvalidInput(
            "Invalid domain format. Use 'example.com' without protocol or path".to_string(),
        ));
    }

    // Check for invalid characters (only alphanumeric, hyphens, and dots allowed)
    let valid_domain = domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.');
    if !valid_domain {
        return Err(Error::InvalidInput(
            "Domain contains invalid characters. Only letters, numbers, hyphens, and dots are allowed".to_string(),
        ));
    }

    // Check that domain doesn't start or end with hyphen or dot
    if domain.starts_with('.') || domain.ends_with('.') || domain.starts_with('-') || domain.ends_with('-') {
        return Err(Error::InvalidInput(
            "Domain cannot start or end with a dot or hyphen".to_string(),
        ));
    }

    queries::insert_blocked_item(state.pool(), "website", &domain).await?;

    // Apply hosts file blocking if session is active
    let blocking_enabled = {
        let blocking_state = state.blocking_state.read().await;
        blocking_state.enabled
    };

    if blocking_enabled {
        let all_websites = queries::get_blocked_items(state.pool(), Some("website")).await?;
        let domains: Vec<String> = all_websites.iter().map(|item| item.value.clone()).collect();

        // Update in-memory state for DNS fallback
        {
            let mut blocking_state = state.blocking_state.write().await;
            blocking_state.update_blocked_websites(domains.clone());
        }

        // Try to update hosts file (may fail without privileges)
        if let Err(e) = hosts::update_hosts_file(&domains).await {
            tracing::warn!("Hosts file update failed: {}, DNS fallback active", e);
        }
    }

    tracing::info!("Added blocked website: {}", domain);

    Ok(())
}

/// Remove a website from the block list
#[tauri::command]
pub async fn remove_blocked_website(
    request: AddBlockedItemRequest,
    state: State<'_, AppState>,
) -> Result<()> {
    let domain = request.value.trim().to_lowercase();

    queries::remove_blocked_item(state.pool(), "website", &domain).await?;

    // Update hosts file
    let blocking_enabled = {
        let blocking_state = state.blocking_state.read().await;
        blocking_state.enabled
    };

    if blocking_enabled {
        let all_websites = queries::get_blocked_items(state.pool(), Some("website")).await?;
        let domains: Vec<String> = all_websites.iter().map(|item| item.value.clone()).collect();

        // Update in-memory state for DNS fallback
        {
            let mut blocking_state = state.blocking_state.write().await;
            blocking_state.update_blocked_websites(domains.clone());
        }

        // Try to update hosts file (may fail without privileges)
        if let Err(e) = hosts::update_hosts_file(&domains).await {
            tracing::warn!("Hosts file update failed: {}, DNS fallback active", e);
        }
    }

    tracing::info!("Removed blocked website: {}", domain);

    Ok(())
}

/// Get all currently blocked items
#[tauri::command]
pub async fn get_blocked_items(
    state: State<'_, AppState>,
) -> Result<BlockedItemsResponse> {
    let items = queries::get_blocked_items(state.pool(), None).await?;

    let mut apps = Vec::new();
    let mut websites = Vec::new();

    for item in items {
        match item.item_type.as_str() {
            "app" => apps.push(item.value),
            "website" => websites.push(item.value),
            _ => {}
        }
    }

    Ok(BlockedItemsResponse { apps, websites })
}

/// Toggle blocking on/off manually
#[tauri::command]
pub async fn toggle_blocking(
    enable: bool,
    state: State<'_, AppState>,
) -> Result<()> {
    let websites = queries::get_blocked_items(state.pool(), Some("website")).await?;
    let domains: Vec<String> = websites.iter().map(|item| item.value.clone()).collect();

    {
        let mut blocking_state = state.blocking_state.write().await;
        if enable {
            blocking_state.enable();
            blocking_state.update_blocked_websites(domains.clone());
        } else {
            blocking_state.disable();
            blocking_state.update_blocked_websites(Vec::new());
        }
    }

    if enable {
        // Try to apply hosts file blocking (requires elevated privileges)
        match hosts::update_hosts_file(&domains).await {
            Ok(_) => {
                tracing::info!("Hosts file blocking enabled");
            }
            Err(e) => {
                tracing::warn!("Hosts file blocking failed ({}), DNS fallback available", e);
                // Don't return error - fallback is still available
            }
        }
    } else {
        // Clear website blocking
        if let Err(e) = hosts::clear_hosts_file().await {
            tracing::warn!("Failed to clear hosts file: {}", e);
        }
    }

    tracing::info!("Blocking {}", if enable { "enabled" } else { "disabled" });

    Ok(())
}

// ============================================================================
// DNS Fallback Commands (for frontend-based blocking)
// ============================================================================

/// Get all currently blocked domains for frontend enforcement
///
/// This command provides the blocked domains list to the frontend so it can
/// implement blocking overlays, warnings, or communicate with browser extensions.
/// This works as a fallback when hosts file modification isn't available.
#[tauri::command]
pub async fn get_blocked_domains(
    state: State<'_, AppState>,
) -> Result<dns::BlockedDomainsResponse> {
    let blocking_state = state.blocking_state.read().await;

    let response = dns::BlockedDomainsResponse {
        domains: blocking_state.blocked_websites.clone(),
        enabled: blocking_state.enabled,
        count: blocking_state.blocked_websites.len(),
        last_updated: blocking_state
            .last_check
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339(),
    };

    Ok(response)
}

/// Check if a specific domain is blocked
///
/// Frontend can use this to check before navigating or to show blocking UI
/// When record_attempt is true, this will log the block attempt to the database
#[tauri::command]
pub async fn check_domain_blocked(
    domain: String,
    record_attempt: Option<bool>,
    state: State<'_, AppState>,
) -> Result<dns::DomainCheckResult> {
    let blocking_state = state.blocking_state.read().await;

    if !blocking_state.enabled {
        return Ok(dns::DomainCheckResult {
            blocked: false,
            matched_domain: None,
            match_type: None,
        });
    }

    // Create temporary fallback instance to use domain checking logic
    let mut fallback = dns::DnsBlockingFallback::new();
    fallback.update_blocklist(blocking_state.blocked_websites.clone());
    fallback.enable();

    let result = fallback.is_domain_blocked(&domain);

    // Record the block attempt if requested and blocked
    if record_attempt.unwrap_or(false) && result.blocked {
        drop(blocking_state); // Release the lock before async operation

        let session_id = {
            let active_session = state.active_session.read().await;
            active_session.as_ref().map(|s| s.id.clone())
        };

        let user_id = state.get_user_id().await;
        let blocked_domain = result.matched_domain.as_ref().unwrap_or(&domain);

        if let Err(e) = queries::record_block_attempt(
            state.pool(),
            "website",
            blocked_domain,
            session_id.as_deref(),
            user_id.as_deref(),
        )
        .await
        {
            tracing::error!("Failed to record website block attempt: {}", e);
        } else {
            tracing::debug!("Recorded website block attempt for: {}", blocked_domain);
        }
    }

    Ok(result)
}

/// Check if a URL is blocked
///
/// Convenience endpoint that extracts domain from URL and checks if blocked
/// When record_attempt is true, this will log the block attempt to the database
#[tauri::command]
pub async fn check_url_blocked(
    url: String,
    record_attempt: Option<bool>,
    state: State<'_, AppState>,
) -> Result<dns::DomainCheckResult> {
    let blocking_state = state.blocking_state.read().await;

    if !blocking_state.enabled {
        return Ok(dns::DomainCheckResult {
            blocked: false,
            matched_domain: None,
            match_type: None,
        });
    }

    // Extract domain from URL
    let domain = dns::DnsBlockingFallback::extract_domain_from_url(&url)
        .ok_or_else(|| Error::InvalidInput("Invalid URL format".to_string()))?;

    // Create temporary fallback instance to use domain checking logic
    let mut fallback = dns::DnsBlockingFallback::new();
    fallback.update_blocklist(blocking_state.blocked_websites.clone());
    fallback.enable();

    let result = fallback.is_domain_blocked(&domain);

    // Record the block attempt if requested and blocked
    if record_attempt.unwrap_or(false) && result.blocked {
        drop(blocking_state); // Release the lock before async operation

        let session_id = {
            let active_session = state.active_session.read().await;
            active_session.as_ref().map(|s| s.id.clone())
        };

        let user_id = state.get_user_id().await;
        let blocked_domain = result.matched_domain.as_ref().unwrap_or(&domain);

        if let Err(e) = queries::record_block_attempt(
            state.pool(),
            "website",
            blocked_domain,
            session_id.as_deref(),
            user_id.as_deref(),
        )
        .await
        {
            tracing::error!("Failed to record website block attempt: {}", e);
        } else {
            tracing::debug!("Recorded website block attempt for: {}", blocked_domain);
        }
    }

    Ok(result)
}

/// Get blocking statistics
#[tauri::command]
pub async fn get_blocking_stats(
    state: State<'_, AppState>,
) -> Result<dns::BlockingStats> {
    let blocking_state = state.blocking_state.read().await;

    let mut fallback = dns::DnsBlockingFallback::new();
    fallback.update_blocklist(blocking_state.blocked_websites.clone());

    if blocking_state.enabled {
        fallback.enable();
    }

    let stats = fallback.get_stats();

    Ok(stats)
}

// ============================================================================
// Capability Detection and Permission Guidance
// ============================================================================

/// Get current blocking capabilities
///
/// Returns information about what blocking methods are available based on
/// system permissions. This should be called at app startup and displayed
/// to the user so they understand what blocking features will work.
///
/// Example response:
/// ```json
/// {
///   "hosts_file_writable": false,
///   "hosts_file_path": "/etc/hosts",
///   "process_termination_available": true,
///   "recommended_method": "process_termination",
///   "available_methods": ["process_termination", "frontend_only"],
///   "limitations": [
///     "Hosts file at /etc/hosts is not writable. Website blocking requires elevated privileges."
///   ],
///   "platform": "macOS"
/// }
/// ```
#[tauri::command]
pub async fn get_blocking_capabilities() -> Result<capabilities::BlockingCapabilities> {
    let caps = capabilities::check_capabilities().await;

    if !caps.hosts_file_writable {
        tracing::warn!(
            "Hosts file is not writable. Website blocking will use fallback methods."
        );
    }

    Ok(caps)
}

/// Get platform-specific elevation instructions
///
/// Returns detailed instructions on how to grant the necessary permissions
/// for effective website blocking. The instructions are tailored to the
/// user's operating system.
///
/// Example response for macOS:
/// ```json
/// {
///   "platform": "macOS",
///   "primary_method": "Grant Full Disk Access",
///   "alternative_methods": ["Run with sudo (temporary)"],
///   "steps": [
///     "Open System Settings > Privacy & Security > Full Disk Access",
///     "Click the lock icon and enter your password",
///     "Click the '+' button and add FocusFlow from Applications",
///     "Restart FocusFlow for changes to take effect"
///   ],
///   "security_notes": [
///     "Full Disk Access allows FocusFlow to modify system files like /etc/hosts",
///     "This is required for effective website blocking",
///     "Similar permissions are required by apps like Cold Turkey and SelfControl"
///   ],
///   "requires_restart": true
/// }
/// ```
#[tauri::command]
pub async fn get_elevation_instructions() -> Result<capabilities::ElevationInstructions> {
    let instructions = capabilities::get_elevation_instructions();

    tracing::info!(
        "Providing elevation instructions for platform: {}",
        instructions.platform
    );

    Ok(instructions)
}

/// Check if the app currently has hosts file permissions
///
/// This is a simple boolean check that can be called frequently to detect
/// when permissions have been granted (e.g., after the user follows the
/// elevation instructions and restarts the app).
///
/// Returns true if hosts file is writable, false otherwise.
#[tauri::command]
pub async fn check_hosts_file_permissions() -> Result<bool> {
    let writable = hosts::check_hosts_permissions().await;

    if writable {
        tracing::info!("Hosts file is writable - full website blocking available");
    } else {
        tracing::warn!("Hosts file is not writable - using fallback blocking methods");
    }

    Ok(writable)
}
