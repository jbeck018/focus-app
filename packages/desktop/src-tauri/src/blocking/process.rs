// blocking/process.rs - Efficient process monitoring and termination

use crate::{db::queries, state::AppState, Error, Result};
use regex::Regex;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use std::collections::HashSet;
use tauri_plugin_notification::NotificationExt;
use tokio::time::{interval, Duration};

/// Monitoring loop configuration
const MONITOR_INTERVAL_MS: u64 = 2000; // Check every 2 seconds
const GRACE_PERIOD_MS: u64 = 3000; // Give 3 seconds before killing

/// Match types for process blocking
#[derive(Debug, Clone, PartialEq)]
enum MatchType {
    Exact,
    Contains,
    Regex,
}

impl MatchType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "exact" => MatchType::Exact,
            "contains" => MatchType::Contains,
            "regex" => MatchType::Regex,
            _ => MatchType::Exact, // Default to exact for safety
        }
    }
}

/// Compiled matcher for efficient process matching
struct ProcessMatcher {
    value: String,
    match_type: MatchType,
    regex: Option<Regex>,
}

impl ProcessMatcher {
    fn new(value: String, match_type: String) -> Result<Self> {
        let match_type = MatchType::from_str(&match_type);
        let regex = if match_type == MatchType::Regex {
            Some(Regex::new(&value).map_err(|e| {
                Error::Config(format!("Invalid regex pattern '{}': {}", value, e))
            })?)
        } else {
            None
        };

        Ok(Self {
            value,
            match_type,
            regex,
        })
    }

    /// Check if a process name matches this blocked item
    fn matches(&self, process_name: &str) -> bool {
        match self.match_type {
            MatchType::Exact => self.exact_match(process_name),
            MatchType::Contains => self.contains_match(process_name),
            MatchType::Regex => self.regex_match(process_name),
        }
    }

    /// Exact match with case-insensitive comparison and extension normalization
    fn exact_match(&self, process_name: &str) -> bool {
        let normalized_process = normalize_process_name(process_name);
        let normalized_blocked = normalize_process_name(&self.value);

        normalized_process == normalized_blocked
    }

    /// Contains match (legacy behavior, opt-in only)
    fn contains_match(&self, process_name: &str) -> bool {
        process_name
            .to_lowercase()
            .contains(&self.value.to_lowercase())
    }

    /// Regex match
    fn regex_match(&self, process_name: &str) -> bool {
        self.regex
            .as_ref()
            .map(|r| r.is_match(process_name))
            .unwrap_or(false)
    }
}

/// Normalize a process name for exact matching
///
/// This handles platform-specific variations:
/// - Case-insensitive comparison (Chrome == chrome == CHROME)
/// - Windows .exe extension stripping (chrome.exe == chrome)
/// - Consistent lowercasing
fn normalize_process_name(name: &str) -> String {
    let mut normalized = name.to_lowercase();

    // Strip .exe extension on all platforms for consistency
    // This allows blocking "chrome" to match both "chrome" (macOS/Linux) and "chrome.exe" (Windows)
    if normalized.ends_with(".exe") {
        normalized = normalized[..normalized.len() - 4].to_string();
    }

    normalized
}

/// Start the process monitoring loop
///
/// This runs in the background and checks for blocked processes at regular intervals.
/// Uses efficient delta updates from sysinfo to minimize CPU usage.
pub async fn start_monitoring_loop(state: AppState) -> Result<()> {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    let mut interval = interval(Duration::from_millis(MONITOR_INTERVAL_MS));
    let mut warned_processes: HashSet<String> = HashSet::new();

    loop {
        interval.tick().await;

        // Check if blocking is enabled
        let blocking_enabled = {
            let blocking_state = state.blocking_state.read().await;
            blocking_state.enabled
        };

        if !blocking_enabled {
            warned_processes.clear();
            continue;
        }

        // Get blocked apps from database
        let blocked_items = match queries::get_blocked_items(state.pool(), Some("app")).await {
            Ok(items) => items,
            Err(e) => {
                tracing::error!("Failed to fetch blocked apps: {}", e);
                continue;
            }
        };

        if blocked_items.is_empty() {
            continue;
        }

        // Build process matchers from blocked items
        let matchers: Vec<ProcessMatcher> = blocked_items
            .iter()
            .filter_map(|item| {
                match ProcessMatcher::new(item.value.clone(), item.match_type.clone()) {
                    Ok(matcher) => Some(matcher),
                    Err(e) => {
                        tracing::warn!(
                            "Invalid blocking rule for '{}': {}. Skipping.",
                            item.value,
                            e
                        );
                        None
                    }
                }
            })
            .collect();

        if matchers.is_empty() {
            continue;
        }

        // Refresh process list (only updates changed processes)
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        // Check for blocked processes
        for (pid, process) in system.processes() {
            let process_name = process.name().to_string_lossy();

            // Check if this process matches any blocked rule
            let is_blocked = matchers.iter().any(|matcher| matcher.matches(&process_name));

            if is_blocked {
                let process_key = format!("{}:{}", process_name, pid);

                if !warned_processes.contains(&process_key) {
                    // First detection: send warning notification
                    tracing::warn!("Detected blocked process: {} (PID: {})", process_name, pid);

                    if let Err(e) = state
                        .app_handle
                        .notification()
                        .builder()
                        .title("Blocked Application Detected")
                        .body(format!(
                            "{} is blocked during focus sessions. It will be closed shortly.",
                            process.name().to_string_lossy()
                        ))
                        .show()
                    {
                        tracing::warn!("Failed to send notification: {}", e);
                    }

                    warned_processes.insert(process_key.clone());

                    // Schedule termination after grace period
                    let pid_copy = *pid;
                    let process_name_copy = process_name.to_string();
                    let app_handle = state.app_handle.clone();
                    let state_clone = state.clone();

                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(GRACE_PERIOD_MS)).await;

                        if let Err(e) = terminate_process(pid_copy) {
                            tracing::error!("Failed to terminate {}: {}", process_name_copy, e);
                        } else {
                            tracing::info!("Terminated blocked process: {} (PID: {})", process_name_copy, pid_copy);

                            // Record the block attempt
                            let session_id = {
                                let active_session = state_clone.active_session.read().await;
                                active_session.as_ref().map(|s| s.id.clone())
                            };

                            let user_id = state_clone.get_user_id().await;

                            if let Err(e) = queries::record_block_attempt(
                                state_clone.pool(),
                                "app",
                                &process_name_copy,
                                session_id.as_deref(),
                                user_id.as_deref(),
                            )
                            .await
                            {
                                tracing::error!("Failed to record block attempt: {}", e);
                            } else {
                                tracing::debug!("Recorded block attempt for: {}", process_name_copy);
                            }

                            let _ = app_handle
                                .notification()
                                .builder()
                                .title("Application Closed")
                                .body(format!("{} has been closed to help you focus.", process_name_copy))
                                .show();
                        }
                    });
                }
            }
        }

        // Clean up warned processes that are no longer running
        warned_processes.retain(|key| {
            let pid_str = key.split(':').nth(1).unwrap_or("");
            if let Ok(pid) = pid_str.parse::<usize>() {
                system.process(sysinfo::Pid::from(pid)).is_some()
            } else {
                false
            }
        });

        // Update last check timestamp
        {
            let mut blocking_state = state.blocking_state.write().await;
            blocking_state.update_last_check();
        }
    }
}

/// Terminate a process by PID
///
/// Uses platform-specific APIs for graceful then forceful termination
fn terminate_process(pid: sysinfo::Pid) -> Result<()> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    if let Some(process) = system.process(pid) {
        // Try graceful termination first
        if process.kill() {
            Ok(())
        } else {
            Err(Error::System(format!(
                "Failed to terminate process with PID {}",
                pid
            )))
        }
    } else {
        Err(Error::ProcessNotFound(format!("PID {}", pid)))
    }
}

/// Get list of currently running processes (for debugging/UI)
#[allow(dead_code)]
pub fn get_running_processes() -> Vec<ProcessInfo> {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    system
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
        })
        .collect()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_process_name() {
        // Case insensitivity
        assert_eq!(normalize_process_name("Chrome"), "chrome");
        assert_eq!(normalize_process_name("CHROME"), "chrome");
        assert_eq!(normalize_process_name("ChRoMe"), "chrome");

        // Windows .exe extension handling
        assert_eq!(normalize_process_name("chrome.exe"), "chrome");
        assert_eq!(normalize_process_name("Chrome.exe"), "chrome");
        assert_eq!(normalize_process_name("CHROME.EXE"), "chrome");

        // macOS/Linux names without extension
        assert_eq!(normalize_process_name("slack"), "slack");
        assert_eq!(normalize_process_name("firefox"), "firefox");
    }

    #[test]
    fn test_exact_match_basic() {
        let matcher = ProcessMatcher::new("chrome".to_string(), "exact".to_string()).unwrap();

        // Should match exact name
        assert!(matcher.matches("chrome"));
        assert!(matcher.matches("Chrome"));
        assert!(matcher.matches("CHROME"));

        // Should match with .exe extension
        assert!(matcher.matches("chrome.exe"));
        assert!(matcher.matches("Chrome.exe"));

        // Should NOT match substrings
        assert!(!matcher.matches("chromedriver"));
        assert!(!matcher.matches("chromebook"));
        assert!(!matcher.matches("google-chrome"));
        assert!(!matcher.matches("mychrome"));
    }

    #[test]
    fn test_exact_match_with_exe_in_blocked_value() {
        let matcher = ProcessMatcher::new("chrome.exe".to_string(), "exact".to_string()).unwrap();

        // Should match both with and without .exe
        assert!(matcher.matches("chrome"));
        assert!(matcher.matches("chrome.exe"));
        assert!(matcher.matches("Chrome"));
        assert!(matcher.matches("CHROME.EXE"));

        // Should NOT match substrings
        assert!(!matcher.matches("chromedriver.exe"));
    }

    #[test]
    fn test_contains_match() {
        let matcher = ProcessMatcher::new("chrome".to_string(), "contains".to_string()).unwrap();

        // Should match exact name
        assert!(matcher.matches("chrome"));
        assert!(matcher.matches("Chrome.exe"));

        // Should match substrings (legacy behavior)
        assert!(matcher.matches("chromedriver"));
        assert!(matcher.matches("chromebook"));
        assert!(matcher.matches("google-chrome"));
    }

    #[test]
    fn test_regex_match() {
        let matcher = ProcessMatcher::new("^chrome$".to_string(), "regex".to_string()).unwrap();

        // Should match exact name only
        assert!(matcher.matches("chrome"));

        // Should NOT match substrings
        assert!(!matcher.matches("chromedriver"));
        assert!(!matcher.matches("chrome.exe"));
    }

    #[test]
    fn test_regex_match_complex() {
        let matcher = ProcessMatcher::new(
            r"^(chrome|firefox|safari)\.exe$".to_string(),
            "regex".to_string(),
        )
        .unwrap();

        // Should match any of the browsers with .exe
        assert!(matcher.matches("chrome.exe"));
        assert!(matcher.matches("firefox.exe"));
        assert!(matcher.matches("safari.exe"));

        // Should NOT match without .exe or wrong names
        assert!(!matcher.matches("chrome"));
        assert!(!matcher.matches("edge.exe"));
    }

    #[test]
    fn test_invalid_regex() {
        let result = ProcessMatcher::new("[invalid".to_string(), "regex".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_match_type_default() {
        // Unknown match type defaults to exact
        let matcher = ProcessMatcher::new("chrome".to_string(), "unknown".to_string()).unwrap();

        assert!(matcher.matches("chrome"));
        assert!(!matcher.matches("chromedriver"));
    }

    #[test]
    fn test_real_world_scenarios() {
        // Scenario 1: Blocking "slack" should not block "slack-helper"
        let matcher = ProcessMatcher::new("slack".to_string(), "exact".to_string()).unwrap();
        assert!(matcher.matches("slack"));
        assert!(matcher.matches("Slack"));
        assert!(matcher.matches("slack.exe"));
        assert!(!matcher.matches("slack-helper"));
        assert!(!matcher.matches("slack helper"));

        // Scenario 2: Blocking "discord" should not block "discord.js"
        let matcher = ProcessMatcher::new("discord".to_string(), "exact".to_string()).unwrap();
        assert!(matcher.matches("discord"));
        assert!(matcher.matches("Discord.exe"));
        assert!(!matcher.matches("discordjs"));

        // Scenario 3: Blocking "code" should not block "vscode"
        let matcher = ProcessMatcher::new("code".to_string(), "exact".to_string()).unwrap();
        assert!(matcher.matches("code"));
        assert!(matcher.matches("Code.exe"));
        assert!(!matcher.matches("vscode"));
        assert!(!matcher.matches("code-insiders"));
    }

    #[test]
    fn test_edge_cases() {
        let matcher = ProcessMatcher::new("a".to_string(), "exact".to_string()).unwrap();

        assert!(matcher.matches("a"));
        assert!(matcher.matches("A"));
        assert!(matcher.matches("a.exe"));
        assert!(!matcher.matches("ab"));
        assert!(!matcher.matches("ba"));
    }
}
