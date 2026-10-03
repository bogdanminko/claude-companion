#!/usr/bin/env bash
# Builds Claude Companion and installs it with autostart and restart after a crash.
#   macOS: ~/Applications + LaunchAgent.   Linux: ~/.local/bin + XDG autostart (runs under --supervise).
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/build.sh

if [[ "$(uname -s)" == "Darwin" ]]; then
    LABEL=com.bogdanminko.claude-companion
    DEST="$HOME/Applications/Claude Companion.app"
    PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
    DOMAIN="gui/$(id -u)"

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
    exit 0
fi

BIN="$HOME/.local/bin/claude-companion"
ICON="$HOME/.local/share/icons/hicolor/256x256/apps/claude-companion.png"
APPS="$HOME/.local/share/applications"
AUTOSTART="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"

# stop the running copy, if any
pkill -f "^$BIN( |$)" 2>/dev/null || true

mkdir -p "$(dirname "$BIN")" "$(dirname "$ICON")" "$APPS" "$AUTOSTART"
install -m 755 target/release/claude-companion "$BIN"
install -m 644 Resources/claude-companion.png "$ICON"

entry() {
    cat <<DESKTOP
[Desktop Entry]
Type=Application
Name=Claude Companion
Comment=A tiny pixel Clawd that opens Claude and Claude Code
Exec="$BIN" --supervise
Icon=claude-companion
Terminal=false
Categories=Utility;
StartupNotify=false
DESKTOP
}
entry > "$APPS/claude-companion.desktop"
{ entry; echo "X-GNOME-Autostart-enabled=true"; } > "$AUTOSTART/claude-companion.desktop"

# start now, detached from this terminal
if [[ -n "${DISPLAY:-}" ]]; then
    setsid "$BIN" --supervise >/dev/null 2>&1 < /dev/null &
    echo "✓ Claude Companion installed and running"
else
    echo "✓ Claude Companion installed; it starts with your next desktop session"
fi
echo "  Autostart: $AUTOSTART/claude-companion.desktop"
