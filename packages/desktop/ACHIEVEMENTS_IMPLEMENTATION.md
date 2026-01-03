# Achievement System Implementation Guide

## Overview

The FocusFlow achievement system is a comprehensive gamification feature that rewards users for productivity milestones. The system includes:

- **35+ predefined achievements** across 5 categories
- **Automatic achievement checking** after session completion
- **Beautiful UI components** for displaying achievements
- **Toast notifications** for unlocked achievements
- **Progress tracking** toward next achievements
- **Points and rarity system** (Common, Rare, Epic, Legendary)

## Architecture

### Backend (Rust/Tauri)

#### Database Schema

**achievements table:**
```sql
- id: INTEGER PRIMARY KEY
- key: TEXT (unique identifier)
- name: TEXT
- description: TEXT
- icon: TEXT (emoji)
- category: TEXT (session/streak/time/blocking/special)
- rarity: TEXT (common/rare/epic/legendary)
- threshold: INTEGER (requirement to unlock)
- points: INTEGER (awarded points)
- hidden: BOOLEAN
- display_order: INTEGER
```

**user_achievements table:**
```sql
- id: INTEGER PRIMARY KEY
- user_id: TEXT (nullable for local-only)
- achievement_id: INTEGER (FK to achievements)
- unlocked_at: TIMESTAMP
- notification_sent: BOOLEAN
```

#### Commands

Located in `/packages/desktop/src-tauri/src/commands/achievements.rs`:

- `get_achievements()` - Get all achievements with unlock status
- `get_achievement_stats()` - Get user statistics (total points, completion %)
- `get_recent_achievements(limit)` - Get recently unlocked achievements
- `check_achievements(session_id)` - Check for newly unlocked achievements

#### Database Queries

Located in `/packages/desktop/src-tauri/src/db/queries.rs`:

All achievement-related queries including:
- Achievement retrieval
- Unlock status checking
- Statistics calculations (sessions, streaks, time, blocks)
- Special achievement conditions

### Frontend (React/TypeScript)

#### Type Definitions

Located in `/packages/types/src/achievements.ts`:

- `Achievement` - Core achievement entity
- `AchievementWithStatus` - Achievement with unlock status and progress
- `UserAchievement` - Unlocked achievement record
- `AchievementStats` - User statistics
- And more...

#### Hooks

Located in `/packages/desktop/src/hooks/use-achievements.ts`:

```typescript
// Get all achievements with status
const { data: achievements } = useAchievements();

// Get achievement statistics
const { data: stats } = useAchievementStats();

// Get recent unlocks
const { data: recent } = useRecentAchievements(10);

// Check for new achievements
const checkAchievements = useCheckAchievements();
await checkAchievements.mutateAsync(sessionId);

// Filtered hooks
const { data: sessionAchievements } = useAchievementsByCategory('session');
const { data: unlocked } = useUnlockedAchievements();
const { data: locked } = useLockedAchievements();
const { data: next } = useNextAchievements(3);
```

#### Components

Located in `/packages/desktop/src/features/achievements/`:

**AchievementCard** - Individual achievement display
```typescript
import { AchievementCard } from '@/features/achievements';

<AchievementCard
  achievement={achievement}
  compact={false}        // Use compact layout
  showProgress={true}    // Show progress bar
/>
```

**AchievementGallery** - Full gallery view with tabs and stats
```typescript
import { AchievementGallery } from '@/features/achievements';

// In your page/component:
<AchievementGallery />
```

**AchievementToast** - Toast notifications
```typescript
import { showAchievementToast } from '@/features/achievements';

showAchievementToast(achievement);
```

## Achievement Categories

### 1. Session Achievements
- **first_focus** - First session completed
- **sessions_10** - 10 sessions completed
- **sessions_50** - 50 sessions completed
- **sessions_100** - 100 sessions completed
- **sessions_500** - 500 sessions completed
- **sessions_1000** - 1000 sessions completed

### 2. Streak Achievements
- **streak_3** - 3-day streak
- **streak_7** - 7-day streak
- **streak_14** - 14-day streak
- **streak_30** - 30-day streak
- **streak_100** - 100-day streak
- **streak_365** - 365-day streak

### 3. Time Achievements
- **time_1h** - 1 hour of focus time
- **time_10h** - 10 hours of focus time
- **time_50h** - 50 hours of focus time
- **time_100h** - 100 hours of focus time
- **time_500h** - 500 hours of focus time
- **time_1000h** - 1000 hours of focus time

### 4. Blocking Achievements
- **first_block** - First distraction blocked
- **blocks_100** - 100 distractions blocked
- **blocks_500** - 500 distractions blocked
- **blocks_1000** - 1000 distractions blocked

### 5. Special Achievements
- **night_owl** - Session between 10PM-4AM
- **early_bird** - Session between 5AM-7AM
- **weekend_warrior** - 10 weekend sessions
- **marathon** - Single 2+ hour session
- **perfectionist** - 20 sessions with 100% completion
- **consistency_king** - Sessions every day for a week
- **zero_distractions** - 10 sessions with zero blocks

## Usage Examples

### Add Achievement Gallery to Dashboard

```typescript
// In your main dashboard/settings page
import { AchievementGallery } from '@/features/achievements';

export function Dashboard() {
  return (
    <div className="container">
      <h1>My Achievements</h1>
      <AchievementGallery />
    </div>
  );
}
```

### Show Achievement Stats Widget

```typescript
import { useAchievementStats } from '@/hooks/use-achievements';
import { Trophy } from 'lucide-react';

export function AchievementStatsWidget() {
  const { data: stats } = useAchievementStats();

  return (
    <div className="stats-widget">
      <div className="stat">
        <Trophy className="icon" />
        <span>{stats?.totalPoints || 0} points</span>
      </div>
      <div className="stat">
        <span>
          {stats?.unlockedCount}/{stats?.totalAchievements} unlocked
        </span>
      </div>
      <div className="progress-bar">
        <div style={{ width: `${stats?.completionPercentage || 0}%` }} />
      </div>
    </div>
  );
}
```

### Display Next Achievements to Unlock

```typescript
import { useNextAchievements } from '@/hooks/use-achievements';
import { AchievementCard } from '@/features/achievements';

export function NextAchievements() {
  const { data: nextAchievements } = useNextAchievements(3);

  return (
    <div className="next-achievements">
      <h3>Next to Unlock</h3>
      {nextAchievements?.map(achievement => (
        <AchievementCard
          key={achievement.id}
          achievement={achievement}
          compact
          showProgress
        />
      ))}
    </div>
  );
}
```

### Manual Achievement Check

```typescript
import { useCheckAchievements } from '@/hooks/use-achievements';

export function SessionEndButton() {
  const checkAchievements = useCheckAchievements();

  const handleEndSession = async (sessionId: string) => {
    // End session logic...

    // Check for achievements (automatically shows toasts)
    await checkAchievements.mutateAsync(sessionId);
  };

  return <button onClick={() => handleEndSession('session-id')}>End</button>;
}
```

## Automatic Integration

The achievement system automatically checks for new achievements when a session completes:

**Location:** `/packages/desktop/src-tauri/src/commands/focus.rs`

```rust
// In end_focus_session command:
if completed {
    // Asynchronously check achievements (non-blocking)
    let achievement_state = state.clone();
    let session_id = session.id.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = super::achievements::check_achievements(
            session_id,
            achievement_state.into()
        ).await {
            tracing::warn!("Failed to check achievements: {}", e);
        }
    });
}
```

## Customization

### Adding New Achievements

1. Add to database migration in `/packages/desktop/src-tauri/src/db/migrations.rs`:

```rust
let new_achievements = vec![
    ("my_achievement", "My Achievement", "Description", "🎉", 100, "rare", 50, 0),
];

for (key, name, desc, icon, threshold, rarity, points, order) in new_achievements {
    sqlx::query(/* INSERT */).execute(pool).await?;
}
```

2. Add check logic in `/packages/desktop/src-tauri/src/commands/achievements.rs`:

```rust
// In check_achievements function:
let my_value = queries::get_my_custom_stat(state.pool(), user_id_ref).await?;
newly_unlocked.extend(
    check_threshold_achievement(state.pool(), user_id_ref, "my_achievement", my_value).await?
);
```

3. Add type constant in `/packages/types/src/achievements.ts`:

```typescript
export const AchievementKey = {
  // ...
  MY_ACHIEVEMENT: "my_achievement",
} as const;
```

### Custom Toast Styling

Modify `/packages/desktop/src/features/achievements/achievement-toast.tsx` to customize the appearance of achievement notifications.

### Custom Card Layouts

The `AchievementCard` component accepts a `compact` prop. Extend it for more layouts:

```typescript
<AchievementCard achievement={achievement} compact />  // Compact
<AchievementCard achievement={achievement} />          // Full card
```

## Testing

### Manual Testing

1. Complete a session to unlock "First Steps"
2. Check the achievement gallery
3. Verify toast notification appeared
4. Check stats are updated

### Database Testing

```bash
# View achievements table
sqlite3 ~/Library/Application\ Support/com.focusflow.app/focusflow.db
SELECT * FROM achievements;
SELECT * FROM user_achievements;
```

## Performance Considerations

- Achievement checks run **asynchronously** after session completion (non-blocking)
- Stats queries use **indexed lookups** for performance
- React Query provides **automatic caching** (5-minute stale time)
- Toast notifications are **throttled** (1 second between multiple unlocks)

## File Structure

```
packages/
├── types/src/
│   └── achievements.ts              # TypeScript type definitions
├── desktop/
│   ├── src/
│   │   ├── hooks/
│   │   │   └── use-achievements.ts  # React hooks
│   │   └── features/achievements/
│   │       ├── achievement-card.tsx
│   │       ├── achievement-toast.tsx
│   │       ├── achievement-gallery.tsx
│   │       └── index.ts
│   └── src-tauri/src/
│       ├── commands/
│       │   └── achievements.rs      # Tauri commands
│       └── db/
│           ├── migrations.rs        # Database schema
│           └── queries.rs           # SQL queries
```

## Next Steps

1. Add achievement gallery to main navigation
2. Show achievement widget on dashboard
3. Display achievement notifications with custom UI
4. Add social sharing for achievements (optional)
5. Add achievement export/import for data portability

## Credits

Built with:
- **Tauri 2.0** - Desktop framework
- **React 19** - UI framework
- **TanStack Query** - Data fetching
- **shadcn/ui** - UI components
- **SQLite** - Database
- **Sonner** - Toast notifications
