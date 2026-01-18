// db/migrations.rs - Database schema migrations

use crate::Result;
use sqlx::SqlitePool;

/// Run all database migrations
///
/// Migrations are idempotent and safe to run multiple times.
pub async fn run(pool: &SqlitePool) -> Result<()> {
    // Create migrations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Run migrations in order
    run_if_needed(pool, 1, "create_sessions_table").await?;
    run_if_needed(pool, 2, "create_blocked_items_table").await?;
    run_if_needed(pool, 3, "create_analytics_table").await?;
    run_if_needed(pool, 4, "create_indices").await?;
    run_if_needed(pool, 5, "add_sync_columns").await?;
    run_if_needed(pool, 6, "create_journal_entries_table").await?;
    run_if_needed(pool, 7, "create_user_settings_table").await?;
    run_if_needed(pool, 8, "create_achievements_tables").await?;
    run_if_needed(pool, 9, "create_blocking_schedules_table").await?;
    run_if_needed(pool, 10, "create_blocking_categories_table").await?;
    run_if_needed(pool, 11, "create_block_attempts_table").await?;
    run_if_needed(pool, 12, "create_streak_tables").await?;
    run_if_needed(pool, 13, "add_match_type_to_blocked_items").await?;
    run_if_needed(pool, 14, "create_team_connection_table").await?;
    run_if_needed(pool, 15, "create_shared_sessions_table").await?;
    run_if_needed(pool, 16, "create_oauth_tokens_table").await?;
    run_if_needed(pool, 17, "create_conversations_table").await?;
    run_if_needed(pool, 18, "create_messages_table").await?;
    run_if_needed(pool, 19, "create_memory_table").await?;
    run_if_needed(pool, 20, "create_conversation_summaries_table").await?;
    run_if_needed(pool, 21, "create_chat_indices").await?;
    run_if_needed(pool, 22, "create_team_tables").await?;

    Ok(())
}

/// Check if migration is needed and run the appropriate SQL
async fn run_if_needed(pool: &SqlitePool, id: i32, name: &str) -> Result<()> {
    let exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM _migrations WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    if exists.0 == 0 {
        tracing::info!("Running migration {}: {}", id, name);

        match id {
            1 => create_sessions_table(pool).await?,
            2 => create_blocked_items_table(pool).await?,
            3 => create_analytics_table(pool).await?,
            4 => create_indices(pool).await?,
            5 => add_sync_columns(pool).await?,
            6 => create_journal_entries_table(pool).await?,
            7 => create_user_settings_table(pool).await?,
            8 => create_achievements_tables(pool).await?,
            9 => create_blocking_schedules_table(pool).await?,
            10 => create_blocking_categories_table(pool).await?,
            11 => create_block_attempts_table(pool).await?,
            12 => create_streak_tables(pool).await?,
            13 => add_match_type_to_blocked_items(pool).await?,
            14 => create_team_connection_table(pool).await?,
            15 => create_shared_sessions_table(pool).await?,
            16 => create_oauth_tokens_table(pool).await?,
            17 => create_conversations_table(pool).await?,
            18 => create_messages_table(pool).await?,
            19 => create_memory_table(pool).await?,
            20 => create_conversation_summaries_table(pool).await?,
            21 => create_chat_indices(pool).await?,
            22 => create_team_tables(pool).await?,
            _ => return Err(crate::Error::Config(format!("Unknown migration id: {}", id))),
        }

        sqlx::query("INSERT INTO _migrations (id, name) VALUES (?, ?)")
            .bind(id)
            .bind(name)
            .execute(pool)
            .await?;

        tracing::info!("Migration {} completed", id);
    }

    Ok(())
}

/// Migration 1: Create sessions table
async fn create_sessions_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            start_time TEXT NOT NULL,
            end_time TEXT,
            planned_duration_minutes INTEGER NOT NULL,
            actual_duration_seconds INTEGER,
            session_type TEXT NOT NULL CHECK(session_type IN ('focus', 'break', 'custom')),
            completed BOOLEAN NOT NULL DEFAULT 0,
            notes TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 2: Create blocked items table
async fn create_blocked_items_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE blocked_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            item_type TEXT NOT NULL CHECK(item_type IN ('app', 'website')),
            value TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(item_type, value)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 3: Create analytics aggregation table
async fn create_analytics_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE daily_analytics (
            date TEXT PRIMARY KEY,
            total_focus_seconds INTEGER NOT NULL DEFAULT 0,
            total_break_seconds INTEGER NOT NULL DEFAULT 0,
            sessions_completed INTEGER NOT NULL DEFAULT 0,
            sessions_abandoned INTEGER NOT NULL DEFAULT 0,
            productivity_score REAL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 4: Create performance indices
async fn create_indices(pool: &SqlitePool) -> Result<()> {
    // Index for session lookups by date
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_sessions_start_time
        ON sessions(start_time DESC)
        "#,
    )
    .execute(pool)
    .await?;

    // Index for active session lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_sessions_active
        ON sessions(end_time)
        WHERE end_time IS NULL
        "#,
    )
    .execute(pool)
    .await?;

    // Index for blocked items by type
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocked_items_type_enabled
        ON blocked_items(item_type, enabled)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 5: Add sync columns for cloud integration
async fn add_sync_columns(pool: &SqlitePool) -> Result<()> {
    // Add user_id to sessions for cloud sync
    sqlx::query("ALTER TABLE sessions ADD COLUMN user_id TEXT")
        .execute(pool)
        .await?;

    // Add sync tracking columns to sessions
    sqlx::query("ALTER TABLE sessions ADD COLUMN device_id TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE sessions ADD COLUMN synced_at TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE sessions ADD COLUMN last_modified TEXT DEFAULT CURRENT_TIMESTAMP")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE sessions ADD COLUMN deleted BOOLEAN DEFAULT 0")
        .execute(pool)
        .await?;

    // Add sync columns to blocked_items
    sqlx::query("ALTER TABLE blocked_items ADD COLUMN user_id TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE blocked_items ADD COLUMN device_id TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE blocked_items ADD COLUMN synced_at TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE blocked_items ADD COLUMN last_modified TEXT DEFAULT CURRENT_TIMESTAMP")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE blocked_items ADD COLUMN deleted BOOLEAN DEFAULT 0")
        .execute(pool)
        .await?;

    // Add sync columns to daily_analytics
    sqlx::query("ALTER TABLE daily_analytics ADD COLUMN user_id TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE daily_analytics ADD COLUMN synced_at TEXT")
        .execute(pool)
        .await?;

    // Create index for sync queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_sessions_sync
        ON sessions(user_id, last_modified)
        WHERE synced_at IS NULL OR last_modified > synced_at
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 6: Create journal entries table for trigger journaling
async fn create_journal_entries_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE journal_entries (
            id TEXT PRIMARY KEY,
            session_id TEXT REFERENCES sessions(id),
            user_id TEXT,
            trigger_type TEXT NOT NULL CHECK(trigger_type IN (
                'boredom', 'anxiety', 'stress', 'fatigue',
                'notification', 'person', 'environment', 'other'
            )),
            emotion TEXT CHECK(emotion IN (
                'frustrated', 'anxious', 'tired', 'distracted',
                'curious', 'bored', 'overwhelmed', 'neutral'
            )),
            notes TEXT,
            intensity INTEGER CHECK(intensity >= 1 AND intensity <= 5),
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            device_id TEXT,
            synced_at TEXT,
            last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
            deleted BOOLEAN DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create trigger patterns table for analytics
    sqlx::query(
        r#"
        CREATE TABLE trigger_patterns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            trigger_type TEXT NOT NULL,
            day_of_week INTEGER CHECK(day_of_week >= 0 AND day_of_week <= 6),
            hour_of_day INTEGER CHECK(hour_of_day >= 0 AND hour_of_day <= 23),
            frequency INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Index for journal lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_journal_session
        ON journal_entries(session_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_journal_user_date
        ON journal_entries(user_id, created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 7: Create user settings table for local preferences
async fn create_user_settings_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Insert default settings
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO user_settings (key, value) VALUES
            ('theme', 'system'),
            ('notification_sound', 'true'),
            ('auto_start_breaks', 'false'),
            ('default_focus_minutes', '25'),
            ('default_break_minutes', '5'),
            ('subscription_tier', 'free'),
            ('sessions_today_count', '0'),
            ('device_id', lower(hex(randomblob(16))))
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 8: Create achievements tables (stub - implementation moved to migration 8)
///
/// This was originally a placeholder but the actual implementation was added later
/// as the real migration 8 function. This stub is kept only for migration ordering.
#[allow(dead_code)]
async fn create_achievements_tables_stub(_pool: &SqlitePool) -> Result<()> {
    // Achievements implementation is handled by the achievements feature
    // This is a placeholder to maintain migration order
    Ok(())
}

/// Migration 9: Create blocking schedules table for time-based blocking
async fn create_blocking_schedules_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE blocking_schedules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            day_of_week INTEGER NOT NULL CHECK(day_of_week >= 0 AND day_of_week <= 6),
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index for schedule lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocking_schedules_enabled
        ON blocking_schedules(enabled, day_of_week)
        WHERE enabled = 1
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 10: Create blocking categories table for category-based blocking
async fn create_blocking_categories_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE blocking_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            items TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Pre-populate default categories
    let default_categories = vec![
        (
            "Social Media",
            "Social networking and messaging platforms",
            r#"["facebook.com","twitter.com","instagram.com","tiktok.com","reddit.com","linkedin.com","snapchat.com"]"#,
        ),
        (
            "News",
            "News websites and aggregators",
            r#"["cnn.com","bbc.com","nytimes.com","news.google.com","theguardian.com","washingtonpost.com"]"#,
        ),
        (
            "Gaming",
            "Gaming platforms and stores",
            r#"["steam","epicgames","twitch.tv","discord","origin"]"#,
        ),
        (
            "Video",
            "Video streaming platforms",
            r#"["youtube.com","netflix.com","hulu.com","primevideo.com","disneyplus.com"]"#,
        ),
        (
            "Shopping",
            "E-commerce and shopping sites",
            r#"["amazon.com","ebay.com","etsy.com","alibaba.com","walmart.com"]"#,
        ),
    ];

    for (name, description, items) in default_categories {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO blocking_categories (name, description, items)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(items)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Migration 11: Create block attempts table for statistics tracking
async fn create_block_attempts_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE block_attempts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            item_type TEXT NOT NULL CHECK(item_type IN ('app', 'website')),
            item_value TEXT NOT NULL,
            blocked_at TEXT DEFAULT CURRENT_TIMESTAMP,
            session_id TEXT REFERENCES sessions(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indices for statistics queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_block_attempts_user_date
        ON block_attempts(user_id, blocked_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_block_attempts_item
        ON block_attempts(item_type, item_value, blocked_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 12: Create streak tables for enhanced streak tracking
async fn create_streak_tables(pool: &SqlitePool) -> Result<()> {
    // Create streak_freezes table
    sqlx::query(
        r#"
        CREATE TABLE streak_freezes (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            used_at TEXT,
            source TEXT NOT NULL CHECK(source IN ('weekly', 'achievement', 'purchase')),
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            expires_at TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create streak_history table
    sqlx::query(
        r#"
        CREATE TABLE streak_history (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            date TEXT NOT NULL,
            sessions_count INTEGER NOT NULL DEFAULT 0,
            focus_minutes INTEGER NOT NULL DEFAULT 0,
            was_frozen BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, date)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indices for performance
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_streak_freezes_user
        ON streak_freezes(user_id, used_at)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_streak_freezes_expires
        ON streak_freezes(expires_at)
        WHERE expires_at IS NOT NULL
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_streak_history_user_date
        ON streak_history(user_id, date DESC)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_streak_history_date
        ON streak_history(date DESC)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 8: Create achievements tables for gamification
async fn create_achievements_tables(pool: &SqlitePool) -> Result<()> {
    // Create achievements master table
    sqlx::query(
        r#"
        CREATE TABLE achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL,
            category TEXT NOT NULL CHECK(category IN ('session', 'streak', 'time', 'blocking', 'special')),
            rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary')),
            threshold INTEGER NOT NULL,
            points INTEGER NOT NULL DEFAULT 10,
            hidden BOOLEAN NOT NULL DEFAULT 0,
            display_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create user achievements table (unlocked achievements)
    sqlx::query(
        r#"
        CREATE TABLE user_achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            achievement_id INTEGER NOT NULL,
            unlocked_at TEXT DEFAULT CURRENT_TIMESTAMP,
            notification_sent BOOLEAN NOT NULL DEFAULT 0,
            device_id TEXT,
            synced_at TEXT,
            FOREIGN KEY (achievement_id) REFERENCES achievements(id) ON DELETE CASCADE,
            UNIQUE(user_id, achievement_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indices for performance
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_achievements_category
        ON achievements(category, display_order)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_achievements_user
        ON user_achievements(user_id, unlocked_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    // Insert default achievements
    insert_default_achievements(pool).await?;

    Ok(())
}

/// Insert predefined achievements into the database
async fn insert_default_achievements(pool: &SqlitePool) -> Result<()> {
    // Session achievements
    let session_achievements = vec![
        ("first_focus", "First Steps", "Complete your first focus session", "🎯", 1, "common", 10, 0),
        ("sessions_10", "Getting Started", "Complete 10 focus sessions", "⭐", 10, "common", 20, 1),
        ("sessions_50", "Focused Mind", "Complete 50 focus sessions", "🌟", 50, "rare", 50, 2),
        ("sessions_100", "Century Club", "Complete 100 focus sessions", "💫", 100, "rare", 100, 3),
        ("sessions_500", "Focus Master", "Complete 500 focus sessions", "🏆", 500, "epic", 250, 4),
        ("sessions_1000", "Legendary Focus", "Complete 1000 focus sessions", "👑", 1000, "legendary", 500, 5),
    ];

    for (key, name, desc, icon, threshold, rarity, points, order) in session_achievements {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO achievements
            (key, name, description, icon, category, rarity, threshold, points, display_order)
            VALUES (?, ?, ?, ?, 'session', ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(name)
        .bind(desc)
        .bind(icon)
        .bind(rarity)
        .bind(threshold)
        .bind(points)
        .bind(order)
        .execute(pool)
        .await?;
    }

    // Streak achievements
    let streak_achievements = vec![
        ("streak_3", "Hot Streak", "Maintain a 3-day focus streak", "🔥", 3, "common", 15, 0),
        ("streak_7", "Week Warrior", "Maintain a 7-day focus streak", "💪", 7, "rare", 35, 1),
        ("streak_14", "Fortnight Force", "Maintain a 14-day focus streak", "⚡", 14, "rare", 70, 2),
        ("streak_30", "Monthly Master", "Maintain a 30-day focus streak", "🌙", 30, "epic", 150, 3),
        ("streak_100", "Unstoppable", "Maintain a 100-day focus streak", "💎", 100, "legendary", 500, 4),
        ("streak_365", "Year of Focus", "Maintain a 365-day focus streak", "🎖️", 365, "legendary", 1000, 5),
    ];

    for (key, name, desc, icon, threshold, rarity, points, order) in streak_achievements {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO achievements
            (key, name, description, icon, category, rarity, threshold, points, display_order)
            VALUES (?, ?, ?, ?, 'streak', ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(name)
        .bind(desc)
        .bind(icon)
        .bind(rarity)
        .bind(threshold)
        .bind(points)
        .bind(order)
        .execute(pool)
        .await?;
    }

    // Time achievements (in hours)
    let time_achievements = vec![
        ("time_1h", "First Hour", "Accumulate 1 hour of focus time", "⏰", 1, "common", 10, 0),
        ("time_10h", "Ten Hours Deep", "Accumulate 10 hours of focus time", "⏳", 10, "common", 25, 1),
        ("time_50h", "Dedicated", "Accumulate 50 hours of focus time", "🕐", 50, "rare", 75, 2),
        ("time_100h", "Centurion", "Accumulate 100 hours of focus time", "🕒", 100, "epic", 150, 3),
        ("time_500h", "Time Lord", "Accumulate 500 hours of focus time", "🕗", 500, "epic", 400, 4),
        ("time_1000h", "Grandmaster", "Accumulate 1000 hours of focus time", "⌛", 1000, "legendary", 1000, 5),
    ];

    for (key, name, desc, icon, threshold, rarity, points, order) in time_achievements {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO achievements
            (key, name, description, icon, category, rarity, threshold, points, display_order)
            VALUES (?, ?, ?, ?, 'time', ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(name)
        .bind(desc)
        .bind(icon)
        .bind(rarity)
        .bind(threshold)
        .bind(points)
        .bind(order)
        .execute(pool)
        .await?;
    }

    // Blocking achievements
    let blocking_achievements = vec![
        ("first_block", "Distraction Defender", "Block your first distraction", "🛡️", 1, "common", 10, 0),
        ("blocks_100", "Guardian", "Block 100 distractions", "🚫", 100, "rare", 50, 1),
        ("blocks_500", "Gatekeeper", "Block 500 distractions", "🔒", 500, "epic", 150, 2),
        ("blocks_1000", "Fortress", "Block 1000 distractions", "🏰", 1000, "legendary", 300, 3),
    ];

    for (key, name, desc, icon, threshold, rarity, points, order) in blocking_achievements {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO achievements
            (key, name, description, icon, category, rarity, threshold, points, display_order)
            VALUES (?, ?, ?, ?, 'blocking', ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(name)
        .bind(desc)
        .bind(icon)
        .bind(rarity)
        .bind(threshold)
        .bind(points)
        .bind(order)
        .execute(pool)
        .await?;
    }

    // Special achievements
    let special_achievements = vec![
        ("night_owl", "Night Owl", "Complete a session between 10PM-4AM", "🦉", 1, "rare", 30, 0),
        ("early_bird", "Early Bird", "Complete a session between 5AM-7AM", "🐦", 1, "rare", 30, 1),
        ("weekend_warrior", "Weekend Warrior", "Complete 10 sessions on weekends", "⚔️", 10, "rare", 50, 2),
        ("marathon", "Marathon Runner", "Complete a single 2+ hour session", "🏃", 120, "epic", 100, 3),
        ("perfectionist", "Perfectionist", "Complete 20 sessions with 100% completion", "✨", 20, "epic", 100, 4),
        ("consistency_king", "Consistency King", "Complete sessions every day for a week", "👑", 7, "epic", 150, 5),
        ("zero_distractions", "Zen Master", "Complete 10 sessions with zero blocks", "🧘", 10, "legendary", 200, 6),
    ];

    for (key, name, desc, icon, threshold, rarity, points, order) in special_achievements {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO achievements
            (key, name, description, icon, category, rarity, threshold, points, display_order)
            VALUES (?, ?, ?, ?, 'special', ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(name)
        .bind(desc)
        .bind(icon)
        .bind(rarity)
        .bind(threshold)
        .bind(points)
        .bind(order)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Migration 13: Add match_type column to blocked_items for configurable matching
async fn add_match_type_to_blocked_items(pool: &SqlitePool) -> Result<()> {
    // Add match_type column with default 'exact' for safety
    sqlx::query(
        r#"
        ALTER TABLE blocked_items
        ADD COLUMN match_type TEXT NOT NULL DEFAULT 'exact'
        CHECK(match_type IN ('exact', 'contains', 'regex'))
        "#,
    )
    .execute(pool)
    .await?;

    // Update existing rows to use 'exact' matching for safety
    // Users can change to 'contains' or 'regex' if they want the old behavior
    sqlx::query(
        r#"
        UPDATE blocked_items
        SET match_type = 'exact'
        WHERE match_type IS NULL
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 14: Create team connection table
async fn create_team_connection_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE team_connection (
            id TEXT PRIMARY KEY,
            server_url TEXT NOT NULL,
            team_id TEXT,
            user_id TEXT,
            api_key TEXT,
            connected_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 15: Create shared sessions table
async fn create_shared_sessions_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE shared_sessions (
            id TEXT PRIMARY KEY,
            local_session_id TEXT NOT NULL,
            remote_session_id TEXT,
            team_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT,
            planned_duration_minutes INTEGER NOT NULL,
            actual_duration_seconds INTEGER,
            completed BOOLEAN NOT NULL DEFAULT 0,
            shared_at INTEGER NOT NULL,
            last_modified TEXT NOT NULL,
            sync_status TEXT DEFAULT 'pending' CHECK(sync_status IN ('pending', 'synced', 'failed')),
            FOREIGN KEY (local_session_id) REFERENCES sessions(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indices for shared sessions
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_shared_sessions_team
        ON shared_sessions(team_id, start_time DESC)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_shared_sessions_user
        ON shared_sessions(user_id, start_time DESC)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_shared_sessions_sync
        ON shared_sessions(sync_status, last_modified)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 16: Create oauth_tokens table for calendar integrations
async fn create_oauth_tokens_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE oauth_tokens (
            id TEXT PRIMARY KEY,
            provider TEXT NOT NULL UNIQUE,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            expires_at INTEGER NOT NULL,
            scopes TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on provider for faster lookups
    sqlx::query(
        r#"
        CREATE INDEX idx_oauth_tokens_provider ON oauth_tokens(provider)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 17: Create conversations table for chat history
async fn create_conversations_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE conversations (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            title TEXT NOT NULL,
            summary TEXT,
            message_count INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            archived BOOLEAN NOT NULL DEFAULT 0,
            device_id TEXT,
            synced_at TEXT,
            last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
            deleted BOOLEAN DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 18: Create messages table for chat messages
async fn create_messages_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
            content TEXT NOT NULL,
            token_count INTEGER,
            tool_calls TEXT,
            model_used TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            device_id TEXT,
            synced_at TEXT,
            last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
            deleted BOOLEAN DEFAULT 0,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 19: Create memory table for LLM context persistence
async fn create_memory_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE memory (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            category TEXT NOT NULL CHECK(category IN ('preference', 'fact', 'pattern', 'goal', 'context')),
            confidence REAL NOT NULL DEFAULT 1.0 CHECK(confidence >= 0.0 AND confidence <= 1.0),
            source_conversation_id TEXT,
            source_message_id TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            access_count INTEGER NOT NULL DEFAULT 0,
            last_accessed TEXT,
            expires_at TEXT,
            device_id TEXT,
            synced_at TEXT,
            last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
            deleted BOOLEAN DEFAULT 0,
            FOREIGN KEY (source_conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
            FOREIGN KEY (source_message_id) REFERENCES messages(id) ON DELETE SET NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 20: Create conversation_summaries table for old conversations
async fn create_conversation_summaries_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE conversation_summaries (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            summary_text TEXT NOT NULL,
            key_topics TEXT,
            summary_tokens INTEGER,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            summarized_messages_count INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 21: Create indices for chat and memory tables
async fn create_chat_indices(pool: &SqlitePool) -> Result<()> {
    // Conversations indices
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_conversations_user_updated
        ON conversations(user_id, updated_at DESC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_conversations_created
        ON conversations(created_at DESC)
        WHERE deleted = 0 AND archived = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_conversations_sync
        ON conversations(user_id, last_modified)
        WHERE synced_at IS NULL OR last_modified > synced_at
        "#,
    )
    .execute(pool)
    .await?;

    // Messages indices
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_conversation
        ON messages(conversation_id, created_at ASC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_role
        ON messages(conversation_id, role, created_at DESC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_recent
        ON messages(created_at DESC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    // Memory indices
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_user_category
        ON memory(user_id, category, last_updated DESC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_key
        ON memory(key, user_id)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_confidence
        ON memory(user_id, confidence DESC, last_updated DESC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_accessed
        ON memory(user_id, last_accessed DESC)
        WHERE deleted = 0
        "#,
    )
    .execute(pool)
    .await?;

    // Conversation summaries indices
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_summaries_conversation
        ON conversation_summaries(conversation_id, created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Migration 22: Create team tables for TrailBase integration
async fn create_team_tables(pool: &SqlitePool) -> Result<()> {
    // Create teams table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            name TEXT NOT NULL,
            invite_code TEXT NOT NULL UNIQUE,
            created_by TEXT NOT NULL,
            member_count INTEGER DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            sync_status TEXT DEFAULT 'pending'
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create team_members table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS team_members (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            team_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT,
            role TEXT NOT NULL DEFAULT 'member',
            joined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            sync_status TEXT DEFAULT 'pending',
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
            UNIQUE(team_id, user_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create team_shared_sessions table (for team activity)
    // Named differently to avoid conflict with existing shared_sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS team_shared_sessions (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            team_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT,
            planned_duration_minutes INTEGER NOT NULL,
            actual_duration_seconds INTEGER,
            completed INTEGER DEFAULT 0,
            shared_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            sync_status TEXT DEFAULT 'pending',
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create team_blocklist table (shared blocking rules)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS team_blocklist (
            id TEXT PRIMARY KEY,
            remote_id TEXT,
            team_id TEXT NOT NULL,
            item_type TEXT NOT NULL,
            value TEXT NOT NULL,
            added_by TEXT NOT NULL,
            added_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            sync_status TEXT DEFAULT 'pending',
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
            UNIQUE(team_id, item_type, value)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indices for performance
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_team_members_team_id
        ON team_members(team_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_team_members_user_id
        ON team_members(user_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_team_shared_sessions_team_id
        ON team_shared_sessions(team_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_team_shared_sessions_user_id
        ON team_shared_sessions(user_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_team_blocklist_team_id
        ON team_blocklist(team_id)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
