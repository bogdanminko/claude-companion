#!/usr/bin/env bash
# Builds Claude Companion, puts it in ~/Applications and registers a LaunchAgent (autostart + restart on crash).
set -euo pipefail
cd "$(dirname "$0")/.."

LABEL=com.bogdanminko.claude-companion
DEST="$HOME/Applications/Claude Companion.app"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
DOMAIN="gui/$(id -u)"

./scripts/build.sh

mkdir -p "$HOME/Applications" "$HOME/Library/LaunchAgents"

# remove the old version, if any
launchctl bootout "$DOMAIN/$LABEL" 2>/dev/null || true

rm -rf "$DEST"
cp -R "build/Claude Companion.app" "$DEST"

# the ad-hoc signature changes every build → the old Accessibility checkbox no longer applies.
# Reset it so macOS asks again instead of showing "enabled but not working".
tccutil reset Accessibility "$LABEL" >/dev/null 2>&1 || true

sed "s|__EXECUTABLE__|$DEST/Contents/MacOS/ClaudeCompanion|" "launchd/$LABEL.plist" > "$PLIST"
launchctl bootstrap "$DOMAIN" "$PLIST"

echo "✓ Claude Companion installed and running as LaunchAgent ($LABEL)"
echo "  Grant access: System Settings → Privacy & Security → Accessibility → Claude Companion"
