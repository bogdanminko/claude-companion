//! Demo GIFs for the README: drives the real Companion offscreen, one GIF per scene.
//!   cargo run --release --example demo -- docs   →  docs/*.gif

use claude_companion::canvas::Canvas;
use claude_companion::companion::{Companion, Trick, SIZE};

const SCALE: f32 = 2.0;
const BG: [u8; 3] = [20, 20, 23];

fn record(dir: &str, name: &str, frames: usize, setup: impl FnOnce(&mut Companion)) {
    let mut c = Companion::new();
    setup(&mut c);
    let mut canvas = Canvas::new(SIZE, SCALE);
    let (w, h) = (canvas.width as u16, canvas.height as u16);
    let path = format!("{dir}/{name}.gif");
    let mut enc = gif::Encoder::new(std::fs::File::create(&path).unwrap(), w, h, &[]).unwrap();
    enc.set_repeat(gif::Repeat::Infinite).unwrap();
    for _ in 0..frames {
        c.step();
        c.draw(&mut canvas);
        // flatten onto the dark background
        let mut rgb: Vec<u8> = Vec::with_capacity(canvas.data.len() / 4 * 3);
        for p in canvas.data.chunks_exact(4) {
            let a = p[3] as u32;
            for k in 0..3 {
                rgb.push(((p[k] as u32 * a + BG[k] as u32 * (255 - a)) / 255) as u8);
            }
        }
        let mut frame = gif::Frame::from_rgb_speed(w, h, &rgb, 10);
        frame.delay = 10; // 0.1 s
        enc.write_frame(&frame).unwrap();
    }
    println!("✓ {path}");
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "docs".into());
    std::fs::create_dir_all(&dir).unwrap();
    record(&dir, "idle", 50, |_| {});
    record(&dir, "fiesta", 60, |c| c.start_trick(Trick::Fiesta));
    record(&dir, "jump", 24, |c| c.start_trick(Trick::Jump));
    record(&dir, "glitch", 20, |c| c.start_trick(Trick::Glitch));
    record(&dir, "wave", 24, |c| c.start_trick(Trick::Wave));
    record(&dir, "look", 24, |c| c.start_trick(Trick::Look));
    record(&dir, "shake", 20, |c| c.start_trick(Trick::Shake));
    record(&dir, "sleep", 40, |c| c.go_to_sleep());
}
