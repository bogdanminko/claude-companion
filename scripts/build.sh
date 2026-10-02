#!/usr/bin/env bash
# Builds "build/Claude Companion.app" from the SwiftPM target.
set -euo pipefail
cd "$(dirname "$0")/.."

swift build -c release

APP="build/Claude Companion.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp .build/release/ClaudeCompanion "$APP/Contents/MacOS/ClaudeCompanion"
cp Resources/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
cp Resources/Info.plist "$APP/Contents/Info.plist"

# ad-hoc signature: needed so macOS remembers the Accessibility permission
codesign --force --sign - --identifier com.bogdanminko.claude-companion "$APP"

echo "✓ Built: $APP"
