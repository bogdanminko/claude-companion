//! Dark capsule with two icon buttons: Claude and Claude Code. Shows on hover over Pixel.

use crate::canvas::{Canvas, Rgba};
use crate::sprite;

pub const SIZE: (f32, f32) = (112.0, 50.0);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Button {
    Claude,
    Code,
}

const BG: Rgba = Rgba::rgba(41, 41, 43, 245);
const BORDER: Rgba = Rgba::rgba(255, 255, 255, 31);
const DIVIDER: Rgba = Rgba::rgba(255, 255, 255, 46);
const HIGHLIGHT: Rgba = Rgba::rgba(255, 255, 255, 26);

pub fn button_at(x: f32, y: f32) -> Option<Button> {
    let (w, h) = SIZE;
    if x < 0.0 || y < 0.0 || x >= w || y >= h {
        None
    } else if x < w / 2.0 {
        Some(Button::Claude)
    } else {
        Some(Button::Code)
    }
}

pub fn draw(c: &mut Canvas, hovered: Option<Button>) {
    let (w, h) = SIZE;
    c.clear(Rgba::rgba(0, 0, 0, 0));
    c.fill_round_rect(0.0, 0.0, w, h, h / 2.0, BG);
    c.stroke_round_rect(0.0, 0.0, w, h, h / 2.0, 1.0, BORDER);

    let half = w / 2.0;
    match hovered {
        Some(Button::Claude) => c.fill_round_rect(6.0, 5.0, half - 8.0, 40.0, 20.0, HIGHLIGHT),
        Some(Button::Code) => c.fill_round_rect(half + 2.0, 5.0, half - 8.0, 40.0, 20.0, HIGHLIGHT),
        None => {}
    }
    c.fill_rect(half - 0.5, 13.0, 1.0, 24.0, DIVIDER);

    claude_logo(c, (half + 2.0) / 2.0, h / 2.0, 13.0);

    // Claude Code: the same Clawd sprite at 2 pt per pixel
    let p = 2.0;
    let grid = sprite::standing();
    let ox = half + (half - sprite::W as f32 * p) / 2.0 - 1.0;
    let oy = (h - grid.len() as f32 * p) / 2.0;
    for (y, row) in grid.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            if let Some(color) = color {
                c.fill_rect(ox + x as f32 * p, oy + y as f32 * p, p, p, *color);
            }
        }
    }
}

/// Claude's spark: uneven rays around a centre, in the Claude orange.
pub fn claude_logo(c: &mut Canvas, cx: f32, cy: f32, r: f32) {
    const RAYS: [f32; 12] = [1.0, 0.78, 0.92, 0.72, 0.98, 0.8, 0.9, 0.74, 1.0, 0.82, 0.88, 0.76];
    for (i, len) in RAYS.iter().enumerate() {
        let a = i as f32 / RAYS.len() as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let (s, co) = a.sin_cos();
        let inner = r * 0.22;
        c.line((cx + co * inner, cy + s * inner), (cx + co * r * len, cy + s * r * len), r * 0.2, sprite::BODY);
    }
}
