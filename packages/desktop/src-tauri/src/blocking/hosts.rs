// blocking/hosts.rs - Cross-platform hosts file manipulation

use crate::{Error, Result};
use std::path::PathBuf;

const FOCUSFLOW_MARKER_START: &str = "# FocusFlow BLOCK START";
const FOCUSFLOW_MARKER_END: &str = "# FocusFlow BLOCK END";

/// Get platform-specific hosts file path
pub fn get_hosts_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/hosts")
    }
}

/// Check if we have write permissions to the hosts file
///
/// This should be called BEFORE attempting to modify the hosts file
/// to provide early feedback to the user about permission requirements.
///
/// Returns true if we can write to the hosts file, false otherwise.
pub async fn check_hosts_permissions() -> bool {
    let hosts_path = get_hosts_path();

    // First check if file exists and is readable
    if !hosts_path.exists() {
        tracing::warn!("Hosts file does not exist at {}", hosts_path.display());
        return false;
    }

    // Try to read the file first
    if tokio::fs::read_to_string(&hosts_path).await.is_err() {
        tracing::warn!("Cannot read hosts file at {}", hosts_path.display());
        return false;
    }

    // Try to open the file in write mode (but don't actually write)
    // This is the most reliable way to check write permissions without side effects
    match std::fs::OpenOptions::new()
        .write(true)
        .open(&hosts_path)
    {
        Ok(_) => {
            tracing::info!("Hosts file is writable at {}", hosts_path.display());
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            tracing::warn!(
                "Permission denied for hosts file at {}. Elevated privileges required.",
                hosts_path.display()
            );
            false
        }
        Err(e) => {
            tracing::warn!("Error checking hosts file permissions: {}", e);
            false
        }
    }
}

/// Update hosts file with blocked domains
///
/// This function requires elevated privileges on all platforms.
/// Uses atomic write pattern: read -> modify -> write to temp -> rename
pub async fn update_hosts_file(domains: &[String]) -> Result<()> {
    // Read existing hosts file
    let content = read_hosts_file().await?;

    // Remove old FocusFlow entries
    let cleaned = remove_focusflow_entries(&content);

    // Add new entries if any domains provided
    let new_content = if domains.is_empty() {
        cleaned
    } else {
        add_focusflow_entries(&cleaned, domains)
    };

    // Write atomically
    write_hosts_file(&new_content).await?;

    tracing::info!("Updated hosts file with {} domains", domains.len());

    Ok(())
}

/// Clear all FocusFlow entries from hosts file
pub async fn clear_hosts_file() -> Result<()> {
    update_hosts_file(&[]).await
}

/// Read hosts file with error handling for permission issues
async fn read_hosts_file() -> Result<String> {
    let hosts_path = get_hosts_path();

    match tokio::fs::read_to_string(&hosts_path).await {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(Error::PermissionDenied(format!(
                "Cannot read hosts file at {}. Please run with elevated privileges.",
                hosts_path.display()
            )))
        }
        Err(e) => Err(e.into()),
    }
}

/// Write hosts file atomically with proper error handling
async fn write_hosts_file(content: &str) -> Result<()> {
    let hosts_path = get_hosts_path();
    let temp_path = hosts_path.with_extension("focusflow.tmp");

    // Write to temporary file first
    match tokio::fs::write(&temp_path, content).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(Error::PermissionDenied("Cannot write hosts file. Please run with elevated privileges.\n\
                 On macOS/Linux: Use sudo or grant accessibility permissions\n\
                 On Windows: Run as administrator".to_string()));
        }
        Err(e) => return Err(e.into()),
    }

    // Atomic rename
    tokio::fs::rename(&temp_path, &hosts_path).await?;

    // Flush DNS cache after modifying hosts file
    flush_dns_cache().await;

    Ok(())
}

/// Remove existing FocusFlow entries from hosts content
fn remove_focusflow_entries(content: &str) -> String {
    let mut result = String::new();
    let mut skip = false;

    for line in content.lines() {
        if line.contains(FOCUSFLOW_MARKER_START) {
            skip = true;
            continue;
        }

        if line.contains(FOCUSFLOW_MARKER_END) {
            skip = false;
            continue;
        }

        if !skip {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Add FocusFlow entries to hosts content
///
/// Security: Validates domains before writing to hosts file to prevent injection
fn add_focusflow_entries(content: &str, domains: &[String]) -> String {
    let mut result = content.to_string();

    // Ensure content ends with newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    // Add FocusFlow section
    result.push_str(FOCUSFLOW_MARKER_START);
    result.push('\n');

    for domain in domains {
        // SECURITY: Sanitize domain to prevent hosts file injection
        let sanitized = sanitize_domain_for_hosts(domain);

        // Skip invalid domains
        if sanitized.is_empty() {
            tracing::warn!("Skipping invalid domain: {}", domain);
            continue;
        }

        // Add both with and without www
        result.push_str(&format!("127.0.0.1 {}\n", sanitized));

        if !sanitized.starts_with("www.") {
            result.push_str(&format!("127.0.0.1 www.{}\n", sanitized));
        }

        // Also block IPv6
        result.push_str(&format!("::1 {}\n", sanitized));
        if !sanitized.starts_with("www.") {
            result.push_str(&format!("::1 www.{}\n", sanitized));
        }
    }

    result.push_str(FOCUSFLOW_MARKER_END);
    result.push('\n');

    result
}

/// Sanitize domain name for safe hosts file entry
///
/// Removes any characters that could be used for injection attacks
/// Returns empty string if domain is invalid
fn sanitize_domain_for_hosts(domain: &str) -> String {
    let domain = domain.trim().to_lowercase();

    // Reject empty, very long, or domains with null bytes
    if domain.is_empty() || domain.len() > 253 || domain.contains('\0') {
        return String::new();
    }

    // Reject domains with newlines, carriage returns, or tabs (hosts file injection)
    if domain.contains('\n') || domain.contains('\r') || domain.contains('\t') {
        return String::new();
    }

    // Reject domains with spaces or hash (could break hosts file format)
    if domain.contains(' ') || domain.contains('#') {
        return String::new();
    }

    // Only allow valid DNS characters: alphanumeric, hyphens, and dots
    if !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
        return String::new();
    }

    // Reject invalid formats
    if domain.starts_with('.') || domain.ends_with('.')
        || domain.starts_with('-') || domain.ends_with('-')
        || domain.contains("..") || !domain.contains('.') {
        return String::new();
    }

    domain
}

/// Flush DNS cache after modifying hosts file
///
/// Security: Uses Command::new() with separate arguments to prevent command injection.
/// All commands are hardcoded system binaries with static arguments.
async fn flush_dns_cache() {
    #[cfg(target_os = "macos")]
    {
        // Execute dscacheutil with static argument
        if let Err(e) = tokio::process::Command::new("dscacheutil")
            .arg("-flushcache")
            .output()
            .await
        {
            tracing::debug!("Failed to execute dscacheutil: {}", e);
        }

        // Execute killall with static arguments
        if let Err(e) = tokio::process::Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .output()
            .await
        {
            tracing::debug!("Failed to execute killall: {}", e);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Execute ipconfig with static argument
        if let Err(e) = tokio::process::Command::new("ipconfig")
            .arg("/flushdns")
            .output()
            .await
        {
            tracing::debug!("Failed to execute ipconfig: {}", e);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try multiple DNS cache flush methods for different distributions
        // All use hardcoded commands with static arguments

        // systemd-resolved (Ubuntu, Debian, Fedora, etc.)
        let _ = tokio::process::Command::new("systemd-resolve")
            .arg("--flush-caches")
            .output()
            .await;

        // resolvectl (newer systemd)
        let _ = tokio::process::Command::new("resolvectl")
            .arg("flush-caches")
            .output()
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_focusflow_entries() {
        let content = "\
127.0.0.1 localhost\n\
# FocusFlow BLOCK START\n\
127.0.0.1 facebook.com\n\
# FocusFlow BLOCK END\n\
192.168.1.1 router\n";

        let result = remove_focusflow_entries(content);
        assert!(!result.contains("facebook.com"));
        assert!(result.contains("localhost"));
        assert!(result.contains("router"));
    }

    #[test]
    fn test_add_focusflow_entries() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec!["twitter.com".to_string()];

        let result = add_focusflow_entries(content, &domains);
        assert!(result.contains("127.0.0.1 twitter.com"));
        assert!(result.contains("127.0.0.1 www.twitter.com"));
        assert!(result.contains(FOCUSFLOW_MARKER_START));
        assert!(result.contains(FOCUSFLOW_MARKER_END));
    }
}
