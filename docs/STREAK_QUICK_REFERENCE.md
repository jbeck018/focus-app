# Enhanced Streak System - Quick Reference

## 🎯 Key Components

```tsx
// Full Dashboard
import { StreakDashboard } from '@/features/streaks';
<StreakDashboard />

// Individual Components
import { StreakCalendar, StreakStats, StreakFreezeModal, MilestoneCelebration } from '@/features/streaks';
```

## 🪝 Essential Hooks

```tsx
// All-in-one hook
import { useStreaks } from '@/hooks/use-streaks';
const { currentStreak, heatmap, weekStats, milestones, freezes, useFreeze, updateHistory } = useStreaks();

// Individual hooks
import { useCurrentStreak, useStreakHeatmap, useStreakStats, useStreakMilestones, useAvailableFreezes, useUpdateStreakHistory } from '@/hooks/use-streaks';
```

## 🔥 Common Tasks

### Update Streak After Session
```tsx
const updateStreak = useUpdateStreakHistory();
updateStreak.mutate(); // Call when session completes
```

### Show Current Streak
```tsx
const { data: currentStreak } = useCurrentStreak();
<div>{currentStreak?.currentCount || 0} days</div>
```

### Display Heatmap
```tsx
<StreakCalendar months={12} />
```

### Handle Notifications
```tsx
const { shouldNotifyGracePeriod, shouldNotifyRiskBroken } = useStreakNotifications();
```

### Use a Freeze
```tsx
const useFreeze = useUseStreakFreeze();
useFreeze.mutate({ freezeId: "...", date: "2024-12-24" });
```

## 📊 Milestone Tiers

| Tier | Days | Reward | Animation |
|------|------|--------|-----------|
| Bronze | 7 | - | Sparkle |
| Silver | 30 | Freeze | Confetti |
| Gold | 90 | Freeze | Confetti |
| Platinum | 180 | Freeze | Fireworks |
| Diamond | 365 | Badge | Fireworks |

## 🛠️ Backend Commands (Rust)

```rust
get_current_streak() -> CurrentStreak
get_streak_heatmap(months?) -> StreakHeatmapData
get_streak_stats(period) -> StreakStats
get_streak_milestones() -> Vec<StreakMilestone>
get_available_freezes() -> AvailableFreezes
use_streak_freeze(request) -> StreakHistoryEntry
update_streak_history() -> StreakHistoryEntry
create_weekly_freeze() -> StreakFreeze
```

## 💾 Database Tables

```sql
-- Freeze tracking
streak_freezes (id, user_id, used_at, source, created_at, expires_at)

-- Daily activity
streak_history (id, user_id, date, sessions_count, focus_minutes, was_frozen, created_at)
```

## 🎨 CSS Animations

- `.celebration-card` - Pulsing celebration modal
- `.milestone-badge` - Glowing badge
- `.confetti-piece` - Falling confetti
- `.firework` - Exploding fireworks
- `.sparkle` - Twinkling stars
- `.frozen-cell` - Pulse ring for frozen days

## ⚙️ Constants

```typescript
GRACE_PERIOD_HOURS = 2
MIN_SESSIONS_FOR_STREAK = 1
PERFECT_DAY_SESSIONS = 4
```

## 📁 File Locations

```
Backend:
  src-tauri/src/commands/streaks.rs
  src-tauri/src/db/migrations.rs (migration #12)

Frontend:
  src/features/streaks/
    - streak-calendar.tsx
    - streak-stats.tsx
    - streak-freeze-modal.tsx
    - streak-milestone-celebration.tsx
    - streak-dashboard.tsx
    - index.tsx
    - StreakExample.tsx
  src/hooks/use-streaks.ts

Types:
  packages/types/src/streaks.ts

Styles:
  src/index.css (animations section)
```

## 🚀 Quick Start

1. **Add to your app**:
   ```tsx
   import { StreakDashboard } from '@/features/streaks';

   function App() {
     return <StreakDashboard />;
   }
   ```

2. **Integrate with sessions**:
   ```tsx
   import { useUpdateStreakHistory } from '@/hooks/use-streaks';

   function SessionTimer() {
     const updateStreak = useUpdateStreakHistory();

     const onComplete = () => {
       updateStreak.mutate();
     };
   }
   ```

3. **Done!** The system handles everything else automatically.

## 📚 Full Documentation

See `STREAK_SYSTEM.md` for complete documentation with detailed examples, troubleshooting, and advanced usage.
