# Achievement System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FocusFlow Achievement System                   │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (React)                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                      App.tsx (Main App)                       │      │
│  │  ┌─────────────────────────────────────────────────────┐     │      │
│  │  │        Achievements Tab (Trophy Icon)                │     │      │
│  │  │  ┌────────────────────────────────────────────┐     │     │      │
│  │  │  │      <AchievementGallery />                │     │     │      │
│  │  │  └────────────────────────────────────────────┘     │     │      │
│  │  └─────────────────────────────────────────────────────┘     │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                    │                                     │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │         /features/achievements/ Components                    │      │
│  │                                                               │      │
│  │  ┌─────────────────────┐  ┌──────────────────────┐          │      │
│  │  │ achievement-gallery │  │  achievement-card    │          │      │
│  │  │  - Category tabs    │  │  - Full/Compact view │          │      │
│  │  │  - Stats overview   │  │  - Progress bar      │          │      │
│  │  │  - Filtering        │  │  - Rarity badges     │          │      │
│  │  └─────────────────────┘  └──────────────────────┘          │      │
│  │                                                               │      │
│  │  ┌──────────────────────────────────────────────┐           │      │
│  │  │         achievement-toast                     │           │      │
│  │  │  - Custom toast with rarity styling          │           │      │
│  │  │  - Sparkle animations for legendary          │           │      │
│  │  └──────────────────────────────────────────────┘           │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                    │                                     │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │         /hooks/use-achievements.ts                            │      │
│  │                                                               │      │
│  │  • useAchievements()          - All with status              │      │
│  │  • useAchievementStats()      - Points, completion %         │      │
│  │  • useRecentAchievements()    - Recent unlocks               │      │
│  │  • useCheckAchievements()     - Trigger checks + toasts      │      │
│  │  • useAchievementsByCategory()- Filtered by category         │      │
│  │  • useUnlockedAchievements()  - Only unlocked                │      │
│  │  • useLockedAchievements()    - Only locked                  │      │
│  │  • useNextAchievements()      - Closest to unlock            │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                    │                                     │
│                                    │ TanStack Query                      │
│                                    │ (auto-caching, 5min stale)          │
│                                    ▼                                     │
└────────────────────────────────────┼─────────────────────────────────────┘
                                     │
                                     │ Tauri IPC (invoke)
                                     │
┌────────────────────────────────────┼─────────────────────────────────────┐
│                              BACKEND (Rust/Tauri)                        │
├────────────────────────────────────┼─────────────────────────────────────┤
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │      /commands/achievements.rs - Tauri Commands              │      │
│  │                                                               │      │
│  │  #[tauri::command]                                           │      │
│  │  • get_achievements()                                        │      │
│  │  • get_achievement_stats()                                   │      │
│  │  • get_recent_achievements(limit)                            │      │
│  │  • check_achievements(session_id) ◄──────────┐              │      │
│  │                                               │              │      │
│  └───────────────────────────────────────────────┼──────────────┘      │
│                                    │             │                      │
│                                    ▼             │                      │
│  ┌──────────────────────────────────────────────┼──────────────┐      │
│  │         /db/queries.rs - Achievement Queries │              │      │
│  │                                               │              │      │
│  │  • get_all_achievements()                    │              │      │
│  │  • get_achievements_with_status()            │              │      │
│  │  • unlock_achievement()                      │              │      │
│  │  • get_completed_sessions_count()            │              │      │
│  │  • get_current_streak()                      │              │      │
│  │  • get_total_focus_hours()                   │              │      │
│  │  • get_total_blocks_count()                  │              │      │
│  │  • ... (20+ specialized queries)             │              │      │
│  └──────────────────────────────────────────────┼──────────────┘      │
│                                    │             │                      │
│                                    ▼             │                      │
│  ┌──────────────────────────────────────────────┼──────────────┐      │
│  │                  SQLite Database              │              │      │
│  │  ┌──────────────────────────┐                │              │      │
│  │  │   achievements           │                │              │      │
│  │  │  - 35 predefined         │                │              │      │
│  │  │  - 5 categories          │                │              │      │
│  │  │  - 4 rarity levels       │                │              │      │
│  │  └──────────────────────────┘                │              │      │
│  │  ┌──────────────────────────┐                │              │      │
│  │  │   user_achievements      │                │              │      │
│  │  │  - Unlock records        │                │              │      │
│  │  │  - Timestamps            │                │              │      │
│  │  └──────────────────────────┘                │              │      │
│  └──────────────────────────────────────────────┼──────────────┘      │
│                                                  │                      │
│  ┌──────────────────────────────────────────────┼──────────────┐      │
│  │      /commands/focus.rs - Session Commands   │              │      │
│  │                                               │              │      │
│  │  #[tauri::command]                           │              │      │
│  │  async fn end_focus_session(...) {           │              │      │
│  │    // ... session logic ...                  │              │      │
│  │                                               │              │      │
│  │    if completed {                            │              │      │
│  │      // Async spawn achievement check ───────┘              │      │
│  │      tauri::async_runtime::spawn(async {                    │      │
│  │        check_achievements(session_id, state).await          │      │
│  │      });                                                     │      │
│  │    }                                                         │      │
│  │  }                                                           │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                       ACHIEVEMENT FLOW (Session End)                     │
└─────────────────────────────────────────────────────────────────────────┘

1. User completes focus session
   │
   ▼
2. Frontend calls end_focus_session()
   │
   ▼
3. Backend updates session in DB
   │
   ▼
4. Backend spawns async achievement check (non-blocking)
   │
   ├──► Queries session count
   ├──► Queries streak count
   ├──► Queries total hours
   ├──► Queries block count
   ├──► Checks special conditions (time of day, etc.)
   │
   ▼
5. For each achievement threshold met:
   ├──► Unlock achievement in DB
   ├──► Send native notification
   └──► Return list of newly unlocked
   │
   ▼
6. Frontend receives achievement check result
   │
   ├──► Shows toast notification (Sonner)
   ├──► Invalidates achievement queries
   └──► Updates UI automatically
   │
   ▼
7. User sees toast + updated gallery

┌─────────────────────────────────────────────────────────────────────────┐
│                          ACHIEVEMENT CATEGORIES                          │
└─────────────────────────────────────────────────────────────────────────┘

SESSION (6)         STREAK (6)          TIME (6)
├─ first_focus      ├─ streak_3         ├─ time_1h
├─ sessions_10      ├─ streak_7         ├─ time_10h
├─ sessions_50      ├─ streak_14        ├─ time_50h
├─ sessions_100     ├─ streak_30        ├─ time_100h
├─ sessions_500     ├─ streak_100       ├─ time_500h
└─ sessions_1000    └─ streak_365       └─ time_1000h

BLOCKING (4)        SPECIAL (7)
├─ first_block      ├─ night_owl        (10PM-4AM)
├─ blocks_100       ├─ early_bird       (5AM-7AM)
├─ blocks_500       ├─ weekend_warrior  (10 weekend sessions)
└─ blocks_1000      ├─ marathon         (2+ hour session)
                    ├─ perfectionist    (20 100% sessions)
                    ├─ consistency_king (7 day daily streak)
                    └─ zero_distractions(10 zero-block sessions)

┌─────────────────────────────────────────────────────────────────────────┐
│                           RARITY & POINTS SYSTEM                         │
└─────────────────────────────────────────────────────────────────────────┘

COMMON (Gray)       10-20 pts   ├─ First achievements
RARE (Blue)         30-75 pts   ├─ Mid-tier milestones
EPIC (Purple)      100-400 pts  ├─ Major achievements
LEGENDARY (Gold)   500-1000 pts └─ Ultimate goals

┌─────────────────────────────────────────────────────────────────────────┐
│                          PERFORMANCE OPTIMIZATIONS                       │
└─────────────────────────────────────────────────────────────────────────┘

• Achievement checks run asynchronously (non-blocking)
• Database uses indexed lookups for all queries
• React Query caches achievement data (5-minute stale time)
• Toast notifications throttled (1 second between multiple unlocks)
• Efficient SQL queries with JOINs and aggregations
• Progress calculation cached on backend
