//! App icons: Clawd on a dark rounded square.
//!   cargo run --example icon -- build/icons   →  macOS .iconset PNGs, a 256 px PNG for Linux, an .ico for Windows

use claude_companion::canvas::{Canvas, Rgba};
use claude_companion::sprite;
use std::io::{BufWriter, Write};

fn render(size: usize) -> Vec<u8> {
    let s = size as f32;
    let mut c = Canvas::new((s, s), 1.0);
    // macOS icon grid: 824/1024 rounded square, centred
    let inset = s * 100.0 / 1024.0;
    let side = s - 2.0 * inset;
    c.fill_round_rect(inset, inset, side, side, side * 0.225, Rgba::rgba(26, 26, 28, 255));
    let grid = sprite::standing();
    let p = (side * 0.78 / sprite::W as f32).floor().max(1.0);
    let ox = ((s - p * sprite::W as f32) / 2.0).round();
    let oy = ((s - p * grid.len() as f32) / 2.0).round();
    for (y, row) in grid.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            if let Some(color) = color {
                c.fill_rect(ox + x as f32 * p, oy + y as f32 * p, p, p, *color);
            }
        }
    }
    c.data
}

fn png_bytes(size: usize) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, size as u32, size as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&render(size)).unwrap();
    }
    out
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "build/icons".into());
    let iconset = format!("{dir}/AppIcon.iconset");
    std::fs::create_dir_all(&iconset).unwrap();
    for base in [16, 32, 128, 256, 512] {
        std::fs::write(format!("{iconset}/icon_{base}x{base}.png"), png_bytes(base)).unwrap();
        std::fs::write(format!("{iconset}/icon_{base}x{base}@2x.png"), png_bytes(base * 2)).unwrap();
    }
    std::fs::write(format!("{dir}/claude-companion.png"), png_bytes(256)).unwrap();

    // .ico with embedded PNGs (Vista+)
    let sizes = [16usize, 24, 32, 48, 64, 128, 256];
    let images: Vec<Vec<u8>> = sizes.iter().map(|&s| png_bytes(s)).collect();
    let mut ico = BufWriter::new(std::fs::File::create(format!("{dir}/claude-companion.ico")).unwrap());
    ico.write_all(&[0, 0, 1, 0]).unwrap();
    ico.write_all(&(sizes.len() as u16).to_le_bytes()).unwrap();
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (s, img) in sizes.iter().zip(&images) {
        let dim = if *s >= 256 { 0 } else { *s as u8 };
        ico.write_all(&[dim, dim, 0, 0]).unwrap();
        ico.write_all(&1u16.to_le_bytes()).unwrap(); // planes
        ico.write_all(&32u16.to_le_bytes()).unwrap(); // bits per pixel
        ico.write_all(&(img.len() as u32).to_le_bytes()).unwrap();
        ico.write_all(&offset.to_le_bytes()).unwrap();
        offset += img.len() as u32;
    }
    for img in &images {
        ico.write_all(img).unwrap();
    }
    println!("✓ {dir}");
}
