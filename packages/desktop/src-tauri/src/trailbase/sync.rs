// trailbase/sync.rs - Bi-directional sync infrastructure
//
// Provides trait-based sync with offline queue and conflict resolution.

#![allow(clippy::type_complexity)]

use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fmt::Debug;

/// Trait for entities that can be synced with TrailBase
///
/// Implement this trait to enable bi-directional sync for your entity type.
pub trait SyncableEntity: Serialize + DeserializeOwned + Clone + Debug + Send + Sync {
    /// Entity type identifier (e.g., "teams", "members", "sessions")
    fn entity_type() -> &'static str;

    /// Local database ID
    fn local_id(&self) -> &str;

    /// Remote TrailBase ID (None if not yet synced)
    fn remote_id(&self) -> Option<&str>;

    /// Set the remote ID after successful sync
    fn set_remote_id(&mut self, remote_id: String);

    /// Last modification timestamp
    fn last_modified(&self) -> DateTime<Utc>;

    /// Set last modified timestamp
    fn set_last_modified(&mut self, timestamp: DateTime<Utc>);

    /// Merge local changes with remote changes (conflict resolution)
    ///
    /// Default strategy: last-write-wins based on timestamp.
    /// Override for custom merge logic.
    fn merge_with(&self, remote: &Self) -> Self {
        if self.last_modified() > remote.last_modified() {
            self.clone()
        } else {
            remote.clone()
        }
    }

    /// Validate entity before sync
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SyncResult {
    Success {
        local_id: String,
        remote_id: String,
    },
    Conflict {
        local_id: String,
        remote_id: String,
        merged: bool,
    },
    Failed {
        local_id: String,
        error: String,
    },
}

/// Queued sync operation for offline support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: SyncOperationType,
    pub payload: String, // JSON-serialized entity
    pub created_at: DateTime<Utc>,
    pub retry_count: i32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncOperationType {
    Create,
    Update,
    Delete,
}

/// Offline-first sync queue
pub struct SyncQueue {
    pool: SqlitePool,
}

impl SyncQueue {
    /// Create a new sync queue
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize sync queue table
    pub async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_queue (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                operation TEXT NOT NULL CHECK(operation IN ('create', 'update', 'delete')),
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0,
                last_error TEXT,
                UNIQUE(entity_type, entity_id, operation)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Sync queue initialized");
        Ok(())
    }

    /// Enqueue a sync operation
    pub async fn enqueue<E: SyncableEntity>(&self, entity: &E, operation: SyncOperationType) -> Result<()> {
        // Validate entity before queueing
        entity.validate()?;

        let id = uuid::Uuid::new_v4().to_string();
        let entity_type = E::entity_type();
        let entity_id = entity.local_id();
        let payload = serde_json::to_string(entity)?;
        let created_at = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO sync_queue (id, entity_type, entity_id, operation, payload, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(entity_type, entity_id, operation) DO UPDATE SET
                payload = excluded.payload,
                created_at = excluded.created_at,
                retry_count = 0,
                last_error = NULL
            "#,
        )
        .bind(&id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(serde_json::to_string(&operation)?)
        .bind(&payload)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        tracing::debug!(
            "Enqueued {} operation for {} entity: {}",
            match operation {
                SyncOperationType::Create => "create",
                SyncOperationType::Update => "update",
                SyncOperationType::Delete => "delete",
            },
            entity_type,
            entity_id
        );

        Ok(())
    }

    /// Get pending sync operations
    pub async fn get_pending(&self, limit: i32) -> Result<Vec<SyncOperation>> {
        let operations: Vec<(String, String, String, String, String, String, i32, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, entity_type, entity_id, operation, payload, created_at, retry_count, last_error
            FROM sync_queue
            WHERE retry_count < 5
            ORDER BY created_at ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        operations
            .into_iter()
            .map(|(id, entity_type, entity_id, operation_str, payload, created_at, retry_count, last_error)| {
                let operation: SyncOperationType = serde_json::from_str(&operation_str)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                let created_at = DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| Error::Serialization(e.to_string()))?
                    .with_timezone(&Utc);

                Ok(SyncOperation {
                    id,
                    entity_type,
                    entity_id,
                    operation,
                    payload,
                    created_at,
                    retry_count,
                    last_error,
                })
            })
            .collect()
    }

    /// Mark operation as completed
    pub async fn mark_completed(&self, operation_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sync_queue WHERE id = ?")
            .bind(operation_id)
            .execute(&self.pool)
            .await?;

        tracing::debug!("Marked sync operation {} as completed", operation_id);
        Ok(())
    }

    /// Mark operation as failed with retry
    pub async fn mark_failed(&self, operation_id: &str, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE sync_queue
            SET retry_count = retry_count + 1,
                last_error = ?
            WHERE id = ?
            "#,
        )
        .bind(error)
        .bind(operation_id)
        .execute(&self.pool)
        .await?;

        tracing::debug!("Marked sync operation {} as failed: {}", operation_id, error);
        Ok(())
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> Result<SyncQueueStats> {
        let pending_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_queue WHERE retry_count < 5")
            .fetch_one(&self.pool)
            .await?;

        let failed_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_queue WHERE retry_count >= 5")
            .fetch_one(&self.pool)
            .await?;

        let oldest_pending: Option<String> = sqlx::query_scalar(
            "SELECT created_at FROM sync_queue WHERE retry_count < 5 ORDER BY created_at ASC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(SyncQueueStats {
            pending_count,
            failed_count,
            oldest_pending,
        })
    }

    /// Clear all failed operations (manual intervention)
    pub async fn clear_failed(&self) -> Result<i32> {
        let result = sqlx::query("DELETE FROM sync_queue WHERE retry_count >= 5")
            .execute(&self.pool)
            .await?;

        let deleted = result.rows_affected() as i32;
        tracing::info!("Cleared {} failed sync operations", deleted);
        Ok(deleted)
    }

    /// Retry all failed operations (reset retry count)
    pub async fn retry_all_failed(&self) -> Result<i32> {
        let result = sqlx::query(
            r#"
            UPDATE sync_queue
            SET retry_count = 0,
                last_error = NULL
            WHERE retry_count >= 5
            "#,
        )
        .execute(&self.pool)
        .await?;

        let updated = result.rows_affected() as i32;
        tracing::info!("Reset {} failed sync operations for retry", updated);
        Ok(updated)
    }
}

/// Sync queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueStats {
    pub pending_count: i32,
    pub failed_count: i32,
    pub oldest_pending: Option<String>,
}

/// Conflict resolution strategy (planned for future use)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictStrategy {
    /// Use local version (client wins)
    LocalWins,
    /// Use remote version (server wins)
    RemoteWins,
    /// Use latest based on timestamp
    LastWriteWins,
    /// Use custom merge function
    CustomMerge,
}

/// Resolve conflicts between local and remote entities
///
/// Currently unused but planned for advanced conflict resolution in team sync.
/// For now, the SyncableEntity trait's merge_with method provides default behavior.
#[allow(dead_code)]
pub fn resolve_conflict<E: SyncableEntity>(
    local: &E,
    remote: &E,
    strategy: ConflictStrategy,
) -> E {
    match strategy {
        ConflictStrategy::LocalWins => local.clone(),
        ConflictStrategy::RemoteWins => remote.clone(),
        ConflictStrategy::LastWriteWins => {
            if local.last_modified() > remote.last_modified() {
                local.clone()
            } else {
                remote.clone()
            }
        }
        ConflictStrategy::CustomMerge => local.merge_with(remote),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEntity {
        local_id: String,
        remote_id: Option<String>,
        last_modified: DateTime<Utc>,
        data: String,
    }

    impl SyncableEntity for TestEntity {
        fn entity_type() -> &'static str {
            "test_entities"
        }

        fn local_id(&self) -> &str {
            &self.local_id
        }

        fn remote_id(&self) -> Option<&str> {
            self.remote_id.as_deref()
        }

        fn set_remote_id(&mut self, remote_id: String) {
            self.remote_id = Some(remote_id);
        }

        fn last_modified(&self) -> DateTime<Utc> {
            self.last_modified
        }

        fn set_last_modified(&mut self, timestamp: DateTime<Utc>) {
            self.last_modified = timestamp;
        }
    }

    #[test]
    fn test_conflict_resolution() {
        let now = Utc::now();
        let earlier = now - chrono::Duration::hours(1);

        let local = TestEntity {
            local_id: "test-1".to_string(),
            remote_id: Some("remote-1".to_string()),
            last_modified: now,
            data: "local data".to_string(),
        };

        let remote = TestEntity {
            local_id: "test-1".to_string(),
            remote_id: Some("remote-1".to_string()),
            last_modified: earlier,
            data: "remote data".to_string(),
        };

        // Local wins
        let result = resolve_conflict(&local, &remote, ConflictStrategy::LocalWins);
        assert_eq!(result.data, "local data");

        // Remote wins
        let result = resolve_conflict(&local, &remote, ConflictStrategy::RemoteWins);
        assert_eq!(result.data, "remote data");

        // Last write wins (local is newer)
        let result = resolve_conflict(&local, &remote, ConflictStrategy::LastWriteWins);
        assert_eq!(result.data, "local data");
    }
}
