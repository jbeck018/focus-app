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

/// Critical system processes that should never be terminated
/// Platform-specific protection lists to prevent accidental system damage
#[cfg(target_os = "windows")]
const PROTECTED_PROCESSES: &[&str] = &[
    "system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "dwm.exe",
    "explorer.exe",
    "taskmgr.exe",
    "conhost.exe",
    "audiodg.exe",
    "fontdrvhost.exe",
    "spoolsv.exe",
    "runtimebroker.exe",
    "sihost.exe",
    "taskhostw.exe",
    "registry",
    "memory compression",
];

#[cfg(target_os = "macos")]
const PROTECTED_PROCESSES: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Dock",
    "Finder",
    "Activity Monitor",
    "Console",
    "Terminal",
    "iTerm2",
    "sysmond",
    "diskarbitrationd",
    "configd",
    "notifyd",
    "opendirectoryd",
    "powerd",
    "mds",
    "mds_stores",
    "mdworker",
    "coreaudiod",
    "hidd",
    "coreservicesd",
    "UserEventAgent",
];

#[cfg(target_os = "linux")]
const PROTECTED_PROCESSES: &[&str] = &[
    "systemd",
    "init",
    "kthreadd",
    "ksoftirqd",
    "kworker",
    "rcu_sched",
    "rcu_bh",
    "migration",
    "watchdog",
    "dbus-daemon",
    "X",
    "Xorg",
    "xfce4-session",
    "gnome-session",
    "kde-session",
    "lightdm",
    "gdm",
    "sddm",
    "systemd-logind",
    "systemd-journald",
    "systemd-udevd",
    "pulseaudio",
    "pipewire",
    "NetworkManager",
    "wpa_supplicant",
];

/// Default fallback for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
const PROTECTED_PROCESSES: &[&str] = &[];

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

/// Check if a process is protected from termination
///
/// Returns true if the process is a critical system process that should never be killed.
/// This prevents accidental system instability or crashes.
fn is_protected_process(process_name: &str) -> bool {
    let normalized = normalize_process_name(process_name);

    PROTECTED_PROCESSES.iter().any(|&protected| {
        let normalized_protected = normalize_process_name(protected);
        normalized == normalized_protected
    })
}

/// Check if a process is owned by the current user
///
/// Returns true if the process is owned by the current user.
/// System processes (PID < 1000 on Unix, user id 0/SYSTEM on Windows) are considered non-user processes.
#[cfg(target_os = "windows")]
fn is_user_owned_process(process: &sysinfo::Process) -> bool {
    // On Windows, check if the process is running under the current user
    // System processes typically run under SYSTEM or other service accounts
    // We can use the process user id to determine ownership

    // As a safety measure, if we can't determine ownership, we assume it's not user-owned
    if let Some(user_id) = process.user_id() {
        // Get current user's SID (simplified check - system processes have well-known SIDs)
        let user_id_str = format!("{:?}", user_id);

        // System account SIDs start with S-1-5-18, S-1-5-19, S-1-5-20
        // Local Service: S-1-5-19
        // Network Service: S-1-5-20
        // System: S-1-5-18
        if user_id_str.starts_with("S-1-5-18") ||
           user_id_str.starts_with("S-1-5-19") ||
           user_id_str.starts_with("S-1-5-20") {
            return false;
        }

        true
    } else {
        // If we can't determine ownership, err on the side of caution
        false
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_user_owned_process(process: &sysinfo::Process) -> bool {
    // On Unix-like systems, check if the process UID matches current user
    // System processes typically run as root (UID 0) or low UIDs (< 1000)

    if let Some(user_id) = process.user_id() {
        // Get the numeric UID
        let user_id_str = format!("{:?}", user_id);

        // Try to parse as a number
        if let Ok(uid) = user_id_str.parse::<u32>() {
            // System processes typically have UID < 1000
            // User processes have UID >= 1000 on most systems
            return uid >= 1000;
        }
    }

    // If we can't determine ownership, err on the side of caution
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn is_user_owned_process(_process: &sysinfo::Process) -> bool {
    // On unsupported platforms, assume processes are not user-owned for safety
    false
}

/// Start the process monitoring loop
///
/// This runs in the background and checks for blocked processes at regular intervals.
/// Uses efficient delta updates from sysinfo to minimize CPU usage.
///
/// Supports two blocking modes:
/// 1. Standard blocking: Blocks specific apps in the blocklist
/// 2. Focus Time (inverse blocking): Blocks ALL apps EXCEPT those in the allowed list
pub async fn start_monitoring_loop(state: AppState) -> Result<()> {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    let mut interval = interval(Duration::from_millis(MONITOR_INTERVAL_MS));
    let mut warned_processes: HashSet<String> = HashSet::new();

    loop {
        interval.tick().await;

        // Check if standard blocking is enabled
        let blocking_enabled = {
            let blocking_state = state.blocking_state.read().await;
            blocking_state.enabled
        };

        // Check if Focus Time (inverse blocking) is active
        let focus_time_state = {
            let ft_state = state.focus_time_state.read().await;
            if ft_state.active {
                Some(ft_state.clone())
            } else {
                None
            }
        };

        // If neither mode is active, clear warnings and continue
        if !blocking_enabled && focus_time_state.is_none() {
            warned_processes.clear();
            continue;
        }

        // Refresh process list (only updates changed processes)
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        // Handle Focus Time inverse blocking (takes priority when active)
        if let Some(ref ft_state) = focus_time_state {
            for (pid, process) in system.processes() {
                let process_name = process.name().to_string_lossy();

                // Safety check: Skip protected system processes
                if is_protected_process(&process_name) {
                    continue;
                }

                // Safety check: Only terminate user-owned processes
                if !is_user_owned_process(process) {
                    continue;
                }

                // In Focus Time mode, check if app is NOT in the allowed list
                if !ft_state.is_app_allowed(&process_name) {
                    let process_key = format!("ft:{}:{}", process_name, pid);

                    if !warned_processes.contains(&process_key) {
                        tracing::warn!(
                            "Focus Time: Detected non-allowed process: {} (PID: {})",
                            process_name,
                            pid
                        );

                        if let Err(e) = state
                            .app_handle
                            .notification()
                            .builder()
                            .title("Focus Time: App Not Allowed")
                            .body(format!(
                                "{} is not in your Focus Time allowed list. It will be closed shortly.",
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

                            // Re-check if Focus Time is still active and app is still not allowed
                            let still_blocked = {
                                let ft_state = state_clone.focus_time_state.read().await;
                                ft_state.active && !ft_state.is_app_allowed(&process_name_copy)
                            };

                            if !still_blocked {
                                tracing::debug!(
                                    "Focus Time ended or app now allowed, skipping termination of {}",
                                    process_name_copy
                                );
                                return;
                            }

                            if let Err(e) = terminate_process(pid_copy) {
                                tracing::error!("Failed to terminate {}: {}", process_name_copy, e);
                            } else {
                                tracing::info!(
                                    "Focus Time: Terminated non-allowed process: {} (PID: {})",
                                    process_name_copy,
                                    pid_copy
                                );

                                // Record the block attempt
                                let session_id = {
                                    let active_session = state_clone.active_session.read().await;
                                    active_session.as_ref().map(|s| s.id.clone())
                                };

                                let user_id = state_clone.get_user_id().await;

                                if let Err(e) = queries::record_block_attempt(
                                    state_clone.pool(),
                                    "focus_time_app",
                                    &process_name_copy,
                                    session_id.as_deref(),
                                    user_id.as_deref(),
                                )
                                .await
                                {
                                    tracing::error!("Failed to record block attempt: {}", e);
                                }

                                let _ = app_handle
                                    .notification()
                                    .builder()
                                    .title("Focus Time: Application Closed")
                                    .body(format!(
                                        "{} was closed (not in allowed apps list).",
                                        process_name_copy
                                    ))
                                    .show();
                            }
                        });
                    }
                }
            }

            // Clean up Focus Time warned processes and continue to next iteration
            warned_processes.retain(|key| {
                if !key.starts_with("ft:") {
                    return true; // Keep non-focus-time entries
                }
                let parts: Vec<&str> = key.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(pid) = parts[2].parse::<usize>() {
                        return system.process(sysinfo::Pid::from(pid)).is_some();
                    }
                }
                false
            });

            // Update last check timestamp
            {
                let mut blocking_state = state.blocking_state.write().await;
                blocking_state.update_last_check();
            }

            // Skip standard blocking when Focus Time is active
            continue;
        }

        // Standard blocking mode (only when Focus Time is not active)
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

        // Check for blocked processes (standard mode)
        for (pid, process) in system.processes() {
            let process_name = process.name().to_string_lossy();

            // Safety check: Skip protected system processes
            if is_protected_process(&process_name) {
                continue;
            }

            // Safety check: Only terminate user-owned processes
            if !is_user_owned_process(process) {
                continue;
            }

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
/// Uses platform-specific APIs for graceful then forceful termination.
/// Includes safety checks to prevent terminating critical system processes.
fn terminate_process(pid: sysinfo::Pid) -> Result<()> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    if let Some(process) = system.process(pid) {
        let process_name = process.name().to_string_lossy();

        // Safety check 1: Verify it's not a protected system process
        if is_protected_process(&process_name) {
            return Err(Error::System(format!(
                "Refusing to terminate protected system process: {} (PID: {})",
                process_name, pid
            )));
        }

        // Safety check 2: Verify it's user-owned
        if !is_user_owned_process(process) {
            return Err(Error::System(format!(
                "Refusing to terminate non-user-owned process: {} (PID: {})",
                process_name, pid
            )));
        }

        // All safety checks passed, attempt termination
        tracing::debug!(
            "Terminating user process: {} (PID: {})",
            process_name,
            pid
        );

        // Try graceful termination first
        if process.kill() {
            Ok(())
        } else {
            Err(Error::System(format!(
                "Failed to terminate process {} (PID: {})",
                process_name, pid
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

    #[test]
    fn test_protected_processes() {
        // Test Windows protected processes
        #[cfg(target_os = "windows")]
        {
            assert!(is_protected_process("explorer.exe"));
            assert!(is_protected_process("Explorer.exe"));
            assert!(is_protected_process("EXPLORER.EXE"));
            assert!(is_protected_process("explorer"));
            assert!(is_protected_process("csrss.exe"));
            assert!(is_protected_process("winlogon.exe"));
            assert!(is_protected_process("system"));

            // Should not protect user applications
            assert!(!is_protected_process("chrome.exe"));
            assert!(!is_protected_process("notepad.exe"));
            assert!(!is_protected_process("slack.exe"));
        }

        // Test macOS protected processes
        #[cfg(target_os = "macos")]
        {
            assert!(is_protected_process("kernel_task"));
            assert!(is_protected_process("launchd"));
            assert!(is_protected_process("WindowServer"));
            assert!(is_protected_process("Finder"));
            assert!(is_protected_process("Dock"));

            // Should not protect user applications
            assert!(!is_protected_process("Chrome"));
            assert!(!is_protected_process("Slack"));
            assert!(!is_protected_process("Safari"));
        }

        // Test Linux protected processes
        #[cfg(target_os = "linux")]
        {
            assert!(is_protected_process("systemd"));
            assert!(is_protected_process("init"));
            assert!(is_protected_process("Xorg"));
            assert!(is_protected_process("dbus-daemon"));

            // Should not protect user applications
            assert!(!is_protected_process("chrome"));
            assert!(!is_protected_process("firefox"));
            assert!(!is_protected_process("slack"));
        }
    }

    #[test]
    fn test_protected_processes_normalization() {
        // Test that protected process checking uses same normalization
        #[cfg(target_os = "windows")]
        {
            // explorer.exe is in the protected list
            assert!(is_protected_process("explorer"));
            assert!(is_protected_process("explorer.exe"));
            assert!(is_protected_process("Explorer"));
            assert!(is_protected_process("EXPLORER"));
            assert!(is_protected_process("EXPLORER.EXE"));
        }
    }

    #[test]
    fn test_safety_prevents_system_damage() {
        // Verify that critical system processes are protected on all platforms

        #[cfg(target_os = "windows")]
        {
            // Windows critical processes
            let critical = ["system", "csrss.exe", "winlogon.exe", "services.exe", "lsass.exe"];
            for process in critical.iter() {
                assert!(
                    is_protected_process(process),
                    "Critical Windows process '{}' should be protected",
                    process
                );
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS critical processes
            let critical = ["kernel_task", "launchd", "WindowServer"];
            for process in critical.iter() {
                assert!(
                    is_protected_process(process),
                    "Critical macOS process '{}' should be protected",
                    process
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux critical processes
            let critical = ["systemd", "init", "kthreadd"];
            for process in critical.iter() {
                assert!(
                    is_protected_process(process),
                    "Critical Linux process '{}' should be protected",
                    process
                );
            }
        }
    }
}
