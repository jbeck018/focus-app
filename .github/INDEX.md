# GitHub Actions Workflows - Complete Index

## Project Structure

```
focus-app/
├── .github/
│   ├── INDEX.md (this file)                    # Navigation guide
│   ├── WORKFLOWS_SUMMARY.md                    # Comprehensive overview
│   └── workflows/
│       ├── QUICK_REFERENCE.md                  # ⚡ Start here for quick answers
│       ├── SETUP.md                            # 📖 Detailed setup guide
│       ├── TROUBLESHOOTING.md                  # 🔧 Problem solving
│       ├── .env.example                        # Secret configuration template
│       ├── ci.yml                              # CI pipeline
│       ├── release-desktop.yml                 # Desktop app releases
│       ├── deploy-backend.yml                  # Backend deployment
│       ├── preview.yml                         # PR previews
│       └── scripts/
│           ├── setup-signing.sh                # Generate credentials
│           └── validate-config.sh              # Validate setup
├── fly.toml.example                            # Fly.io configuration
└── docker/
    └── Dockerfile.example                      # Container image definition
```

## Getting Started (Choose Your Path)

### Path 1: I'm in a Hurry
**Time: 5 minutes**

1. Read: [QUICK_REFERENCE.md](./workflows/QUICK_REFERENCE.md)
2. Run: `.github/workflows/scripts/validate-config.sh`
3. Add secrets to GitHub UI
4. Push and watch workflows run

### Path 2: I Want to Understand Everything
**Time: 30 minutes**

1. Read: [WORKFLOWS_SUMMARY.md](./WORKFLOWS_SUMMARY.md) - Overview
2. Read: [SETUP.md](./workflows/SETUP.md) - Detailed setup
3. Run: `.github/workflows/scripts/setup-signing.sh`
4. Follow checklist in SETUP.md

### Path 3: I'm Stuck/Troubleshooting
**Time: Varies**

1. Check: [TROUBLESHOOTING.md](./workflows/TROUBLESHOOTING.md)
2. Search for your issue
3. Follow the solution steps
4. Run: `.github/workflows/scripts/validate-config.sh`

## File Guide

### Workflow Files (`.github/workflows/*.yml`)

#### `ci.yml` - Continuous Integration
**When:** Every push to main, every PR
**What:** Lint, type check, test, build check
**Duration:** 3-5 minutes
**Secrets:** None required

**Read More:** [QUICK_REFERENCE.md](./workflows/QUICK_REFERENCE.md#common-commands)

#### `release-desktop.yml` - Desktop Releases
**When:** Tag push (v*) or manual trigger
**What:** Build macOS (arm64 + x86_64), Windows, Linux
**Duration:** 15-25 minutes
**Secrets:** TAURI_SIGNING_PRIVATE_KEY, APPLE_* secrets, etc.

**Read More:** [SETUP.md](./workflows/SETUP.md#2-desktop-release-workflow) → Desktop Release Workflow

#### `deploy-backend.yml` - Backend Deployment
**When:** Changes in docker/, or manual trigger
**What:** Build Docker, push to GHCR, deploy to Fly.io, health checks
**Duration:** 5-10 minutes
**Secrets:** FLY_API_TOKEN, FLY_APP_NAME

**Read More:** [SETUP.md](./workflows/SETUP.md#3-backend-deploy-workflow) → Backend Deploy Workflow

#### `preview.yml` - PR Preview Deployments
**When:** PR opened with docker/ changes, PR closed
**What:** Create temporary preview app, comment PR, cleanup on close
**Duration:** 5-10 minutes
**Secrets:** FLY_API_TOKEN, FLY_ORG (optional)

**Read More:** [SETUP.md](./workflows/SETUP.md#4-preview-deploy-workflow) → Preview Deploy Workflow

### Documentation Files

#### `WORKFLOWS_SUMMARY.md` (Root Level)
**What:** Complete project overview
**Best for:** Understanding the big picture
**Contains:**
- Architecture overview
- All workflows at a glance
- Caching strategy
- Security practices
- Cost analysis
- Customization guide

#### `SETUP.md`
**What:** Step-by-step setup instructions
**Best for:** First-time configuration
**Contains:**
- Environment requirements
- Secret generation
- Platform-specific setup (macOS, Windows, Fly.io)
- Cache configuration
- Troubleshooting for setup issues

#### `QUICK_REFERENCE.md`
**What:** Quick commands and checklists
**Best for:** Day-to-day operations
**Contains:**
- Common commands
- File structure
- Required secrets checklist
- Performance benchmarks
- Emergency procedures

#### `TROUBLESHOOTING.md`
**What:** Solutions for common issues
**Best for:** When something breaks
**Contains:**
- CI workflow issues
- Release workflow issues
- Backend deploy issues
- Preview deploy issues
- Debugging techniques
- Monitoring links

#### `.env.example`
**What:** Template for all secrets
**Best for:** Reference when adding secrets
**Contains:**
- All secret names
- Which secrets are required
- How to generate each secret
- Descriptions of each secret

### Script Files (`.github/workflows/scripts/`)

#### `setup-signing.sh`
**Purpose:** Interactive secret generation
**Usage:**
```bash
chmod +x .github/workflows/scripts/setup-signing.sh
.github/workflows/scripts/setup-signing.sh
```
**What it does:**
- Generates Tauri signing keys
- Encodes macOS certificates (base64)
- Encodes Windows certificates (base64)
- Shows exact values to paste into GitHub Secrets

#### `validate-config.sh`
**Purpose:** Verify workflow configuration
**Usage:**
```bash
chmod +x .github/workflows/scripts/validate-config.sh
.github/workflows/scripts/validate-config.sh
```
**What it does:**
- Checks file structure
- Validates YAML syntax
- Verifies project dependencies
- Checks for hardcoded secrets
- Provides recommendations

### Example Configuration Files

#### `fly.toml.example` (Root Level)
**Purpose:** Fly.io configuration template
**Copy to:** `fly.toml` (in project root)
**Configure:**
- App name
- Region
- Machine resources
- Health check endpoints

#### `docker/Dockerfile.example`
**Purpose:** Docker image for backend
**Copy to:** `docker/Dockerfile`
**Contains:**
- Multi-stage build
- Node.js 20 Alpine base
- Non-root user
- Health checks

## Quick Decision Tree

```
I need to...

├─ Setup workflows for first time
│  └─ Run: setup-signing.sh
│  └─ Read: SETUP.md
│
├─ Deploy desktop application
│  └─ Create: git tag v1.0.0
│  └─ Push: git push origin v1.0.0
│  └─ Monitor: GitHub Actions → Release Desktop
│
├─ Deploy backend changes
│  └─ Push: code with docker/ changes
│  └─ Monitor: GitHub Actions → Deploy Backend
│
├─ Create PR preview
│  └─ Make: PR with docker/ changes
│  └─ Wait: Preview Deploy workflow
│  └─ Check: PR comments for preview URL
│
├─ Debug a failing workflow
│  └─ Check: TROUBLESHOOTING.md for your error
│  └─ Run: .github/workflows/scripts/validate-config.sh
│
├─ Understand what's happening
│  └─ Read: WORKFLOWS_SUMMARY.md
│
└─ Find a specific command
   └─ Check: QUICK_REFERENCE.md
```

## Common Tasks

### Before First CI Run
1. Run validation: `.github/workflows/scripts/validate-config.sh`
2. Read: [SETUP.md - CI Workflow](./workflows/SETUP.md#1-ci-workflow)
3. Test locally: `pnpm lint && pnpm typecheck && pnpm test`
4. Push and verify workflow runs

### Before First Release
1. Generate signing keys: `.github/workflows/scripts/setup-signing.sh`
2. Add secrets to GitHub (use [.env.example](./workflows/.env.example) as reference)
3. Test build locally: `pnpm tauri build --debug`
4. Create tag: `git tag v0.1.0`
5. Push tag: `git push origin v0.1.0`
6. Monitor release workflow

### Before First Backend Deploy
1. Copy `fly.toml.example` to `fly.toml`
2. Customize Fly.io configuration
3. Add FLY_API_TOKEN and FLY_APP_NAME secrets
4. Add health check endpoint to backend
5. Push changes to `docker/`
6. Monitor deploy workflow

### Before First Preview Deploy
1. Ensure FLY_API_TOKEN secret exists
2. Make a PR with changes in `docker/`
3. Verify preview app is created
4. Check PR for preview URL comment
5. Test preview deployment

## Key Metrics

| Workflow | Duration | Cost | Frequency |
|----------|----------|------|-----------|
| CI | 3-5 min | Free | Per push/PR |
| Release | 15-25 min | Free | Per tag (~2x/month) |
| Backend Deploy | 5-10 min | Free | Per change (~2x/week) |
| Preview Deploy | 5-10 min | Free | Per PR (~1x/day) |
| **Total/Month** | **~5-6 hours** | **Free** | **~300 min** |

## Secret Management

### All Required Secrets
See [.env.example](./workflows/.env.example) for complete list

### Generate Secrets
1. Tauri keys: Run `setup-signing.sh`
2. macOS cert: Run `setup-signing.sh`
3. Windows cert: Run `setup-signing.sh`
4. Fly.io token: `flyctl tokens create`

### Add to GitHub
1. Go to: Settings → Secrets and variables → Actions
2. Click: "New repository secret"
3. Copy from above and paste

## Architecture at a Glance

```
Source Code
  ├─ Push to main
  │  └─ CI Workflow (lint, test, build)
  │
  ├─ Docker/ changes
  │  └─ Backend Deploy (Docker → GHCR → Fly.io)
  │
  ├─ Tag push (v1.0.0)
  │  └─ Release Workflow (macOS, Windows, Linux)
  │     └─ GitHub Releases
  │
  └─ PR with docker/ changes
     └─ Preview Deploy (temp Fly.io app)
        └─ PR comment with URL
        └─ Auto-cleanup on PR close
```

## Support & Help

### I'm stuck on...

**Setup Issues:**
→ Read [SETUP.md](./workflows/SETUP.md)
→ Run `.github/workflows/scripts/validate-config.sh`

**Workflow Problems:**
→ Check [TROUBLESHOOTING.md](./workflows/TROUBLESHOOTING.md)
→ Search for your error message

**Quick Commands:**
→ See [QUICK_REFERENCE.md](./workflows/QUICK_REFERENCE.md)

**Project Overview:**
→ Read [WORKFLOWS_SUMMARY.md](./WORKFLOWS_SUMMARY.md)

### External Resources

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Tauri Docs](https://tauri.app/docs)
- [Fly.io Docs](https://fly.io/docs)
- [Rust Guide](https://doc.rust-lang.org/book)

## Maintenance Checklist

### Monthly
- [ ] Review workflow logs for warnings
- [ ] Check GitHub Actions billing
- [ ] Update dependencies

### Quarterly
- [ ] Review and update action versions
- [ ] Audit secrets and permissions
- [ ] Test disaster recovery procedures

### Annually
- [ ] Rotate code signing certificates
- [ ] Update security practices
- [ ] Plan for growth

## File Checksums & Timestamps

```
Created: 2026-01-03
Last Updated: 2026-01-03
Status: Production Ready
Version: 1.0.0

Files:
- .github/WORKFLOWS_SUMMARY.md
- .github/workflows/ci.yml
- .github/workflows/release-desktop.yml
- .github/workflows/deploy-backend.yml
- .github/workflows/preview.yml
- .github/workflows/SETUP.md
- .github/workflows/TROUBLESHOOTING.md
- .github/workflows/QUICK_REFERENCE.md
- .github/workflows/.env.example
- .github/workflows/scripts/setup-signing.sh
- .github/workflows/scripts/validate-config.sh
- fly.toml.example
- docker/Dockerfile.example
```

## Next Steps

### Immediately (Today)
1. ✓ Copy all workflow files
2. ✓ Run validation script
3. ✓ Read QUICK_REFERENCE.md

### This Week
1. ✓ Generate signing credentials
2. ✓ Add secrets to GitHub
3. ✓ Test CI on a PR

### This Month
1. ✓ Create first release
2. ✓ Deploy backend
3. ✓ Create preview instance

---

**Navigation:**
- [WORKFLOWS_SUMMARY.md](./WORKFLOWS_SUMMARY.md) - Overview
- [SETUP.md](./workflows/SETUP.md) - Detailed guide
- [TROUBLESHOOTING.md](./workflows/TROUBLESHOOTING.md) - Issues
- [QUICK_REFERENCE.md](./workflows/QUICK_REFERENCE.md) - Commands

**Ready to get started?** Start with [QUICK_REFERENCE.md](./workflows/QUICK_REFERENCE.md)
