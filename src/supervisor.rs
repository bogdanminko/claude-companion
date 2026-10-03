//! `--supervise`: runs Pixel as a child process and restarts it after a crash, like launchd's KeepAlive on macOS.
//! Quit (exit 0) ends both. Used by the Windows and Linux autostart entries; output goes to a log file
//! (`log_path()`), since there is no terminal to print to.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const THROTTLE: Duration = Duration::from_secs(10);
const MAX_LOG: u64 = 1 << 20;

pub fn log_path() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("claude-companion").join("companion.log"))
}

fn open_log() -> Option<File> {
    let path = log_path()?;
    std::fs::create_dir_all(path.parent()?).ok()?;
    let truncate = path.metadata().map(|m| m.len() > MAX_LOG).unwrap_or(false);
    OpenOptions::new().create(true).append(!truncate).write(true).truncate(truncate).open(path).ok()
}

pub fn run() -> ! {
    let exe = std::env::current_exe().expect("own path");
    let mut log = open_log();
    loop {
        let started = Instant::now();
        let mut cmd = Command::new(&exe);
        if let Some(f) = &log {
            if let (Ok(out), Ok(err)) = (f.try_clone(), f.try_clone()) {
                cmd.stdout(Stdio::from(out)).stderr(Stdio::from(err));
            }
        }
        let message = match cmd.status() {
            Ok(s) if s.success() => std::process::exit(0),
            Ok(s) => format!("claude-companion: exited with {s}, restarting"),
            Err(e) => format!("claude-companion: can't start: {e}"),
        };
        match &mut log {
            Some(f) => {
                let _ = writeln!(f, "{message}");
            }
            None => eprintln!("{message}"),
        }
        if let Some(wait) = THROTTLE.checked_sub(started.elapsed()) {
            std::thread::sleep(wait);
        }
    }
}
