# OAuth Integration Module

This module implements OAuth 2.0 authentication with PKCE flow for Google Calendar and Microsoft Outlook integration in the FocusFlow desktop application.

## Architecture

### Module Structure

```
src/oauth/
├── mod.rs           # Module exports and PKCE utilities
├── provider.rs      # OAuthProvider trait and shared types
├── token.rs         # TokenManager for secure storage
├── google.rs        # Google Calendar OAuth implementation
├── microsoft.rs     # Microsoft Outlook OAuth implementation
└── README.md        # This file
```

### Design Principles

1. **Trait-Based Architecture**: `OAuthProvider` trait enables easy addition of new providers
2. **PKCE Flow**: Uses Proof Key for Code Exchange (no client secret in desktop app)
3. **Secure Token Storage**: SQLite database with automatic token refresh
4. **DRY Code**: Shared logic in trait, provider-specific code isolated
5. **Thread-Safe**: Arc + RwLock for concurrent access across Tauri commands

## OAuth Flow

### 1. Start OAuth Flow

**Tauri Command**: `start_calendar_oauth(provider: String)`

1. Generate PKCE code verifier (random 32 bytes, base64url encoded)
2. Generate code challenge (SHA256 hash of verifier, base64url encoded)
3. Generate CSRF state token (random 32 bytes, base64url encoded)
4. Store PKCE verifier in memory (keyed by state)
5. Build authorization URL with challenge and state
6. Return URL to frontend to open in browser

```rust
// Frontend call
const { url, state } = await invoke('start_calendar_oauth', {
    provider: 'google'
});
window.open(url, '_blank');
```

### 2. Complete OAuth Flow

**Tauri Command**: `complete_calendar_oauth(provider: String, code: String, state: String)`

1. Validate state parameter (CSRF protection)
2. Retrieve PKCE verifier from memory
3. Exchange authorization code for tokens using verifier
4. Fetch user email from provider API
5. Store tokens in database via TokenManager
6. Return connection status

```rust
// Frontend call (from deep link handler)
await invoke('complete_calendar_oauth', {
    provider: 'google',
    code: code,
    state: state
});
```

### 3. Fetch Calendar Events

**Tauri Command**: `get_calendar_events(start_date: String, end_date: String)`

1. Check if provider tokens exist
2. Get valid access token (auto-refresh if expired)
3. Call provider API to fetch events
4. Merge events from all connected providers
5. Return sorted list of events

## Provider Implementations

### Google Calendar

**OAuth Endpoints**:
- Authorization: `https://accounts.google.com/o/oauth2/v2/auth`
- Token Exchange: `https://oauth2.googleapis.com/token`
- API: `https://www.googleapis.com/calendar/v3/`

**Scopes**:
- `https://www.googleapis.com/auth/calendar.readonly`
- `https://www.googleapis.com/auth/userinfo.email`

**Configuration**:
- Client ID: Set via `GOOGLE_CLIENT_ID` environment variable
- Redirect URI: `focusflow://oauth/callback`

### Microsoft Outlook

**OAuth Endpoints**:
- Authorization: `https://login.microsoftonline.com/common/oauth2/v2.0/authorize`
- Token Exchange: `https://login.microsoftonline.com/common/oauth2/v2.0/token`
- API: `https://graph.microsoft.com/v1.0/me/calendar/`

**Scopes**:
- `offline_access`
- `Calendars.Read`
- `User.Read`

**Configuration**:
- Client ID: Set via `MICROSOFT_CLIENT_ID` environment variable
- Redirect URI: `focusflow://oauth/callback`

## Token Management

### TokenManager

Thread-safe token storage and refresh manager.

**Features**:
- Automatic token refresh (5 min buffer before expiry)
- In-memory cache for performance
- SQLite database for persistence
- Support for multiple providers

**Database Schema** (Migration 16):
```sql
CREATE TABLE oauth_tokens (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL UNIQUE,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at INTEGER NOT NULL,
    scopes TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

**Methods**:
```rust
// Store tokens
token_manager.store_token(provider, &token_response, Some(email)).await?;

// Get valid token (auto-refresh)
let token = token_manager.get_valid_token(provider, provider_impl).await?;

// Delete token
token_manager.delete_token(provider).await?;

// Check if token exists
let has_token = token_manager.has_token(provider).await;
```

## Security Considerations

### PKCE Flow

**Why PKCE?**
Desktop applications cannot securely store client secrets. PKCE (RFC 7636) provides security without requiring a client secret.

**How it works**:
1. Generate random `code_verifier` (43-128 chars)
2. Hash verifier with SHA256 → `code_challenge`
3. Send challenge to authorization server
4. Exchange code + verifier for tokens
5. Server validates: `SHA256(verifier) == challenge`

### CSRF Protection

- Random `state` parameter generated for each OAuth flow
- State stored in memory during authorization
- Validated on callback before token exchange
- Prevents authorization code interception attacks

### Token Storage

- Tokens stored in SQLite database (encrypted at OS level)
- Access tokens refreshed automatically before expiry
- Refresh tokens stored securely for long-term access
- No tokens exposed to frontend (managed in Rust backend)

## Adding a New Provider

To add support for a new calendar provider (e.g., Apple Calendar):

### 1. Create Provider Implementation

Create `src/oauth/apple.rs`:

```rust
use async_trait::async_trait;
use crate::oauth::provider::{OAuthProvider, TokenResponse, CalendarEvent};

pub struct AppleCalendar {
    client_id: String,
    redirect_uri: String,
    http_client: reqwest::Client,
}

#[async_trait]
impl OAuthProvider for AppleCalendar {
    fn provider_name(&self) -> &'static str {
        "apple"
    }

    fn auth_url(&self, state: &str, code_challenge: &str) -> String {
        // Build Apple OAuth URL with PKCE
        format!(...)
    }

    async fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<TokenResponse> {
        // Exchange code for tokens
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        // Refresh access token
    }

    async fn fetch_events(&self, token: &str, start: DateTime<Utc>, end: DateTime<Utc>)
        -> Result<Vec<CalendarEvent>> {
        // Fetch calendar events
    }

    async fn get_user_email(&self, token: &str) -> Result<String> {
        // Get user email
    }
}
```

### 2. Update mod.rs

```rust
pub mod apple;
```

### 3. Add to AppState

In `src/state.rs`:

```rust
pub struct AppState {
    // ... existing fields
    pub apple_calendar: Arc<AppleCalendar>,
}

// In AppState::new():
let apple_calendar = AppleCalendar::default(apple_client_id);
// ... add to AppState construction
```

### 4. Update Calendar Commands

In `src/commands/calendar.rs`:

```rust
pub enum CalendarProvider {
    Google,
    Microsoft,
    Apple,  // Add new provider
}

// Add to start_calendar_oauth
CalendarProvider::Apple => {
    state.apple_calendar.auth_url(&csrf_state, &pkce.code_challenge)
}

// Add to complete_calendar_oauth
CalendarProvider::Apple => {
    let token = state.apple_calendar.exchange_code(&code, &pkce.code_verifier).await?;
    let email = state.apple_calendar.get_user_email(&token.access_token).await?;
    (token, email)
}

// Add fetch helper function
async fn fetch_apple_events(...) -> Result<Vec<CalendarEvent>> {
    // Implementation
}
```

## Environment Variables

Set these environment variables before running the app:

```bash
# Google Calendar
export GOOGLE_CLIENT_ID="your-google-client-id.apps.googleusercontent.com"

# Microsoft Outlook
export MICROSOFT_CLIENT_ID="your-microsoft-client-id"

# Optional: Apple Calendar (if implemented)
export APPLE_CLIENT_ID="your-apple-client-id"
```

## Registering OAuth Applications

### Google Cloud Console

1. Go to https://console.cloud.google.com
2. Create a new project or select existing
3. Enable Google Calendar API
4. Create OAuth 2.0 credentials:
   - Application type: Desktop app
   - Authorized redirect URI: `focusflow://oauth/callback`
5. Copy Client ID

### Microsoft Azure Portal

1. Go to https://portal.azure.com
2. Navigate to Azure Active Directory → App registrations
3. Create new registration:
   - Name: FocusFlow
   - Supported account types: Personal Microsoft accounts
   - Redirect URI: `focusflow://oauth/callback` (Public client/native)
4. Copy Application (client) ID
5. Add API permissions:
   - Microsoft Graph → Delegated → Calendars.Read
   - Microsoft Graph → Delegated → User.Read
   - Microsoft Graph → Delegated → offline_access

## Testing

### Unit Tests

```bash
cd src-tauri
cargo test oauth
```

### Manual Testing

1. Set environment variables with test OAuth credentials
2. Run app in development mode
3. Open Calendar Integration page
4. Click "Connect Google Calendar" or "Connect Microsoft Outlook"
5. Complete OAuth flow in browser
6. Verify events appear in calendar view

## Troubleshooting

### "Invalid OAuth state parameter"

- Check that redirect URI matches exactly in OAuth provider settings
- Verify deep link handler is properly configured in Tauri

### Token refresh fails

- Check that `offline_access` scope is requested (Microsoft)
- Verify `access_type=offline` is set (Google)
- Ensure refresh token is being stored

### Events not fetching

- Verify scopes include calendar read permissions
- Check token expiry and refresh logic
- Review API rate limits

### CORS errors

- OAuth flow happens in browser, not affected by CORS
- API calls happen server-side (Rust), not affected by CORS
- If seeing CORS, check that you're using backend API calls

## Dependencies

Required Cargo dependencies (already added):

```toml
async-trait = "0.1"       # Async trait methods
sha2 = "0.10"             # SHA256 for PKCE
rand = "0.8"              # Random generation
base64 = "0.22"           # Base64 encoding
urlencoding = "2.1"       # URL parameter encoding
reqwest = "0.12"          # HTTP client
chrono = "0.4"            # Date/time handling
```

## Future Enhancements

1. **Token encryption**: Encrypt tokens at rest using OS keychain
2. **Token revocation**: Implement token revocation on disconnect
3. **Multiple calendars**: Support multiple Google/Microsoft accounts
4. **Calendar sync**: Write events back to calendar (requires write scopes)
5. **Offline mode**: Cache events for offline access
6. **Rate limiting**: Implement exponential backoff for API calls
7. **Health checks**: Periodic token validation and health monitoring

## References

- [RFC 7636: PKCE](https://tools.ietf.org/html/rfc7636)
- [Google Calendar API](https://developers.google.com/calendar/api)
- [Microsoft Graph API](https://docs.microsoft.com/en-us/graph/api/resources/calendar)
- [OAuth 2.0 for Desktop Apps](https://datatracker.ietf.org/doc/html/rfc8252)
