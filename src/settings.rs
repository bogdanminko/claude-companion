//! Remembers where Pixel sits, in desktop units (see `platform`).

use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("claude-companion").join("position"))
}

pub fn load_position() -> Option<(f64, f64)> {
    let text = std::fs::read_to_string(path()?).ok()?;
    let mut it = text.split_whitespace().map(|v| v.parse::<f64>());
    match (it.next(), it.next()) {
        (Some(Ok(x)), Some(Ok(y))) if x.is_finite() && y.is_finite() => Some((x, y)),
        _ => None,
    }
}

pub fn save_position(p: (f64, f64)) {
    let Some(path) = path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, format!("{} {}\n", p.0.round(), p.1.round()));
}
