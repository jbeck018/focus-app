// trailbase/models.rs - Team collaboration data models
//
// Models for teams, members, and shared sessions with SyncableEntity implementation.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::sync::SyncableEntity;
use crate::Result;

/// Team entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub local_id: String,
    pub remote_id: Option<String>,
    pub name: String,
    pub invite_code: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

impl SyncableEntity for Team {
    fn entity_type() -> &'static str {
        "teams"
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

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::Error::Validation("Team name cannot be empty".to_string()));
        }

        if self.name.len() > 100 {
            return Err(crate::Error::Validation("Team name too long (max 100 characters)".to_string()));
        }

        Ok(())
    }

    fn merge_with(&self, remote: &Self) -> Self {
        // For teams, prefer remote for name changes but keep local invite_code
        let mut merged = if self.last_modified > remote.last_modified {
            self.clone()
        } else {
            remote.clone()
        };

        // Always use the most recent modification timestamp
        merged.last_modified = self.last_modified.max(remote.last_modified);

        merged
    }
}

/// Team member entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub local_id: String,
    pub remote_id: Option<String>,
    pub team_id: String,
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

impl SyncableEntity for Member {
    fn entity_type() -> &'static str {
        "members"
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

    fn validate(&self) -> Result<()> {
        if self.email.trim().is_empty() {
            return Err(crate::Error::Validation("Email cannot be empty".to_string()));
        }

        if !self.email.contains('@') {
            return Err(crate::Error::Validation("Invalid email format".to_string()));
        }

        Ok(())
    }

    fn merge_with(&self, remote: &Self) -> Self {
        // For members, prefer remote for role changes
        let mut merged = if self.last_modified > remote.last_modified {
            self.clone()
        } else {
            remote.clone()
        };

        // Always prefer remote role (server authority)
        merged.role = remote.role.clone();

        // Use most recent display_name
        if let Some(ref remote_name) = remote.display_name {
            merged.display_name = Some(remote_name.clone());
        }

        merged.last_modified = self.last_modified.max(remote.last_modified);

        merged
    }
}

/// Shared focus session entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedSession {
    pub local_id: String,
    pub remote_id: Option<String>,
    pub team_id: String,
    pub user_id: String,
    pub session_id: String, // Reference to local session
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub planned_duration_minutes: i32,
    pub actual_duration_seconds: Option<i32>,
    pub completed: bool,
    pub shared_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Pending,
    Synced,
    Failed,
}

impl SyncableEntity for SharedSession {
    fn entity_type() -> &'static str {
        "shared_sessions"
    }

    fn local_id(&self) -> &str {
        &self.local_id
    }

    fn remote_id(&self) -> Option<&str> {
        self.remote_id.as_deref()
    }

    fn set_remote_id(&mut self, remote_id: String) {
        self.remote_id = Some(remote_id);
        self.sync_status = SyncStatus::Synced;
    }

    fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }

    fn set_last_modified(&mut self, timestamp: DateTime<Utc>) {
        self.last_modified = timestamp;
    }

    fn validate(&self) -> Result<()> {
        if self.planned_duration_minutes <= 0 {
            return Err(crate::Error::Validation(
                "Planned duration must be positive".to_string(),
            ));
        }

        if self.planned_duration_minutes > 480 {
            // Max 8 hours
            return Err(crate::Error::Validation(
                "Planned duration too long (max 480 minutes)".to_string(),
            ));
        }

        if let Some(end) = self.end_time {
            if end < self.start_time {
                return Err(crate::Error::Validation(
                    "End time cannot be before start time".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn merge_with(&self, remote: &Self) -> Self {
        // For shared sessions, prefer the version with completion data
        if self.completed && !remote.completed {
            return self.clone();
        }

        if remote.completed && !self.completed {
            return remote.clone();
        }

        // Otherwise use last-write-wins
        if self.last_modified > remote.last_modified {
            self.clone()
        } else {
            remote.clone()
        }
    }
}

/// Team activity summary (aggregated view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamActivity {
    pub team_id: String,
    pub date: String, // ISO date
    pub total_sessions: i32,
    pub total_focus_minutes: i32,
    pub active_members: i32,
    pub top_productivity_member: Option<String>,
}

/// Team settings (local configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSettings {
    pub team_id: String,
    pub auto_sync: bool,
    pub sync_interval_minutes: i32,
    pub share_sessions: bool,
    pub share_stats: bool,
    pub notification_on_member_join: bool,
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            team_id: String::new(),
            auto_sync: true,
            sync_interval_minutes: 5,
            share_sessions: true,
            share_stats: true,
            notification_on_member_join: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_validation() {
        let mut team = Team {
            local_id: "team-1".to_string(),
            remote_id: None,
            name: "Test Team".to_string(),
            invite_code: "FOCUS-1234".to_string(),
            created_by: "user-1".to_string(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
        };

        assert!(team.validate().is_ok());

        // Empty name should fail
        team.name = "".to_string();
        assert!(team.validate().is_err());

        // Too long name should fail
        team.name = "a".repeat(101);
        assert!(team.validate().is_err());
    }

    #[test]
    fn test_member_validation() {
        let mut member = Member {
            local_id: "member-1".to_string(),
            remote_id: None,
            team_id: "team-1".to_string(),
            user_id: "user-1".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            role: MemberRole::Member,
            joined_at: Utc::now(),
            last_modified: Utc::now(),
        };

        assert!(member.validate().is_ok());

        // Invalid email should fail
        member.email = "invalid-email".to_string();
        assert!(member.validate().is_err());

        // Empty email should fail
        member.email = "".to_string();
        assert!(member.validate().is_err());
    }

    #[test]
    fn test_shared_session_validation() {
        let now = Utc::now();
        let mut session = SharedSession {
            local_id: "session-1".to_string(),
            remote_id: None,
            team_id: "team-1".to_string(),
            user_id: "user-1".to_string(),
            session_id: "local-session-1".to_string(),
            start_time: now,
            end_time: Some(now + chrono::Duration::minutes(25)),
            planned_duration_minutes: 25,
            actual_duration_seconds: Some(1500),
            completed: true,
            shared_at: now,
            last_modified: now,
            sync_status: SyncStatus::Pending,
        };

        assert!(session.validate().is_ok());

        // Negative duration should fail
        session.planned_duration_minutes = -1;
        assert!(session.validate().is_err());

        // Too long duration should fail
        session.planned_duration_minutes = 500;
        assert!(session.validate().is_err());

        // End before start should fail
        session.planned_duration_minutes = 25;
        session.end_time = Some(now - chrono::Duration::minutes(10));
        assert!(session.validate().is_err());
    }

    #[test]
    fn test_shared_session_merge() {
        let now = Utc::now();
        let earlier = now - chrono::Duration::hours(1);

        let local = SharedSession {
            local_id: "session-1".to_string(),
            remote_id: Some("remote-1".to_string()),
            team_id: "team-1".to_string(),
            user_id: "user-1".to_string(),
            session_id: "local-session-1".to_string(),
            start_time: now,
            end_time: None,
            planned_duration_minutes: 25,
            actual_duration_seconds: None,
            completed: false,
            shared_at: now,
            last_modified: now,
            sync_status: SyncStatus::Pending,
        };

        let remote = SharedSession {
            local_id: "session-1".to_string(),
            remote_id: Some("remote-1".to_string()),
            team_id: "team-1".to_string(),
            user_id: "user-1".to_string(),
            session_id: "local-session-1".to_string(),
            start_time: earlier,
            end_time: Some(earlier + chrono::Duration::minutes(25)),
            planned_duration_minutes: 25,
            actual_duration_seconds: Some(1500),
            completed: true,
            shared_at: earlier,
            last_modified: earlier,
            sync_status: SyncStatus::Synced,
        };

        // Remote is completed, should win even though local is newer
        let merged = local.merge_with(&remote);
        assert!(merged.completed);
        assert_eq!(merged.actual_duration_seconds, Some(1500));
    }
}
