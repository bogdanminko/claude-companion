#!/usr/bin/env bash
# Собирает Octo, кладёт в ~/Applications и регистрирует LaunchAgent (автозапуск + перезапуск при падении).
set -euo pipefail
cd "$(dirname "$0")/.."

LABEL=com.bogdanminko.octo
DEST="$HOME/Applications/Octo.app"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
DOMAIN="gui/$(id -u)"

./scripts/build.sh

mkdir -p "$HOME/Applications" "$HOME/Library/LaunchAgents"

# снять старую версию, если была
launchctl bootout "$DOMAIN/$LABEL" 2>/dev/null || true

rm -rf "$DEST"
cp -R build/Octo.app "$DEST"

sed "s|__EXECUTABLE__|$DEST/Contents/MacOS/Octo|" "launchd/$LABEL.plist" > "$PLIST"
launchctl bootstrap "$DOMAIN" "$PLIST"

echo "✓ Octo установлен и запущен как LaunchAgent ($LABEL)"
echo "  Если Octo не открывает quick entry — выдайте ему доступ:"
echo "  System Settings → Privacy & Security → Accessibility → Octo"
