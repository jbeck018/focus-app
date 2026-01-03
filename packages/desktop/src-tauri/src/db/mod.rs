// db/mod.rs - Database module with SQLx integration

pub mod migrations;
pub mod queries;
pub mod chat_queries;

use crate::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

/// Database abstraction with connection pooling
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// Create new database connection with migrations
    ///
    /// Uses WAL mode for better concurrency and sets pragmas for performance.
    pub async fn new(db_path: &Path) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path.display());

        let options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            // Enable Write-Ahead Logging for better concurrency
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            // Enable foreign key constraints
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Run migrations
        migrations::run(&pool).await?;

        Ok(Self { pool })
    }

    /// Execute health check query
    pub async fn health_check(&self) -> Result<bool> {
        let result: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0 == 1)
    }
}

// Ensure pool is Send + Sync for cross-thread sharing
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_all() {
        assert_send_sync::<Database>();
    }
};
