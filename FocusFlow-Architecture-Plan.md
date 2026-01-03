# FocusFlow Architecture Plan

> **Version:** 1.0  
> **Date:** December 2024  
> **Status:** Ready for Development

---

## Executive Summary

FocusFlow is a privacy-first desktop productivity app built on the **Indistractable** framework. This document defines the technical architecture, technology choices, and implementation roadmap.

### Technology Stack Decision

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Desktop App** | Tauri 2.0 + React/TypeScript | Small bundles (3-10MB), Rust performance, web dev familiarity |
| **Local Database** | SQLite via SQLx | Privacy-first (data stays local), compile-time checked queries |
| **Cloud Backend** | TrailBase | 11x faster than PocketBase, built-in auth, Rust-native |
| **Scale-up Path** | PostgreSQL (Supabase) | When team analytics require advanced queries |

---

## Part 1: Desktop Application (Tauri)

### 1.1 Why Tauri

After comprehensive research, Tauri is confirmed as the optimal choice for FocusFlow:

**Verified Capabilities:**
- ✅ **System Tray** — Native support on macOS/Windows/Linux with dynamic icons and context menus
- ✅ **Menu Bar** — Full native menu integration with keyboard shortcuts
- ✅ **Notifications** — `tauri-plugin-notification` with sounds, actions, and scheduling
- ✅ **Autostart** — `tauri-plugin-autostart` for launch-at-login on all platforms
- ✅ **SQLite** — `tauri-plugin-sql` via SQLx with migrations and type-safe queries
- ✅ **File System Access** — Full Rust-level access for hosts file modification
- ✅ **Process Management** — Rust can spawn/monitor processes for app blocking
- ✅ **Small Bundles** — 3-10MB vs Electron's 150MB+
- ✅ **Security** — Capability-based permissions, CSP enforcement

**Alternatives Considered:**
- **Flutter**: Good cross-platform but desktop features less mature, requires Dart
- **Swift**: macOS-only, would need separate Windows/Linux apps
- **Electron**: 150MB+ bundles, higher resource usage

### 1.2 Application Structure

```
focusflow/
├── src/                          # React frontend
│   ├── components/
│   │   ├── FocusTimer/           # Pomodoro-style timer
│   │   ├── Dashboard/            # Analytics & insights
│   │   ├── BlockList/            # App/site blocking UI
│   │   ├── TriggerJournal/       # Distraction logging
│   │   └── Settings/             # Preferences
│   ├── hooks/                    # React hooks
│   ├── stores/                   # Zustand state management
│   └── utils/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # Plugin registration
│   │   ├── commands/             # Tauri commands (IPC)
│   │   │   ├── focus.rs          # Session management
│   │   │   ├── blocking.rs       # App/site blocking
│   │   │   ├── analytics.rs      # Local analytics
│   │   │   └── sync.rs           # Cloud sync
│   │   ├── db/                   # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs
│   │   │   └── queries.rs
│   │   ├── blocking/             # Blocking engine
│   │   │   ├── hosts.rs          # Hosts file modification
│   │   │   ├── dns.rs            # DNS-level blocking
│   │   │   └── process.rs        # App process monitoring
│   │   └── system/               # System integration
│   │       ├── tray.rs           # System tray
│   │       ├── notifications.rs  # Desktop notifications
│   │       └── calendar.rs       # Calendar integration
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

### 1.3 Tauri Configuration

```json
// tauri.conf.json (key sections)
{
  "productName": "FocusFlow",
  "version": "1.0.0",
  "identifier": "app.focusflow.desktop",
  "build": {
    "frontendDist": "../dist"
  },
  "app": {
    "trayIcon": {
      "iconPath": "icons/tray.png",
      "iconAsTemplate": true
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self'"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi", "deb", "appimage"],
    "macOS": {
      "entitlements": "./entitlements.plist",
      "signingIdentity": "-"
    }
  }
}
```

### 1.4 Required Tauri Plugins

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "macos-private-api"] }
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
tauri-plugin-notification = "2"
tauri-plugin-autostart = "2"
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-http = "2"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }

# System integration
sysinfo = "0.30"          # Process monitoring
dns-lookup = "2.0"        # DNS resolution
reqwest = { version = "0.11", features = ["json"] }

# AI (local inference)
llama-cpp = "0.3"         # Local LLM for AI coach (optional)
```

### 1.5 Blocking Implementation

FocusFlow uses multiple blocking strategies for robustness:

#### Strategy 1: Hosts File (Primary for websites)

```rust
// src-tauri/src/blocking/hosts.rs
use std::fs::{self, OpenOptions};
use std::io::Write;

const HOSTS_PATH_MACOS: &str = "/etc/hosts";
const HOSTS_PATH_WINDOWS: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";

pub fn block_domain(domain: &str) -> Result<(), BlockError> {
    let hosts_path = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        HOSTS_PATH_MACOS
    } else {
        HOSTS_PATH_WINDOWS
    };
    
    let entry = format!("\n127.0.0.1 {}\n127.0.0.1 www.{}", domain, domain);
    
    // Requires elevated permissions - prompt user
    let mut file = OpenOptions::new()
        .append(true)
        .open(hosts_path)?;
    
    file.write_all(entry.as_bytes())?;
    
    // Flush DNS cache
    flush_dns_cache()?;
    
    Ok(())
}
```

#### Strategy 2: Process Blocking (for apps)

```rust
// src-tauri/src/blocking/process.rs
use sysinfo::{System, ProcessExt, SystemExt};

pub fn monitor_blocked_apps(blocked_apps: Vec<String>) {
    let mut sys = System::new();
    
    loop {
        sys.refresh_processes();
        
        for (pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            
            if blocked_apps.iter().any(|app| name.contains(app)) {
                // Terminate the process
                process.kill();
                
                // Log the block event
                log_block_event(&name);
                
                // Show notification
                show_block_notification(&name);
            }
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

#### Strategy 3: DNS-Level (network-wide)

```rust
// src-tauri/src/blocking/dns.rs
// For more advanced blocking, can integrate with local DNS proxy
// Consider: trust-dns or hickory-dns crates
```

---

## Part 2: Local Database (SQLite)

### 2.1 Schema Design

All sensitive data stays local. Only aggregates sync to cloud.

```sql
-- Focus Sessions
CREATE TABLE focus_sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at      DATETIME NOT NULL,
    ended_at        DATETIME,
    planned_minutes INTEGER NOT NULL,
    actual_minutes  INTEGER,
    session_type    TEXT CHECK(session_type IN ('focus', 'break', 'long_break')),
    completed       BOOLEAN DEFAULT FALSE,
    interrupted     BOOLEAN DEFAULT FALSE,
    notes           TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Trigger Journal (Indistractable feature)
CREATE TABLE trigger_journal (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      INTEGER REFERENCES focus_sessions(id),
    trigger_time    DATETIME NOT NULL,
    trigger_type    TEXT CHECK(trigger_type IN ('internal', 'external')),
    category        TEXT, -- 'boredom', 'anxiety', 'fatigue', 'notification', etc.
    description     TEXT,
    feeling_before  INTEGER CHECK(feeling_before BETWEEN 1 AND 10),
    action_taken    TEXT, -- 'resisted', 'gave_in', 'scheduled'
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Block Rules
CREATE TABLE block_rules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_type       TEXT CHECK(rule_type IN ('website', 'app', 'category')),
    target          TEXT NOT NULL, -- domain, app name, or category
    schedule_type   TEXT CHECK(schedule_type IN ('always', 'focus_only', 'scheduled')),
    schedule_cron   TEXT, -- For scheduled blocks
    enabled         BOOLEAN DEFAULT TRUE,
    strictness      TEXT CHECK(strictness IN ('soft', 'medium', 'hard')),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Block Events (for analytics)
CREATE TABLE block_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id         INTEGER REFERENCES block_rules(id),
    blocked_at      DATETIME NOT NULL,
    target          TEXT NOT NULL,
    was_bypassed    BOOLEAN DEFAULT FALSE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Daily Aggregates (for sync)
CREATE TABLE daily_aggregates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    date            DATE NOT NULL UNIQUE,
    focus_minutes   INTEGER DEFAULT 0,
    sessions_completed INTEGER DEFAULT 0,
    distractions_blocked INTEGER DEFAULT 0,
    triggers_logged INTEGER DEFAULT 0,
    internal_triggers INTEGER DEFAULT 0,
    external_triggers INTEGER DEFAULT 0,
    synced_at       DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- User Settings
CREATE TABLE settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_sessions_date ON focus_sessions(date(started_at));
CREATE INDEX idx_triggers_session ON trigger_journal(session_id);
CREATE INDEX idx_blocks_date ON block_events(date(blocked_at));
CREATE INDEX idx_aggregates_date ON daily_aggregates(date);
```

### 2.2 SQLx Integration

```rust
// src-tauri/src/db/mod.rs
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tauri::AppHandle;

pub async fn init_db(app: &AppHandle) -> Result<SqlitePool, sqlx::Error> {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let db_path = app_dir.join("focusflow.db");
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite:{}?mode=rwc", db_path.display()))
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(pool)
}
```

---

## Part 3: Cloud Backend (TrailBase)

### 3.1 Why TrailBase

| Feature | TrailBase | PocketBase | Notes |
|---------|-----------|------------|-------|
| **Performance** | 11x faster | Baseline | Rust + Wasmtime vs Go |
| **Type-safe APIs** | ✅ | ❌ | Auto-generated with validation |
| **Vector Search** | ✅ Built-in | ❌ | Useful for AI coach features |
| **Auth** | ✅ | ✅ | OAuth, email/password |
| **Real-time** | ✅ | ✅ | WebSocket subscriptions |
| **Admin UI** | ✅ | ✅ | Built-in dashboard |
| **Maturity** | < 1 year | 3 years | Newer but actively developed |
| **License** | OSL-3.0 | MIT | Copyleft for modifications only |

### 3.2 TrailBase Schema

```sql
-- Users (managed by TrailBase auth)
-- TrailBase handles: id, email, password_hash, created_at, etc.

-- Organizations
CREATE TABLE organizations (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    plan            TEXT CHECK(plan IN ('free', 'team', 'enterprise')),
    seat_limit      INTEGER DEFAULT 5,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Organization Members
CREATE TABLE org_members (
    id              TEXT PRIMARY KEY,
    org_id          TEXT REFERENCES organizations(id),
    user_id         TEXT NOT NULL, -- TrailBase user ID
    role            TEXT CHECK(role IN ('owner', 'admin', 'member')),
    joined_at       DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(org_id, user_id)
);

-- Team Metrics (aggregated, anonymized)
CREATE TABLE team_metrics (
    id              TEXT PRIMARY KEY,
    org_id          TEXT REFERENCES organizations(id),
    date            DATE NOT NULL,
    member_count    INTEGER NOT NULL, -- Must be >= 5 for privacy
    total_focus_minutes INTEGER DEFAULT 0,
    total_sessions  INTEGER DEFAULT 0,
    total_blocks    INTEGER DEFAULT 0,
    avg_focus_score REAL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(org_id, date)
);

-- License Keys
CREATE TABLE licenses (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL,
    org_id          TEXT REFERENCES organizations(id),
    plan            TEXT CHECK(plan IN ('pro', 'team', 'enterprise')),
    seats           INTEGER DEFAULT 1,
    valid_from      DATE NOT NULL,
    valid_until     DATE,
    stripe_subscription_id TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 3.3 TrailBase Configuration

```yaml
# trailbase.yaml
server:
  host: 0.0.0.0
  port: 4000

database:
  path: ./data/focusflow.db

auth:
  jwt_secret: ${JWT_SECRET}
  providers:
    - type: email
      enabled: true
    - type: google
      client_id: ${GOOGLE_CLIENT_ID}
      client_secret: ${GOOGLE_CLIENT_SECRET}
    - type: github
      client_id: ${GITHUB_CLIENT_ID}
      client_secret: ${GITHUB_CLIENT_SECRET}

# Record API access rules
tables:
  organizations:
    read: "auth.id IN (SELECT user_id FROM org_members WHERE org_id = id)"
    create: "auth.id IS NOT NULL"
    update: "auth.id IN (SELECT user_id FROM org_members WHERE org_id = id AND role IN ('owner', 'admin'))"
    
  team_metrics:
    read: "auth.id IN (SELECT user_id FROM org_members WHERE org_id = team_metrics.org_id)"
    create: "auth.id IN (SELECT user_id FROM org_members WHERE org_id = @org_id)"
    # Only allow if member_count >= 5 (privacy threshold)
    
  licenses:
    read: "auth.id = user_id OR auth.id IN (SELECT user_id FROM org_members WHERE org_id = licenses.org_id AND role = 'owner')"
```

### 3.4 API Endpoints

TrailBase auto-generates REST APIs. Custom endpoints for business logic:

```typescript
// trailbase/endpoints/sync-metrics.ts
export async function POST(req: Request) {
  const { orgId, date, metrics } = await req.json();
  
  // Validate member count >= 5 for privacy
  if (metrics.memberCount < 5) {
    return new Response(
      JSON.stringify({ error: "Minimum 5 members required for team metrics" }),
      { status: 400 }
    );
  }
  
  // Upsert daily metrics
  await db.execute(`
    INSERT INTO team_metrics (id, org_id, date, member_count, total_focus_minutes, total_sessions, total_blocks)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    ON CONFLICT (org_id, date) DO UPDATE SET
      member_count = $4,
      total_focus_minutes = $5,
      total_sessions = $6,
      total_blocks = $7
  `, [generateId(), orgId, date, metrics.memberCount, metrics.focusMinutes, metrics.sessions, metrics.blocks]);
  
  return new Response(JSON.stringify({ success: true }));
}
```

---

## Part 4: Sync Strategy

### 4.1 Privacy Model

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER DEVICE                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  LOCAL SQLite (PRIVATE - NEVER LEAVES DEVICE)           │    │
│  │  • Focus sessions (start/end times, notes)              │    │
│  │  • Trigger journal entries (feelings, descriptions)     │    │
│  │  • Block events (what sites/apps were blocked)          │    │
│  │  • Personal settings                                    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  AGGREGATION LAYER                                       │    │
│  │  • Compute daily totals                                 │    │
│  │  • Strip personally identifiable information            │    │
│  │  • Only upload if user has opted-in                     │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
└──────────────────────────────┼───────────────────────────────────┘
                               │ HTTPS (encrypted)
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                       TRAILBASE CLOUD                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  AGGREGATED DATA ONLY                                    │    │
│  │  • User ID → focus_minutes_today                        │    │
│  │  • User ID → sessions_completed_today                   │    │
│  │  • User ID → distractions_blocked_today                 │    │
│  │  (No session details, no trigger descriptions, no URLs) │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  TEAM AGGREGATION (if >= 5 members)                      │    │
│  │  • org_id → team_average_focus_time                     │    │
│  │  • org_id → team_total_sessions                         │    │
│  │  • org_id → focus_distribution_by_hour                  │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Sync Implementation

```rust
// src-tauri/src/commands/sync.rs
use chrono::{Local, NaiveDate};
use reqwest::Client;
use sqlx::SqlitePool;

#[derive(serde::Serialize)]
struct DailyAggregate {
    date: NaiveDate,
    focus_minutes: i32,
    sessions_completed: i32,
    distractions_blocked: i32,
}

#[tauri::command]
pub async fn sync_to_cloud(
    pool: tauri::State<'_, SqlitePool>,
    auth_token: String,
) -> Result<(), String> {
    // 1. Compute today's aggregates from local data
    let today = Local::now().date_naive();
    
    let aggregate: DailyAggregate = sqlx::query_as!(
        DailyAggregate,
        r#"
        SELECT 
            date(started_at) as date,
            COALESCE(SUM(actual_minutes), 0) as focus_minutes,
            COUNT(*) FILTER (WHERE completed = true) as sessions_completed,
            (SELECT COUNT(*) FROM block_events WHERE date(blocked_at) = date(?)) as distractions_blocked
        FROM focus_sessions
        WHERE date(started_at) = date(?)
        GROUP BY date(started_at)
        "#,
        today, today
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;
    
    // 2. Upload to TrailBase
    let client = Client::new();
    let response = client
        .post("https://api.focusflow.app/api/v1/sync")
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&aggregate)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err("Sync failed".to_string());
    }
    
    // 3. Mark as synced locally
    sqlx::query!(
        "UPDATE daily_aggregates SET synced_at = CURRENT_TIMESTAMP WHERE date = ?",
        today
    )
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}
```

---

## Part 5: Deployment

### 5.1 Desktop App Distribution

```yaml
# GitHub Actions: .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
          
      - name: Install dependencies
        run: npm ci
        
      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
        with:
          tagName: v__VERSION__
          releaseName: 'FocusFlow v__VERSION__'
          releaseBody: 'See the changelog for details.'
          releaseDraft: true
```

### 5.2 TrailBase Deployment

```dockerfile
# Dockerfile for TrailBase
FROM debian:bookworm-slim

# Download TrailBase binary
RUN apt-get update && apt-get install -y curl && \
    curl -L https://github.com/trailbaseio/trailbase/releases/latest/download/trail-linux-x86_64 \
    -o /usr/local/bin/trail && \
    chmod +x /usr/local/bin/trail

WORKDIR /app
COPY trailbase.yaml .
COPY migrations/ ./migrations/
COPY endpoints/ ./endpoints/

EXPOSE 4000

CMD ["trail", "serve", "--config", "trailbase.yaml"]
```

**Hosting Options:**
1. **Fly.io** — Simple, $5-10/month for small instances
2. **Railway** — Easy Dockerfile deployment
3. **DigitalOcean App Platform** — Managed containers
4. **Self-hosted VPS** — Most control, $5/month Hetzner/OVH

### 5.3 Auto-Update

```rust
// src-tauri/src/main.rs
use tauri_plugin_updater::UpdaterExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                if let Some(update) = handle.updater().check().await.unwrap() {
                    // Show update available notification
                    update.download_and_install(|_, _| {}, || {}).await.unwrap();
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}
```

---

## Part 6: Development Roadmap

### Phase 1: MVP (Weeks 1-6)

- [ ] **Week 1-2**: Tauri + React scaffold, basic UI
- [ ] **Week 3**: Local SQLite integration, focus timer
- [ ] **Week 4**: Website blocking (hosts file)
- [ ] **Week 5**: System tray, notifications, autostart
- [ ] **Week 6**: Basic analytics dashboard

### Phase 2: Core Features (Weeks 7-10)

- [ ] **Week 7**: App blocking (process monitoring)
- [ ] **Week 8**: Trigger journal UI
- [ ] **Week 9**: TrailBase setup, auth integration
- [ ] **Week 10**: Cloud sync (aggregates only)

### Phase 3: Team Features (Weeks 11-14)

- [ ] **Week 11**: Organization creation, member invites
- [ ] **Week 12**: Team dashboard (anonymized metrics)
- [ ] **Week 13**: Shared blocklists
- [ ] **Week 14**: Manager reports (5+ member threshold)

### Phase 4: Polish & Launch (Weeks 15-18)

- [ ] **Week 15**: Calendar integration (Google, Outlook)
- [ ] **Week 16**: AI coach (local LLM or API)
- [ ] **Week 17**: Onboarding, tutorials
- [ ] **Week 18**: App store submissions, marketing

---

## Part 7: Technical Decisions Log

| Decision | Choice | Alternatives Considered | Rationale |
|----------|--------|------------------------|-----------|
| Desktop framework | Tauri | Flutter, Electron, Swift | Small bundles, Rust performance, web dev familiarity |
| Frontend | React + TypeScript | Vue, Svelte | Ecosystem, hiring pool, component libraries |
| Local DB | SQLite (SQLx) | IndexedDB, LevelDB | ACID, SQL queries, compile-time checks |
| Cloud backend | TrailBase | PocketBase, Supabase | Performance (11x faster), Rust-native, vector search |
| State management | Zustand | Redux, Jotai | Simple, TypeScript-native, no boilerplate |
| Styling | Tailwind CSS | styled-components, CSS modules | Utility-first, design system friendly |
| Auth | TrailBase built-in | Auth0, Clerk | Bundled, self-hostable, simpler |
| Payments | Stripe | Paddle, LemonSqueezy | Market leader, good docs |

---

## Appendix A: Environment Setup

```bash
# Prerequisites
# - Rust (rustup.rs)
# - Node.js 20+
# - pnpm (preferred) or npm

# Clone and setup
git clone https://github.com/yourorg/focusflow.git
cd focusflow
pnpm install

# Development
pnpm tauri dev

# Build for production
pnpm tauri build

# Run TrailBase locally
docker run -p 4000:4000 -v $(pwd)/data:/app/data trailbase/trailbase
```

---

## Appendix B: Security Considerations

1. **Local Data Encryption**: Use SQLCipher for encrypted SQLite
2. **Hosts File Access**: Require admin/sudo elevation, explain to user
3. **Cloud Communication**: TLS 1.3, certificate pinning
4. **API Keys**: Store in system keychain (tauri-plugin-store + keytar)
5. **Bypass Prevention**: "Hard mode" with time-locked bypass codes
6. **Code Signing**: Required for macOS notarization and Windows SmartScreen

---

## Appendix C: Monitoring & Analytics

- **Error Tracking**: Sentry (client + server)
- **Usage Analytics**: PostHog (privacy-focused, self-hostable)
- **Uptime**: UptimeRobot or Better Stack
- **Logging**: TrailBase built-in + structured JSON logs

---

*This document is the single source of truth for FocusFlow architecture. Update as decisions evolve.*
