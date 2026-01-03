# GitHub Actions Workflows - Troubleshooting Guide

## Common Issues & Solutions

### CI Workflow Issues

#### Issue: "Workflow not triggered on push"

**Cause:** Workflow file syntax error, branch protection rules, or disabled actions

**Solutions:**
1. Verify workflow file syntax:
   ```bash
   yamllint .github/workflows/ci.yml
   ```

2. Check if Actions are enabled:
   - Repository Settings → Actions → General
   - Ensure "Allow all actions and reusable workflows" is selected

3. Verify branch matching:
   ```yaml
   on:
     push:
       branches:
         - main  # Check this matches your default branch
   ```

4. Check branch protection rules:
   - Settings → Branches → Branch protection rules
   - Verify rule applies to your branch

#### Issue: "pnpm cache not being reused"

**Symptoms:** Every run reinstalls all dependencies (slow CI)

**Solutions:**
1. Verify pnpm-lock.yaml is committed:
   ```bash
   git ls-files | grep pnpm-lock.yaml
   ```

2. Check cache key matches:
   ```yaml
   key: pnpm-cache-${{ runner.os }}-${{ hashFiles('**/pnpm-lock.yaml') }}
   ```

3. Verify cache path:
   ```bash
   pnpm store path  # Should be ~/.pnpm-store
   ```

4. Clear cache if corrupted:
   - Settings → Actions → General → Actions cache → Clear all

#### Issue: "Turbo cache not working"

**Symptoms:** Every job rebuilds packages, no caching benefit

**Solutions:**
1. Verify Turbo configuration exists:
   ```bash
   cat turbo.json | head -20
   ```

2. Check Turbo cache settings:
   ```json
   {
     "globalDependencies": ["**/pnpm-lock.yaml"],
     "pipeline": {
       "build": {
         "cache": true
       }
     }
   }
   ```

3. If using remote caching:
   ```bash
   # Verify tokens
   echo $TURBO_TOKEN  # Should not be empty
   turbo link        # Link local repo to Vercel
   ```

4. Force cache refresh:
   - Delete all jobs' cache in Actions settings
   - Re-run workflow

#### Issue: "Build fails with 'Tauri CLI not found'"

**Symptoms:**
```
pnpm: tauri: command not found
```

**Solutions:**
1. Ensure Tauri CLI is in devDependencies:
   ```bash
   cd packages/desktop
   pnpm list @tauri-apps/cli
   ```

2. If missing, install:
   ```bash
   pnpm add -D @tauri-apps/cli@next
   ```

3. Update workflow to install CLI:
   ```yaml
   - run: pnpm add -D @tauri-apps/cli@next
   ```

#### Issue: "TypeScript compilation fails"

**Symptoms:**
```
error TS2304: Cannot find name 'react'
```

**Solutions:**
1. Check TypeScript configuration:
   ```bash
   cat tsconfig.json | grep -A5 "compilerOptions"
   ```

2. Verify type definitions are installed:
   ```bash
   pnpm list @types/react @types/node
   ```

3. Check for circular dependencies:
   ```bash
   pnpm list --depth=10
   ```

#### Issue: "Lint fails on workflow CI"

**Symptoms:**
```
ESLint error: Unexpected token
```

**Solutions:**
1. Run linting locally:
   ```bash
   pnpm turbo lint
   ```

2. Fix issues:
   ```bash
   pnpm turbo lint -- --fix
   ```

3. For specific package:
   ```bash
   cd packages/desktop
   pnpm lint --fix
   ```

### Desktop Release Workflow Issues

#### Issue: "Release not triggered on tag push"

**Cause:** Tag format mismatch or git tag not pushed

**Solutions:**
1. Use correct tag format (semver with 'v' prefix):
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

   Or for multiple tags:
   ```bash
   git push origin --tags
   ```

2. Verify tag was created:
   ```bash
   git tag -l
   git ls-remote origin 'refs/tags/v*'
   ```

3. Check workflow trigger condition:
   ```yaml
   on:
     push:
       tags:
         - "v*"
   ```

#### Issue: "macOS code signing fails"

**Symptoms:**
```
Error: The identity "Developer ID Application: ..." is not available
```

**Solutions:**
1. Verify certificate in keychain:
   ```bash
   security find-identity -v -p codesigning
   ```

2. Re-import certificate in CI:
   ```bash
   # Verify base64 encoding
   file <(echo $APPLE_CERTIFICATE | base64 -d)
   # Should show: data (P12 certificate)
   ```

3. Check certificate password:
   ```bash
   security import /tmp/cert.p12 -k build.keychain -P "$PASSWORD"
   # If fails, password is wrong
   ```

4. Verify signing identity matches:
   - GitHub Secret: `APPLE_SIGNING_IDENTITY`
   - Local keychain: `security find-identity -v -p codesigning`
   - Must match exactly including team ID

#### Issue: "Windows signing certificate error"

**Symptoms:**
```
The certificate is not found in the store
```

**Solutions:**
1. Verify certificate encoding:
   ```bash
   # On Windows:
   $cert = [System.Convert]::FromBase64String($env:WINDOWS_SIGN_CERT)
   [IO.File]::WriteAllBytes("cert.pfx", $cert)
   certutil -d cert.pfx  # Should show details
   ```

2. Check certificate password:
   ```powershell
   $pwd = ConvertTo-SecureString -String $env:WINDOWS_SIGN_CERT_PASSWORD -AsPlainText -Force
   Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\CurrentUser\My -Password $pwd
   ```

3. If certificate signing fails, build unsigned:
   - Comment out signing steps
   - Builds will work but be unsigned

#### Issue: "Artifact upload fails"

**Symptoms:**
```
Error: Failed to upload artifact
```

**Solutions:**
1. Check artifact path exists:
   ```bash
   ls -la ./release-artifacts/
   ```

2. Verify artifact collection step:
   ```bash
   # In build script
   find ./packages/desktop/src-tauri/target -name "*.dmg" -o -name "*.exe"
   ```

3. Check artifact size limits:
   - GitHub limit: 400MB per artifact
   - Workflow limit: 5GB total

4. Upload to external storage instead:
   ```yaml
   - name: Upload to S3
     uses: actions/upload-artifact@v4
     with:
       name: release-${{ matrix.os }}
       path: release-artifacts/
   ```

#### Issue: "Tauri build fails with 'Unexpected end of JSON'"

**Symptoms:**
```
Error: Unexpected end of JSON input while parsing
```

**Solutions:**
1. Verify `tauri.conf.json` is valid:
   ```bash
   jq . src-tauri/tauri.conf.json
   ```

2. Check for missing required fields:
   ```json
   {
     "productName": "YourApp",
     "version": "1.0.0",
     "build": {
       "devPath": "...",
       "frontendDist": "..."
     }
   }
   ```

3. Verify bundle configuration:
   ```json
   "bundle": {
     "active": true,
     "targets": ["deb", "dmg", "msi"]
   }
   ```

### Backend Deploy Workflow Issues

#### Issue: "Fly.io deployment fails"

**Symptoms:**
```
Error: Request failed with status 401
```

**Solutions:**
1. Verify API token:
   ```bash
   echo $FLY_API_TOKEN | wc -c  # Should be > 50 chars
   ```

2. Check token permissions:
   ```bash
   flyctl tokens list  # Verify token exists
   ```

3. Generate new token if needed:
   ```bash
   flyctl tokens create
   ```

4. Verify app name:
   ```bash
   flyctl apps list
   ```

#### Issue: "Health check fails after deployment"

**Symptoms:**
```
Health check failed after 5 minutes
```

**Solutions:**
1. Verify health endpoint exists:
   ```bash
   curl -v https://your-app.fly.dev/health
   ```

2. Check application logs:
   ```bash
   flyctl logs --app your-app-name
   ```

3. Verify port configuration:
   ```bash
   flyctl config show  # Check internal_port
   ```

4. Increase health check timeout:
   ```yaml
   - name: Wait for deployment
     timeout-minutes: 10  # Increase if needed
   ```

#### Issue: "Automatic rollback triggered"

**Cause:** Health check failed or deployment timeout

**Solutions:**
1. Check deployment logs:
   ```bash
   flyctl releases --limit 10
   flyctl releases view <release-id>
   ```

2. View application logs during failure:
   ```bash
   flyctl logs --since 10m
   ```

3. Verify application startup:
   - Add health check endpoint
   - Ensure app listens on correct port (8000)
   - Check for missing environment variables

4. Disable auto-rollback for debugging:
   ```yaml
   # Temporarily disable rollback-on-failure
   # Fix issue, then re-enable
   ```

#### Issue: "Docker build fails with 'Cannot find module'"

**Symptoms:**
```
Error: Cannot find module 'express'
```

**Solutions:**
1. Verify dependencies in Dockerfile:
   ```dockerfile
   RUN pnpm install --frozen-lockfile --prod
   ```

2. Check package.json paths are correct:
   ```dockerfile
   COPY packages/backend/package.json ./packages/backend/
   COPY pnpm-lock.yaml .
   ```

3. Build Docker locally:
   ```bash
   docker build -t test:latest docker/
   docker run test:latest  # Verify it works
   ```

### Preview Deploy Workflow Issues

#### Issue: "Preview app not created"

**Cause:** Missing `FLY_API_TOKEN` or `FLY_ORG`

**Solutions:**
1. Verify secrets:
   ```bash
   # In GitHub Actions UI
   Settings → Secrets → Check FLY_API_TOKEN exists
   ```

2. Check organization:
   ```bash
   flyctl orgs list  # Get correct org
   ```

3. Add FLY_ORG secret if missing

#### Issue: "Preview cleanup fails"

**Symptoms:**
```
Error: App not found or already deleted
```

**Solutions:**
1. This is usually non-critical (app already deleted)

2. To prevent error messages:
   ```yaml
   continue-on-error: true  # Already in workflow
   ```

3. Manually cleanup old preview apps:
   ```bash
   flyctl apps list | grep "^preview-pr-"
   flyctl apps destroy preview-pr-123 --force
   ```

#### Issue: "PR comment not posted"

**Cause:** Missing write permissions

**Solutions:**
1. Verify workflow permissions:
   ```yaml
   permissions:
     pull-requests: write  # Need this
   ```

2. Check GITHUB_TOKEN scope:
   - Workflow defaults should be sufficient
   - If using custom token, ensure it has `repo` scope

3. Test manually:
   ```bash
   gh issue comment <number> -b "Test comment"
   ```

### Performance Issues

#### Issue: "CI takes 15+ minutes"

**Solutions:**
1. Check for cache hits:
   - View workflow run logs
   - Look for "Cache hit" messages

2. Enable parallel jobs:
   ```yaml
   strategy:
     max-parallel: 4
   ```

3. Disable unnecessary jobs:
   ```yaml
   # Comment out Windows build if not needed
   - os: windows
     runner: windows-latest
   ```

4. Use self-hosted runners for faster builds

#### Issue: "Release builds take 25+ minutes"

**Solutions:**
1. Check individual platform times:
   - macOS arm64: Usually 5-8 min
   - macOS x86_64: Usually 5-8 min
   - Windows: Usually 8-12 min
   - Linux: Usually 5-7 min

2. If unusually slow:
   - Clear Rust cache
   - Check for resource constraints

3. Build locally for debugging:
   ```bash
   pnpm tauri build --debug
   ```

### Debugging Techniques

#### Enable Debug Logging

```yaml
env:
  RUST_LOG: debug
  TURBO_LOG_ORDER: stream
  TURBO_VERBOSITY: verbose
```

#### SSH into Runner (for macOS only)

```yaml
- name: Setup SSH
  if: failure()
  uses: mxschmitt/action-tmate@v3
```

Then SSH into the runner and debug manually.

#### View Complete Logs

1. Go to Actions → Run → Logs
2. Expand failing job
3. Use browser search to find errors

#### Test Locally

```bash
# Test CI locally with act
act push -j lint

# Test release build
pnpm tauri build --ci

# Test Docker build
docker build -t test:latest docker/
```

### Getting Help

1. Check GitHub Actions documentation:
   - https://docs.github.com/en/actions

2. Platform-specific docs:
   - Tauri: https://tauri.app/docs
   - Fly.io: https://fly.io/docs
   - Docker: https://docs.docker.com

3. Community resources:
   - Tauri Discord: https://discord.gg/tauri
   - GitHub Discussions

4. Enable debug logging:
   - Workflow logs → enable step debug logging
   - https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/enabling-debug-logging

## Workflow Run Monitoring

### Check Workflow Status

```bash
# List recent runs
gh run list

# View specific run
gh run view <run-id>

# View logs
gh run view <run-id> --log
```

### Common Success Patterns

**Successful CI:**
- All jobs show green checkmarks
- Cache hits visible in logs
- Total duration: 3-5 minutes
- No warnings or errors

**Successful Release:**
- All matrix combinations pass
- Artifacts upload successfully
- GitHub Release created
- Total duration: 15-25 minutes

**Successful Deploy:**
- Docker build completes
- Health checks pass
- Deployment status shows "active"
- Logs show no errors

## Prevention Tips

1. **Test locally first:**
   ```bash
   pnpm lint && pnpm typecheck && pnpm test
   ```

2. **Use pre-commit hooks:**
   ```bash
   npx husky install
   npx husky add .husky/pre-commit "pnpm lint"
   ```

3. **Monitor workflow costs:**
   - Settings → Billing and plans
   - Track minutes used per month

4. **Keep dependencies updated:**
   ```bash
   pnpm up --latest
   cargo update
   ```

5. **Regular security audits:**
   ```bash
   pnpm audit
   cargo audit
   ```
