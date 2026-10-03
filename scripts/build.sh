#!/usr/bin/env bash
# Builds the release binary; on macOS also wraps it into "build/Claude Companion.app".
set -euo pipefail
cd "$(dirname "$0")/.."

# UNIVERSAL=1 on macOS: one binary for Apple Silicon and Intel (for releases)
if [[ "$(uname -s)" == "Darwin" && "${UNIVERSAL:-0}" == "1" ]]; then
    rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    BIN=target/release/claude-companion
    mkdir -p target/release
    lipo -create -output "$BIN" \
        target/aarch64-apple-darwin/release/claude-companion \
        target/x86_64-apple-darwin/release/claude-companion
else
    cargo build --release
    BIN=target/release/claude-companion
fi

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
