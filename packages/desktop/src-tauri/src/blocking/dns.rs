// blocking/dns.rs - DNS-level blocking fallback mechanism
//
// This module provides a fallback blocking mechanism when hosts file modification
// isn't available (no elevated privileges). It exposes blocked domains to the
// frontend, which can implement blocking overlays or communicate with browser extensions.
//
// Architecture:
// - Backend maintains the authoritative list of blocked domains
// - Frontend polls or listens for blocklist updates
// - Frontend implements blocking UI (overlays, warnings, redirects)
// - Optional: Browser extension integration for deeper blocking

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Response containing blocked domains and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedDomainsResponse {
    /// List of blocked domains
    pub domains: Vec<String>,
    /// Whether blocking is currently active
    pub enabled: bool,
    /// Total number of blocked domains
    pub count: usize,
    /// Last updated timestamp (ISO 8601)
    pub last_updated: String,
}

/// Domain block check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCheckResult {
    /// Whether the domain is blocked
    pub blocked: bool,
    /// The matched blocked domain (if any)
    pub matched_domain: Option<String>,
    /// Match type (exact, subdomain, wildcard)
    pub match_type: Option<String>,
}

/// Statistics about blocking attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingStats {
    /// Total domains in blocklist
    pub total_blocked_domains: usize,
    /// Whether blocking is enabled
    pub blocking_enabled: bool,
    /// Domains grouped by category (if categorization is implemented)
    pub categories: HashMap<String, Vec<String>>,
}

/// DNS-level blocking fallback provider
///
/// This doesn't actually run a DNS server, but provides domain blocking information
/// to the frontend for client-side enforcement.
pub struct DnsBlockingFallback {
    /// Set of blocked domains for O(1) lookup
    blocked_domains: HashSet<String>,
    /// Whether blocking is currently enabled
    enabled: bool,
    /// Last time the blocklist was updated
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl DnsBlockingFallback {
    /// Create a new DNS blocking fallback instance
    pub fn new() -> Self {
        Self {
            blocked_domains: HashSet::new(),
            enabled: false,
            last_updated: chrono::Utc::now(),
        }
    }

    /// Update the blocklist with new domains
    pub fn update_blocklist(&mut self, domains: Vec<String>) {
        self.blocked_domains.clear();

        for domain in domains {
            // Normalize domain: lowercase, trim whitespace
            let normalized = domain.trim().to_lowercase();

            if !normalized.is_empty() {
                self.blocked_domains.insert(normalized.clone());

                // Also add www. variant if not present
                if !normalized.starts_with("www.") {
                    self.blocked_domains.insert(format!("www.{}", normalized));
                }
            }
        }

        self.last_updated = chrono::Utc::now();

        tracing::info!(
            "Updated DNS fallback blocklist: {} domains (including www. variants)",
            self.blocked_domains.len()
        );
    }

    /// Enable blocking
    pub fn enable(&mut self) {
        self.enabled = true;
        tracing::info!("DNS fallback blocking enabled");
    }

    /// Disable blocking
    #[allow(dead_code)]
    pub fn disable(&mut self) {
        self.enabled = false;
        tracing::info!("DNS fallback blocking disabled");
    }

    /// Check if a domain is blocked
    ///
    /// Supports exact matches and subdomain matching.
    /// Example: blocking "example.com" will also block "www.example.com" and "api.example.com"
    pub fn is_domain_blocked(&self, domain: &str) -> DomainCheckResult {
        if !self.enabled {
            return DomainCheckResult {
                blocked: false,
                matched_domain: None,
                match_type: None,
            };
        }

        let normalized = domain.trim().to_lowercase();

        // Check exact match
        if self.blocked_domains.contains(&normalized) {
            return DomainCheckResult {
                blocked: true,
                matched_domain: Some(normalized.clone()),
                match_type: Some("exact".to_string()),
            };
        }

        // Check if this is a subdomain of a blocked domain
        // Example: "api.facebook.com" should match blocked "facebook.com"
        let parts: Vec<&str> = normalized.split('.').collect();

        // Try progressively shorter domains
        // api.example.com -> example.com -> com
        for i in 1..parts.len() {
            let parent_domain = parts[i..].join(".");
            if self.blocked_domains.contains(&parent_domain) {
                return DomainCheckResult {
                    blocked: true,
                    matched_domain: Some(parent_domain.clone()),
                    match_type: Some("subdomain".to_string()),
                };
            }
        }

        DomainCheckResult {
            blocked: false,
            matched_domain: None,
            match_type: None,
        }
    }

    /// Get all blocked domains as a response
    #[allow(dead_code)]
    pub fn get_blocked_domains(&self) -> BlockedDomainsResponse {
        // Return unique base domains (without www. duplicates)
        let unique_domains: Vec<String> = self
            .blocked_domains
            .iter()
            .filter(|d| !d.starts_with("www."))
            .cloned()
            .collect();

        BlockedDomainsResponse {
            domains: unique_domains.clone(),
            enabled: self.enabled,
            count: unique_domains.len(),
            last_updated: self.last_updated.to_rfc3339(),
        }
    }

    /// Get blocking statistics
    pub fn get_stats(&self) -> BlockingStats {
        // Basic implementation - can be extended with categorization
        let mut categories = HashMap::new();

        let unique_domains: Vec<String> = self
            .blocked_domains
            .iter()
            .filter(|d| !d.starts_with("www."))
            .cloned()
            .collect();

        categories.insert("all".to_string(), unique_domains);

        BlockingStats {
            total_blocked_domains: self.blocked_domains.len() / 2, // Divide by 2 for www. variants
            blocking_enabled: self.enabled,
            categories,
        }
    }

    /// Extract domain from a URL
    ///
    /// Helper function to extract domain from full URLs
    /// Example: "https://www.example.com/path" -> "www.example.com"
    pub fn extract_domain_from_url(url: &str) -> Option<String> {
        // Handle URLs with protocol
        if let Some(without_protocol) = url.strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .or_else(|| url.strip_prefix("//"))
        {
            // Extract domain part (before first /)
            let domain = without_protocol
                .split('/')
                .next()
                .unwrap_or(without_protocol);

            // Remove port if present
            let domain = domain.split(':').next().unwrap_or(domain);

            return Some(domain.to_lowercase().trim().to_string());
        }

        // If no protocol, assume it's already a domain
        let domain = url.split('/').next().unwrap_or(url);
        let domain = domain.split(':').next().unwrap_or(domain);

        Some(domain.to_lowercase().trim().to_string())
    }

    /// Check if a URL is blocked
    ///
    /// Convenience method that extracts domain from URL and checks if blocked
    #[allow(dead_code)]
    pub fn is_url_blocked(&self, url: &str) -> DomainCheckResult {
        match Self::extract_domain_from_url(url) {
            Some(domain) => self.is_domain_blocked(&domain),
            None => DomainCheckResult {
                blocked: false,
                matched_domain: None,
                match_type: None,
            },
        }
    }
}

impl Default for DnsBlockingFallback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_domain_blocking() {
        let mut fallback = DnsBlockingFallback::new();
        fallback.update_blocklist(vec!["facebook.com".to_string()]);
        fallback.enable();

        let result = fallback.is_domain_blocked("facebook.com");
        assert!(result.blocked);
        assert_eq!(result.match_type, Some("exact".to_string()));
    }

    #[test]
    fn test_subdomain_blocking() {
        let mut fallback = DnsBlockingFallback::new();
        fallback.update_blocklist(vec!["facebook.com".to_string()]);
        fallback.enable();

        let result = fallback.is_domain_blocked("www.facebook.com");
        assert!(result.blocked);

        let result = fallback.is_domain_blocked("api.facebook.com");
        assert!(result.blocked);
        assert_eq!(result.match_type, Some("subdomain".to_string()));
    }

    #[test]
    fn test_disabled_blocking() {
        let mut fallback = DnsBlockingFallback::new();
        fallback.update_blocklist(vec!["facebook.com".to_string()]);
        // Don't enable

        let result = fallback.is_domain_blocked("facebook.com");
        assert!(!result.blocked);
    }

    #[test]
    fn test_url_extraction() {
        assert_eq!(
            DnsBlockingFallback::extract_domain_from_url("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );

        assert_eq!(
            DnsBlockingFallback::extract_domain_from_url("http://example.com:8080/path"),
            Some("example.com".to_string())
        );

        assert_eq!(
            DnsBlockingFallback::extract_domain_from_url("example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_url_blocking() {
        let mut fallback = DnsBlockingFallback::new();
        fallback.update_blocklist(vec!["twitter.com".to_string()]);
        fallback.enable();

        let result = fallback.is_url_blocked("https://twitter.com/home");
        assert!(result.blocked);

        let result = fallback.is_url_blocked("https://www.twitter.com/home");
        assert!(result.blocked);
    }

    #[test]
    fn test_case_insensitive_blocking() {
        let mut fallback = DnsBlockingFallback::new();
        fallback.update_blocklist(vec!["Facebook.COM".to_string()]);
        fallback.enable();

        let result = fallback.is_domain_blocked("facebook.com");
        assert!(result.blocked);

        let result = fallback.is_domain_blocked("FACEBOOK.COM");
        assert!(result.blocked);
    }
}
