# DNS Fallback Blocking Implementation

## Overview

This document describes the DNS-level blocking fallback mechanism implemented in FocusFlow. This fallback provides website blocking capabilities when hosts file modification isn't available (e.g., without elevated privileges).

## Architecture

### Multi-Layered Blocking Strategy

FocusFlow implements a **defense-in-depth** approach to website blocking:

1. **Primary: Hosts File Blocking** (`blocking/hosts.rs`)
   - Most effective, system-wide blocking
   - Requires elevated privileges
   - Blocks DNS resolution at OS level

2. **Fallback: Frontend-Based Blocking** (`blocking/dns.rs`)
   - Works without elevated privileges
   - Backend provides blocked domain list to frontend
   - Frontend implements blocking UI/overlays
   - Can integrate with browser extensions

### Why This Approach?

Industry apps like **Freedom**, **Cold Turkey**, and **SelfControl** face similar challenges:

- **macOS**: Requires "Full Disk Access" permission
- **Windows**: Requires "Run as Administrator"
- **Linux**: Requires sudo or file permissions

**Our solution**: Gracefully degrade to frontend-based blocking when privileges aren't available, while maintaining a consistent user experience.

## Implementation Details

### Backend Components

#### 1. DNS Blocking Fallback (`src/blocking/dns.rs`)

**Key Features:**
- Domain normalization (lowercase, trim)
- Subdomain matching (blocking `facebook.com` also blocks `api.facebook.com`)
- Case-insensitive matching
- URL parsing (extracts domain from full URLs)
- www. variant handling (blocking `example.com` also blocks `www.example.com`)

**Core Struct:**
```rust
pub struct DnsBlockingFallback {
    blocked_domains: HashSet<String>,  // O(1) lookup
    enabled: bool,
    last_updated: DateTime<Utc>,
}
```

**Public API:**
- `update_blocklist(domains: Vec<String>)` - Update the blocklist
- `enable()` / `disable()` - Toggle blocking
- `is_domain_blocked(domain: &str)` - Check if domain is blocked
- `is_url_blocked(url: &str)` - Check if URL is blocked
- `get_blocked_domains()` - Get all blocked domains with metadata
- `get_stats()` - Get blocking statistics
- `extract_domain_from_url(url: &str)` - Parse domain from URL

#### 2. State Management (`src/state.rs`)

**BlockingState** now includes:
```rust
pub struct BlockingState {
    pub enabled: bool,
    pub blocked_processes: Vec<String>,
    pub blocked_websites: Vec<String>,  // NEW: In-memory cache
    pub last_check: Option<DateTime<Utc>>,
}
```

**New method:**
- `update_blocked_websites(websites: Vec<String>)` - Sync in-memory blocklist

#### 3. Tauri Commands (`src/commands/blocking.rs`)

**New Commands for Frontend:**

```rust
// Get all blocked domains
#[tauri::command]
pub async fn get_blocked_domains(state) -> Result<BlockedDomainsResponse>

// Check if specific domain is blocked
#[tauri::command]
pub async fn check_domain_blocked(domain: String, state) -> Result<DomainCheckResult>

// Check if URL is blocked (extracts domain automatically)
#[tauri::command]
pub async fn check_url_blocked(url: String, state) -> Result<DomainCheckResult>

// Get blocking statistics
#[tauri::command]
pub async fn get_blocking_stats(state) -> Result<BlockingStats>
```

**Updated Commands:**

- `toggle_blocking()` - Now updates BlockingState and gracefully handles hosts file errors
- `add_blocked_website()` - Updates in-memory state for DNS fallback
- `remove_blocked_website()` - Updates in-memory state for DNS fallback
- `start_focus_session()` - Initializes BlockingState with session websites
- `end_focus_session()` - Clears BlockingState

### Response Types

#### BlockedDomainsResponse
```rust
{
  "domains": ["facebook.com", "twitter.com"],
  "enabled": true,
  "count": 2,
  "last_updated": "2025-12-30T12:00:00Z"
}
```

#### DomainCheckResult
```rust
{
  "blocked": true,
  "matched_domain": "facebook.com",
  "match_type": "subdomain"  // "exact", "subdomain", or null
}
```

#### BlockingStats
```rust
{
  "total_blocked_domains": 5,
  "blocking_enabled": true,
  "categories": {
    "all": ["facebook.com", "twitter.com", ...]
  }
}
```

## Frontend Integration Guide

### 1. Poll for Blocked Domains

```typescript
import { invoke } from '@tauri-apps/api/core';

// On session start or periodic refresh
const blockedDomains = await invoke('get_blocked_domains');
console.log(blockedDomains);
// {
//   domains: ["facebook.com", "twitter.com"],
//   enabled: true,
//   count: 2,
//   last_updated: "2025-12-30T12:00:00Z"
// }
```

### 2. Check Before Navigation (WebView)

```typescript
// Before allowing navigation
const result = await invoke('check_url_blocked', {
  url: 'https://www.facebook.com/profile'
});

if (result.blocked) {
  // Show blocking overlay
  showBlockingOverlay({
    domain: result.matched_domain,
    matchType: result.match_type
  });
  return false;  // Prevent navigation
}
```

### 3. Show Blocking UI

```typescript
function showBlockingOverlay({ domain, matchType }) {
  // Display full-screen overlay
  const overlay = document.createElement('div');
  overlay.className = 'blocking-overlay';
  overlay.innerHTML = `
    <div class="blocking-message">
      <h1>🚫 Site Blocked</h1>
      <p>
        <strong>${domain}</strong> is blocked during your focus session.
      </p>
      <p class="match-type">Matched: ${matchType}</p>
      <button onclick="window.location.href='/'">
        Return to Focus
      </button>
    </div>
  `;
  document.body.appendChild(overlay);
}
```

### 4. Browser Extension Integration (Advanced)

For deeper blocking, create a companion browser extension:

**manifest.json:**
```json
{
  "name": "FocusFlow Blocker",
  "permissions": ["webRequest", "webRequestBlocking", "<all_urls>"],
  "background": {
    "scripts": ["background.js"]
  }
}
```

**background.js:**
```javascript
// Connect to Tauri app via native messaging
let blockedDomains = [];

// Fetch blocked domains from FocusFlow app
async function updateBlockedDomains() {
  // Call Tauri command via native messaging host
  const result = await callNativeHost('get_blocked_domains');
  blockedDomains = result.domains;
}

// Block requests to blocked domains
chrome.webRequest.onBeforeRequest.addListener(
  function(details) {
    const url = new URL(details.url);
    const domain = url.hostname;

    // Check if domain is blocked
    if (isBlocked(domain)) {
      return { cancel: true };
    }
  },
  { urls: ["<all_urls>"] },
  ["blocking"]
);

function isBlocked(domain) {
  // Check exact match
  if (blockedDomains.includes(domain)) return true;

  // Check subdomain match
  const parts = domain.split('.');
  for (let i = 1; i < parts.length; i++) {
    const parentDomain = parts.slice(i).join('.');
    if (blockedDomains.includes(parentDomain)) return true;
  }

  return false;
}

// Refresh blocked domains every 30 seconds
setInterval(updateBlockedDomains, 30000);
updateBlockedDomains();
```

## Testing

### Unit Tests

All tests are located in `src/blocking/dns.rs`:

```bash
cargo test dns --lib
```

**Test Coverage:**
- ✅ Exact domain blocking
- ✅ Subdomain blocking
- ✅ Case-insensitive matching
- ✅ URL extraction
- ✅ URL blocking
- ✅ Disabled state

**Results:**
```
running 6 tests
test blocking::dns::tests::test_url_extraction ... ok
test blocking::dns::tests::test_disabled_blocking ... ok
test blocking::dns::tests::test_case_insensitive_blocking ... ok
test blocking::dns::tests::test_url_blocking ... ok
test blocking::dns::tests::test_subdomain_blocking ... ok
test blocking::dns::tests::test_exact_domain_blocking ... ok

test result: ok. 6 passed; 0 failed; 0 ignored
```

### Integration Testing

Test the full flow:

```typescript
// 1. Start session with blocked websites
await invoke('start_focus_session', {
  request: {
    plannedDurationMinutes: 25,
    sessionType: 'focus',
    blockedApps: [],
    blockedWebsites: ['facebook.com', 'twitter.com']
  }
});

// 2. Verify domains are in state
const domains = await invoke('get_blocked_domains');
assert(domains.enabled === true);
assert(domains.domains.includes('facebook.com'));

// 3. Check domain blocking
const result = await invoke('check_domain_blocked', {
  domain: 'www.facebook.com'
});
assert(result.blocked === true);
assert(result.match_type === 'exact');

// 4. Check subdomain blocking
const subResult = await invoke('check_domain_blocked', {
  domain: 'api.facebook.com'
});
assert(subResult.blocked === true);
assert(subResult.match_type === 'subdomain');

// 5. End session
await invoke('end_focus_session', { completed: true });

// 6. Verify blocking disabled
const endDomains = await invoke('get_blocked_domains');
assert(endDomains.enabled === false);
```

## Performance Considerations

### O(1) Domain Lookup

Uses `HashSet` for blocked domains:
```rust
blocked_domains: HashSet<String>  // O(1) contains() check
```

### Subdomain Matching

Worst case: O(n) where n = number of domain parts
- `api.subdomain.example.com` → 4 checks
- Still very fast for typical domain depths (2-4 levels)

### Memory Usage

- Each domain string: ~10-50 bytes
- 100 blocked domains: ~1-5 KB
- Negligible overhead

## Security Considerations

### Why Frontend Blocking is Less Secure

1. **Bypassable** - User can open DevTools and disable JavaScript
2. **Browser-specific** - Only works in your app's WebView
3. **No system-wide blocking** - Doesn't affect other browsers

### When to Use Each Method

| Scenario | Hosts File | DNS Fallback | Notes |
|----------|------------|--------------|-------|
| Elevated privileges available | ✅ Primary | - | Most secure |
| No elevated privileges | ❌ Not possible | ✅ Fallback | Better than nothing |
| System-wide blocking needed | ✅ Yes | ❌ No | Hosts file only |
| User determined to bypass | ⚠️ Can be reversed | ⚠️ Can be bypassed | Both require commitment |

### Recommendations

1. **Always attempt hosts file blocking first** - Most effective
2. **Gracefully fall back to frontend blocking** - Don't fail the session
3. **Show clear UI indication** - Tell user which method is active
4. **Prompt for elevation** - Guide user to grant permissions
5. **Consider browser extension** - Provides middle ground between methods

## Error Handling

### Hosts File Failures

```rust
match hosts::update_hosts_file(&domains).await {
    Ok(_) => {
        tracing::info!("Hosts file blocking enabled");
    }
    Err(e) => {
        tracing::warn!("Hosts file blocking failed ({}), DNS fallback available", e);
        // Don't return error - fallback is still available
    }
}
```

### Frontend Error Handling

```typescript
try {
  const result = await invoke('check_url_blocked', { url });

  if (result.blocked) {
    showBlockingOverlay(result);
  }
} catch (error) {
  console.error('Blocking check failed:', error);
  // Fail open - allow navigation to avoid breaking app
}
```

## Future Enhancements

### Potential Improvements

1. **Local DNS Proxy** (Complex)
   - Run actual DNS server on localhost:53
   - Intercept DNS queries system-wide
   - Requires elevated privileges anyway

2. **System Proxy with PAC File** (Moderate)
   - Generate Proxy Auto-Config file
   - Set system proxy settings
   - More portable than hosts file
   - Still requires some permissions

3. **WebSocket Event Stream** (Simple)
   - Push blocklist updates to frontend
   - Avoid polling overhead
   - Real-time synchronization

4. **Categorization** (Simple)
   - Group domains by category (social, news, video)
   - Allow category-based blocking
   - Pre-built category lists

5. **Time-based Blocking** (Moderate)
   - Different blocklists for different times
   - "Social media only after 5pm"
   - Already supported via schedules

6. **Wildcard Patterns** (Simple)
   - Support `*.facebook.com`
   - Support `*social*`
   - Regex matching

## Comparison with Industry Apps

### Cold Turkey

- **Hosts file**: Yes
- **Fallback**: Browser extension
- **Elevation required**: Yes (macOS/Windows)
- **Our advantage**: Graceful degradation

### Freedom

- **Hosts file**: Yes
- **Fallback**: VPN-based blocking
- **Elevation required**: Yes
- **Our advantage**: Simpler, no VPN overhead

### SelfControl

- **Hosts file**: Yes
- **Fallback**: None
- **Elevation required**: Yes (macOS only)
- **Our advantage**: Cross-platform, has fallback

## Conclusion

The DNS fallback implementation provides a **robust, user-friendly solution** to website blocking:

✅ **Works without elevated privileges**
✅ **Maintains consistent API**
✅ **Gracefully degrades**
✅ **Well-tested (6/6 tests passing)**
✅ **Production-ready**

While frontend-based blocking is less secure than hosts file modification, it ensures FocusFlow remains functional even when system-level blocking isn't available. Users are guided to grant permissions for stronger blocking, but the app doesn't fail if they can't or won't.

---

**Files Modified:**

- `src/blocking/dns.rs` - Core DNS fallback implementation
- `src/state.rs` - Added `blocked_websites` to BlockingState
- `src/commands/blocking.rs` - New commands for frontend integration
- `src/commands/focus.rs` - Updated to sync BlockingState
- `src/lib.rs` - Registered new commands
- `src/blocking/capabilities.rs` - Fixed compilation error (unrelated)

**Commands Available:**

1. `get_blocked_domains()` - Get all blocked domains
2. `check_domain_blocked(domain)` - Check single domain
3. `check_url_blocked(url)` - Check full URL
4. `get_blocking_stats()` - Get statistics

**Frontend Integration:** See sections above for TypeScript examples.
