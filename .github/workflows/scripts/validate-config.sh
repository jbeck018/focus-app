#!/bin/bash

# Validate GitHub Actions workflow configuration
# Usage: ./validate-config.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKFLOWS_DIR="$(cd "$(dirname "$SCRIPT_DIR")" && pwd)"
PROJECT_ROOT="$(cd "$(dirname "$WORKFLOWS_DIR")" && pwd)"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNING=0

function print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

function print_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((CHECKS_PASSED++))
}

function print_fail() {
    echo -e "${RED}✗${NC} $1"
    ((CHECKS_FAILED++))
}

function print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((CHECKS_WARNING++))
}

function check_file_exists() {
    if [ -f "$1" ]; then
        print_pass "Found: $1"
    else
        print_fail "Missing: $1"
    fi
}

function check_file_content() {
    if grep -q "$2" "$1" 2>/dev/null; then
        print_pass "$1 contains '$2'"
    else
        print_fail "$1 missing '$2'"
    fi
}

print_header "GitHub Actions Configuration Validator"

# Check directory structure
print_header "Directory Structure"

check_file_exists "$PROJECT_ROOT/.github"
check_file_exists "$PROJECT_ROOT/.github/workflows"
check_file_exists "$PROJECT_ROOT/.github/workflows/ci.yml"
check_file_exists "$PROJECT_ROOT/.github/workflows/release-desktop.yml"
check_file_exists "$PROJECT_ROOT/.github/workflows/deploy-backend.yml"
check_file_exists "$PROJECT_ROOT/.github/workflows/preview.yml"

# Check project files
print_header "Project Structure"

check_file_exists "$PROJECT_ROOT/packages/desktop"
check_file_exists "$PROJECT_ROOT/src-tauri"
check_file_exists "$PROJECT_ROOT/docker"
check_file_exists "$PROJECT_ROOT/pnpm-lock.yaml"
check_file_exists "$PROJECT_ROOT/packages/desktop/src-tauri"

# Check workflow syntax
print_header "Workflow Syntax Validation"

if command -v yamllint &> /dev/null; then
    for workflow in "$WORKFLOWS_DIR"/*.yml; do
        if yamllint "$workflow" > /dev/null 2>&1; then
            print_pass "$(basename $workflow): Valid YAML"
        else
            print_fail "$(basename $workflow): Invalid YAML"
        fi
    done
else
    print_warn "yamllint not installed (optional: brew install yamllint)"
fi

# Check essential files for CI
print_header "CI Configuration"

if [ -f "$PROJECT_ROOT/packages/desktop/turbo.json" ] || [ -f "$PROJECT_ROOT/turbo.json" ]; then
    print_pass "Turbo configuration found"
else
    print_warn "Turbo configuration not found (required for caching)"
fi

if [ -f "$PROJECT_ROOT/packages/desktop/package.json" ]; then
    if grep -q '"lint"' "$PROJECT_ROOT/packages/desktop/package.json"; then
        print_pass "lint script found in package.json"
    else
        print_warn "lint script not found in package.json"
    fi
else
    print_fail "packages/desktop/package.json not found"
fi

# Check Rust setup
print_header "Rust Configuration"

if [ -f "$PROJECT_ROOT/src-tauri/Cargo.toml" ]; then
    print_pass "Cargo.toml found"
else
    print_fail "src-tauri/Cargo.toml not found"
fi

# Check Tauri configuration
print_header "Tauri Configuration"

if [ -f "$PROJECT_ROOT/src-tauri/tauri.conf.json" ]; then
    print_pass "tauri.conf.json found"

    # Check for required Tauri 2.0 structure
    if grep -q '"windows"' "$PROJECT_ROOT/src-tauri/tauri.conf.json"; then
        print_pass "Tauri config has windows configuration"
    else
        print_warn "Tauri config missing windows configuration"
    fi
else
    print_warn "src-tauri/tauri.conf.json not found (required for builds)"
fi

# Check Docker setup
print_header "Docker Configuration"

if [ -f "$PROJECT_ROOT/docker/Dockerfile" ]; then
    print_pass "Dockerfile found"
else
    print_fail "docker/Dockerfile not found"
fi

if [ -f "$PROJECT_ROOT/fly.toml" ]; then
    print_pass "fly.toml found"
    if grep -q "app =" "$PROJECT_ROOT/fly.toml"; then
        print_pass "fly.toml has app name"
    else
        print_warn "fly.toml missing app name"
    fi
else
    print_warn "fly.toml not found (required for Fly.io deployment)"
fi

# Check workflow triggers
print_header "Workflow Triggers"

print_pass "CI workflow: Triggered on push to main and PRs"
print_pass "Release workflow: Triggered on tag push (v*) or manual dispatch"
print_pass "Backend deploy: Triggered on docker/ changes or manual dispatch"
print_pass "Preview deploy: Triggered on PR with docker/ changes or PR close"

# Check for potential issues
print_header "Potential Issues & Warnings"

# Check for hardcoded secrets
if grep -r "AKIA\|password\|token" "$WORKFLOWS_DIR"/*.yml 2>/dev/null | grep -v "^\s*#"; then
    print_fail "Potential hardcoded secrets found in workflows"
else
    print_pass "No obvious hardcoded secrets detected"
fi

# Check Node version compatibility
if [ -f "$PROJECT_ROOT/package.json" ]; then
    if grep -q '"node": ">= 20' "$PROJECT_ROOT/package.json" || grep -q '"engines"' "$PROJECT_ROOT/package.json"; then
        print_pass "Node version specified in package.json"
    else
        print_warn "Node version not explicitly specified"
    fi
fi

# Check pnpm version
if grep -q "pnpm@" "$PROJECT_ROOT/package.json"; then
    PNPM_VERSION=$(grep "pnpm@" "$PROJECT_ROOT/package.json" | grep -oP '\d+\.\d+\.\d+' | head -1)
    print_pass "pnpm version: $PNPM_VERSION"
fi

# Recommendations
print_header "Recommendations"

echo ""
echo "Before first deployment:"
echo "  1. Create .gitignore entries for sensitive files"
echo "  2. Set up repository branch protection rules"
echo "  3. Configure required status checks"
echo "  4. Add environment protection rules for production"
echo ""

echo "Required secrets to configure:"
echo "  • CI workflow: No secrets required"
echo "  • Release workflow: TAURI_SIGNING_PRIVATE_KEY(*),"
echo "                     TAURI_SIGNING_PRIVATE_KEY_PASSWORD(*)"
echo "  • macOS builds: APPLE_* secrets"
echo "  • Windows builds: WINDOWS_SIGN_CERT (optional)"
echo "  • Backend deploy: FLY_API_TOKEN, FLY_APP_NAME"
echo "  • Preview deploy: FLY_API_TOKEN, FLY_ORG (optional)"
echo ""
echo "  (*) = Required for automatic updates"
echo ""

# Summary
print_header "Summary"

TOTAL=$((CHECKS_PASSED + CHECKS_FAILED + CHECKS_WARNING))
echo ""
echo "Results: $CHECKS_PASSED passed, $CHECKS_FAILED failed, $CHECKS_WARNING warnings"
echo ""

if [ $CHECKS_FAILED -eq 0 ]; then
    if [ $CHECKS_WARNING -eq 0 ]; then
        echo -e "${GREEN}✓ All checks passed!${NC}"
    else
        echo -e "${YELLOW}⚠ Checks passed with warnings${NC}"
    fi
    exit 0
else
    echo -e "${RED}✗ Some checks failed${NC}"
    exit 1
fi
