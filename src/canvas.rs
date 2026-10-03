//! A tiny software canvas: everything Pixel shows is drawn here, then blitted to the window as one texture.
//! Coordinates are logical points; `scale` maps them to physical pixels so pixel art stays crisp on HiDPI.

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const fn hex(rgb: u32) -> Self {
        Rgba([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255])
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Rgba([r, g, b, a])
    }
    pub fn alpha(self, a: f32) -> Self {
        let mut c = self.0;
        c[3] = (c[3] as f32 * a.clamp(0.0, 1.0)).round() as u8;
        Rgba(c)
    }
}

pub struct Canvas {
    pub width: usize, // physical pixels
    pub height: usize,
    pub scale: f32,
    /// Straight (non-premultiplied) RGBA, row-major, top row first.
    pub data: Vec<u8>,
}

impl Canvas {
    /// `size` in logical points.
    pub fn new(size: (f32, f32), scale: f32) -> Self {
        let width = (size.0 * scale).round().max(1.0) as usize;
        let height = (size.1 * scale).round().max(1.0) as usize;
        Self::sized(width, height, scale)
    }

    /// `width`×`height` physical pixels.
    pub fn sized(width: usize, height: usize, scale: f32) -> Self {
        Canvas { width, height, scale, data: vec![0; width * height * 4] }
    }

    pub fn clear(&mut self, c: Rgba) {
        for px in self.data.chunks_exact_mut(4) {
            px.copy_from_slice(&c.0);
        }
    }

    fn blend(&mut self, x: usize, y: usize, c: Rgba, coverage: f32) {
        let i = (y * self.width + x) * 4;
        let a = c.0[3] as f32 / 255.0 * coverage;
        if a <= 0.0 {
            return;
        }
        let d = &mut self.data[i..i + 4];
        let da = d[3] as f32 / 255.0;
        let oa = a + da * (1.0 - a);
        for k in 0..3 {
            let v = (c.0[k] as f32 * a + d[k] as f32 * da * (1.0 - a)) / oa;
            d[k] = v.round() as u8;
        }
        d[3] = (oa * 255.0).round() as u8;
    }

    /// Hard-edged rectangle, edges snapped to physical pixels (no seams between neighbours).
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        let s = self.scale;
        let x0 = ((x * s).round().max(0.0) as usize).min(self.width);
        let y0 = ((y * s).round().max(0.0) as usize).min(self.height);
        let x1 = (((x + w) * s).round().max(0.0) as usize).min(self.width);
        let y1 = (((y + h) * s).round().max(0.0) as usize).min(self.height);
        for py in y0..y1 {
            for px in x0..x1 {
                self.blend(px, py, c, 1.0);
            }
        }
    }

    /// Anti-aliased shape from a signed distance (in logical points, negative inside) over a bounding box.
    pub fn fill_sdf(&mut self, bbox: (f32, f32, f32, f32), c: Rgba, sdf: impl Fn(f32, f32) -> f32) {
        let s = self.scale;
        let (bx, by, bw, bh) = bbox;
        let x0 = ((bx * s).floor().max(0.0) as usize).min(self.width);
        let y0 = ((by * s).floor().max(0.0) as usize).min(self.height);
        let x1 = (((bx + bw) * s).ceil().max(0.0) as usize).min(self.width);
        let y1 = (((by + bh) * s).ceil().max(0.0) as usize).min(self.height);
        for py in y0..y1 {
            for px in x0..x1 {
                let d = sdf((px as f32 + 0.5) / s, (py as f32 + 0.5) / s) * s;
                let coverage = (0.5 - d).clamp(0.0, 1.0);
                if coverage > 0.0 {
                    self.blend(px, py, c, coverage);
                }
            }
        }
    }

    pub fn fill_round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, c: Rgba) {
        self.fill_sdf((x, y, w, h), c, |px, py| round_rect_sdf(px, py, x, y, w, h, r));
    }

    /// One-pixel ring just inside a rounded rect.
    pub fn stroke_round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, width: f32, c: Rgba) {
        self.fill_sdf((x, y, w, h), c, |px, py| {
            let d = round_rect_sdf(px, py, x, y, w, h, r);
            d.max(-(d + width))
        });
    }

    /// Thick line with round caps.
    pub fn line(&mut self, a: (f32, f32), b: (f32, f32), width: f32, c: Rgba) {
        let r = width / 2.0;
        let bbox = (a.0.min(b.0) - r, a.1.min(b.1) - r, (a.0 - b.0).abs() + width, (a.1 - b.1).abs() + width);
        self.fill_sdf(bbox, c, |px, py| segment_distance((px, py), a, b) - r);
    }

    /// 8×8 bitmap text, `size` points per font pixel. Returns the advance width.
    pub fn text(&mut self, x: f32, y: f32, size: f32, s: &str, c: Rgba) -> f32 {
        let mut cx = x;
        for ch in s.chars() {
            if let Some(glyph) = font8x8::legacy::BASIC_LEGACY.get(ch as usize) {
                for (row, bits) in glyph.iter().enumerate() {
                    for col in 0..8 {
                        if bits & (1 << col) != 0 {
                            self.fill_rect(cx + col as f32 * size, y + row as f32 * size, size, size, c);
                        }
                    }
                }
            }
            cx += 8.0 * size;
        }
        cx - x
    }

    /// Alpha of the pixel under a logical point (for click-through checks).
    pub fn alpha_at(&self, x: f32, y: f32) -> u8 {
        let px = (x * self.scale) as isize;
        let py = (y * self.scale) as isize;
        if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {
            return 0;
        }
        self.data[(py as usize * self.width + px as usize) * 4 + 3]
    }
}

pub fn round_rect_sdf(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let qx = (px - cx).abs() - (w / 2.0 - r);
    let qy = (py - cy).abs() - (h / 2.0 - r);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - r
}

fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let (apx, apy) = (p.0 - a.0, p.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 > 0.0 { ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0) } else { 0.0 };
    let (dx, dy) = (apx - abx * t, apy - aby * t);
    (dx * dx + dy * dy).sqrt()
}
