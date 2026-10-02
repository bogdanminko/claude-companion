#!/usr/bin/env bash
# Собирает build/Octo.app из SwiftPM-таргета.
set -euo pipefail
cd "$(dirname "$0")/.."

swift build -c release

APP=build/Octo.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp .build/release/Octo "$APP/Contents/MacOS/Octo"
cp Resources/Info.plist "$APP/Contents/Info.plist"

# ad-hoc подпись: нужна, чтобы macOS запомнила разрешение Accessibility
codesign --force --sign - --identifier com.bogdanminko.octo "$APP"

echo "✓ Собрано: $APP"
