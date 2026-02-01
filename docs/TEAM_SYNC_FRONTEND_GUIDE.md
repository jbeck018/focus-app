# Team Sync - Frontend Integration Guide

Quick reference for integrating the TrailBase backend in your React components.

## Installation (Already Done)

The backend is ready to use - no additional dependencies needed in the frontend.

## Available Commands

### 1. Connect to Team

```typescript
import { invoke } from '@tauri-apps/api/core';

interface Credentials {
  email: string;
  password: string;
}

interface AuthResponse {
  user_id: string;
  email: string;
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

const connectToTeam = async (serverUrl: string, email: string, password: string) => {
  try {
    const auth: AuthResponse = await invoke('connect_team', {
      serverUrl,
      credentials: { email, password }
    });

    console.log(`Connected as ${auth.email}`);
    return auth;
  } catch (error) {
    console.error('Connection failed:', error);
    throw error;
  }
};
```

### 2. Disconnect from Team

```typescript
const disconnectFromTeam = async () => {
  await invoke('disconnect_team');
  console.log('Disconnected from team');
};
```

### 3. Get Team Members

```typescript
interface TeamMemberInfo {
  id: string;
  email: string;
  display_name?: string;
  role: 'owner' | 'admin' | 'member';
  joined_at: string;
}

const getTeamMembers = async (): Promise<TeamMemberInfo[]> => {
  const members: TeamMemberInfo[] = await invoke('get_team_members_sync');
  return members;
};
```

### 4. Share a Session

```typescript
const shareSession = async (sessionId: string, teamId: string) => {
  const sharedId: string = await invoke('share_session', {
    sessionId,
    teamId
  });

  console.log(`Shared session ${sessionId}, shared ID: ${sharedId}`);
  return sharedId;
};
```

### 5. Get Team Activity

```typescript
interface RecentSessionInfo {
  user_id: string;
  display_name?: string;
  start_time: string;
  duration_minutes: number;
  completed: boolean;
}

interface TeamActivitySummary {
  total_sessions: number;
  total_focus_hours: number;
  active_members: number;
  recent_sessions: RecentSessionInfo[];
}

const getTeamActivity = async (): Promise<TeamActivitySummary> => {
  const activity: TeamActivitySummary = await invoke('get_team_activity');
  return activity;
};
```

### 6. Sync with Team

```typescript
interface SyncStats {
  synced: number;
  failed: number;
  pending: number;
}

const syncWithTeam = async (): Promise<SyncStats> => {
  const stats: SyncStats = await invoke('sync_with_team');
  console.log(`Synced: ${stats.synced}, Failed: ${stats.failed}, Pending: ${stats.pending}`);
  return stats;
};
```

### 7. Get Sync Status

```typescript
const getSyncStatus = async (): Promise<SyncStats> => {
  const stats: SyncStats = await invoke('get_sync_status');
  return stats;
};
```

### 8. Retry Failed Syncs

```typescript
const retryFailedSyncs = async () => {
  const retried: number = await invoke('retry_failed_syncs');
  console.log(`Retried ${retried} failed sync operations`);
  return retried;
};
```

### 9. Clear Failed Syncs

```typescript
const clearFailedSyncs = async () => {
  const cleared: number = await invoke('clear_failed_syncs');
  console.log(`Cleared ${cleared} failed sync operations`);
  return cleared;
};
```

## Example: Complete Team Component

```typescript
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface TeamMemberInfo {
  id: string;
  email: string;
  display_name?: string;
  role: 'owner' | 'admin' | 'member';
  joined_at: string;
}

interface TeamActivitySummary {
  total_sessions: number;
  total_focus_hours: number;
  active_members: number;
  recent_sessions: RecentSessionInfo[];
}

interface SyncStats {
  synced: number;
  failed: number;
  pending: number;
}

export function TeamSync() {
  const [connected, setConnected] = useState(false);
  const [members, setMembers] = useState<TeamMemberInfo[]>([]);
  const [activity, setActivity] = useState<TeamActivitySummary | null>(null);
  const [syncStats, setSyncStats] = useState<SyncStats | null>(null);
  const [serverUrl, setServerUrl] = useState('https://api.trailbase.io');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Auto-sync every 5 minutes
  useEffect(() => {
    if (!connected) return;

    const syncInterval = setInterval(async () => {
      try {
        const stats = await invoke<SyncStats>('sync_with_team');
        setSyncStats(stats);
      } catch (err) {
        console.error('Auto-sync failed:', err);
      }
    }, 5 * 60 * 1000);

    return () => clearInterval(syncInterval);
  }, [connected]);

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      await invoke('connect_team', {
        serverUrl,
        credentials: { email, password }
      });
      setConnected(true);

      // Load initial data
      await loadMembers();
      await loadActivity();
      await loadSyncStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Connection failed');
    } finally {
      setLoading(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await invoke('disconnect_team');
      setConnected(false);
      setMembers([]);
      setActivity(null);
      setSyncStats(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Disconnect failed');
    }
  };

  const loadMembers = async () => {
    try {
      const data = await invoke<TeamMemberInfo[]>('get_team_members_sync');
      setMembers(data);
    } catch (err) {
      console.error('Failed to load members:', err);
    }
  };

  const loadActivity = async () => {
    try {
      const data = await invoke<TeamActivitySummary>('get_team_activity');
      setActivity(data);
    } catch (err) {
      console.error('Failed to load activity:', err);
    }
  };

  const loadSyncStatus = async () => {
    try {
      const stats = await invoke<SyncStats>('get_sync_status');
      setSyncStats(stats);
    } catch (err) {
      console.error('Failed to load sync status:', err);
    }
  };

  const handleSync = async () => {
    setLoading(true);
    try {
      const stats = await invoke<SyncStats>('sync_with_team');
      setSyncStats(stats);
      await loadActivity(); // Refresh activity after sync
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sync failed');
    } finally {
      setLoading(false);
    }
  };

  const handleRetryFailed = async () => {
    try {
      const retried = await invoke<number>('retry_failed_syncs');
      console.log(`Retried ${retried} operations`);
      await loadSyncStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Retry failed');
    }
  };

  if (!connected) {
    return (
      <div className="team-sync-connect">
        <h2>Connect to Team</h2>
        <form onSubmit={handleConnect}>
          <div>
            <label>Server URL</label>
            <input
              type="url"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              required
            />
          </div>
          <div>
            <label>Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <div>
            <label>Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          {error && <div className="error">{error}</div>}
          <button type="submit" disabled={loading}>
            {loading ? 'Connecting...' : 'Connect'}
          </button>
        </form>
      </div>
    );
  }

  return (
    <div className="team-sync">
      <div className="header">
        <h2>Team Collaboration</h2>
        <button onClick={handleDisconnect}>Disconnect</button>
      </div>

      {/* Sync Status */}
      {syncStats && (
        <div className="sync-status">
          <h3>Sync Status</h3>
          <div className="stats">
            <span>Synced: {syncStats.synced}</span>
            <span>Pending: {syncStats.pending}</span>
            <span>Failed: {syncStats.failed}</span>
          </div>
          <div className="actions">
            <button onClick={handleSync} disabled={loading}>
              {loading ? 'Syncing...' : 'Sync Now'}
            </button>
            {syncStats.failed > 0 && (
              <button onClick={handleRetryFailed}>
                Retry Failed
              </button>
            )}
          </div>
        </div>
      )}

      {/* Team Activity */}
      {activity && (
        <div className="team-activity">
          <h3>Team Activity</h3>
          <div className="summary">
            <div>Total Sessions: {activity.total_sessions}</div>
            <div>Focus Hours: {activity.total_focus_hours.toFixed(1)}</div>
            <div>Active Members: {activity.active_members}</div>
          </div>
          <div className="recent-sessions">
            <h4>Recent Sessions</h4>
            {activity.recent_sessions.map((session, i) => (
              <div key={i} className="session">
                <span>{session.display_name || session.user_id}</span>
                <span>{session.duration_minutes} min</span>
                <span>{session.completed ? '✓' : '⏳'}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Team Members */}
      <div className="team-members">
        <h3>Team Members ({members.length})</h3>
        {members.map((member) => (
          <div key={member.id} className="member">
            <div className="info">
              <strong>{member.display_name || member.email}</strong>
              <span className={`role ${member.role}`}>{member.role}</span>
            </div>
            <div className="meta">
              Joined {new Date(member.joined_at).toLocaleDateString()}
            </div>
          </div>
        ))}
      </div>

      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

## Error Handling

All commands can throw errors. Wrap in try-catch:

```typescript
try {
  await invoke('connect_team', { serverUrl, credentials });
} catch (error) {
  // Error types:
  // - Network errors: "Network error: ..."
  // - Auth errors: "Authentication error: ..."
  // - Not found: "Not connected to team"
  // - Validation: "Validation error: ..."
  console.error('Error:', error);
}
```

## Tips

1. **Auto-sync**: Set up interval to call `sync_with_team()` every 5 minutes
2. **Loading states**: Show spinners during long operations
3. **Error recovery**: Show retry button for failed syncs
4. **Optimistic updates**: Update UI immediately, sync in background
5. **Offline indicator**: Check sync status, show when pending > 0
6. **Session sharing**: Auto-share completed sessions with team

## TypeScript Types

Create a `types/team.ts` file:

```typescript
export interface Credentials {
  email: string;
  password: string;
}

export interface AuthResponse {
  user_id: string;
  email: string;
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

export interface TeamMemberInfo {
  id: string;
  email: string;
  display_name?: string;
  role: 'owner' | 'admin' | 'member';
  joined_at: string;
}

export interface RecentSessionInfo {
  user_id: string;
  display_name?: string;
  start_time: string;
  duration_minutes: number;
  completed: boolean;
}

export interface TeamActivitySummary {
  total_sessions: number;
  total_focus_hours: number;
  active_members: number;
  recent_sessions: RecentSessionInfo[];
}

export interface SyncStats {
  synced: number;
  failed: number;
  pending: number;
}
```

## Testing

Test with mock TrailBase server or use:
- `https://api.trailbase.io` (production)
- `http://localhost:8000` (local development)

## Troubleshooting

**"Not connected to team"**
- User must call `connect_team()` first
- Connection is stored in database, persists across app restarts

**"Network error"**
- Check server URL is correct
- Ensure HTTPS (HTTP not allowed in production)
- Check network connectivity

**"Authentication error"**
- Invalid credentials
- Token expired (call `connect_team()` again)

**Sync not working**
- Check `get_sync_status()` for errors
- Use `retry_failed_syncs()` to retry
- Use `clear_failed_syncs()` to reset

## Next Steps

1. Update `src/features/Team.tsx` with new commands
2. Add team settings page with connection UI
3. Add activity feed component
4. Add sync status indicator to header
5. Test with real TrailBase instance
