// commands/permissions.rs - Comprehensive permission checking for blocking features
//
// This module provides detailed permission status checks and platform-specific
// instructions for granting the necessary permissions for FocusFlow's blocking features.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Comprehensive permission status for all blocking capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    /// Can we modify the hosts file for website blocking?
    pub hosts_file_writable: bool,

    /// Detailed error if hosts file is not writable
    pub hosts_file_error: Option<String>,

    /// Path to the hosts file for reference
    pub hosts_file_path: String,

    /// Can we enumerate and monitor running processes?
    pub process_monitoring_available: bool,

    /// Detailed error if process monitoring is not available
    pub process_monitoring_error: Option<String>,

    /// Can we terminate blocked processes?
    pub process_termination_available: bool,

    /// Detailed error if process termination is not available
    pub process_termination_error: Option<String>,

    /// Overall assessment of the blocking system's functionality
    pub overall_status: OverallPermissionStatus,

    /// Recommendations for the user
    pub recommendations: Vec<String>,

    /// Current platform
    pub platform: String,
}

/// Overall permission status categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverallPermissionStatus {
    /// All blocking features work perfectly
    FullyFunctional,

    /// Some features work, but not all (e.g., process blocking works but not hosts file)
    Degraded,

    /// No privileged features work, only frontend-based fallbacks available
    NonFunctional,
}

/// Platform-specific instructions for granting permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInstructions {
    /// Operating system name
    pub platform: String,

    /// Primary recommended method for granting permissions
    pub primary_method: PermissionMethod,

    /// Alternative methods available
    pub alternative_methods: Vec<PermissionMethod>,

    /// Whether app needs restart after granting permissions
    pub requires_restart: bool,

    /// General security notes about the permissions
    pub security_notes: Vec<String>,
}

/// A specific method for granting permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionMethod {
    /// Name of the method (e.g., "Full Disk Access", "Run as Administrator")
    pub name: String,

    /// Step-by-step instructions
    pub steps: Vec<String>,

    /// Whether this is a permanent solution or temporary
    pub is_permanent: bool,

    /// Whether this method is recommended (vs alternative)
    pub is_recommended: bool,

    /// Specific capability this method grants
    pub grants: Vec<String>,
}

/// Check all permissions and return detailed status
///
/// This command provides a comprehensive overview of what blocking features
/// are currently available based on the app's permissions. It checks:
/// - Hosts file write access
/// - Process enumeration capabilities
/// - Process termination capabilities
///
/// Returns detailed error messages for any permission issues to help guide
/// the user toward granting the necessary permissions.
#[tauri::command]
pub async fn check_permissions() -> Result<PermissionStatus> {
    let platform = get_platform_name();

    // Check hosts file permissions
    let (hosts_writable, hosts_error) = check_hosts_file_detailed().await;

    // Check process monitoring permissions
    let (process_monitoring, monitoring_error) = check_process_monitoring_detailed();

    // Check process termination permissions
    let (process_termination, termination_error) = check_process_termination_detailed();

    // Determine overall status
    let overall_status = if hosts_writable && process_monitoring && process_termination {
        OverallPermissionStatus::FullyFunctional
    } else if process_monitoring || process_termination {
        OverallPermissionStatus::Degraded
    } else {
        OverallPermissionStatus::NonFunctional
    };

    // Build recommendations
    let mut recommendations = Vec::new();

    if !hosts_writable {
        recommendations.push(
            "Grant file system permissions to enable website blocking through hosts file."
                .to_string(),
        );
    }

    if !process_monitoring {
        recommendations.push(
            "Grant process monitoring permissions to enable app blocking.".to_string(),
        );
    }

    if overall_status == OverallPermissionStatus::NonFunctional {
        recommendations.push(
            "Consider using frontend-based blocking as a temporary fallback.".to_string(),
        );
    }

    if overall_status != OverallPermissionStatus::FullyFunctional {
        recommendations.push(format!(
            "See detailed setup instructions for {} to enable full blocking capabilities.",
            platform
        ));
    }

    let status = PermissionStatus {
        hosts_file_writable: hosts_writable,
        hosts_file_error: hosts_error,
        hosts_file_path: get_hosts_path().display().to_string(),
        process_monitoring_available: process_monitoring,
        process_monitoring_error: monitoring_error,
        process_termination_available: process_termination,
        process_termination_error: termination_error,
        overall_status: overall_status.clone(),
        recommendations,
        platform: platform.clone(),
    };

    // Log the permission check results
    tracing::info!(
        "Permission check complete: {:?} - hosts:{} process:{} termination:{}",
        overall_status,
        hosts_writable,
        process_monitoring,
        process_termination
    );

    Ok(status)
}

/// Get platform-specific instructions for granting permissions
///
/// Returns detailed, step-by-step instructions tailored to the user's
/// operating system. Includes both primary recommended methods and
/// alternative approaches.
#[tauri::command]
pub async fn get_permission_instructions(platform: String) -> Result<PlatformInstructions> {
    let instructions = match platform.to_lowercase().as_str() {
        "macos" | "darwin" => get_macos_instructions(),
        "windows" => get_windows_instructions(),
        "linux" => get_linux_instructions(),
        _ => {
            // Try to detect platform if not provided or invalid
            let detected_platform = get_platform_name();
            match detected_platform.as_str() {
                "macOS" => get_macos_instructions(),
                "Windows" => get_windows_instructions(),
                "Linux" => get_linux_instructions(),
                _ => {
                    return Err(Error::System(format!(
                        "Unsupported platform: {}",
                        platform
                    )))
                }
            }
        }
    };

    tracing::info!(
        "Providing permission instructions for platform: {}",
        instructions.platform
    );

    Ok(instructions)
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Check hosts file with detailed error information
async fn check_hosts_file_detailed() -> (bool, Option<String>) {
    let hosts_path = get_hosts_path();

    // Check if file exists
    if !hosts_path.exists() {
        return (
            false,
            Some(format!(
                "Hosts file not found at {}",
                hosts_path.display()
            )),
        );
    }

    // Try to read the file
    if let Err(e) = tokio::fs::read_to_string(&hosts_path).await {
        return (
            false,
            Some(format!("Cannot read hosts file: {}", e.to_string())),
        );
    }

    // Try to open in write mode (without actually writing)
    match std::fs::OpenOptions::new()
        .write(true)
        .open(&hosts_path)
    {
        Ok(_) => {
            tracing::debug!("Hosts file is writable");
            (true, None)
        }
        Err(e) => {
            let error_msg = match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    "Permission denied. Elevated privileges required.".to_string()
                }
                std::io::ErrorKind::NotFound => {
                    "File not found after existence check (race condition?)".to_string()
                }
                _ => format!("Unexpected error: {}", e),
            };

            tracing::warn!(
                "Hosts file not writable at {}: {}",
                hosts_path.display(),
                error_msg
            );
            (false, Some(error_msg))
        }
    }
}

/// Check process monitoring with detailed error information
fn check_process_monitoring_detailed() -> (bool, Option<String>) {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let count = system.processes().len();
        count > 0
    })) {
        Ok(true) => {
            tracing::debug!("Process monitoring is available");
            (true, None)
        }
        Ok(false) => {
            let error_msg = "Process list is empty (unexpected)".to_string();
            tracing::warn!("Process monitoring check failed: {}", error_msg);
            (false, Some(error_msg))
        }
        Err(_) => {
            let error_msg = "Process enumeration panicked (likely permission issue)".to_string();
            tracing::error!("Process monitoring check panicked: {}", error_msg);
            (false, Some(error_msg))
        }
    }
}

/// Check process termination with detailed error information
fn check_process_termination_detailed() -> (bool, Option<String>) {
    // Process termination is generally available on all platforms
    // We can't test it without actually terminating a process,
    // so we check if we can at least enumerate processes as a proxy
    let (can_monitor, monitor_error) = check_process_monitoring_detailed();

    if !can_monitor {
        return (
            false,
            Some(format!(
                "Cannot enumerate processes: {}",
                monitor_error.unwrap_or_else(|| "Unknown error".to_string())
            )),
        );
    }

    // On most platforms, if we can enumerate processes, we can terminate user processes
    // System processes may still require elevation, but that's handled at runtime
    tracing::debug!("Process termination assumed available (can enumerate processes)");
    (true, None)
}

/// Get platform name
fn get_platform_name() -> String {
    #[cfg(target_os = "macos")]
    return "macOS".to_string();

    #[cfg(target_os = "windows")]
    return "Windows".to_string();

    #[cfg(target_os = "linux")]
    return "Linux".to_string();

    #[allow(unreachable_code)]
    "Unknown".to_string()
}

/// Get hosts file path for current platform
fn get_hosts_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/hosts")
    }
}

// ============================================================================
// Platform-Specific Instructions
// ============================================================================

/// Get macOS-specific permission instructions
fn get_macos_instructions() -> PlatformInstructions {
    let primary = PermissionMethod {
        name: "Grant Full Disk Access".to_string(),
        steps: vec![
            "Open System Settings (or System Preferences on older macOS)".to_string(),
            "Navigate to Privacy & Security > Full Disk Access".to_string(),
            "Click the lock icon in the bottom left and authenticate".to_string(),
            "Click the '+' button to add an application".to_string(),
            "Navigate to Applications and select FocusFlow".to_string(),
            "Ensure the checkbox next to FocusFlow is enabled".to_string(),
            "Close System Settings and restart FocusFlow".to_string(),
        ],
        is_permanent: true,
        is_recommended: true,
        grants: vec![
            "Read/write access to /etc/hosts for website blocking".to_string(),
            "Process monitoring and termination capabilities".to_string(),
        ],
    };

    let sudo_method = PermissionMethod {
        name: "Run with sudo (Temporary)".to_string(),
        steps: vec![
            "Close FocusFlow if it's currently running".to_string(),
            "Open Terminal application".to_string(),
            "Navigate to FocusFlow.app: cd /Applications/FocusFlow.app/Contents/MacOS".to_string(),
            "Run with sudo: sudo ./FocusFlow".to_string(),
            "Enter your password when prompted".to_string(),
            "Note: This grants temporary permissions for this session only".to_string(),
        ],
        is_permanent: false,
        is_recommended: false,
        grants: vec!["Temporary elevated access for this session".to_string()],
    };

    PlatformInstructions {
        platform: "macOS".to_string(),
        primary_method: primary,
        alternative_methods: vec![sudo_method],
        requires_restart: true,
        security_notes: vec![
            "Full Disk Access allows FocusFlow to modify system files like /etc/hosts"
                .to_string(),
            "FocusFlow only modifies the hosts file and does not access other system files"
                .to_string(),
            "This permission is required by all effective website blockers on macOS".to_string(),
            "Similar apps like Cold Turkey and SelfControl require the same permissions"
                .to_string(),
            "You can revoke this permission at any time from System Settings".to_string(),
        ],
    }
}

/// Get Windows-specific permission instructions
fn get_windows_instructions() -> PlatformInstructions {
    let primary = PermissionMethod {
        name: "Set to Always Run as Administrator".to_string(),
        steps: vec![
            "Close FocusFlow if it's currently running".to_string(),
            "Right-click the FocusFlow desktop shortcut or Start Menu icon".to_string(),
            "Select 'Properties' from the context menu".to_string(),
            "Navigate to the 'Compatibility' tab".to_string(),
            "Check the box 'Run this program as an administrator'".to_string(),
            "Click 'Apply' and then 'OK'".to_string(),
            "Launch FocusFlow - you may see a UAC prompt, click 'Yes'".to_string(),
        ],
        is_permanent: true,
        is_recommended: true,
        grants: vec![
            "Administrator access to modify C:\\Windows\\System32\\drivers\\etc\\hosts"
                .to_string(),
            "Process monitoring and termination capabilities".to_string(),
        ],
    };

    let temp_method = PermissionMethod {
        name: "Run as Administrator (One Time)".to_string(),
        steps: vec![
            "Right-click the FocusFlow icon".to_string(),
            "Select 'Run as administrator'".to_string(),
            "Click 'Yes' on the User Account Control (UAC) prompt".to_string(),
            "Note: This grants temporary permissions for this session only".to_string(),
        ],
        is_permanent: false,
        is_recommended: false,
        grants: vec!["Temporary administrator access for this session".to_string()],
    };

    PlatformInstructions {
        platform: "Windows".to_string(),
        primary_method: primary,
        alternative_methods: vec![temp_method],
        requires_restart: true,
        security_notes: vec![
            "Administrator access is required to modify the Windows hosts file".to_string(),
            "The hosts file is located at C:\\Windows\\System32\\drivers\\etc\\hosts"
                .to_string(),
            "FocusFlow only modifies the hosts file for website blocking purposes".to_string(),
            "This is the same permission required by Freedom, Cold Turkey, and similar apps"
                .to_string(),
            "Windows will show a UAC prompt each time FocusFlow starts (this is normal)"
                .to_string(),
        ],
    }
}

/// Get Linux-specific permission instructions
fn get_linux_instructions() -> PlatformInstructions {
    let primary = PermissionMethod {
        name: "Create sudoers rule (Recommended)".to_string(),
        steps: vec![
            "Open a terminal".to_string(),
            "Run: sudo visudo".to_string(),
            "Add this line at the end (replace 'username' with your username):".to_string(),
            "  username ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts".to_string(),
            "Save and exit (Ctrl+X, then Y, then Enter in nano)".to_string(),
            "Restart FocusFlow".to_string(),
        ],
        is_permanent: true,
        is_recommended: true,
        grants: vec![
            "Passwordless sudo access for modifying /etc/hosts".to_string(),
            "Process monitoring and termination capabilities".to_string(),
        ],
    };

    let sudo_method = PermissionMethod {
        name: "Run with sudo".to_string(),
        steps: vec![
            "Open a terminal".to_string(),
            "Run: sudo focusflow".to_string(),
            "Enter your password when prompted".to_string(),
            "Note: You'll need to do this every time you launch FocusFlow".to_string(),
        ],
        is_permanent: false,
        is_recommended: false,
        grants: vec!["Temporary root access for this session".to_string()],
    };

    let chmod_method = PermissionMethod {
        name: "Make hosts file world-writable (Not Recommended)".to_string(),
        steps: vec![
            "Open a terminal".to_string(),
            "Run: sudo chmod 666 /etc/hosts".to_string(),
            "Warning: This is a security risk as any program can modify your hosts file"
                .to_string(),
            "Only use this if you understand the security implications".to_string(),
        ],
        is_permanent: true,
        is_recommended: false,
        grants: vec!["Write access to /etc/hosts for all users (security risk)".to_string()],
    };

    PlatformInstructions {
        platform: "Linux".to_string(),
        primary_method: primary,
        alternative_methods: vec![sudo_method, chmod_method],
        requires_restart: false,
        security_notes: vec![
            "Root access is required to modify /etc/hosts on Linux".to_string(),
            "The sudoers rule option is the most secure approach".to_string(),
            "Making /etc/hosts world-writable is NOT recommended for security reasons"
                .to_string(),
            "FocusFlow only modifies the hosts file for website blocking".to_string(),
            "Different Linux distributions may have different DNS caching mechanisms"
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_name() {
        let platform = get_platform_name();
        assert!(!platform.is_empty());
        assert!(platform == "macOS" || platform == "Windows" || platform == "Linux");
    }

    #[test]
    fn test_get_hosts_path() {
        let path = get_hosts_path();
        assert!(path.as_os_str().len() > 0);

        #[cfg(target_os = "windows")]
        assert!(path
            .to_string_lossy()
            .contains("Windows\\System32\\drivers\\etc\\hosts"));

        #[cfg(not(target_os = "windows"))]
        assert_eq!(path.to_str().unwrap(), "/etc/hosts");
    }

    #[tokio::test]
    async fn test_check_permissions() {
        let result = check_permissions().await;
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(!status.platform.is_empty());
        assert!(!status.hosts_file_path.is_empty());

        // Should always have at least some status
        match status.overall_status {
            OverallPermissionStatus::FullyFunctional
            | OverallPermissionStatus::Degraded
            | OverallPermissionStatus::NonFunctional => {}
        }
    }

    #[tokio::test]
    async fn test_get_permission_instructions_macos() {
        let result = get_permission_instructions("macos".to_string()).await;
        assert!(result.is_ok());

        let instructions = result.unwrap();
        assert_eq!(instructions.platform, "macOS");
        assert!(!instructions.primary_method.steps.is_empty());
        assert!(!instructions.alternative_methods.is_empty());
        assert!(!instructions.security_notes.is_empty());
    }

    #[tokio::test]
    async fn test_get_permission_instructions_windows() {
        let result = get_permission_instructions("windows".to_string()).await;
        assert!(result.is_ok());

        let instructions = result.unwrap();
        assert_eq!(instructions.platform, "Windows");
        assert!(instructions.primary_method.is_recommended);
    }

    #[tokio::test]
    async fn test_get_permission_instructions_linux() {
        let result = get_permission_instructions("linux".to_string()).await;
        assert!(result.is_ok());

        let instructions = result.unwrap();
        assert_eq!(instructions.platform, "Linux");
        assert!(instructions.alternative_methods.len() >= 2);
    }

    #[tokio::test]
    async fn test_get_permission_instructions_auto_detect() {
        // Passing empty or invalid platform should auto-detect
        let result = get_permission_instructions("".to_string()).await;
        assert!(result.is_ok());

        let instructions = result.unwrap();
        assert!(!instructions.platform.is_empty());
    }

    #[test]
    fn test_check_process_monitoring_detailed() {
        let (available, error) = check_process_monitoring_detailed();

        // Process monitoring should generally be available
        // This might fail in restricted environments, but that's okay for testing
        if available {
            assert!(error.is_none());
        } else {
            assert!(error.is_some());
        }
    }

    #[tokio::test]
    async fn test_check_hosts_file_detailed() {
        let (writable, error) = check_hosts_file_detailed().await;

        // Either writable with no error, or not writable with an error
        if writable {
            assert!(error.is_none());
        } else {
            assert!(error.is_some());
            let error_msg = error.unwrap();
            assert!(!error_msg.is_empty());
        }
    }

    #[test]
    fn test_overall_status_serialization() {
        use serde_json;

        let status = OverallPermissionStatus::FullyFunctional;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"fully_functional\"");

        let status = OverallPermissionStatus::Degraded;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"degraded\"");

        let status = OverallPermissionStatus::NonFunctional;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"non_functional\"");
    }
}
