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

# ad-hoc подпись меняется при каждой сборке → старая галочка Accessibility больше не действует.
# Сбрасываем её, чтобы macOS честно спросила заново, а не показывала «включено, но не работает».
tccutil reset Accessibility "$LABEL" >/dev/null 2>&1 || true

sed "s|__EXECUTABLE__|$DEST/Contents/MacOS/Octo|" "launchd/$LABEL.plist" > "$PLIST"
launchctl bootstrap "$DOMAIN" "$PLIST"

echo "✓ Octo установлен и запущен как LaunchAgent ($LABEL)"
echo "  1. Выдайте доступ: System Settings → Privacy & Security → Accessibility → Octo"
echo "  2. В Claude: Settings → General → Desktop app → Quick access shortcut → Option + Space"
