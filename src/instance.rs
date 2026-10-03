//! One Pixel per user: a second launch (double-clicking the app again, the Start menu, autostart) doesn't add
//! another one; it asks the running copy to show itself and exits.

use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;

fn dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("claude-companion"))
}

/// The lock is held as long as the returned file is alive. `None`: another copy is running.
pub fn acquire() -> Option<Option<File>> {
    let Some(dir) = dir() else { return Some(None) };
    let _ = std::fs::create_dir_all(&dir);
    let Ok(file) = File::options().create(true).truncate(false).write(true).open(dir.join("instance.lock")) else {
        return Some(None); // can't tell: run anyway
    };
    match file.try_lock() {
        Ok(()) => Some(Some(file)),
        Err(std::fs::TryLockError::WouldBlock) => None,
        Err(_) => Some(Some(file)),
    }
}

fn summon_path() -> Option<PathBuf> {
    Some(dir()?.join("summon"))
}

/// Ask the running copy to show Pixel.
pub fn summon_running() {
    if let Some(p) = summon_path() {
        let _ = std::fs::write(p, format!("{:?}", SystemTime::now()));
    }
}

/// Notices `summon_running()` from another launch.
pub struct SummonWatch {
    seen: Option<SystemTime>,
}

impl SummonWatch {
    pub fn new() -> Self {
        SummonWatch { seen: Self::stamp() }
    }

    fn stamp() -> Option<SystemTime> {
        summon_path()?.metadata().ok()?.modified().ok()
    }

    pub fn check(&mut self) -> bool {
        let now = Self::stamp();
        let fired = now.is_some() && now != self.seen;
        self.seen = now;
        fired
    }
}
