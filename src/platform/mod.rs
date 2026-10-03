//! Everything that differs between macOS, Windows and Linux lives behind this module.
//!
//! "Desktop units" (DU): the coordinate space window positions are kept in. Physical pixels of the virtual
//! desktop on Windows and Linux (what winit and the OS cursor APIs use), logical points with a top-left
//! origin on macOS (what AppKit and winit use there). Origin top-left, y grows down everywhere.
//!
//! Each OS module provides:
//! - `key_state()` — polled keyboard / mouse snapshot for the double-Alt and Caps Lock detector;
//! - `CAPS_IS_TOGGLE` — whether `KeyState::caps` is the lock flag (macOS) or the key itself;
//! - `tap_caps_lock(on)` — press Caps Lock, to toggle Claude's voice input;
//! - `cursor_position()` — global cursor in DU;
//! - `monitors(event_loop)` and `work_area(point, monitors)` — screen rects in DU, work area without dock / taskbar;
//! - `prepare_window(window, kind)` — float on all desktops, keep out of the taskbar, don't steal focus;
//! - `ensure_accessibility(prompt)` — macOS permission for global keys and synthetic Caps Lock;
//! - `claude_running()`, `open_claude()`, `open_claude_code()`.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(all(unix, not(target_os = "macos")))]
pub use linux::*;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn contains(&self, p: (f64, f64)) -> bool {
        p.0 >= self.x && p.0 < self.x + self.w && p.1 >= self.y && p.1 < self.y + self.h
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
    pub fn intersect(&self, o: &Rect) -> Option<Rect> {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        (r > x && b > y).then_some(Rect { x, y, w: r - x, h: b - y })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WindowKind {
    /// Pixel itself: lives on the desktop for good.
    Pixel,
    /// Capsule and menu: short-lived popups.
    Popup,
}

/// Starts a program without tying it to Pixel's lifetime or console.
pub fn spawn_detached(cmd: &mut Command) -> bool {
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    match cmd.spawn() {
        Ok(_) => true,
        Err(e) => {
            eprintln!("claude-companion: failed to start {:?}: {e}", cmd.get_program());
            false
        }
    }
}

/// Finds a program in PATH, plus the usual per-user install spots that a login session may not have in PATH.
pub fn which(name: &str) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".claude/local"));
        dirs.push(home.join(".npm-global/bin"));
    }
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(|e| e.to_ascii_lowercase())
            .chain(std::iter::once(String::new()))
            .collect()
    } else {
        vec![String::new()]
    };
    dirs.iter().find_map(|d| exts.iter().map(|e| d.join(format!("{name}{e}"))).find(|p| is_executable(p)))
}

fn is_executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        p.metadata().map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

pub const CLAUDE_WEB: &str = "https://claude.ai/new";
pub const CLAUDE_CODE_WEB: &str = "https://claude.ai/code";
pub const CLAUDE_CODE_URL: &str = "claude://code/new"; // new session in the Code tab of the desktop app
