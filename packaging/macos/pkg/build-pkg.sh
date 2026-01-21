#!/bin/bash
set -e

# Build macOS PKG installer for Endpoint Assessment Agent
# Run this script on macOS after building the agent binary

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
VERSION="${VERSION:-0.1.0}"
ARCH="${ARCH:-$(uname -m)}"
PKG_NAME="endpoint-agent-${VERSION}-macos-${ARCH}.pkg"

echo "Building macOS package: $PKG_NAME"

# Create staging directory
STAGING_DIR=$(mktemp -d)
trap "rm -rf $STAGING_DIR" EXIT

# Create directory structure
mkdir -p "$STAGING_DIR/usr/local/bin"
mkdir -p "$STAGING_DIR/Library/LaunchDaemons"

# Copy files
cp "$PROJECT_ROOT/target/release/agent" "$STAGING_DIR/usr/local/bin/endpoint-agent"
cp "$SCRIPT_DIR/../launchd/com.endpointassessment.agent.plist" "$STAGING_DIR/Library/LaunchDaemons/"

# Build component package
pkgbuild \
    --root "$STAGING_DIR" \
    --scripts "$SCRIPT_DIR/scripts" \
    --identifier "com.endpointassessment.agent" \
    --version "$VERSION" \
    --install-location "/" \
    "$PROJECT_ROOT/target/release/$PKG_NAME"

echo ""
echo "Package created: target/release/$PKG_NAME"
echo ""
echo "To install: sudo installer -pkg $PKG_NAME -target /"
