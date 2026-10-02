#!/usr/bin/env bash
# Останавливает и удаляет Octo и его LaunchAgent.
set -euo pipefail

LABEL=com.bogdanminko.octo
launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
rm -f "$HOME/Library/LaunchAgents/$LABEL.plist"
rm -rf "$HOME/Applications/Octo.app"

echo "✓ Octo удалён"
