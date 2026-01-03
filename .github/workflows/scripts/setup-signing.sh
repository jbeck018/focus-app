#!/bin/bash

# Helper script to generate and encode signing credentials for GitHub Actions
# Usage: ./setup-signing.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==================================="
echo "GitHub Actions Signing Setup"
echo "==================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_step() {
    echo -e "${GREEN}→${NC} $1"
}

function print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

function print_error() {
    echo -e "${RED}✗${NC} $1"
}

function print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
else
    print_error "Unsupported OS: $OSTYPE"
    exit 1
fi

# Menu
echo "Select what to setup:"
echo "1) Tauri signing keys"
echo "2) macOS code signing"
echo "3) Windows code signing"
echo "4) All"
echo ""
read -p "Enter choice [1-4]: " choice

case $choice in
    1|4)
        echo ""
        print_step "Setting up Tauri signing keys"
        echo ""

        if ! command -v cargo &> /dev/null; then
            print_error "Rust not found. Install from https://rustup.rs/"
            exit 1
        fi

        read -p "Enter path to save private key [~/.tauri/private.key]: " KEY_PATH
        KEY_PATH=${KEY_PATH:-~/.tauri/private.key}

        # Expand home directory
        KEY_PATH="${KEY_PATH/#\~/$HOME}"

        # Create directory
        mkdir -p "$(dirname "$KEY_PATH")"

        # Check if key already exists
        if [ -f "$KEY_PATH" ]; then
            read -p "Key already exists. Overwrite? [y/N]: " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                print_warning "Skipping key generation"
                echo ""
            else
                rm "$KEY_PATH"
                cd packages/desktop
                pnpm tauri signer generate -w "$KEY_PATH"
                cd - > /dev/null
            fi
        else
            cd packages/desktop
            pnpm tauri signer generate -w "$KEY_PATH"
            cd - > /dev/null
        fi

        if [ -f "$KEY_PATH" ]; then
            print_success "Generated private key: $KEY_PATH"
            echo ""
            print_step "Encoding for GitHub Actions..."
            echo ""

            # Read the key file
            PRIVATE_KEY=$(cat "$KEY_PATH")

            # Create temporary file for base64
            KEY_B64=$(echo "$PRIVATE_KEY" | base64)

            echo "TAURI_SIGNING_PRIVATE_KEY (base64):"
            echo "======================================"
            echo "$KEY_B64"
            echo "======================================"
            echo ""

            read -s -p "Enter password for the signing key: " KEY_PASSWORD
            echo ""

            echo "TAURI_SIGNING_PRIVATE_KEY_PASSWORD:"
            echo "======================================"
            echo "$KEY_PASSWORD"
            echo "======================================"
            echo ""

            print_warning "Save these values as GitHub Secrets:"
            echo "  1. TAURI_SIGNING_PRIVATE_KEY = (base64 encoded key above)"
            echo "  2. TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (password above)"
            echo ""

            # Also save public key
            if [ -f "${KEY_PATH}.pub" ]; then
                PUBLIC_KEY=$(cat "${KEY_PATH}.pub")
                echo "Public key for reference (save in code/config):"
                echo "======================================"
                echo "$PUBLIC_KEY"
                echo "======================================"
                echo ""
            fi
        fi
        ;;
esac

# macOS Code Signing
if [[ $choice == "2" || $choice == "4" ]]; then
    if [[ $OS != "macos" ]]; then
        print_warning "macOS setup requires macOS machine. Skipping."
    else
        echo ""
        print_step "Setting up macOS code signing"
        echo ""

        print_step "Finding signing identities..."
        echo ""
        security find-identity -v -p codesigning | grep "Developer ID"
        echo ""

        read -p "Enter Developer ID Application identity (copy from above): " SIGNING_IDENTITY
        echo ""

        if [ -z "$SIGNING_IDENTITY" ]; then
            print_error "Identity required for macOS signing"
            exit 1
        fi

        # Extract Team ID
        TEAM_ID=$(echo "$SIGNING_IDENTITY" | grep -oP '\([A-Z0-9]+\)$' | tr -d '()')

        if [ -z "$TEAM_ID" ]; then
            print_error "Could not extract Team ID from identity"
            exit 1
        fi

        print_success "Extracted Team ID: $TEAM_ID"
        echo ""

        print_step "Exporting certificate..."
        echo ""

        read -p "Enter path to certificate file [~/Desktop/certificate.p12]: " CERT_PATH
        CERT_PATH=${CERT_PATH:-~/Desktop/certificate.p12}
        CERT_PATH="${CERT_PATH/#\~/$HOME}"

        if [ ! -f "$CERT_PATH" ]; then
            print_error "Certificate not found at $CERT_PATH"
            echo ""
            echo "To export your certificate:"
            echo "1. Open Keychain Access"
            echo "2. Find your 'Developer ID Application' certificate"
            echo "3. Right-click → Export"
            echo "4. Format: Personal Information Exchange (.p12)"
            echo "5. Remember the password you set"
            exit 1
        fi

        print_success "Found certificate: $CERT_PATH"
        echo ""

        # Encode certificate
        print_step "Base64 encoding certificate..."
        CERT_B64=$(base64 -i "$CERT_PATH")

        echo "APPLE_CERTIFICATE (base64):"
        echo "======================================"
        echo "$CERT_B64" | head -c 100
        echo "... (truncated)"
        echo "======================================"
        echo ""

        read -s -p "Enter certificate password: " CERT_PASSWORD
        echo ""

        print_step "Getting Apple ID credentials..."
        echo ""

        read -p "Enter Apple ID email: " APPLE_ID
        read -p "Enter Team ID: " APPLE_TEAM_ID_INPUT
        APPLE_TEAM_ID=${APPLE_TEAM_ID_INPUT:-$TEAM_ID}

        echo ""
        print_warning "Generate app-specific password:"
        echo "1. Visit https://appleid.apple.com/account/security"
        echo "2. Sign in with your Apple ID"
        echo "3. Security section → App-specific passwords"
        echo "4. Generate password for 'GitHub Actions'"
        echo ""

        read -s -p "Enter app-specific password: " APPLE_PASSWORD
        echo ""
        echo ""

        echo "Summary:"
        echo "======================================"
        echo "APPLE_CERTIFICATE=(base64 from above)"
        echo "APPLE_CERTIFICATE_PASSWORD=$CERT_PASSWORD"
        echo "APPLE_SIGNING_IDENTITY=$SIGNING_IDENTITY"
        echo "APPLE_ID=$APPLE_ID"
        echo "APPLE_TEAM_ID=$APPLE_TEAM_ID"
        echo "APPLE_PASSWORD=(app-specific password)"
        echo "======================================"
        echo ""

        print_warning "Save these as GitHub Secrets"
        echo ""
    fi
fi

# Windows Code Signing
if [[ $choice == "3" || $choice == "4" ]]; then
    echo ""
    print_step "Setting up Windows code signing (optional)"
    echo ""

    echo "Windows code signing setup requires a code signing certificate."
    echo "Options:"
    echo "1. Use existing .pfx certificate file"
    echo "2. Skip (builds will be unsigned)"
    echo ""

    read -p "Enter choice [1-2] (default 2): " win_choice
    win_choice=${win_choice:-2}

    if [[ $win_choice == "1" ]]; then
        read -p "Enter path to .pfx certificate: " PFX_PATH
        PFX_PATH="${PFX_PATH/#\~/$HOME}"

        if [ ! -f "$PFX_PATH" ]; then
            print_error "Certificate not found at $PFX_PATH"
            exit 1
        fi

        print_success "Found certificate: $PFX_PATH"
        echo ""

        # Encode certificate
        print_step "Base64 encoding certificate..."

        if [[ "$OSTYPE" == "darwin"* ]]; then
            CERT_B64=$(base64 -i "$PFX_PATH")
        else
            CERT_B64=$(base64 "$PFX_PATH")
        fi

        echo "WINDOWS_SIGN_CERT (base64):"
        echo "======================================"
        echo "$CERT_B64" | head -c 100
        echo "... (truncated)"
        echo "======================================"
        echo ""

        read -s -p "Enter certificate password: " WIN_CERT_PASSWORD
        echo ""
        echo ""

        echo "Save as GitHub Secrets:"
        echo "======================================"
        echo "WINDOWS_SIGN_CERT=(base64 from above)"
        echo "WINDOWS_SIGN_CERT_PASSWORD=$WIN_CERT_PASSWORD"
        echo "======================================"
        echo ""
    else
        print_warning "Windows code signing disabled"
        print_step "To enable later, run this script again and select option 3"
    fi
fi

echo ""
print_success "Setup complete!"
echo ""
echo "Next steps:"
echo "1. Go to GitHub repository Settings"
echo "2. Navigate to Secrets and variables → Actions"
echo "3. Add each secret from above"
echo "4. Trigger a workflow to verify"
echo ""
