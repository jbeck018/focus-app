// BACKEND_EXAMPLE.rs - Example Rust backend implementation for permission checking
// This file shows how to implement the check_permissions command in your Tauri backend

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverallStatus {
    FullyFunctional,
    Degraded,
    NonFunctional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub hosts_file_writable: bool,
    pub hosts_file_error: Option<String>,
    pub process_monitoring_available: bool,
    pub process_monitoring_error: Option<String>,
    pub overall_status: OverallStatus,
}

/// Check if we can write to the system hosts file
fn check_hosts_file_access() -> (bool, Option<String>) {
    #[cfg(target_os = "windows")]
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";

    #[cfg(not(target_os = "windows"))]
    let hosts_path = "/etc/hosts";

    // Try to open the hosts file in append mode (non-destructive check)
    match OpenOptions::new().append(true).open(hosts_path) {
        Ok(_) => (true, None),
        Err(e) => {
            let error_msg = match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    format!("Permission denied: Cannot write to hosts file at {}", hosts_path)
                }
                std::io::ErrorKind::NotFound => {
                    format!("Hosts file not found at {}", hosts_path)
                }
                _ => {
                    format!("Cannot access hosts file: {}", e)
                }
            };
            (false, Some(error_msg))
        }
    }
}

/// Check if we can monitor running processes
fn check_process_monitoring() -> (bool, Option<String>) {
    #[cfg(target_os = "macos")]
    {
        // On macOS, we need Accessibility permissions to monitor processes
        // Try to list processes as a test
        use std::process::Command;

        match Command::new("ps").arg("-A").output() {
            Ok(output) => {
                if output.status.success() {
                    (true, None)
                } else {
                    (false, Some("Unable to list processes. Accessibility permissions may be required.".to_string()))
                }
            }
            Err(e) => (false, Some(format!("Process monitoring error: {}", e))),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, try to enumerate processes
        use std::process::Command;

        match Command::new("tasklist").output() {
            Ok(output) => {
                if output.status.success() {
                    (true, None)
                } else {
                    (false, Some("Unable to list processes. Admin privileges may be required.".to_string()))
                }
            }
            Err(e) => (false, Some(format!("Process monitoring error: {}", e))),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, check if we can read /proc
        use std::path::Path;

        if Path::new("/proc").exists() {
            match std::fs::read_dir("/proc") {
                Ok(_) => (true, None),
                Err(e) => (false, Some(format!("Cannot read /proc: {}", e))),
            }
        } else {
            (false, Some("/proc filesystem not available".to_string()))
        }
    }
}

/// Main permission check command
#[tauri::command]
pub fn check_permissions() -> Result<PermissionStatus, String> {
    // Check hosts file access
    let (hosts_file_writable, hosts_file_error) = check_hosts_file_access();

    // Check process monitoring
    let (process_monitoring_available, process_monitoring_error) = check_process_monitoring();

    // Determine overall status
    let overall_status = match (hosts_file_writable, process_monitoring_available) {
        (true, true) => OverallStatus::FullyFunctional,
        (true, false) | (false, true) => OverallStatus::Degraded,
        (false, false) => OverallStatus::NonFunctional,
    };

    Ok(PermissionStatus {
        hosts_file_writable,
        hosts_file_error,
        process_monitoring_available,
        process_monitoring_error,
        overall_status,
    })
}

// ============================================================================
// INTEGRATION INTO YOUR TAURI APP
// ============================================================================

/*
In your main.rs or lib.rs, register the command:

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_permissions,
            // ... your other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Optional: Add periodic permission checks
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Check permissions on startup
            let window = app.get_window("main").unwrap();
            tauri::async_runtime::spawn(async move {
                // Emit permission status to frontend
                if let Ok(status) = check_permissions() {
                    window.emit("permission-status", status).ok();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_permissions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
*/

// ============================================================================
// ENHANCED PERMISSION CHECKS WITH MORE DETAILS
// ============================================================================

/// More comprehensive permission check with additional platform details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedPermissionStatus {
    pub hosts_file: HostsFileStatus,
    pub process_monitoring: ProcessMonitoringStatus,
    pub overall_status: OverallStatus,
    pub platform: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsFileStatus {
    pub writable: bool,
    pub path: String,
    pub error: Option<String>,
    pub test_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMonitoringStatus {
    pub available: bool,
    pub error: Option<String>,
    pub test_passed: bool,
    pub process_count: Option<usize>,
}

#[tauri::command]
pub fn check_permissions_detailed() -> Result<DetailedPermissionStatus, String> {
    let platform = std::env::consts::OS.to_string();
    let mut recommendations = Vec::new();

    // Check hosts file
    #[cfg(target_os = "windows")]
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";

    #[cfg(not(target_os = "windows"))]
    let hosts_path = "/etc/hosts";

    let (hosts_writable, hosts_error) = check_hosts_file_access();

    if !hosts_writable {
        #[cfg(target_os = "macos")]
        recommendations.push("Run: sudo chmod 644 /etc/hosts".to_string());

        #[cfg(target_os = "windows")]
        recommendations.push("Run FocusFlow as Administrator".to_string());

        #[cfg(target_os = "linux")]
        recommendations.push("Run: sudo chmod 644 /etc/hosts".to_string());
    }

    let hosts_file_status = HostsFileStatus {
        writable: hosts_writable,
        path: hosts_path.to_string(),
        error: hosts_error,
        test_passed: hosts_writable,
    };

    // Check process monitoring
    let (process_available, process_error) = check_process_monitoring();

    if !process_available {
        #[cfg(target_os = "macos")]
        recommendations.push("Grant Accessibility permissions in System Preferences".to_string());

        #[cfg(target_os = "windows")]
        recommendations.push("Add FocusFlow to antivirus exceptions".to_string());

        #[cfg(target_os = "linux")]
        recommendations.push("Ensure /proc filesystem is accessible".to_string());
    }

    let process_monitoring_status = ProcessMonitoringStatus {
        available: process_available,
        error: process_error,
        test_passed: process_available,
        process_count: None, // Could be populated with actual count
    };

    // Determine overall status
    let overall_status = match (hosts_writable, process_available) {
        (true, true) => OverallStatus::FullyFunctional,
        (true, false) | (false, true) => OverallStatus::Degraded,
        (false, false) => OverallStatus::NonFunctional,
    };

    Ok(DetailedPermissionStatus {
        hosts_file: hosts_file_status,
        process_monitoring: process_monitoring_status,
        overall_status,
        platform,
        recommendations,
    })
}

// ============================================================================
// HELPER FUNCTIONS FOR PERMISSION FIXING
// ============================================================================

/// Attempt to fix permissions (macOS/Linux only)
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn attempt_fix_permissions() -> Result<String, String> {
    use std::process::Command;

    // Try to fix hosts file permissions
    let output = Command::new("osascript")
        .arg("-e")
        .arg("do shell script \"chmod 644 /etc/hosts\" with administrator privileges")
        .output()
        .map_err(|e| format!("Failed to execute permission fix: {}", e))?;

    if output.status.success() {
        Ok("Permissions fixed successfully. Please restart FocusFlow.".to_string())
    } else {
        Err("Failed to fix permissions. Please follow manual instructions.".to_string())
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn attempt_fix_permissions() -> Result<String, String> {
    Err("Automatic permission fixing not supported on Windows. Please run as Administrator.".to_string())
}
