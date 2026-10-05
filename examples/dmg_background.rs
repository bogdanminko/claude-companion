//! Background of the macOS disk image window: "drag Pixel into Applications", in the same pixel style.
//!   cargo run --example dmg_background -- Resources   →  dmg-background.png + dmg-background@2x.png (600×400 pt)
//! The icon spots in release.yml (app at 160,190, Applications at 440,190) must match the arrow here.

use claude_companion::canvas::{Canvas, Rgba};
use claude_companion::sprite;
use std::io::BufWriter;

const SIZE: (f32, f32) = (600.0, 400.0);
const PAPER: Rgba = Rgba::hex(0xF4EFE9);
const INK: Rgba = Rgba::hex(0x3A2A24);
const MUTED: Rgba = Rgba::hex(0x9A8A80);

fn centered(c: &mut Canvas, y: f32, size: f32, s: &str, color: Rgba) {
    let w = s.len() as f32 * 8.0 * size;
    c.text(((SIZE.0 - w) / 2.0).round(), y, size, s, color);
}

fn render(scale: f32) -> Canvas {
    let mut c = Canvas::new(SIZE, scale);
    c.clear(PAPER);
    centered(&mut c, 44.0, 2.0, "Drag Pixel to Applications", INK);

    // a pixel arrow between the two icons: shaft, then a chevron whose tip ends the shaft
    let (p, y) = (6.0, 190.0);
    let tip = 348.0;
    c.fill_rect(246.0, y - p / 2.0, tip - 246.0, p, sprite::BODY);
    for k in 0..5 {
        let x = tip - p - k as f32 * p;
        for dy in [-1.0, 1.0] {
            c.fill_rect(x - p, y - p / 2.0 + dy * k as f32 * p, 2.0 * p, p, sprite::BODY);
        }
    }

    centered(&mut c, 330.0, 1.0, "Then open it from Applications", MUTED);
    centered(&mut c, 346.0, 1.0, "and allow Accessibility when asked", MUTED);
    c
}

fn write_png(path: &std::path::Path, c: &Canvas) {
    let mut enc = png::Encoder::new(BufWriter::new(std::fs::File::create(path).unwrap()), c.width as u32, c.height as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&c.data).unwrap();
    println!("✓ {}", path.display());
}

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "Resources".into()));
    std::fs::create_dir_all(&dir).unwrap();
    write_png(&dir.join("dmg-background.png"), &render(1.0));
    write_png(&dir.join("dmg-background@2x.png"), &render(2.0));
}
