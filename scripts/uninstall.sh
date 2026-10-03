#!/usr/bin/env bash
# Stops and removes Claude Companion and its autostart entry.
set -euo pipefail

if [[ "$(uname -s)" == "Darwin" ]]; then
    LABEL=com.bogdanminko.claude-companion
    launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
    rm -f "$HOME/Library/LaunchAgents/$LABEL.plist"
    rm -rf "$HOME/Applications/Claude Companion.app"
else
    BIN="$HOME/.local/bin/claude-companion"
    pkill -f "^$BIN( |$)" 2>/dev/null || true
    rm -f "$BIN" \
        "$HOME/.local/share/icons/hicolor/256x256/apps/claude-companion.png" \
        "$HOME/.local/share/applications/claude-companion.desktop" \
        "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/claude-companion.desktop"
fi

echo "✓ Claude Companion removed"
