---
name: install-claude-companion
description: Install (or update, or uninstall) Claude Companion — the pixel Clawd that lives on the desktop and opens Claude or Claude Code — on the user's macOS, Windows or Linux computer. Use when the user asks to install, set up, update or remove Claude Companion / Pixel.
---

# Install Claude Companion

You are installing Claude Companion on the user's own computer. Work through the steps in order, run the
commands yourself, and tell the user in one short line what you are doing at each step.

- Repository: https://github.com/bogdanminko/claude-companion
- Latest release files: `https://github.com/bogdanminko/claude-companion/releases/latest/download/<file>`

Rules:
- Match running copies by process name or an anchored pattern (as below), never a bare `pkill -f claude-companion`:
  that also matches the shell running your command.
- Ask the user before any command that needs `sudo` or an admin password.
- If a step fails, show the error, try the fallback given for that step, and only then ask the user.
- If the user asked to **uninstall**, skip to [Uninstall](#uninstall).

## 1. Detect the system

- macOS / Linux: `uname -s` and `uname -m`.
- Windows (PowerShell): `$env:PROCESSOR_ARCHITECTURE`.

| System | Install with |
| --- | --- |
| macOS (arm64 or x86_64) | [macOS](#2a-macos) |
| Windows x64 (`AMD64`) | [Windows](#2b-windows) |
| Linux x86_64 with `apt` (Debian, Ubuntu, Mint, Pop!_OS…) | [Linux .deb](#2c-linux-deb) |
| Linux x86_64, other distributions | [Linux tarball](#2d-linux-tarball) |
| Anything else (Windows ARM, Linux arm64…) | [Build from source](#3-build-from-source) |

If a download returns 404 (no release published yet), go to [Build from source](#3-build-from-source).

## 2a. macOS

```bash
set -e
TMP=$(mktemp -d)
curl -fL -o "$TMP/ClaudeCompanion.dmg" https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-macOS.dmg
MNT=$(hdiutil attach -nobrowse -readonly "$TMP/ClaudeCompanion.dmg" | awk -F'\t' '/\/Volumes\//{print $NF}')
pkill -x ClaudeCompanion 2>/dev/null || true
DEST=/Applications; [ -w "$DEST" ] || DEST="$HOME/Applications"; mkdir -p "$DEST"
rm -rf "$DEST/Claude Companion.app"
cp -R "$MNT/Claude Companion.app" "$DEST/"
hdiutil detach "$MNT" -quiet
# the build isn't notarized: clear the download quarantine so Gatekeeper doesn't block the first launch
xattr -dr com.apple.quarantine "$DEST/Claude Companion.app"
open "$DEST/Claude Companion.app"
```

Then tell the user to grant **Accessibility** (needed for double Option and the Caps Lock voice key):
System Settings → Privacy & Security → Accessibility → enable **Claude Companion**. Offer to open that pane:
`open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"`.

Go to [Verify](#4-verify).

## 2b. Windows

In PowerShell:

```powershell
$setup = Join-Path $env:TEMP "ClaudeCompanion-Setup.exe"
Invoke-WebRequest -UseBasicParsing -OutFile $setup https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-Setup.exe
Start-Process -Wait $setup -ArgumentList "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"
$exe = Join-Path $env:LOCALAPPDATA "Programs\ClaudeCompanion\claude-companion.exe"
if (-not (Get-Process claude-companion -ErrorAction SilentlyContinue)) { Start-Process $exe -ArgumentList "--supervise" }
```

The installer is per-user (no admin), adds a Start menu entry and starts Pixel at login.
If SmartScreen blocks it, tell the user: **More info → Run anyway**.

Go to [Verify](#4-verify).

## 2c. Linux .deb

Ask the user before running `sudo`:

```bash
set -e
TMP=$(mktemp -d)
curl -fL -o "$TMP/claude-companion_amd64.deb" https://github.com/bogdanminko/claude-companion/releases/latest/download/claude-companion_amd64.deb
sudo apt-get install -y "$TMP/claude-companion_amd64.deb"
pkill -f "^(/usr/bin/)?claude-companion( |$)" 2>/dev/null || true
setsid claude-companion --supervise >/dev/null 2>&1 < /dev/null &
```

Go to [Linux notes](#linux-notes), then [Verify](#4-verify).

## 2d. Linux tarball

No `sudo` needed:

```bash
set -e
TMP=$(mktemp -d)
curl -fL https://github.com/bogdanminko/claude-companion/releases/latest/download/claude-companion-linux-x86_64.tar.gz | tar -xz -C "$TMP"
BIN="$HOME/.local/bin/claude-companion"
pkill -f "^$BIN( |$)" 2>/dev/null || true
install -Dm755 "$TMP/claude-companion/claude-companion" "$BIN"
install -Dm644 "$TMP/claude-companion/claude-companion.png" "$HOME/.local/share/icons/hicolor/256x256/apps/claude-companion.png"
sed "s|^Exec=.*|Exec=\"$BIN\" --supervise|" "$TMP/claude-companion/claude-companion.desktop" \
  | install -Dm644 /dev/stdin "$HOME/.local/share/applications/claude-companion.desktop"
setsid "$BIN" --supervise >/dev/null 2>&1 < /dev/null &
```

### Linux notes

- Pixel needs an X11 or XWayland session (`echo $DISPLAY` must not be empty) and a compositor for transparency.
- Runtime libraries: libX11, libXtst, libxkbcommon-x11, libGL/libEGL. If it fails to start with a missing
  `lib…so`, install them (ask before `sudo`): Debian/Ubuntu `libxtst6 libxkbcommon-x11-0 libgl1`,
  Fedora `libXtst libxkbcommon-x11 mesa-libGL`, Arch `libxtst libxkbcommon-x11 libglvnd`.
- Tray icon on GNOME needs the AppIndicator extension; the right-click menu on Pixel works without it.

## 3. Build from source

Fallback for other architectures or when there is no release yet.

1. Check for `git` and Rust: `cargo --version`. If Rust is missing, ask the user, then install it:
   macOS / Linux `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . "$HOME/.cargo/env"`;
   Windows `winget install --id Rustlang.Rustup -e` (Rust on Windows also needs the Visual Studio C++ Build Tools:
   `winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"`).
   macOS also needs the Command Line Tools: `xcode-select --install` if `xcode-select -p` fails.
2. Clone and install:
   - macOS / Linux:
     ```bash
     git clone --depth 1 https://github.com/bogdanminko/claude-companion.git ~/claude-companion
     cd ~/claude-companion && make install
     ```
   - Windows:
     ```powershell
     git clone --depth 1 https://github.com/bogdanminko/claude-companion.git "$HOME\claude-companion"
     cd "$HOME\claude-companion"; powershell -ExecutionPolicy Bypass -File scripts\install.ps1
     ```
3. On macOS, ask for Accessibility as in [macOS](#2a-macos).

## 4. Verify

- macOS: `pgrep -x ClaudeCompanion`
- Windows: `Get-Process claude-companion`
- Linux: `pgrep -fa "^[^ ]*claude-companion( |$)"`

If it is not running, show the log and fix what it says:
macOS `/tmp/claude-companion.err.log`; Linux `~/.cache/claude-companion/companion.log`;
Windows `%LOCALAPPDATA%\claude-companion\companion.log`.

Finish with a short note for the user:
- Pixel sits in the bottom-right corner; hover it for the **Claude** / **Claude Code** capsule.
- **Double Alt** (double **Option** on a Mac) from any app shows the capsule; right-click Pixel for the menu.
- It starts at login; turn that off with **Start at login** in the menu.
- Claude Desktop (https://claude.ai/download) is optional: without it Pixel opens claude.ai and the `claude` CLI.

## Uninstall

- macOS:
  ```bash
  launchctl bootout "gui/$(id -u)/com.bogdanminko.claude-companion" 2>/dev/null || true
  pkill -x ClaudeCompanion 2>/dev/null || true
  rm -f ~/Library/LaunchAgents/com.bogdanminko.claude-companion.plist
  rm -rf "/Applications/Claude Companion.app" "$HOME/Applications/Claude Companion.app"
  ```
- Windows: run `"$env:LOCALAPPDATA\Programs\ClaudeCompanion\unins000.exe" /VERYSILENT` if it exists,
  otherwise `scripts\uninstall.ps1` from the repository.
- Linux .deb (ask before `sudo`): `sudo apt-get remove -y claude-companion`, then
  `rm -f ~/.config/autostart/claude-companion.desktop`.
- Linux tarball or source install:
  ```bash
  pkill -f "^[^ ]*claude-companion( |$)" 2>/dev/null || true
  rm -f ~/.local/bin/claude-companion ~/.local/share/applications/claude-companion.desktop \
        ~/.local/share/icons/hicolor/256x256/apps/claude-companion.png ~/.config/autostart/claude-companion.desktop
  ```
