#!/bin/bash
# Build a macOS .app bundle and installer DMG for Baeus.
#
# Usage:
#   ./macos/build-app.sh              # Build release, create .app and .dmg
#   ./macos/build-app.sh --skip-build # Use existing release binary
#
# Output:
#   target/release/Baeus.app
#   target/release/Baeus-macos-arm64.dmg

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Baeus"
BINARY_NAME="baeus-app"
BUNDLE_DIR="$PROJECT_ROOT/target/release/$APP_NAME.app"
DMG_PATH="$PROJECT_ROOT/target/release/$APP_NAME-macos-arm64.dmg"

# --- Build release binary (unless --skip-build) ---
if [[ "${1:-}" != "--skip-build" ]]; then
    echo "Building release binary..."
    cd "$PROJECT_ROOT"
    RUST_MIN_STACK=268435456 "${HOME}/.cargo/bin/cargo" build --release
fi

BINARY="$PROJECT_ROOT/target/release/$BINARY_NAME"
if [[ ! -f "$BINARY" ]]; then
    echo "ERROR: Release binary not found at $BINARY"
    echo "Run 'cargo build --release' first or remove --skip-build"
    exit 1
fi

# --- Create .app bundle structure ---
echo "Creating $APP_NAME.app bundle..."

rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$BUNDLE_DIR/Contents/Info.plist"

# Copy binary
cp "$BINARY" "$BUNDLE_DIR/Contents/MacOS/$BINARY_NAME"

# Copy app icon
ICNS="$BUNDLE_DIR/Contents/Resources/AppIcon.icns"
if [[ -f "$SCRIPT_DIR/AppIcon.icns" ]]; then
    cp "$SCRIPT_DIR/AppIcon.icns" "$ICNS"
    echo "  Copied AppIcon.icns"
else
    echo "  WARNING: No AppIcon.icns found in macos/ — app will use default macOS icon"
fi

# Write PkgInfo
echo -n "APPL????" > "$BUNDLE_DIR/Contents/PkgInfo"

echo ""
echo "Built: $BUNDLE_DIR"

# --- Create installer DMG ---
echo ""
echo "Creating installer DMG..."

rm -f "$DMG_PATH"

if command -v create-dmg &>/dev/null; then
    # create-dmg returns exit code 2 when it can't set the icon position
    # (non-fatal — the DMG is still created correctly)
    create-dmg \
        --volname "$APP_NAME" \
        --volicon "$SCRIPT_DIR/AppIcon.icns" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "$APP_NAME.app" 150 185 \
        --app-drop-link 450 185 \
        --no-internet-enable \
        "$DMG_PATH" \
        "$BUNDLE_DIR" \
    || [[ $? -eq 2 ]]

    echo "Built: $DMG_PATH"
else
    # Fallback: plain hdiutil DMG with Applications symlink
    echo "  (create-dmg not found, using hdiutil fallback)"
    STAGING_DIR=$(mktemp -d)
    cp -r "$BUNDLE_DIR" "$STAGING_DIR/"
    ln -s /Applications "$STAGING_DIR/Applications"
    hdiutil create -volname "$APP_NAME" -srcfolder "$STAGING_DIR" \
        -ov -format UDZO "$DMG_PATH"
    rm -rf "$STAGING_DIR"
    echo "Built: $DMG_PATH"
fi

echo ""
echo "To install:"
echo "  open \"$DMG_PATH\"    # then drag Baeus to Applications"
echo ""
echo "To run directly:"
echo "  open \"$BUNDLE_DIR\""
