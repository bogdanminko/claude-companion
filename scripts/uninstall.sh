#!/usr/bin/env bash
# Stops and removes Claude Companion and its LaunchAgent.
set -euo pipefail

LABEL=com.bogdanminko.claude-companion
launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
rm -f "$HOME/Library/LaunchAgents/$LABEL.plist"
rm -rf "$HOME/Applications/Claude Companion.app"

echo "✓ Claude Companion removed"
