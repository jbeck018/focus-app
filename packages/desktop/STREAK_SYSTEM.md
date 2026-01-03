# Enhanced Streak System

The Enhanced Streak System for FocusFlow provides comprehensive streak tracking with visual feedback, freezes, milestones, and detailed statistics.

## Features

### 1. Visual Streak Calendar (GitHub-style Heatmap)
- **Component**: `StreakCalendar`
- **Location**: `/src/features/streaks/streak-calendar.tsx`
- Color-coded cells showing daily activity intensity (0-4 levels)
- Tooltips with detailed session and focus time data
- Special indicators for frozen days
- Responsive grid layout with month/day labels
- 12-month default view (configurable)

### 2. Streak Freezes
- **Component**: `StreakFreezeModal`
- **Location**: `/src/features/streaks/streak-freeze-modal.tsx`

#### Freeze Types:
- **Weekly Freeze**: Automatically available every Monday, expires the following Monday
- **Earned Freezes**: Awarded for achieving milestones (Silver, Gold, Platinum)
- **Purchased Freezes**: (Future implementation)

#### Features:
- View all available freezes
- Apply freezes to past or current dates
- Expiration warnings for weekly freezes
- Confirmation dialog before usage

### 3. Streak Milestones
- **Component**: `MilestoneCelebration`
- **Location**: `/src/features/streaks/streak-milestone-celebration.tsx`

#### Tiers:
- **Bronze** (7 days): Sparkle animation
- **Silver** (30 days): Confetti animation + Freeze reward
- **Gold** (90 days): Confetti animation + Freeze reward
- **Platinum** (180 days): Fireworks animation + Freeze reward
- **Diamond** (365 days): Fireworks animation + Badge reward

#### Animations:
- **Sparkle**: Twinkling stars for smaller milestones
- **Confetti**: Falling colored pieces for mid-tier achievements
- **Fireworks**: Exploding particles for legendary achievements
- Auto-dismiss after 5 seconds
- Manual dismiss option

### 4. Grace Period Recovery
- **2-hour window** after midnight to complete a session
- Automatic detection and notification
- Visual indicators in the UI
- Countdown timer showing remaining grace period

### 5. Weekly/Monthly Statistics
- **Component**: `StreakStats`
- **Location**: `/src/features/streaks/streak-stats.tsx`

#### Metrics:
- **Active Days**: Days with at least 1 completed session
- **Total Sessions**: Aggregate session count
- **Focus Time**: Total and average focus minutes
- **Perfect Days**: Days with 4+ completed sessions
- **Activity Rate**: Percentage of active days
- **Consistency Score**: Calculated from activity rate, perfect days, and streak length

## Database Schema

### `streak_freezes` Table
```sql
CREATE TABLE streak_freezes (
  id TEXT PRIMARY KEY,
  user_id TEXT,
  used_at TEXT,
  source TEXT CHECK(source IN ('weekly', 'achievement', 'purchase')),
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  expires_at TEXT
);
```

### `streak_history` Table
```sql
CREATE TABLE streak_history (
  id TEXT PRIMARY KEY,
  user_id TEXT,
  date TEXT NOT NULL,
  sessions_count INTEGER NOT NULL DEFAULT 0,
  focus_minutes INTEGER NOT NULL DEFAULT 0,
  was_frozen BOOLEAN NOT NULL DEFAULT 0,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(user_id, date)
);
```

## Backend Commands (Rust)

### Available Tauri Commands:
- `get_current_streak()` - Get current streak status with grace period
- `get_streak_heatmap(months?)` - Get heatmap data for visualization
- `get_streak_stats(period)` - Get weekly or monthly statistics
- `get_streak_milestones()` - Get milestone progress
- `get_available_freezes()` - Get all available freezes
- `use_streak_freeze(request)` - Apply a freeze to a date
- `update_streak_history()` - Update today's streak data (called after session)
- `create_weekly_freeze()` - Create/refresh weekly freeze

### File Location:
`/src-tauri/src/commands/streaks.rs`

## Frontend Hooks

### Primary Hook:
```typescript
import { useStreaks } from '@/hooks/use-streaks';

function MyComponent() {
  const {
    currentStreak,      // Current streak data
    heatmap,           // Heatmap visualization data
    weekStats,         // Weekly statistics
    monthStats,        // Monthly statistics
    milestones,        // Milestone progress
    freezes,           // Available freezes
    useFreeze,         // Mutation to use a freeze
    updateHistory,     // Mutation to update today's history
    isLoading,         // Loading state
    error,             // Error state
  } = useStreaks();
}
```

### Individual Hooks:
- `useCurrentStreak()` - Current streak status only
- `useStreakHeatmap(months)` - Heatmap data
- `useStreakStats(period)` - Statistics for week or month
- `useStreakMilestones()` - Milestone progress
- `useAvailableFreezes()` - Available freezes
- `useUseStreakFreeze()` - Mutation to use freeze
- `useUpdateStreakHistory()` - Mutation to update history
- `useNextMilestone()` - Helper for next milestone calculation
- `useStreakNotifications()` - Helper for notification logic

### File Location:
`/src/hooks/use-streaks.ts`

## Type Definitions

All types are defined in `/packages/types/src/streaks.ts`:

```typescript
- CurrentStreak
- StreakHistoryEntry
- StreakFreeze
- StreakMilestone
- HeatmapCell
- StreakHeatmapData
- StreakStats
- AvailableFreezes
- UseStreakFreezeRequest
- FreezeSource
- MilestoneTier
```

## Integration Guide

### 1. Display Streak Dashboard
```tsx
import { StreakDashboard } from '@/features/streaks';

function App() {
  return <StreakDashboard />;
}
```

### 2. Update Streak After Session
```tsx
import { useUpdateStreakHistory } from '@/hooks/use-streaks';

function SessionCompleteHandler() {
  const updateStreak = useUpdateStreakHistory();

  const handleSessionComplete = async () => {
    // Complete the session
    await completeSession();

    // Update streak history
    updateStreak.mutate();
  };
}
```

### 3. Handle Milestone Achievement
```tsx
import { useStreakMilestones } from '@/hooks/use-streaks';
import { MilestoneCelebration } from '@/features/streaks';

function MilestoneHandler() {
  const { data: milestones } = useStreakMilestones();
  const [celebrating, setCelebrating] = useState(null);

  useEffect(() => {
    // Check for newly achieved milestones
    const newMilestone = milestones?.find(m =>
      m.isAchieved &&
      isRecent(m.achievedAt)
    );

    if (newMilestone) {
      setCelebrating(newMilestone);
    }
  }, [milestones]);

  return celebrating && (
    <MilestoneCelebration
      milestone={celebrating}
      onDismiss={() => setCelebrating(null)}
    />
  );
}
```

### 4. Streak Notifications
```tsx
import { useStreakNotifications } from '@/hooks/use-streaks';

function StreakNotifications() {
  const {
    shouldNotifyGracePeriod,
    shouldNotifyRiskBroken,
    gracePeriodEndsAt,
    currentCount,
  } = useStreakNotifications();

  if (shouldNotifyGracePeriod) {
    return (
      <Alert>
        Grace period active! Complete a session before {gracePeriodEndsAt}
      </Alert>
    );
  }

  if (shouldNotifyRiskBroken) {
    return (
      <Alert variant="destructive">
        Your {currentCount}-day streak is at risk! Complete a session today.
      </Alert>
    );
  }
}
```

## Styling

### Custom CSS Classes:
- `.celebration-card` - Pulsing animation for celebration modal
- `.milestone-badge` - Glowing animation for milestone badges
- `.confetti-piece` - Falling confetti animation
- `.firework` - Exploding firework animation
- `.sparkle` - Twinkling sparkle animation
- `.frozen-cell` - Pulse ring for frozen days in calendar

All animations are defined in `/src/index.css`

## Configuration Constants

```typescript
export const GRACE_PERIOD_HOURS = 2;
export const MIN_SESSIONS_FOR_STREAK = 1;
export const PERFECT_DAY_SESSIONS = 4;

export const MILESTONE_DAYS = {
  bronze: 7,
  silver: 30,
  gold: 90,
  platinum: 180,
  diamond: 365,
};
```

## Performance Optimizations

1. **Query Caching**: All queries use TanStack Query with appropriate stale times
2. **Incremental Updates**: Streak history updates only today's entry
3. **Efficient Heatmap**: Uses HashMap for O(1) lookups when generating calendar
4. **Database Indices**: Optimized indices on user_id and date columns
5. **Lazy Loading**: Celebration animations only render when triggered

## Testing Checklist

- [ ] Streak increments after completing first session of the day
- [ ] Grace period activates within 2 hours after midnight
- [ ] Weekly freeze appears every Monday
- [ ] Freezes can be applied to missed days
- [ ] Milestones trigger celebrations at correct day counts
- [ ] Heatmap displays correct intensity colors
- [ ] Statistics calculate correctly for week/month
- [ ] Streak breaks when day is missed without freeze
- [ ] Calendar tooltips show accurate data
- [ ] Animations perform smoothly

## Future Enhancements

1. **Streak Leaderboards**: Compare streaks with team members
2. **Custom Streak Goals**: Set personal streak targets
3. **Streak Predictions**: AI-based risk assessment
4. **Social Sharing**: Share milestone achievements
5. **Streak Recovery**: Purchase freezes with points
6. **Streak Challenges**: Time-limited streak competitions
7. **Historical Analysis**: Trend analysis over multiple years
8. **Export/Import**: Backup and restore streak data

## Troubleshooting

### Streak not updating after session
- Ensure `update_streak_history()` is called after session completion
- Check database permissions
- Verify session is marked as completed

### Grace period not showing
- Check system time is correct
- Verify last activity date is yesterday
- Ensure grace period hasn't expired

### Heatmap not rendering
- Check date format in database (YYYY-MM-DD)
- Verify streak_history table exists
- Check console for errors

### Animations not playing
- Ensure CSS is imported in index.css
- Check for CSS class name conflicts
- Verify browser supports CSS animations

## Support

For issues or questions, refer to:
- Type definitions: `/packages/types/src/streaks.ts`
- Backend logic: `/src-tauri/src/commands/streaks.rs`
- Frontend components: `/src/features/streaks/`
- React hooks: `/src/hooks/use-streaks.ts`
