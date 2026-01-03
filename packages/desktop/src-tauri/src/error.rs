// error.rs - Centralized error handling using thiserror

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application-wide Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the application
///
/// Uses thiserror for ergonomic error handling and automatic Display impl.
/// All errors are serializable for safe transmission to the frontend.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Invalid session: {0}")]
    InvalidSession(String),

    #[error("Session limit reached: {0}")]
    SessionLimitReached(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("System error: {0}")]
    System(String),

    #[error("Hosts file error: {0}")]
    HostsFile(String),

    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Window error: {0}")]
    Window(String),

    #[error("AI error: {0}")]
    Ai(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

// Implement conversions from common error types
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Database(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => {
                Error::PermissionDenied(err.to_string())
            }
            std::io::ErrorKind::NotFound => {
                Error::Io(format!("File not found: {}", err))
            }
            _ => Error::Io(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<tauri::Error> for Error {
    fn from(err: tauri::Error) -> Self {
        Error::System(err.to_string())
    }
}

