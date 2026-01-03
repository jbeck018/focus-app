# OAuth Setup Guide

Quick guide to set up OAuth credentials for Google Calendar and Microsoft Outlook integration.

## Prerequisites

- Google Cloud Console account
- Microsoft Azure account
- FocusFlow app (this repository)

## Google Calendar Setup

### 1. Create Google Cloud Project

1. Go to https://console.cloud.google.com
2. Click "Select a project" → "New Project"
3. Project name: `FocusFlow` (or your preferred name)
4. Click "Create"

### 2. Enable Google Calendar API

1. In the project dashboard, go to "APIs & Services" → "Library"
2. Search for "Google Calendar API"
3. Click "Google Calendar API" → "Enable"

### 3. Create OAuth 2.0 Credentials

1. Go to "APIs & Services" → "Credentials"
2. Click "+ CREATE CREDENTIALS" → "OAuth client ID"
3. If prompted, configure OAuth consent screen:
   - User Type: External
   - App name: FocusFlow
   - User support email: your email
   - Developer contact: your email
   - Scopes: Add `calendar.readonly` and `userinfo.email`
   - Test users: Add your Google account email
   - Click "Save and Continue"
4. Back to "Create OAuth client ID":
   - Application type: **Desktop app**
   - Name: FocusFlow Desktop
   - Click "Create"
5. Copy the **Client ID** (looks like `xxx.apps.googleusercontent.com`)

### 4. Configure Redirect URI

1. Click on your OAuth client in the credentials list
2. Under "Authorized redirect URIs", add:
   ```
   focusflow://oauth/callback
   ```
3. Click "Save"

### 5. Set Environment Variable

```bash
export GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
```

Or add to your shell profile (`~/.bashrc`, `~/.zshrc`):
```bash
echo 'export GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"' >> ~/.zshrc
source ~/.zshrc
```

## Microsoft Outlook Setup

### 1. Register Azure AD Application

1. Go to https://portal.azure.com
2. Navigate to "Azure Active Directory"
3. Click "App registrations" → "+ New registration"
4. Fill in details:
   - Name: FocusFlow
   - Supported account types: **Accounts in any organizational directory and personal Microsoft accounts**
   - Redirect URI: Select "Public client/native (mobile & desktop)"
   - Add redirect URI: `focusflow://oauth/callback`
5. Click "Register"

### 2. Copy Application ID

1. In the app overview page, copy the **Application (client) ID**
2. This is your Microsoft Client ID

### 3. Configure API Permissions

1. Click "API permissions" in the left sidebar
2. Click "+ Add a permission"
3. Select "Microsoft Graph"
4. Select "Delegated permissions"
5. Add these permissions:
   - `Calendars.Read`
   - `User.Read`
   - `offline_access`
6. Click "Add permissions"

Note: You don't need admin consent for these permissions when using personal accounts.

### 4. Set Environment Variable

```bash
export MICROSOFT_CLIENT_ID="your-application-id"
```

Or add to your shell profile:
```bash
echo 'export MICROSOFT_CLIENT_ID="your-application-id"' >> ~/.zshrc
source ~/.zshrc
```

## Verify Setup

### Check Environment Variables

```bash
echo $GOOGLE_CLIENT_ID
echo $MICROSOFT_CLIENT_ID
```

Both should output your client IDs.

### Run the App

```bash
cd packages/desktop
npm run tauri dev
```

The app should start without errors related to OAuth configuration.

## Testing OAuth Flow

### 1. Connect Google Calendar

1. Open FocusFlow app
2. Navigate to Settings → Calendar Integration
3. Click "Connect Google Calendar"
4. Browser opens with Google OAuth consent screen
5. Sign in and grant permissions
6. Browser redirects to `focusflow://oauth/callback?code=...`
7. App should show "Connected" status with your email

### 2. Connect Microsoft Outlook

1. Click "Connect Microsoft Outlook"
2. Browser opens with Microsoft OAuth consent screen
3. Sign in with Microsoft account
4. Grant permissions
5. Browser redirects to `focusflow://oauth/callback?code=...`
6. App should show "Connected" status with your email

### 3. Fetch Calendar Events

1. Navigate to Calendar view
2. Events from both calendars should appear merged
3. Check console logs for any errors

## Troubleshooting

### Google OAuth Errors

#### "redirect_uri_mismatch"
- Ensure redirect URI is exactly: `focusflow://oauth/callback`
- Check for typos (no trailing slash)
- Verify it's added in Google Cloud Console credentials

#### "access_denied"
- Make sure you've added your email as a test user in OAuth consent screen
- Check that Calendar API is enabled

#### "invalid_client"
- Verify `GOOGLE_CLIENT_ID` environment variable is set correctly
- Restart your terminal/IDE after setting environment variables

### Microsoft OAuth Errors

#### "AADSTS50011: The redirect URI specified in the request does not match"
- Ensure redirect URI is exactly: `focusflow://oauth/callback`
- Verify it's registered as "Public client/native" type
- Check Azure AD app registration settings

#### "AADSTS65001: The user or administrator has not consented"
- Verify API permissions include `Calendars.Read`, `User.Read`, `offline_access`
- Make sure "Accounts in any organizational directory and personal Microsoft accounts" is selected

#### "AADSTS7000218: The request body must contain the following parameter: client_assertion"
- Ensure redirect URI type is "Public client/native", NOT "Web"
- We're using PKCE, so no client secret is needed

### General Issues

#### No events showing
- Check browser console for API errors
- Verify OAuth tokens are stored in database:
  ```sql
  sqlite3 ~/Library/Application\ Support/com.focusflow.app/focusflow.db
  SELECT * FROM oauth_tokens;
  ```
- Check Tauri logs for token refresh errors

#### Deep link not working
- Verify Tauri deep link configuration in `tauri.conf.json`
- On macOS, check if custom URL scheme is registered:
  ```bash
  /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -dump | grep focusflow
  ```

## Security Notes

### Client IDs Are Not Secrets

OAuth 2.0 client IDs for desktop apps are **not** secrets. They're designed to be embedded in applications. Security comes from:

1. **PKCE flow** - Prevents authorization code interception
2. **Redirect URI validation** - OAuth provider validates redirect URI
3. **CSRF protection** - State parameter prevents CSRF attacks

### Never Commit Client Secrets

Desktop apps should **never** use client secrets. If you accidentally created web application credentials instead of desktop app credentials, delete them and create new desktop app credentials.

### Environment Variables vs Hardcoding

We use environment variables for flexibility:
- Development: Set in shell
- Production: Bundle as environment variables in app package
- Alternative: Store in secure config file (not in git)

## Production Deployment

For production builds, you have several options:

### Option 1: Build-time Environment Variables

```bash
GOOGLE_CLIENT_ID="xxx" MICROSOFT_CLIENT_ID="yyy" npm run tauri build
```

### Option 2: Runtime Configuration File

Store credentials in a config file outside the source tree:

```json
{
  "oauth": {
    "google_client_id": "xxx",
    "microsoft_client_id": "yyy"
  }
}
```

Load at runtime (requires code changes to read from config file).

### Option 3: User Configuration

Let users configure their own OAuth apps (advanced users only).

## Recommended: Use Separate OAuth Apps for Production

For production deployments:

1. Create separate OAuth apps for production
2. Use different client IDs for dev vs production
3. Restrict production app to your domain/organization
4. Set up proper OAuth consent screen branding
5. Request OAuth verification from Google (if public)

## Next Steps

After OAuth is working:

1. Test with multiple calendar events
2. Test token refresh (wait for token to expire)
3. Test disconnect and reconnect
4. Test with both providers connected simultaneously
5. Verify events are correctly merged and sorted
6. Test focus suggestions with real calendar data

## Support

If you encounter issues:

1. Check the logs in Tauri DevTools console
2. Review `OAUTH_IMPLEMENTATION.md` for architecture details
3. Check `src/oauth/README.md` for API documentation
4. Open an issue with error logs and steps to reproduce
