// blocking/capabilities.rs - Detection and management of blocking capabilities
//
// This module checks what blocking methods are available based on system
// permissions and provides fallback strategies when elevated privileges
// are not available.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Blocking method availability and capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingCapabilities {
    /// Can we modify the hosts file?
    pub hosts_file_writable: bool,

    /// Path to the hosts file
    pub hosts_file_path: String,

    /// Can we terminate processes?
    pub process_termination_available: bool,

    /// Recommended blocking method based on available capabilities
    pub recommended_method: BlockingMethod,

    /// All available blocking methods
    pub available_methods: Vec<BlockingMethod>,

    /// Why certain methods might not be available
    pub limitations: Vec<String>,

    /// Current operating system
    pub platform: String,
}

/// Available blocking methods in order of effectiveness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockingMethod {
    /// Hosts file modification (most effective, requires elevation)
    HostsFile,

    /// Process termination (effective for apps, may require elevation)
    ProcessTermination,

    /// Frontend-based blocking (least secure, no elevation needed)
    FrontendOnly,
}

/// Platform-specific elevation instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationInstructions {
    /// Operating system
    pub platform: String,

    /// Primary method to gain necessary permissions
    pub primary_method: String,

    /// Alternative methods
    pub alternative_methods: Vec<String>,

    /// Step-by-step instructions
    pub steps: Vec<String>,

    /// Security considerations
    pub security_notes: Vec<String>,

    /// Whether the app needs to be restarted after elevation
    pub requires_restart: bool,
}

/// Check all blocking capabilities on the current system
pub async fn check_capabilities() -> BlockingCapabilities {
    let hosts_writable = check_hosts_file_writable().await;
    let process_termination = check_process_termination_available();
    let platform = get_platform_name();

    let mut available_methods = Vec::new();
    let mut limitations = Vec::new();

    // Check hosts file capability
    if hosts_writable {
        available_methods.push(BlockingMethod::HostsFile);
    } else {
        limitations.push(format!(
            "Hosts file at {} is not writable. Website blocking requires elevated privileges.",
            get_hosts_path().display()
        ));
    }

    // Check process termination capability
    if process_termination {
        available_methods.push(BlockingMethod::ProcessTermination);
    } else {
        limitations.push(
            "Process termination may require elevated privileges on this system.".to_string()
        );
    }

    // Frontend-only is always available as a fallback
    available_methods.push(BlockingMethod::FrontendOnly);

    // Determine recommended method
    let recommended_method = if hosts_writable {
        BlockingMethod::HostsFile
    } else if process_termination {
        BlockingMethod::ProcessTermination
    } else {
        limitations.push(
            "Using frontend-only blocking. This is less secure and can be bypassed.".to_string()
        );
        BlockingMethod::FrontendOnly
    };

    BlockingCapabilities {
        hosts_file_writable: hosts_writable,
        hosts_file_path: get_hosts_path().display().to_string(),
        process_termination_available: process_termination,
        recommended_method,
        available_methods,
        limitations,
        platform,
    }
}

/// Check if we can write to the hosts file
pub async fn check_hosts_file_writable() -> bool {
    let hosts_path = get_hosts_path();

    // First check if file exists and is readable
    if !hosts_path.exists() {
        tracing::warn!("Hosts file does not exist at {}", hosts_path.display());
        return false;
    }

    // Try to read the file
    if tokio::fs::read_to_string(&hosts_path).await.is_err() {
        tracing::warn!("Cannot read hosts file at {}", hosts_path.display());
        return false;
    }

    // Try to open the file in write mode (but don't actually write)
    // This is the most reliable way to check write permissions
    match std::fs::OpenOptions::new()
        .write(true)
        .open(&hosts_path)
    {
        Ok(_) => {
            tracing::info!("Hosts file is writable at {}", hosts_path.display());
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            tracing::warn!("Permission denied for hosts file at {}", hosts_path.display());
            false
        }
        Err(e) => {
            tracing::warn!("Error checking hosts file writability: {}", e);
            false
        }
    }
}

/// Check if process termination is available
fn check_process_termination_available() -> bool {
    // Process termination is generally available on all platforms
    // but may require elevation for system processes
    // We'll be optimistic here and report true, with actual termination
    // attempts handling permission errors gracefully
    true
}

/// Get platform-specific hosts file path
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

/// Get current platform name
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

/// Get platform-specific elevation instructions
pub fn get_elevation_instructions() -> ElevationInstructions {
    #[cfg(target_os = "macos")]
    {
        return ElevationInstructions {
            platform: "macOS".to_string(),
            primary_method: "Grant Full Disk Access".to_string(),
            alternative_methods: vec![
                "Run with sudo (temporary)".to_string(),
                "Grant automation permissions".to_string(),
            ],
            steps: vec![
                "Open System Settings > Privacy & Security > Full Disk Access".to_string(),
                "Click the lock icon and enter your password".to_string(),
                "Click the '+' button and add FocusFlow from Applications".to_string(),
                "Restart FocusFlow for changes to take effect".to_string(),
            ],
            security_notes: vec![
                "Full Disk Access allows FocusFlow to modify system files like /etc/hosts".to_string(),
                "This is required for effective website blocking".to_string(),
                "FocusFlow only modifies the hosts file and does not access other system files".to_string(),
                "Similar permissions are required by apps like Cold Turkey and SelfControl".to_string(),
            ],
            requires_restart: true,
        };
    }

    #[cfg(target_os = "windows")]
    {
        return ElevationInstructions {
            platform: "Windows".to_string(),
            primary_method: "Run as Administrator".to_string(),
            alternative_methods: vec![
                "Set to always run as administrator".to_string(),
            ],
            steps: vec![
                "Right-click the FocusFlow icon".to_string(),
                "Select 'Run as administrator'".to_string(),
                "Or: Right-click > Properties > Compatibility".to_string(),
                "Check 'Run this program as an administrator'".to_string(),
                "Click OK and restart FocusFlow".to_string(),
            ],
            security_notes: vec![
                "Administrator access is required to modify the Windows hosts file".to_string(),
                "Located at C:\\Windows\\System32\\drivers\\etc\\hosts".to_string(),
                "FocusFlow only modifies the hosts file for website blocking".to_string(),
                "This is the same permission required by Freedom, Cold Turkey, and similar apps".to_string(),
            ],
            requires_restart: true,
        };
    }

    #[cfg(target_os = "linux")]
    {
        return ElevationInstructions {
            platform: "Linux".to_string(),
            primary_method: "Run with sudo or add user to hosts file group".to_string(),
            alternative_methods: vec![
                "Create a sudoers rule for /etc/hosts".to_string(),
                "Use capabilities with setcap".to_string(),
            ],
            steps: vec![
                "Option 1 - Run with sudo:".to_string(),
                "  sudo focusflow".to_string(),
                "".to_string(),
                "Option 2 - Make hosts file writable (less secure):".to_string(),
                "  sudo chmod 666 /etc/hosts".to_string(),
                "".to_string(),
                "Option 3 - Create sudoers rule (recommended):".to_string(),
                "  sudo visudo".to_string(),
                "  Add: your_username ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts".to_string(),
            ],
            security_notes: vec![
                "Root access is required to modify /etc/hosts on Linux".to_string(),
                "The sudoers rule option is most secure".to_string(),
                "Making /etc/hosts world-writable is not recommended for security".to_string(),
                "FocusFlow only modifies the hosts file for website blocking".to_string(),
            ],
            requires_restart: false,
        };
    }

    #[allow(unreachable_code)]
    ElevationInstructions {
        platform: "Unknown".to_string(),
        primary_method: "Platform not supported".to_string(),
        alternative_methods: vec![],
        steps: vec![],
        security_notes: vec![],
        requires_restart: false,
    }
}

/// Validate that we have the required capabilities for a blocking method
///
/// Currently unused - validation happens at runtime in blocking commands.
/// Kept for potential future use in UI permission checking.
#[allow(dead_code)]
pub async fn validate_blocking_method(method: &BlockingMethod) -> Result<()> {
    match method {
        BlockingMethod::HostsFile => {
            if !check_hosts_file_writable().await {
                return Err(Error::PermissionDenied(
                    "Hosts file is not writable. Please grant elevated privileges.".to_string()
                ));
            }
        }
        BlockingMethod::ProcessTermination => {
            // Process termination is generally available
            // Actual permission errors will be handled during termination attempts
        }
        BlockingMethod::FrontendOnly => {
            // Always available
        }
    }

    Ok(())
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
    fn test_get_elevation_instructions() {
        let instructions = get_elevation_instructions();
        assert!(!instructions.platform.is_empty());
        assert!(!instructions.primary_method.is_empty());
        assert!(!instructions.steps.is_empty());
    }

    #[tokio::test]
    async fn test_check_capabilities() {
        let capabilities = check_capabilities().await;

        // Should always have at least one method available (FrontendOnly)
        assert!(!capabilities.available_methods.is_empty());
        assert!(capabilities.available_methods.contains(&BlockingMethod::FrontendOnly));

        // Platform should be detected
        assert!(!capabilities.platform.is_empty());
    }
}
