# Frontend Blocking Integration Guide

## Quick Start

This guide shows how to integrate website blocking in your FocusFlow frontend.

## Available Commands

### 1. Get All Blocked Domains

```typescript
import { invoke } from '@tauri-apps/api/core';

const domains = await invoke('get_blocked_domains');
```

**Response:**
```typescript
interface BlockedDomainsResponse {
  domains: string[];        // ["facebook.com", "twitter.com"]
  enabled: boolean;         // Is blocking currently active?
  count: number;           // Total number of blocked domains
  last_updated: string;    // ISO 8601 timestamp
}
```

### 2. Check If Domain Is Blocked

```typescript
const result = await invoke('check_domain_blocked', {
  domain: 'facebook.com'
});
```

**Response:**
```typescript
interface DomainCheckResult {
  blocked: boolean;           // Is this domain blocked?
  matched_domain: string | null;  // Which domain matched (if blocked)
  match_type: string | null;      // "exact" | "subdomain" | null
}
```

**Examples:**
- `facebook.com` → `{ blocked: true, matched_domain: "facebook.com", match_type: "exact" }`
- `www.facebook.com` → `{ blocked: true, matched_domain: "facebook.com", match_type: "exact" }`
- `api.facebook.com` → `{ blocked: true, matched_domain: "facebook.com", match_type: "subdomain" }`

### 3. Check If URL Is Blocked

```typescript
const result = await invoke('check_url_blocked', {
  url: 'https://www.facebook.com/profile'
});
```

Same response as `check_domain_blocked`, but automatically extracts the domain from the URL.

### 4. Get Blocking Statistics

```typescript
const stats = await invoke('get_blocking_stats');
```

**Response:**
```typescript
interface BlockingStats {
  total_blocked_domains: number;
  blocking_enabled: boolean;
  categories: {
    [category: string]: string[];  // e.g., { "all": ["facebook.com", ...] }
  };
}
```

## Usage Patterns

### Pattern 1: WebView Navigation Guard

Intercept navigation in your webview and block if needed:

```typescript
// In your webview setup
webview.addEventListener('will-navigate', async (event) => {
  const url = event.url;

  const result = await invoke('check_url_blocked', { url });

  if (result.blocked) {
    event.preventDefault();
    showBlockingOverlay({
      url,
      domain: result.matched_domain,
      matchType: result.match_type
    });
  }
});
```

### Pattern 2: Periodic Sync

Keep your frontend in sync with backend state:

```typescript
let blockedDomains: Set<string> = new Set();

async function syncBlockedDomains() {
  const response = await invoke('get_blocked_domains');

  if (response.enabled) {
    blockedDomains = new Set(response.domains);
  } else {
    blockedDomains.clear();
  }
}

// Sync on mount
syncBlockedDomains();

// Sync periodically (optional)
setInterval(syncBlockedDomains, 30000);  // Every 30 seconds
```

### Pattern 3: Client-Side Domain Checking

For fast local checks without invoking backend:

```typescript
class DomainBlocker {
  private blockedDomains: Set<string> = new Set();

  constructor() {
    this.sync();
  }

  async sync() {
    const response = await invoke('get_blocked_domains');
    this.blockedDomains = new Set(response.domains);
  }

  isDomainBlocked(domain: string): boolean {
    domain = domain.toLowerCase().trim();

    // Check exact match
    if (this.blockedDomains.has(domain)) {
      return true;
    }

    // Check subdomain match
    const parts = domain.split('.');
    for (let i = 1; i < parts.length; i++) {
      const parent = parts.slice(i).join('.');
      if (this.blockedDomains.has(parent)) {
        return true;
      }
    }

    return false;
  }

  extractDomain(url: string): string | null {
    try {
      const urlObj = new URL(url);
      return urlObj.hostname;
    } catch {
      return null;
    }
  }

  isUrlBlocked(url: string): boolean {
    const domain = this.extractDomain(url);
    return domain ? this.isDomainBlocked(domain) : false;
  }
}

// Usage
const blocker = new DomainBlocker();

// Fast local check
if (blocker.isUrlBlocked('https://facebook.com')) {
  showBlockingOverlay();
}

// Or verify with backend
const result = await invoke('check_url_blocked', { url });
```

### Pattern 4: React Hook

```typescript
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface BlockedDomainsResponse {
  domains: string[];
  enabled: boolean;
  count: number;
  last_updated: string;
}

export function useBlockedDomains() {
  const [blockedDomains, setBlockedDomains] = useState<BlockedDomainsResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadBlockedDomains();

    // Optional: Auto-refresh
    const interval = setInterval(loadBlockedDomains, 30000);
    return () => clearInterval(interval);
  }, []);

  async function loadBlockedDomains() {
    try {
      const domains = await invoke<BlockedDomainsResponse>('get_blocked_domains');
      setBlockedDomains(domains);
    } catch (error) {
      console.error('Failed to load blocked domains:', error);
    } finally {
      setLoading(false);
    }
  }

  async function checkUrl(url: string) {
    return await invoke('check_url_blocked', { url });
  }

  return {
    blockedDomains,
    loading,
    checkUrl,
    refresh: loadBlockedDomains
  };
}

// Usage in component
function BlockingStatus() {
  const { blockedDomains, loading } = useBlockedDomains();

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h3>Blocking Status</h3>
      <p>Active: {blockedDomains?.enabled ? 'Yes' : 'No'}</p>
      <p>Blocked Sites: {blockedDomains?.count}</p>
      <ul>
        {blockedDomains?.domains.map(domain => (
          <li key={domain}>{domain}</li>
        ))}
      </ul>
    </div>
  );
}
```

## UI Components

### Blocking Overlay Component

```typescript
interface BlockingOverlayProps {
  domain: string;
  matchType: string;
  onReturn?: () => void;
}

function BlockingOverlay({ domain, matchType, onReturn }: BlockingOverlayProps) {
  return (
    <div className="fixed inset-0 bg-gray-900 bg-opacity-95 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl p-8 max-w-md text-center">
        <div className="text-6xl mb-4">🚫</div>
        <h1 className="text-3xl font-bold mb-4 text-gray-900">
          Site Blocked
        </h1>
        <p className="text-xl text-gray-700 mb-2">
          <strong>{domain}</strong> is blocked during your focus session.
        </p>
        <p className="text-sm text-gray-500 mb-6">
          Match type: {matchType}
        </p>
        <div className="space-y-3">
          <button
            onClick={onReturn || (() => window.location.href = '/')}
            className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition"
          >
            Return to Focus
          </button>
          <p className="text-xs text-gray-400">
            This site will be available when your session ends
          </p>
        </div>
      </div>
    </div>
  );
}
```

### Blocking Status Badge

```typescript
function BlockingStatusBadge() {
  const { blockedDomains } = useBlockedDomains();

  if (!blockedDomains?.enabled) return null;

  return (
    <div className="flex items-center gap-2 px-3 py-1 bg-red-100 text-red-800 rounded-full text-sm font-medium">
      <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
      <span>{blockedDomains.count} sites blocked</span>
    </div>
  );
}
```

## Testing

### Unit Test Example

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { mockIPC } from '@tauri-apps/api/mocks';

describe('Domain Blocking', () => {
  beforeEach(() => {
    mockIPC((cmd, args) => {
      if (cmd === 'get_blocked_domains') {
        return {
          domains: ['facebook.com', 'twitter.com'],
          enabled: true,
          count: 2,
          last_updated: '2025-12-30T12:00:00Z'
        };
      }

      if (cmd === 'check_url_blocked') {
        const url = args.url;
        const domain = new URL(url).hostname;

        if (domain.includes('facebook.com')) {
          return {
            blocked: true,
            matched_domain: 'facebook.com',
            match_type: 'subdomain'
          };
        }

        return {
          blocked: false,
          matched_domain: null,
          match_type: null
        };
      }
    });
  });

  it('should fetch blocked domains', async () => {
    const domains = await invoke('get_blocked_domains');

    expect(domains.enabled).toBe(true);
    expect(domains.count).toBe(2);
    expect(domains.domains).toContain('facebook.com');
  });

  it('should block Facebook URLs', async () => {
    const result = await invoke('check_url_blocked', {
      url: 'https://www.facebook.com/profile'
    });

    expect(result.blocked).toBe(true);
    expect(result.matched_domain).toBe('facebook.com');
  });

  it('should allow non-blocked URLs', async () => {
    const result = await invoke('check_url_blocked', {
      url: 'https://www.google.com'
    });

    expect(result.blocked).toBe(false);
  });
});
```

## Best Practices

### 1. Always Handle Errors

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

### 2. Cache Blocked Domains Locally

Don't invoke the backend for every URL check. Cache the list and sync periodically:

```typescript
// Good: Cache and check locally
const blocker = new DomainBlocker();
blocker.sync();  // Once on mount
if (blocker.isUrlBlocked(url)) { /* block */ }

// Bad: Invoke backend for every check
if ((await invoke('check_url_blocked', { url })).blocked) { /* block */ }
```

### 3. Show Clear Feedback

Tell users why they can't access a site:

```typescript
// Good: Clear message
"facebook.com is blocked during your focus session"

// Bad: Generic error
"Access denied"
```

### 4. Respect Blocking State

Don't try to bypass blocking. Users chose to block these sites:

```typescript
// Good: Respect user's choice
if (result.blocked) {
  showBlockingOverlay();
  return;
}

// Bad: Try to bypass
if (result.blocked) {
  if (confirm('Site is blocked. Open anyway?')) {
    window.open(url);  // Don't do this!
  }
}
```

### 5. Test Both Blocking States

```typescript
// Test with blocking enabled
const enabledResult = await invoke('check_url_blocked', { url });
expect(enabledResult.blocked).toBe(true);

// End session (disables blocking)
await invoke('end_focus_session', { completed: true });

// Test with blocking disabled
const disabledResult = await invoke('check_url_blocked', { url });
expect(disabledResult.blocked).toBe(false);
```

## Troubleshooting

### Issue: Domains not being blocked

**Check:**
1. Is blocking enabled? `blockedDomains.enabled === true`
2. Are domains in the list? `blockedDomains.domains.includes('facebook.com')`
3. Is the session active? Check `get_active_session()`

### Issue: Subdomains not matching

**Check:**
- Make sure you're checking the full domain, not just the base
- `api.facebook.com` should match `facebook.com` (parent domain)
- The backend handles subdomain matching automatically

### Issue: www. variants not working

**Check:**
- Backend automatically adds www. variants
- Both `example.com` and `www.example.com` are blocked when you block `example.com`

## Performance Tips

### 1. Minimize Backend Calls

```typescript
// Good: Fetch once, check locally
const { domains } = await invoke('get_blocked_domains');
const blockedSet = new Set(domains);

urls.forEach(url => {
  if (blockedSet.has(extractDomain(url))) {
    // block
  }
});

// Bad: Call backend for each URL
urls.forEach(async url => {
  const result = await invoke('check_url_blocked', { url });
  if (result.blocked) { /* block */ }
});
```

### 2. Use Debouncing for Real-Time Checks

```typescript
import { debounce } from 'lodash';

const checkUrl = debounce(async (url: string) => {
  const result = await invoke('check_url_blocked', { url });
  updateUI(result);
}, 300);

// User typing in address bar
input.addEventListener('input', (e) => {
  checkUrl(e.target.value);
});
```

### 3. Lazy Load Blocked Domains

```typescript
let blockedDomainsCache: BlockedDomainsResponse | null = null;

async function getBlockedDomains() {
  if (!blockedDomainsCache) {
    blockedDomainsCache = await invoke('get_blocked_domains');
  }
  return blockedDomainsCache;
}

// Invalidate cache on session changes
async function onSessionStart() {
  blockedDomainsCache = null;
  await getBlockedDomains();  // Reload
}
```

## Examples by Use Case

### Built-in Browser Tab

```typescript
function BrowserTab() {
  const [url, setUrl] = useState('');
  const [blocked, setBlocked] = useState(false);

  async function navigate(newUrl: string) {
    const result = await invoke('check_url_blocked', { url: newUrl });

    if (result.blocked) {
      setBlocked(true);
      showBlockingOverlay({
        domain: result.matched_domain,
        matchType: result.match_type
      });
      return;
    }

    setBlocked(false);
    setUrl(newUrl);
    // Actually navigate webview
  }

  return (
    <div>
      {blocked ? (
        <BlockingOverlay domain={url} />
      ) : (
        <webview src={url} />
      )}
    </div>
  );
}
```

### Settings Page

```typescript
function BlockingSettings() {
  const { blockedDomains, loading, refresh } = useBlockedDomains();

  if (loading) return <Spinner />;

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">Blocked Websites</h2>
        <BlockingStatusBadge />
      </div>

      <div className="bg-gray-100 rounded-lg p-4">
        <p className="text-sm text-gray-600 mb-2">
          {blockedDomains.enabled
            ? `${blockedDomains.count} sites currently blocked`
            : 'Blocking disabled'}
        </p>
        <ul className="space-y-2">
          {blockedDomains.domains.map(domain => (
            <li key={domain} className="flex items-center gap-2">
              <span className="text-red-600">🚫</span>
              <span className="font-mono text-sm">{domain}</span>
            </li>
          ))}
        </ul>
      </div>

      <button
        onClick={refresh}
        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        Refresh
      </button>
    </div>
  );
}
```

---

## Summary

The DNS fallback system provides four simple commands:

1. `get_blocked_domains()` - Get the list
2. `check_domain_blocked(domain)` - Check a domain
3. `check_url_blocked(url)` - Check a URL
4. `get_blocking_stats()` - Get statistics

Use these to build blocking UI that works even without system-level permissions.

For more details, see `DNS_FALLBACK_IMPLEMENTATION.md`.
