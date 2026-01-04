// state.rs - Application state management with Arc for thread-safe sharing

use crate::ai::{LlmEngine, ModelConfig};
use crate::oauth::{google::GoogleCalendar, microsoft::MicrosoftCalendar, Pkce, TokenManager};
use crate::trailbase::TrailBaseClient;
use crate::{commands::auth::AuthState, db::Database, Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Application state shared across all commands
///
/// Uses Arc for cheap cloning across async boundaries and RwLock for
/// interior mutability with reader-writer semantics for performance.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub active_session: Arc<RwLock<Option<ActiveSession>>>,
    pub timer_state: Arc<RwLock<TimerState>>,
    pub blocking_state: Arc<RwLock<BlockingState>>,
    pub strict_mode_state: Arc<RwLock<StrictModeState>>,
    pub nuclear_option_state: Arc<RwLock<NuclearOptionState>>,
    pub auth_state: Arc<RwLock<AuthState>>,
    pub llm_engine: Arc<RwLock<Option<LlmEngine>>>,
    pub trailbase_client: Arc<RwLock<Option<TrailBaseClient>>>,
    pub token_manager: Arc<TokenManager>,
    pub google_calendar: Arc<GoogleCalendar>,
    pub microsoft_calendar: Arc<MicrosoftCalendar>,
    pub oauth_flow_state: Arc<RwLock<HashMap<String, Pkce>>>,
    pub app_handle: tauri::AppHandle,
}

impl AppState {
    /// Initialize application state with database connection
    pub async fn new(app_handle: tauri::AppHandle) -> Result<Self> {
        let db_path = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| Error::Config(format!("Failed to get app data dir: {}", e)))?
            .join("focusflow.db");

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let db = Database::new(&db_path).await?;

        // Initialize LLM engine (lazy-loaded, won't download/load model yet)
        #[cfg(feature = "local-ai")]
        let llm_engine = {
            let models_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| Error::Config(format!("Failed to get app data dir: {}", e)))?
                .join("models");

            // Use Phi-3.5-mini as default model (can be changed via settings)
            match LlmEngine::new(models_dir, ModelConfig::phi_3_5_mini()) {
                Ok(engine) => {
                    info!("LLM engine initialized (model not yet loaded)");
                    Some(engine)
                }
                Err(e) => {
                    info!("LLM engine initialization skipped: {}", e);
                    None
                }
            }
        };

        #[cfg(not(feature = "local-ai"))]
        let llm_engine = {
            info!("LLM engine disabled (local-ai feature not enabled)");
            None
        };

        // Initialize OAuth providers
        // TODO: These client IDs should come from environment variables or config
        // For development, you can set them via environment variables:
        // GOOGLE_CLIENT_ID and MICROSOFT_CLIENT_ID
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_ID".to_string());
        let microsoft_client_id = std::env::var("MICROSOFT_CLIENT_ID")
            .unwrap_or_else(|_| "YOUR_MICROSOFT_CLIENT_ID".to_string());

        let token_manager = TokenManager::new(db.pool.clone());
        let google_calendar = GoogleCalendar::default(google_client_id);
        let microsoft_calendar = MicrosoftCalendar::default(microsoft_client_id);

        Ok(Self {
            db: Arc::new(db),
            active_session: Arc::new(RwLock::new(None)),
            timer_state: Arc::new(RwLock::new(TimerState::default())),
            blocking_state: Arc::new(RwLock::new(BlockingState::default())),
            strict_mode_state: Arc::new(RwLock::new(StrictModeState::default())),
            nuclear_option_state: Arc::new(RwLock::new(NuclearOptionState::default())),
            auth_state: Arc::new(RwLock::new(AuthState::new())),
            llm_engine: Arc::new(RwLock::new(llm_engine)),
            trailbase_client: Arc::new(RwLock::new(None)),
            token_manager: Arc::new(token_manager),
            google_calendar: Arc::new(google_calendar),
            microsoft_calendar: Arc::new(microsoft_calendar),
            oauth_flow_state: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        })
    }

    /// Get database connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.db.pool
    }

    /// Get current user ID from auth state (if authenticated)
    pub async fn get_user_id(&self) -> Option<String> {
        let auth = self.auth_state.read().await;
        auth.user.as_ref().map(|u| u.id.clone())
    }

    /// Get TrailBase client (if connected)
    pub async fn get_trailbase_client(&self) -> Option<TrailBaseClient> {
        let client = self.trailbase_client.read().await;
        client.clone()
    }

    /// Set TrailBase client
    pub async fn set_trailbase_client(&self, client: Option<TrailBaseClient>) {
        let mut trailbase_client = self.trailbase_client.write().await;
        *trailbase_client = client;
    }
}

/// Represents an active focus session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub planned_duration_minutes: i32,
    pub session_type: SessionType,
    pub blocked_apps: Vec<String>,
    pub blocked_websites: Vec<String>,
}

impl ActiveSession {
    pub fn new(
        planned_duration_minutes: i32,
        session_type: SessionType,
        blocked_apps: Vec<String>,
        blocked_websites: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            start_time: chrono::Utc::now(),
            planned_duration_minutes,
            session_type,
            blocked_apps,
            blocked_websites,
        }
    }

    /// Calculate elapsed time in seconds
    pub fn elapsed_seconds(&self) -> i64 {
        (chrono::Utc::now() - self.start_time).num_seconds()
    }

    /// Check if session has exceeded planned duration
    pub fn is_overtime(&self) -> bool {
        self.elapsed_seconds() > (self.planned_duration_minutes as i64 * 60)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    Focus,
    Break,
    Custom,
}

/// Current state of blocking functionality
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockingState {
    pub enabled: bool,
    pub blocked_processes: Vec<String>,
    pub blocked_websites: Vec<String>,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
}

impl BlockingState {
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn update_last_check(&mut self) {
        self.last_check = Some(chrono::Utc::now());
    }

    pub fn update_blocked_websites(&mut self, websites: Vec<String>) {
        self.blocked_websites = websites;
    }
}

/// Strict mode state - cannot disable blocking until session ends
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrictModeState {
    pub enabled: bool,
    pub session_id: Option<String>,
    pub started_at: Option<String>,
}

/// Nuclear option state - irreversible time-locked blocking
#[derive(Debug, Clone, Default)]
pub struct NuclearOptionState {
    pub active: bool,
    pub duration_minutes: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}

/// Timer state for tracking pause/resume during active sessions
///
/// This allows the backend to be the single source of truth for timer state,
/// ensuring all windows stay perfectly synchronized.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct TimerState {
    pub is_running: bool,
    pub is_paused: bool,
    /// Cumulative seconds spent paused (not counting current pause)
    pub pause_elapsed_seconds: i64,
    /// When the current pause started (if paused)
    pub paused_at: Option<DateTime<Utc>>,
}


impl TimerState {
    /// Create a new running timer state (for session start)
    pub fn new_running() -> Self {
        Self {
            is_running: true,
            is_paused: false,
            pause_elapsed_seconds: 0,
            paused_at: None,
        }
    }

    /// Pause the timer
    pub fn pause(&mut self) {
        if self.is_running && !self.is_paused {
            self.is_paused = true;
            self.paused_at = Some(Utc::now());
        }
    }

    /// Resume the timer
    pub fn resume(&mut self) {
        if self.is_paused {
            if let Some(paused_time) = self.paused_at {
                let pause_duration = Utc::now() - paused_time;
                self.pause_elapsed_seconds += pause_duration.num_seconds();
            }
            self.is_paused = false;
            self.paused_at = None;
        }
    }

    /// Toggle pause/resume
    pub fn toggle(&mut self) {
        if self.is_paused {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// Stop the timer completely
    pub fn stop(&mut self) {
        self.is_running = false;
        self.is_paused = false;
    }

    /// Calculate effective elapsed time accounting for pauses
    pub fn calculate_elapsed(&self, session_start: DateTime<Utc>) -> i64 {
        let total_elapsed = (Utc::now() - session_start).num_seconds();

        // Calculate current pause duration if currently paused
        let current_pause_duration = if self.is_paused {
            self.paused_at
                .map(|p| (Utc::now() - p).num_seconds())
                .unwrap_or(0)
        } else {
            0
        };

        // Effective elapsed = total - all pauses
        (total_elapsed - self.pause_elapsed_seconds - current_pause_duration).max(0)
    }
}
