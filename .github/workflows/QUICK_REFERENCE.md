# Quick Reference Guide

## File Structure

```
.github/
├── workflows/
│   ├── ci.yml                          # Lint, test, build check
│   ├── release-desktop.yml             # Multi-platform desktop builds
│   ├── deploy-backend.yml              # Docker build & Fly.io deploy
│   ├── preview.yml                     # PR preview deployments
│   ├── SETUP.md                        # Detailed setup instructions
│   ├── TROUBLESHOOTING.md              # Common issues & solutions
│   ├── QUICK_REFERENCE.md              # This file
│   ├── .env.example                    # Secret names reference
│   └── scripts/
│       ├── setup-signing.sh            # Generate signing credentials
│       └── validate-config.sh          # Validate workflow config
```

## First Time Setup (5 minutes)

```bash
# 1. Copy workflow files
mkdir -p .github/workflows
cp workflows/*.yml .github/workflows/

# 2. Make scripts executable
chmod +x .github/workflows/scripts/*.sh

# 3. Validate configuration
.github/workflows/scripts/validate-config.sh

# 4. Setup secrets (interactive)
.github/workflows/scripts/setup-signing.sh

# 5. Add repository secrets via GitHub UI
# Settings → Secrets and variables → Actions
```

## Workflow Triggers

| Workflow | Trigger | Duration |
|----------|---------|----------|
| **CI** | Push to main, PR | 3-5 min |
| **Release** | Tag push `v*` or manual | 15-25 min |
| **Backend** | Changes in `docker/` or manual | 5-10 min |
| **Preview** | PR with `docker/` changes | 5-10 min |

## Common Commands

```bash
# Test locally before pushing
pnpm lint && pnpm typecheck && pnpm test

# Create a release
git tag v1.0.0
git push origin v1.0.0

# Deploy backend manually
# Go to: Actions → Deploy Backend → Run workflow

# View workflow logs
gh run view <run-id> --log

# Monitor recent runs
gh run list --limit 10
```

## Required Secrets Quick Checklist

```
CI:
  ☐ No secrets required (optional: TURBO_TOKEN, TURBO_TEAM)

Release:
  ☐ TAURI_SIGNING_PRIVATE_KEY
  ☐ TAURI_SIGNING_PRIVATE_KEY_PASSWORD
  ☐ APPLE_CERTIFICATE (base64)
  ☐ APPLE_CERTIFICATE_PASSWORD
  ☐ APPLE_SIGNING_IDENTITY
  ☐ APPLE_ID
  ☐ APPLE_TEAM_ID
  ☐ APPLE_PASSWORD (app-specific)
  ☐ WINDOWS_SIGN_CERT (optional, base64)
  ☐ WINDOWS_SIGN_CERT_PASSWORD (optional)

Backend Deploy:
  ☐ FLY_API_TOKEN
  ☐ FLY_APP_NAME

Preview Deploy:
  ☐ FLY_API_TOKEN (shared)
  ☐ FLY_ORG (optional)
```

## Typical Workflow Progression

```
Feature Development
    ↓
Push to GitHub
    ↓
CI Workflow (automated)
  - Lint ✓
  - Type check ✓
  - Frontend tests ✓
  - Rust tests ✓
  - Build check ✓
    ↓
Code Review & Merge
    ↓
Docker changes in main?
    ↓ YES
Deploy Backend Workflow
  - Build Docker image
  - Push to GHCR
  - Deploy to Fly.io
  - Health checks
  - Auto-rollback if failed
    ↓
Time for release?
    ↓ YES (git tag v1.0.0)
Release Desktop Workflow
  - Build macOS (arm64, x86_64)
  - Build Windows (x86_64)
  - Build Linux (x86_64)
  - Code sign all platforms
  - Upload to GitHub Releases
    ↓
Users download & install
```

## Performance Benchmarks

```
CI Workflow:
  ✓ Pnpm cache hit: 1-2 min
  ✗ Pnpm cache miss: 3-4 min
  ✓ Turbo cache hit: +0.5 min
  ✓ Rust cache hit: +1-2 min
  Expected total: 3-5 min

Release Workflow (per platform):
  macOS arm64:  6-8 min
  macOS x86_64: 6-8 min
  Windows:      8-12 min
  Linux:        5-7 min
  Artifacts:    2-3 min
  Expected total: 15-25 min (parallel)

Backend Deploy:
  Docker build: 2-3 min
  Push to GHCR: 1 min
  Deploy: 2-3 min
  Health check: 1-2 min
  Expected total: 5-10 min
```

## Debugging Quick Tips

| Problem | Check |
|---------|-------|
| Workflow not running | Enable Actions in Settings |
| Cache not working | Verify `pnpm-lock.yaml` in git |
| Release not triggered | Tag format: `v1.0.0` + `git push --tags` |
| Signing fails | Verify secret base64 encoding |
| Deploy fails | Check Fly.io logs: `flyctl logs` |
| Preview not created | Verify `FLY_API_TOKEN` secret exists |

## Secrets Management Cheat Sheet

**Generate Tauri signing key:**
```bash
cd packages/desktop
pnpm tauri signer generate -w ~/.tauri/private.key
```

**Base64 encode for GitHub:**
```bash
# macOS
base64 -i file.p12 | pbcopy

# Linux
base64 file.p12 | xclip -selection clipboard

# Windows
[Convert]::ToBase64String([IO.File]::ReadAllBytes("file.pfx")) | Set-Clipboard
```

**Get Apple Team ID:**
```bash
security find-identity -v -p codesigning | grep "Developer ID"
# Output: "XXX XXXXXXXXXX Developer ID Application: Name (TEAM_ID)"
```

**Create Fly.io token:**
```bash
flyctl tokens create --name "github-actions"
```

## Performance Optimization

### Reduce CI Time
```yaml
# Use matrix builds for parallel execution
strategy:
  matrix:
    job: [lint, typecheck, test]
  max-parallel: 3
```

### Reduce Release Time
```yaml
# Build all platforms in parallel
strategy:
  matrix:
    os: [macos, windows, linux]
  max-parallel: 3
```

### Cache Best Practices
```yaml
# Cache pnpm store
- uses: actions/cache@v4
  with:
    path: ~/.pnpm-store
    key: pnpm-cache-${{ hashFiles('**/pnpm-lock.yaml') }}

# Cache Rust builds
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: src-tauri

# Docker layer caching
cache-from: type=gha
cache-to: type=gha,mode=max
```

## Cost Optimization

| Action | Cost Impact |
|--------|-------------|
| Parallel CI jobs | ~1 min/run = ~150 min/month |
| Release workflow | ~20 min × 4 = ~3 hours/month |
| Preview deploys | ~5 min × 5 = ~25 min/month |
| Backend deploy | ~7 min × 2 = ~14 min/month |
| **Total** | **~4-5 hours/month** |

**Free tier: 2,000 min/month = 33 hours**
Your usage fits comfortably in free tier!

## Environment Variables (Built-in)

```
github.actor              # User who triggered workflow
github.ref                # Branch/tag reference
github.ref_name           # Branch/tag name only
github.sha                # Commit SHA
github.repository         # Owner/repo
github.event_name         # Event type (push, pull_request, etc)
github.server_url         # GitHub server URL
runner.os                 # ubuntu, macos, windows
```

## Useful Actions

```yaml
# Actions used in workflows:
actions/checkout@v4                    # Clone repository
actions/setup-node@v4                  # Setup Node.js
pnpm/action-setup@v2                   # Setup pnpm
dtolnay/rust-toolchain@stable          # Setup Rust
Swatinem/rust-cache@v2                 # Cache Rust builds
docker/setup-buildx-action@v3          # Setup Docker builder
docker/build-push-action@v5            # Build & push Docker
superfly/flyctl-actions/setup-flyctl   # Setup Fly CLI
softprops/action-gh-release@v1         # Create GitHub release
actions/github-script@v7               # Run JS in workflow
```

## Monitoring Links

```
GitHub Actions:
  https://github.com/<owner>/<repo>/actions

Fly.io Console:
  https://fly.io/apps/<app-name>

Docker Images (GHCR):
  https://github.com/<owner>/<repo>/pkgs/container/<image>

GitHub Releases:
  https://github.com/<owner>/<repo>/releases
```

## Support Resources

- **Tauri Docs:** https://tauri.app/docs
- **GitHub Actions:** https://docs.github.com/en/actions
- **Fly.io Docs:** https://fly.io/docs
- **Rust Guide:** https://doc.rust-lang.org/book
- **Docker Guide:** https://docs.docker.com/get-started

## Common Patterns

### Conditional Job Execution
```yaml
if: github.ref == 'refs/heads/main'
if: startsWith(github.ref, 'refs/tags/v')
if: success()  # Run only if previous succeeded
if: failure()  # Run only if previous failed
if: always()   # Always run
```

### Environment-Specific Config
```yaml
jobs:
  deploy:
    environment:
      name: production
      url: https://app.example.com
    steps:
      - run: deploy.sh
```

### Artifact Management
```yaml
# Upload
- uses: actions/upload-artifact@v4
  with:
    name: build-output
    path: dist/
    retention-days: 7

# Download
- uses: actions/download-artifact@v4
  with:
    name: build-output
    path: ./build
```

### Matrix Builds
```yaml
strategy:
  matrix:
    os: [macos, windows, linux]
    arch: [x86_64, arm64]
    exclude:
      - os: windows
        arch: arm64  # Windows only supports x86_64
```

## Emergency Procedures

### Disable workflow temporarily
```bash
# Rename workflow file (actions ignores disabled files)
mv .github/workflows/release-desktop.yml .github/workflows/release-desktop.yml.disabled
git commit -m "Disable release workflow temporarily"
git push
```

### Force rebuild from scratch
```bash
# Clear all caches
# Settings → Actions → General → Actions cache → Clear all

# Or re-run with cache cleared:
gh run rerun <run-id> --cache-only
```

### Rollback production deployment
```bash
flyctl releases rollback  # Rolls back to previous release
```

### Cancel running workflow
```bash
gh run cancel <run-id>
```

## Next Steps

1. ✓ Copy workflows to `.github/workflows/`
2. ✓ Run validation script
3. ✓ Setup secrets via GitHub UI
4. ✓ Push to trigger CI
5. ✓ Create test tag `v0.1.0` for release
6. ✓ Monitor logs and verify all passes
7. ✓ Customize for your project specifics
