// commands/sync.rs - Data export/import for backup and sync

use crate::{
    db::queries::{self, BlockedItem, Session},
    state::AppState,
    Result,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Manager, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub sessions: Vec<Session>,
    pub blocked_items: Vec<BlockedItem>,
}

/// Export all data to JSON file
///
/// Returns the path where the file was saved
#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
) -> Result<String> {
    // Get all sessions (last 90 days)
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(90);
    let sessions = queries::get_sessions_in_range(state.pool(), start, end).await?;

    // Get all blocked items
    let blocked_items = queries::get_blocked_items(state.pool(), None).await?;

    let export_data = ExportData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now(),
        sessions,
        blocked_items,
    };

    // Determine export path
    let app_data_dir = state
        .app_handle
        .path()
        .app_data_dir()
        .map_err(|e| crate::Error::Config(e.to_string()))?;

    let export_path = app_data_dir.join(format!(
        "focusflow_export_{}.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));

    // Write to file
    let json = serde_json::to_string_pretty(&export_data)?;
    tokio::fs::write(&export_path, json).await?;

    tracing::info!("Data exported to: {}", export_path.display());

    Ok(export_path.display().to_string())
}

/// Import data from JSON file
///
/// This will merge the imported data with existing data
#[tauri::command]
pub async fn import_data(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ImportStats> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(crate::Error::FileNotFound { path });
    }

    // Read and parse file
    let content = tokio::fs::read_to_string(&path).await?;
    let import_data: ExportData = serde_json::from_str(&content)?;

    let mut stats = ImportStats::default();

    // Import sessions (skip duplicates by ID)
    for session in import_data.sessions {
        // Check if session already exists
        let existing = queries::get_session(state.pool(), &session.id).await?;

        if existing.is_none() {
            queries::insert_session(
                state.pool(),
                &session.id,
                session.start_time,
                session.planned_duration_minutes,
                &session.session_type,
            )
            .await?;

            // If session is completed, also update end time
            if let Some(end_time) = session.end_time {
                queries::end_session(
                    state.pool(),
                    &session.id,
                    end_time,
                    session.completed,
                )
                .await?;
            }

            stats.sessions_imported += 1;
        } else {
            stats.sessions_skipped += 1;
        }
    }

    // Import blocked items (will enable if already exists)
    for item in import_data.blocked_items {
        queries::insert_blocked_item(
            state.pool(),
            &item.item_type,
            &item.value,
        )
        .await?;

        stats.blocked_items_imported += 1;
    }

    tracing::info!(
        "Import completed: {} sessions, {} blocked items",
        stats.sessions_imported,
        stats.blocked_items_imported
    );

    Ok(stats)
}

#[derive(Debug, Default, Serialize)]
pub struct ImportStats {
    pub sessions_imported: i64,
    pub sessions_skipped: i64,
    pub blocked_items_imported: i64,
}
