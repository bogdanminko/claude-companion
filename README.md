<div align="center">

<img src="docs/fiesta.gif" width="260" alt="Pixel playing guitar in a sombrero">

# Claude Companion

**A tiny pixel Clawd that lives on your desktop and opens Claude or Claude Code in one move.**

macOS · Windows · Linux · Rust · one small binary

**[⬇ Download](#download)**

</div>

> Unofficial fan project, not affiliated with Anthropic. Clawd is the Claude Code mascot from the CLI banner.

## What it does

Pixel sits on top of all your windows. Hover it and a small capsule pops up with two icons:

- **Claude** — opens the desktop app;
- **Claude Code** — opens a new session in the Code tab.

The rest of the time it just lives there: blinks, shuffles its legs, gets bored and does tricks.

| | | | |
|:-:|:-:|:-:|:-:|
| <img src="docs/idle.gif" width="130"><br>idle | <img src="docs/jump.gif" width="130"><br>jump | <img src="docs/wave.gif" width="130"><br>wave | <img src="docs/look.gif" width="130"><br>look around |
| <img src="docs/shake.gif" width="130"><br>shiver | <img src="docs/glitch.gif" width="130"><br>glitch | <img src="docs/fiesta.gif" width="130"><br>fiesta 🎸 | <img src="docs/sleep.gif" width="130"><br>sleep |

<p align="center"><img src="docs/clones.gif" width="260" alt="Pixel vanishes in smoke and five small clones pop out"><br>shadow clone technique 🥷</p>

## Features

- floats above all windows on every desktop / Space; drag it anywhere, it remembers the spot;
- a random trick every 8–25 seconds when idle — walks off sideways, jumps, shivers, waves, glitches, or puts on a sombrero and plays guitar; once in a while it vanishes in a puff of smoke and splits into five small clones;
- blushes and looks up on hover, falls asleep after 5 minutes without attention;
- **double click** — it glitches; **drag** it and its legs run in the air;
- **double Alt** (**double Option** on a Mac) from any app shows the capsule;
- **voice** — Claude voice input via Caps Lock, Pixel flaps its arms while you talk;
- right click / tray or menu bar icon: voice, open Claude / Claude Code, hide, sleep, fiesta, shadow clones, back to corner, start at login, quit;
- starts at login and restarts after a crash (LaunchAgent on macOS, `--supervise` on Windows and Linux).

## Download

<div align="center">

[![Download for macOS](https://img.shields.io/badge/Download-macOS-D97757?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-macOS.dmg)
[![Download for Windows](https://img.shields.io/badge/Download-Windows-D97757?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-Setup.exe)
[![Download for Linux](https://img.shields.io/badge/Download-Linux%20.deb-D97757?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/bogdanminko/claude-companion/releases/latest/download/claude-companion_amd64.deb)

</div>

Each button downloads the file from the [latest release](https://github.com/bogdanminko/claude-companion/releases/latest).

| System | Download | How |
| --- | --- | --- |
| macOS 11+ (Apple Silicon and Intel) | [ClaudeCompanion-macOS.dmg](https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-macOS.dmg) | drag to Applications, open |
| Windows 10 / 11 | [ClaudeCompanion-Setup.exe](https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-Setup.exe) | run the installer |
| Windows, no install | [ClaudeCompanion-portable.exe](https://github.com/bogdanminko/claude-companion/releases/latest/download/ClaudeCompanion-portable.exe) | just run it |
| Linux (Debian, Ubuntu…) | [claude-companion_amd64.deb](https://github.com/bogdanminko/claude-companion/releases/latest/download/claude-companion_amd64.deb) | `sudo apt install ./claude-companion_amd64.deb`, start from the app menu |
| Linux (any) | [claude-companion-linux-x86_64.tar.gz](https://github.com/bogdanminko/claude-companion/releases/latest/download/claude-companion-linux-x86_64.tar.gz) | unpack, run `./claude-companion` |

### Or let Claude install it

Paste into [Claude Code](https://claude.com/claude-code) — it picks the right build for your system, installs it and
starts Pixel:

```bash
claude "Install Claude Companion: https://raw.githubusercontent.com/bogdanminko/claude-companion/main/skills/install/SKILL.md"
```

Any Claude that can run commands on your computer understands the same line. To keep it as a reusable skill
(`/install-claude-companion` — also updates and uninstalls):

```bash
mkdir -p ~/.claude/skills/install-claude-companion && curl -fsSL https://raw.githubusercontent.com/bogdanminko/claude-companion/main/skills/install/SKILL.md -o ~/.claude/skills/install-claude-companion/SKILL.md
```

Pixel turns on **Start at login** on its first launch; switch it off in the right-click or tray menu.
Launching it again while it runs just brings Pixel back.

The builds are not signed with paid certificates yet, so the OS asks once:

- **macOS**: "Apple could not verify…" → System Settings → Privacy & Security → **Open Anyway**
  (or `xattr -dr com.apple.quarantine "/Applications/Claude Companion.app"`). Then grant **Accessibility**, see below.
- **Windows**: SmartScreen "Windows protected your PC" → **More info** → **Run anyway**.

## Build from source

Requires [Rust](https://rustup.rs) to build. [Claude Desktop](https://claude.ai/download) is optional: without it
Pixel opens claude.ai in the browser, and Claude Code in a terminal if the `claude` CLI is installed.

### macOS 11+

```bash
git clone https://github.com/bogdanminko/claude-companion.git
cd claude-companion
make install
```

Then grant **Accessibility** access (needed for double Option and the Caps Lock voice key):
System Settings → Privacy & Security → Accessibility → enable **Claude Companion**. No restart needed.

> The build is ad-hoc signed, so macOS asks for access again after every reinstall —
> `install.sh` resets the stale entry for you.

### Windows 10 / 11

```powershell
git clone https://github.com/bogdanminko/claude-companion.git
cd claude-companion
powershell -ExecutionPolicy Bypass -File scripts\install.ps1
```

Installs to `%LOCALAPPDATA%\Programs\ClaudeCompanion`, adds a Start menu shortcut and a login entry
(`HKCU\…\Run`). Remove with `scripts\uninstall.ps1`.

### Linux

```bash
git clone https://github.com/bogdanminko/claude-companion.git
cd claude-companion
make install
```

Installs `~/.local/bin/claude-companion`, an app menu entry and an XDG autostart entry. Remove with `make uninstall`.

- Needs X11 or XWayland (on Wayland sessions Pixel runs through XWayland: Wayland doesn't let windows place
  themselves) and a compositor for transparency — every mainstream desktop has one.
- Runtime libraries present on any desktop: libX11, libXtst, libxkbcommon-x11, libGL / libEGL.
- Tray icon: KDE, Xfce, Cinnamon, MATE; GNOME needs the AppIndicator extension. Without a tray the right-click menu has everything.
- Double Alt and Caps Lock tracking see key presses in X11 / XWayland apps; native Wayland apps hide them.
- Claude Desktop has no official Linux build; community builds (`claude-desktop`) are picked up automatically.

For voice, enable Caps Lock voice input in Claude settings.

## Commands

| Command | What it does |
| --- | --- |
| `make install` | build, install, register autostart (macOS / Linux) |
| `make run` | build and run without installing |
| `make restart` | restart the installed copy |
| `make logs` | tail the logs |
| `make uninstall` | stop and remove |
| `make test` | unit tests |
| `make preview` | render all sprite states to `build/preview.png` |
| `make demo` | re-record the GIFs in `docs/` |
| `make icon` | regenerate the app icons in `Resources/` from the sprite |

## How it works

```
src/
  main.rs         — windows, mouse, drag, capsule / menu wiring, tray and hotkey polling
  companion.rs    — moods, tricks, animation (no OS code)
  sprite.rs       — Clawd, 18×10 pixels, block for block from the CLI banner
  bubble.rs       — Claude / Claude Code capsule
  menu.rs         — right-click menu, drawn in an 8×8 pixel font
  canvas.rs       — tiny software renderer: pixel blocks, rounded shapes, text
  gfx.rs          — puts canvases on transparent windows (OpenGL texture)
  hotkey.rs       — double Alt / Option and Caps Lock from polled key state
  voice.rs        — Claude voice input via Caps Lock
  tray.rs         — tray / menu bar icon (native on macOS and Windows, StatusNotifierItem on Linux)
  supervisor.rs   — --supervise: restart after a crash, log to the cache folder
  autostart.rs    — start at login: LaunchAgent / HKCU Run / XDG autostart, set up by the app itself
  instance.rs     — one Pixel per user; a second launch brings the running one back
  platform/       — macOS (AppKit, CoreGraphics), Windows (Win32), Linux (X11): keys, cursor,
                    work area, window flags, opening Claude
examples/         — sprite preview, demo GIF recorder, icon generator
launchd/          — LaunchAgent template (macOS)
scripts/          — build / install / uninstall (.sh for macOS and Linux, .ps1 for Windows)
installer/        — Windows installer (Inno Setup)
skills/install/   — SKILL.md: lets Claude install, update or remove Pixel on any system
.github/workflows — CI on all three systems; release.yml builds the downloads on a v* tag
```

Everything is drawn in code, no image assets. Each quadrant of the CLI banner `▐▛███▜▌ / ▝▜█████▛▘ / ▘▘ ▝▝`
becomes two square pixels (terminal cells are twice as tall as wide). Change shapes in `sprite.rs`, check with `make preview`.

On macOS `KeepAlive` is `SuccessfulExit = false`; elsewhere `--supervise` does the same: Quit closes Pixel until
next login, a crash restarts it.

## License

[MIT](LICENSE)
