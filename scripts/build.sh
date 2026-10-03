#!/usr/bin/env bash
# Builds the release binary; on macOS also wraps it into "build/Claude Companion.app".
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release
BIN=target/release/claude-companion

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "✓ Built: $BIN"
    exit 0
fi

APP="build/Claude Companion.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/ClaudeCompanion"
cp Resources/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
cp Resources/Info.plist "$APP/Contents/Info.plist"

# ad-hoc signature: needed so macOS remembers the Accessibility permission
codesign --force --sign - --identifier com.bogdanminko.claude-companion "$APP"

echo "✓ Built: $APP"
