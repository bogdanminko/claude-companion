//! Sprite preview: `cargo run --example preview -- build/preview.png` — all states side by side.

use claude_companion::sprite::{self, Eyes, Screen};
use std::io::BufWriter;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "build/preview.png".into());
    let frames: [(Eyes, bool, Screen, f64, Option<i32>); 5] = [
        (Eyes::Open { look_up: false }, false, Screen::Prompt, 0.0, None),
        (Eyes::Open { look_up: true }, true, Screen::Prompt, 1.2, None),
        (Eyes::Open { look_up: false }, false, Screen::Thinking { active: 0 }, 2.4, Some(0)),
        (Eyes::Open { look_up: false }, false, Screen::Listening { tick: 3 }, 3.6, None),
        (Eyes::Closed, false, Screen::Off, 4.8, Some(1)),
    ];
    let (s, gap) = (14, 40);
    let (cw, ch) = (sprite::W * s, sprite::H * s);
    let (w, h) = (frames.len() * (cw + gap) + gap, ch + 2 * gap);
    let mut img: Vec<u8> = [20u8, 20, 20, 255].repeat(w * h);
    for (i, f) in frames.iter().enumerate() {
        let grid = sprite::render(f.3, f.0, f.1, f.2, f.4);
        let ox = gap + i * (cw + gap);
        for (y, row) in grid.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                let Some(c) = c else { continue };
                for py in gap + y * s..gap + (y + 1) * s {
                    for px in ox + x * s..ox + (x + 1) * s {
                        img[(py * w + px) * 4..][..4].copy_from_slice(&c.0);
                    }
                }
            }
        }
    }
    write_png(&out, w, h, &img);
    println!("✓ {out}");
}

fn write_png(path: &str, w: usize, h: usize, rgba: &[u8]) {
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    let mut enc = png::Encoder::new(BufWriter::new(std::fs::File::create(path).unwrap()), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(rgba).unwrap();
}
