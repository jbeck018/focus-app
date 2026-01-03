# GitHub Actions Workflows Setup Guide

This document provides comprehensive setup instructions for all CI/CD workflows.

## Quick Start

1. Copy workflow files to `.github/workflows/`
2. Configure repository secrets (see sections below)
3. Enable GitHub Actions in repository settings
4. Push code or create PR to trigger workflows

## Workflows Overview

### 1. CI Workflow (`ci.yml`)

Runs on every push to main and pull requests. Tests, lints, and builds the application.

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch

**Jobs:**
- `lint`: ESLint, Prettier checks via Turbo
- `typecheck`: TypeScript compilation checks
- `test-frontend`: Frontend unit tests
- `test-rust`: Rust unit tests and clippy checks
- `build-check`: Full Tauri build verification

**No secrets required** for CI workflow.

**Cache Strategy:**
- pnpm store: `~/.pnpm-store`
- Rust target: `.cargo/registry`
- GitHub Actions cache for build artifacts

### 2. Desktop Release Workflow (`release-desktop.yml`)

Builds and publishes cross-platform desktop applications.

**Triggers:**
- Tag push: `git tag v1.0.0 && git push --tags`
- Manual: Workflow dispatch with version input

**Platforms:**
- macOS: arm64 (Apple Silicon) + x86_64 (Intel)
- Windows: x86_64
- Linux: x86_64

**Supported Formats:**
- macOS: `.dmg`, `.tar.gz`
- Windows: `.msi`, `.exe` (NSIS)
- Linux: `.deb`, `.AppImage`, `.tar.gz`

**Required Secrets:**

#### Tauri Updater
```
TAURI_SIGNING_PRIVATE_KEY          # Private key for signing updates
TAURI_SIGNING_PRIVATE_KEY_PASSWORD # Password for private key
```

Generate these with:
```bash
cd packages/desktop
pnpm tauri signer generate -w ~/.tauri/private.key
```

Store the private key in `TAURI_SIGNING_PRIVATE_KEY` (base64 encoded if multi-line).

#### macOS Code Signing (all platforms)
```
APPLE_CERTIFICATE               # P12 certificate (base64 encoded)
APPLE_CERTIFICATE_PASSWORD      # P12 password
APPLE_SIGNING_IDENTITY          # Identity: "Developer ID Application: Your Name (TEAM_ID)"
APPLE_ID                        # Apple ID email
APPLE_TEAM_ID                   # Team ID
APPLE_PASSWORD                  # App-specific password
```

**macOS Setup:**

1. Get Apple Developer Certificate:
   ```bash
   # Export p12 certificate from Keychain
   security export-ics ~/Desktop/certificate.p12 -k ~/Library/Keychains/login.keychain
   ```

2. Base64 encode:
   ```bash
   base64 -i certificate.p12 | pbcopy
   ```

3. Create app-specific password:
   - Sign in to appleid.apple.com
   - Security settings -> App-specific passwords
   - Generate for "GitHub Actions"

4. Get Team ID:
   ```bash
   security find-identity -v -p codesigning | grep "Developer ID"
   # Output: "XXXXXXXXXX Developer ID Application: Your Name (TEAM_ID)"
   ```

#### Windows Code Signing (optional)
```
WINDOWS_SIGN_CERT               # PFX certificate (base64 encoded)
WINDOWS_SIGN_CERT_PASSWORD      # Certificate password
```

If not provided, Windows builds proceed unsigned.

#### Optional: Turbo Cache
```
TURBO_TOKEN                     # Remote cache token
TURBO_TEAM                      # Team name for Turbo
```

### 3. Backend Deploy Workflow (`deploy-backend.yml`)

Builds Docker image and deploys to Fly.io.

**Triggers:**
- Push to `main` when `docker/` changes
- Manual workflow dispatch

**Jobs:**
1. Build Docker image and push to GHCR
2. Deploy to Fly.io
3. Health checks (HTTP 200 on `/health`)
4. Auto-rollback on failure
5. Deployment summary

**Required Secrets:**

```
FLY_API_TOKEN                   # Fly.io API token
FLY_APP_NAME                    # Fly.io app name (e.g., "focus-app-prod")
```

**Fly.io Setup:**

1. Create Fly.io account and app:
   ```bash
   flyctl auth login
   flyctl launch --name focus-app-prod
   ```

2. Generate API token:
   ```bash
   flyctl tokens create readonly
   ```

3. Store in GitHub Secrets:
   - Go to Settings -> Secrets and variables -> Actions
   - Add `FLY_API_TOKEN`
   - Add `FLY_APP_NAME`

4. Configure `fly.toml` in root:
   ```toml
   app = "focus-app-prod"
   primary_region = "dfw"

   [http_service]
     internal_port = 8000

   [[services]]
     protocol = "tcp"
     internal_port = 8000
     [[services.ports]]
       port = 80
       handlers = ["http"]
     [[services.ports]]
       port = 443
       handlers = ["tls", "http"]
   ```

5. Health check endpoints required:
   - `GET /health` - Returns 200 when ready
   - `GET /api/health` - API health
   - `GET /api/db-status` - Database connectivity

**Rollback Behavior:**
- Automatic rollback triggers on:
  - Health check failure
  - Deployment timeout
- Rolls back to previous release automatically
- Posts failure notification as comment

### 4. Preview Deploy Workflow (`preview.yml`)

Deploys preview instances for pull requests.

**Triggers:**
- PR opened/updated with changes in `docker/`
- PR closed (cleanup)

**Features:**
- Creates unique preview app: `preview-pr-<number>`
- Comments PR with preview URL
- Auto-cleanup on PR close
- Auto-pause idle instances

**Required Secrets:**

```
FLY_API_TOKEN                   # Fly.io API token (shared with main deploy)
FLY_ORG                         # Fly.io organization (optional)
```

**Configuration:**
- Preview apps use 512MB RAM, 1 CPU
- Minimal config for cost efficiency
- Auto-stop when idle
- Force HTTPS disabled for local testing

## Setting Up Repository Secrets

1. Go to repository Settings
2. Navigate to Secrets and variables → Actions
3. Click "New repository secret"
4. Add each secret from above

### Secret Format Notes

**Multi-line secrets (like PEM files):**
- Base64 encode them first
- Or: copy entire content including newlines
- GitHub Actions will handle escaping

**Environment-specific secrets:**
- Can use repository environments
- Settings → Environments → Production
- Restrict to specific branches/tags

## Caching Strategy

### pnpm Cache
```yaml
- uses: actions/cache@v4
  with:
    path: ~/.pnpm-store
    key: pnpm-cache-${{ runner.os }}-${{ hashFiles('**/pnpm-lock.yaml') }}
```

### Rust Cache
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: src-tauri
```

### Docker Build Cache
```yaml
cache-from: type=gha
cache-to: type=gha,mode=max
```

**Cache hit rate targets:**
- CI runs: 80%+ (pnpm + Rust)
- Release builds: 70%+ (Docker)

## Performance Tuning

### CI Workflow
- Parallel job execution: lint, typecheck, test run simultaneously
- Turbo caching: Reuses built packages and artifacts
- Expected duration: 3-5 minutes

### Release Workflow
- Matrix builds: All platforms built in parallel
- Artifact upload: Direct to GitHub Releases
- Expected duration: 15-25 minutes per platform

### Backend Deploy
- Docker layer caching: Reuses most layers
- Fly.io API: Streamlined deployment
- Health checks: 5-minute timeout
- Expected duration: 5-10 minutes

## Troubleshooting

### Workflow Not Triggering

**CI Workflow:**
- Verify `.github/workflows/ci.yml` exists
- Check branch protection rules (may block PR merges)
- Ensure no path filters blocking changes

**Release Workflow:**
- Tag format must be `v*` (e.g., `v1.0.0`)
- Command: `git tag v1.0.0 && git push --tags`
- Check "Releases" tab in repository

**Backend Deploy:**
- Check if `docker/` directory has actual changes
- Manual trigger via "Actions" tab

### Secret-Related Issues

**"Secrets not found" error:**
- Verify secret name matches exactly
- Check organization vs. repository secrets
- For environment secrets, verify job references environment

**Signing failures (macOS/Windows):**
- Verify certificate base64 encoding
- Check certificate password is correct
- Ensure signing identity exists: `security find-identity -v`

### Build Failures

**Cache misses causing slowdown:**
- First run is slower due to cache population
- Subsequent runs should be faster
- Force cache refresh: Delete actions cache in Settings

**Out of disk space:**
- Linux builds can be large (Rust + Node)
- Consider using self-hosted runners for releases
- Monitor artifact retention settings

## Self-Hosted Runners (Optional)

For faster builds, use self-hosted macOS runner for release builds:

```yaml
runs-on: [self-hosted, macos, arm64]
```

**Setup:**
1. Go to Settings → Actions → Runners
2. Add self-hosted runner
3. Follow GitHub's setup script
4. Tag as `macos` and `arm64`

## Monitoring & Alerts

### GitHub Notifications
- Enable: Settings → Code security and analysis
- Subscribe to workflow runs

### Custom Webhooks
Add webhook to Slack/Discord for deployments:

```bash
# In deploy-backend.yml
- name: Notify Slack
  uses: slackapi/slack-github-action@v1.24
  with:
    webhook-url: ${{ secrets.SLACK_WEBHOOK }}
    payload: |
      {
        "text": "Deployment to Fly.io: ${{ job.status }}"
      }
```

## Best Practices

1. **Use concurrency groups:** Prevents duplicate runs
2. **Set environment protection rules:** Require approvals for production
3. **Lock action versions:** Use commit SHA or specific releases
4. **Rotate secrets regularly:** Especially signing certificates
5. **Monitor job duration:** Alert on unusual slowness
6. **Review logs:** Check for warnings or deprecations
7. **Test locally:** Run Tauri build locally before release
8. **Validate artifacts:** Spot-check downloaded releases

## Cost Optimization

### GitHub-Hosted Runners
- Free tier: 2,000 minutes/month
- With CI + releases: ~100-150 min/month
- With previews: ~50-100 min/month per active PR

### Fly.io
- Free tier includes resources
- Preview apps: ~$2-5/month each
- Production: Depends on machine size

**Cost saving tips:**
- Merge PRs regularly to limit active previews
- Use auto-pause for preview apps
- Cache aggressively to reduce build time
- Disable unnecessary jobs (e.g., Windows builds if not needed)

## Next Steps

1. Copy all workflow files to `.github/workflows/`
2. Configure secrets for your platform
3. Test CI on a test branch
4. Create a test release with `v0.1.0` tag
5. Monitor first few deployments
6. Adjust timeouts/resources as needed
