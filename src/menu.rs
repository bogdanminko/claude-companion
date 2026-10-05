//! Right-click menu on Pixel, drawn in the same pixel style (8×8 bitmap font), identical on every OS.
//! The tray / menu bar icon uses the native menu with the same actions.

use crate::canvas::{Canvas, Rgba};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    Voice,
    OpenClaude,
    OpenCode,
    Toggle,
    Sleep,
    Fiesta,
    Clones,
    ResetPosition,
    /// Toggle: start Pixel at login (checked when on).
    Autostart,
    Quit,
}

pub enum Entry {
    Item(Action),
    Separator,
}

pub const ENTRIES: [Entry; 12] = [
    Entry::Item(Action::Voice),
    Entry::Item(Action::OpenClaude),
    Entry::Item(Action::OpenCode),
    Entry::Separator,
    Entry::Item(Action::Toggle),
    Entry::Item(Action::Sleep),
    Entry::Item(Action::Fiesta),
    Entry::Item(Action::Clones),
    Entry::Item(Action::ResetPosition),
    Entry::Separator,
    Entry::Item(Action::Autostart),
    Entry::Item(Action::Quit),
];

pub fn title(a: Action, listening: bool) -> &'static str {
    match a {
        Action::Voice if listening => "Stop voice",
        Action::Voice => "Voice",
        Action::OpenClaude => "Open Claude",
        Action::OpenCode => "Open Claude Code",
        Action::Toggle => "Show / hide Pixel",
        Action::Sleep => "Put to sleep",
        Action::Fiesta => "Fiesta!",
        Action::Clones => "Shadow clones!",
        Action::ResetPosition => "Back to corner",
        Action::Autostart => "Start at login",
        Action::Quit => "Quit",
    }
}

const PAD: f32 = 6.0;
const ITEM_H: f32 = 22.0;
const SEP_H: f32 = 9.0;
const BG: Rgba = Rgba::rgba(41, 41, 43, 248);
const BORDER: Rgba = Rgba::rgba(255, 255, 255, 31);
const TEXT: Rgba = Rgba::rgba(236, 236, 236, 255);
const SEP: Rgba = Rgba::rgba(255, 255, 255, 36);
const HIGHLIGHT: Rgba = crate::sprite::BODY;

/// Font pixel size snapped to whole physical pixels, so the 8×8 glyphs stay even.
fn font_px(scale: f32) -> f32 {
    (1.5 * scale).round().max(1.0) / scale
}

pub fn size(scale: f32) -> (f32, f32) {
    let fp = font_px(scale);
    let longest = ENTRIES
        .iter()
        .filter_map(|e| if let Entry::Item(a) = e { Some(title(*a, true).len().max(title(*a, false).len())) } else { None })
        .max()
        .unwrap_or(0);
    let h: f32 = ENTRIES.iter().map(|e| if matches!(e, Entry::Item(_)) { ITEM_H } else { SEP_H }).sum();
    // + room for the check mark
    (((longest + 2) as f32 * 8.0 * fp + 4.0 * PAD).ceil(), h + 2.0 * PAD)
}

pub fn item_at(x: f32, y: f32, scale: f32) -> Option<Action> {
    let (w, _) = size(scale);
    if x < 0.0 || x >= w {
        return None;
    }
    let mut top = PAD;
    for e in &ENTRIES {
        match e {
            Entry::Item(a) => {
                if y >= top && y < top + ITEM_H {
                    return Some(*a);
                }
                top += ITEM_H;
            }
            Entry::Separator => top += SEP_H,
        }
    }
    None
}

/// Pixel check mark for toggles that are on.
const CHECK: [&str; 5] = ["......x", ".....x.", "x...x..", ".x.x...", "..x...."];

pub fn draw(c: &mut Canvas, hovered: Option<Action>, listening: bool, autostart: bool) {
    let (w, h) = size(c.scale);
    let fp = font_px(c.scale);
    c.clear(Rgba::rgba(0, 0, 0, 0));
    c.fill_round_rect(0.0, 0.0, w, h, 7.0, BG);
    c.stroke_round_rect(0.0, 0.0, w, h, 7.0, 1.0, BORDER);
    let mut top = PAD;
    for e in &ENTRIES {
        match e {
            Entry::Item(a) => {
                if hovered == Some(*a) {
                    c.fill_round_rect(PAD - 2.0, top, w - 2.0 * PAD + 4.0, ITEM_H, 4.0, HIGHLIGHT);
                }
                let ty = top + ((ITEM_H - 8.0 * fp) / 2.0 * c.scale).round() / c.scale;
                c.text(2.0 * PAD, ty, fp, title(*a, listening), TEXT);
                if *a == Action::Autostart && autostart {
                    let (cx, cy) = (w - 2.0 * PAD - 7.0 * fp, ty + 1.5 * fp);
                    for (r, line) in CHECK.iter().enumerate() {
                        for (col, ch) in line.chars().enumerate() {
                            if ch == 'x' {
                                c.fill_rect(cx + col as f32 * fp, cy + r as f32 * fp, fp, fp, TEXT);
                            }
                        }
                    }
                }
                top += ITEM_H;
            }
            Entry::Separator => {
                c.fill_rect(PAD, top + (SEP_H / 2.0).floor(), w - 2.0 * PAD, 1.0, SEP);
                top += SEP_H;
            }
        }
    }
}
