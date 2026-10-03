//! Start at login, managed by the app itself, so a downloaded app needs no installer.
//! macOS: a LaunchAgent (the same one `make install` writes), Windows: `HKCU\…\Run`, Linux: XDG autostart.
//! Windows and Linux run Pixel under `--supervise` to restart it after a crash, like launchd's KeepAlive.

use std::path::{Path, PathBuf};

const MARKER: &str = "autostart-configured";

/// The program autostart should launch.
fn program() -> Option<PathBuf> {
    // an AppImage runs from a temporary mount; autostart must point at the image itself
    if let Some(image) = std::env::var_os("APPIMAGE") {
        return Some(image.into());
    }
    std::env::current_exe().ok()
}

/// Builds from the source tree, a mounted .dmg or a quarantined (translocated) app: not a place to start from at login.
fn is_transient(exe: &Path) -> bool {
    let s = exe.to_string_lossy();
    s.contains("/target/") || s.contains("\\target\\") || s.starts_with("/Volumes/") || s.contains("/AppTranslocation/")
}

/// First launch of an installed copy turns autostart on once; later the menu toggle decides.
/// If it is on, re-point it at this copy (the app may have been moved or updated).
pub fn sync() {
    let Some(exe) = program() else { return };
    if is_transient(&exe) {
        return;
    }
    let Some(marker) = dirs::config_dir().map(|d| d.join("claude-companion").join(MARKER)) else { return };
    if !marker.exists() {
        if let Some(dir) = marker.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&marker, "");
        set(true);
    } else if is_enabled() {
        set(true);
    }
}

pub fn set(enabled: bool) {
    let result = match program() {
        Some(exe) if enabled => imp::enable(&exe),
        Some(_) => imp::disable(),
        None => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("claude-companion: can't change autostart: {e}");
    }
}

pub fn is_enabled() -> bool {
    imp::is_enabled()
}

#[cfg(target_os = "macos")]
mod imp {
    use std::path::{Path, PathBuf};

    const LABEL: &str = "com.bogdanminko.claude-companion";

    fn plist() -> Option<PathBuf> {
        Some(dirs::home_dir()?.join("Library/LaunchAgents").join(format!("{LABEL}.plist")))
    }

    fn escape(s: &str) -> String {
        s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
    }

    /// Same agent as launchd/com.bogdanminko.claude-companion.plist: start at login, restart only after a crash.
    pub fn enable(exe: &Path) -> std::io::Result<()> {
        let path = plist().ok_or(std::io::ErrorKind::NotFound)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>10</integer>
    <key>LimitLoadToSessionType</key>
    <string>Aqua</string>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>StandardOutPath</key>
    <string>/tmp/claude-companion.out.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/claude-companion.err.log</string>
</dict>
</plist>
"#,
            escape(&exe.to_string_lossy())
        );
        // takes effect at the next login; loading it now would start a second copy
        if std::fs::read_to_string(&path).ok().as_deref() != Some(body.as_str()) {
            std::fs::write(&path, body)?;
        }
        Ok(())
    }

    pub fn disable() -> std::io::Result<()> {
        match plist() {
            Some(p) if p.exists() => std::fs::remove_file(p),
            _ => Ok(()),
        }
    }

    pub fn is_enabled() -> bool {
        plist().is_some_and(|p| p.exists())
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use std::path::Path;
    use windows_sys::Win32::System::Registry::{
        RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
    };

    const KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const NAME: &str = "ClaudeCompanion";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn enable(exe: &Path) -> std::io::Result<()> {
        let value = wide(&format!("\"{}\" --supervise", exe.display()));
        let (key, name) = (wide(KEY), wide(NAME));
        let r = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                value.as_ptr().cast(),
                (value.len() * 2) as u32,
            )
        };
        if r == 0 {
            Ok(())
        } else {
            Err(std::io::Error::from_raw_os_error(r as i32))
        }
    }

    pub fn disable() -> std::io::Result<()> {
        let (key, name) = (wide(KEY), wide(NAME));
        unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr()) };
        Ok(())
    }

    pub fn is_enabled() -> bool {
        let (key, name) = (wide(KEY), wide(NAME));
        let r = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        r == 0
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::path::{Path, PathBuf};

    fn entry() -> Option<PathBuf> {
        Some(dirs::config_dir()?.join("autostart").join("claude-companion.desktop"))
    }

    pub fn enable(exe: &Path) -> std::io::Result<()> {
        let path = entry().ok_or(std::io::ErrorKind::NotFound)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        let exe = exe.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"");
        let body = format!(
            "[Desktop Entry]\nType=Application\nName=Claude Companion\n\
             Comment=A tiny pixel Clawd that opens Claude and Claude Code\n\
             Exec=\"{exe}\" --supervise\nIcon=claude-companion\nTerminal=false\n\
             Categories=Utility;\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\n"
        );
        if std::fs::read_to_string(&path).ok().as_deref() != Some(body.as_str()) {
            std::fs::write(&path, body)?;
        }
        Ok(())
    }

    pub fn disable() -> std::io::Result<()> {
        match entry() {
            Some(p) if p.exists() => std::fs::remove_file(p),
            _ => Ok(()),
        }
    }

    pub fn is_enabled() -> bool {
        entry().is_some_and(|p| p.exists())
    }
}
