//! Pixel Clawd: moods, tricks and animation. Knows nothing about windows: the app feeds it ticks and mouse
//! events, it draws itself onto a canvas and asks the app to move the window while walking.

use crate::canvas::{Canvas, Rgba};
use crate::sprite::{self, Eyes, Screen};
use std::time::{Duration, Instant};

pub const PX: f32 = 5.0; // 18 sprite px = 90 pt
pub const SIZE: (f32, f32) = (130.0, 112.0);
pub const TICK: Duration = Duration::from_millis(100);
const IDLE_TIMEOUT: Duration = Duration::from_secs(300); // falls asleep after 5 minutes without attention

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mood {
    Idle,
    Hover,
    Thinking,
    Listening,
    Sleeping,
}

/// Random idle tricks, so Pixel feels alive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Trick {
    Walk,
    Jump,
    Shake,
    Wave,
    Look,
    Glitch,
    Fiesta,
    /// Shadow clone technique: Pixel vanishes in a puff of smoke, five small copies pop out, then merge back.
    Clones,
}

impl Trick {
    /// Everyday tricks; clones are a rare treat on top.
    const ALL: [Trick; 7] = [Trick::Walk, Trick::Jump, Trick::Shake, Trick::Wave, Trick::Look, Trick::Glitch, Trick::Fiesta];
}

/// What the app should do after a tick.
#[derive(Default)]
pub struct Step {
    /// Walking: move the window horizontally by this many points.
    pub walk_dx: Option<f32>,
    /// A walk just ended: remember the new spot.
    pub walk_ended: bool,
}

const GLITCH_COLORS: [Rgba; 2] = [Rgba::rgba(51, 242, 255, 255), Rgba::rgba(255, 64, 191, 255)];
const NOTE_COLOR: Rgba = Rgba::rgba(250, 217, 115, 255);
const ZZZ_COLOR: Rgba = Rgba::rgba(166, 179, 242, 255);
const SMOKE: Rgba = Rgba::rgba(242, 238, 232, 255);
const SMOKE_SHADE: Rgba = Rgba::rgba(184, 178, 172, 255);

// Shadow clones, in ticks from the start of the trick
const CLONES_LENGTH: i32 = 76;
const BIG_POOF: i32 = 4; // Pixel vanishes
const CLONES_IN: i32 = 7; // first clone pops out, the others one tick apart
const CLONES_OUT: i32 = 60; // first clone vanishes
const BIG_BACK: i32 = 66; // Pixel is back
const CLONE_PX: f32 = 2.0;
/// Where the clones stand (sprite top-left): three at the back, two in front.
const CLONE_SPOTS: [(f32, f32); 5] = [(2.0, 44.0), (40.0, 42.0), (78.0, 44.0), (21.0, 64.0), (59.0, 64.0)];

pub struct Companion {
    mood: Mood,
    tick: i32,
    last_interaction: Instant,
    blink_until: i32,
    next_blink: i32,
    think_until: i32,
    trick: Option<Trick>,
    trick_start: i32,
    trick_length: i32,
    walk_dir: f32,
    next_trick: i32,
    /// Being carried: legs run in the air.
    pub dragged: bool,
    rng: Rng,
}

impl Companion {
    pub fn new() -> Self {
        let mut rng = Rng::seeded();
        let next_trick = rng.range(80, 250);
        Companion {
            mood: Mood::Idle,
            tick: 0,
            last_interaction: Instant::now(),
            blink_until: 0,
            next_blink: 30,
            think_until: 0,
            trick: None,
            trick_start: 0,
            trick_length: 0,
            walk_dir: 1.0,
            next_trick,
            dragged: false,
            rng,
        }
    }

    pub fn mood(&self) -> Mood {
        self.mood
    }

    pub fn step(&mut self) -> Step {
        self.tick += 1;
        if self.mood == Mood::Thinking && self.tick >= self.think_until {
            self.mood = Mood::Idle;
        }
        if !matches!(self.mood, Mood::Sleeping | Mood::Thinking | Mood::Listening)
            && self.last_interaction.elapsed() > IDLE_TIMEOUT
        {
            self.mood = Mood::Sleeping;
        }
        let step = self.update_trick();
        if self.tick >= self.next_blink {
            self.blink_until = self.tick + 2;
            self.next_blink = self.tick + self.rng.range(25, 70);
        }
        step
    }

    fn update_trick(&mut self) -> Step {
        let keeps_on_hover =
            self.mood == Mood::Hover && matches!(self.trick, Some(Trick::Fiesta | Trick::Glitch | Trick::Clones));
        if self.mood != Mood::Idle && !keeps_on_hover {
            return Step { walk_dx: None, walk_ended: self.stop_trick() };
        }
        if self.trick.is_none() && self.tick >= self.next_trick {
            let t = if self.rng.range(0, 19) == 0 {
                Trick::Clones
            } else {
                Trick::ALL[self.rng.range(0, Trick::ALL.len() as i32 - 1) as usize]
            };
            self.start_trick(t);
        }
        let Some(trick) = self.trick else { return Step::default() };
        if self.tick - self.trick_start >= self.trick_length {
            return Step { walk_dx: None, walk_ended: self.stop_trick() };
        }
        Step { walk_dx: (trick == Trick::Walk).then_some(self.walk_dir * 3.0), walk_ended: false }
    }

    /// Bumped into the screen edge while walking: turn around. Returns the correction to apply.
    pub fn turn_around(&mut self) -> f32 {
        self.walk_dir = -self.walk_dir;
        self.walk_dir * 6.0
    }

    pub fn start_trick(&mut self, t: Trick) {
        self.trick = Some(t);
        self.trick_start = self.tick;
        self.trick_length = match t {
            Trick::Walk => self.rng.range(15, 40),
            Trick::Glitch => 15,
            Trick::Fiesta => 60,
            Trick::Clones => CLONES_LENGTH,
            _ => 12,
        };
        self.walk_dir = if self.rng.range(0, 1) == 0 { 1.0 } else { -1.0 };
    }

    /// Returns true if a walk was interrupted (the window moved: save the spot).
    pub fn stop_trick(&mut self) -> bool {
        let Some(t) = self.trick.take() else { return false };
        self.next_trick = self.tick + self.rng.range(80, 250);
        t == Trick::Walk
    }

    /// Sombrero on, a few seconds of guitar.
    pub fn fiesta(&mut self) {
        self.wake();
        self.mood = Mood::Idle;
        self.start_trick(Trick::Fiesta);
    }

    /// Shadow clone technique on demand.
    pub fn clones(&mut self) {
        self.wake();
        self.mood = Mood::Idle;
        self.start_trick(Trick::Clones);
    }

    /// Double click: glitch.
    pub fn glitch(&mut self) {
        self.wake();
        self.mood = Mood::Idle;
        self.start_trick(Trick::Glitch);
    }

    pub fn go_to_sleep(&mut self) {
        self.mood = Mood::Sleeping;
    }

    pub fn wake(&mut self) {
        self.last_interaction = Instant::now();
        if self.mood == Mood::Sleeping {
            self.mood = Mood::Idle;
        }
    }

    pub fn set_hover(&mut self, inside: bool) {
        if inside {
            self.wake();
            if self.mood == Mood::Idle {
                self.mood = Mood::Hover;
            }
        } else if self.mood == Mood::Hover {
            self.mood = Mood::Idle;
        }
    }

    pub fn set_listening(&mut self, on: bool) {
        self.last_interaction = Instant::now();
        self.mood = if on { Mood::Listening } else { Mood::Idle };
    }

    pub fn think(&mut self) {
        self.last_interaction = Instant::now();
        self.mood = Mood::Thinking;
        self.think_until = self.tick + 20;
    }

    // MARK: Drawing

    pub fn draw(&mut self, c: &mut Canvas) {
        c.clear(Rgba::rgba(0, 0, 0, 0));
        let p = PX;
        let tick = self.tick;
        let sleeping = self.mood == Mood::Sleeping;
        // bob by one sprite pixel
        let bob_period = if sleeping { 40.0 } else { 16.0 };
        let bob = if (tick as f64 / bob_period * 2.0 * std::f64::consts::PI).sin() > 0.0 { p } else { 0.0 };
        let t = tick - self.trick_start;
        if self.trick == Some(Trick::Clones) {
            self.draw_clones(c, t, bob);
            return;
        }
        let mut ox: f32 = 10.0;
        let mut oy: f32 = 26.0 + bob;
        match self.trick {
            Some(Trick::Jump) => oy -= ((t as f64 / self.trick_length as f64 * std::f64::consts::PI).sin() * 5.0) as f32 * p,
            Some(Trick::Shake) => ox += if t % 2 == 0 { p } else { -p },
            _ => {}
        }

        let eyes =
            if sleeping || tick < self.blink_until { Eyes::Closed } else { Eyes::Open { look_up: self.mood == Mood::Hover } };
        let screen = match self.mood {
            Mood::Idle if self.trick == Some(Trick::Wave) => Screen::Listening { tick },
            Mood::Idle if self.trick == Some(Trick::Look) => Screen::Thinking { active: (t / 4) % 3 },
            Mood::Idle | Mood::Hover => Screen::Prompt,
            Mood::Thinking => Screen::Thinking { active: (tick / 3) % 3 },
            Mood::Listening => Screen::Listening { tick },
            Mood::Sleeping => Screen::Off,
        };
        let speed = if sleeping {
            0.05
        } else if self.trick == Some(Trick::Walk) {
            0.9
        } else if self.dragged {
            1.6 // carried: legs run in the air
        } else {
            0.18
        };
        let phase = tick as f64 * speed;
        let fiesta = (self.trick == Some(Trick::Fiesta)).then_some(t / 2);
        let grid = sprite::render(phase, eyes, self.mood == Mood::Hover, screen, fiesta);

        let glitching = self.trick == Some(Trick::Glitch);
        for (y, row) in grid.iter().enumerate() {
            // glitch: rows tear sideways, some get a cyan / magenta fringe like broken video
            let torn = glitching && self.rng.range(0, 1) == 0;
            let shift = if torn { self.rng.range(-3, 3) as f32 * p } else { 0.0 };
            let fringe = (torn && self.rng.range(0, 2) == 0).then(|| GLITCH_COLORS[self.rng.range(0, 1) as usize]);
            for (x, color) in row.iter().enumerate() {
                let Some(color) = color else { continue };
                c.fill_rect(ox + shift + x as f32 * p, oy + y as f32 * p, p, p, fringe.unwrap_or(*color));
            }
        }

        // notes floating up from the guitar
        if self.trick == Some(Trick::Fiesta) {
            let note = ["..x", "..x", "xxx", "xx."];
            for k in 0..2 {
                let ph = (t + k * 8) % 16;
                let color = NOTE_COLOR.alpha(1.0 - ph as f32 / 16.0);
                let x0 = ox + 18.0 * p + 2.0 + k as f32 * 2.0 * p;
                let y0 = oy + 8.0 * p - ph as f32 * 2.5;
                draw_pattern(c, &note, x0, y0, p * 0.6, color);
            }
        }

        // zzz above the head
        if sleeping {
            let pattern = ["xxxx", "..x.", ".x..", "xxxx"];
            for k in 0..2 {
                let phase = (tick / 2 + k * 10) % 20;
                let x0 = ox + 15.0 * p + phase as f32 * 0.6;
                let y0 = oy - 2.0 - phase as f32 * 0.9;
                let s = if k == 0 { p } else { p * 0.8 };
                draw_pattern(c, &pattern, x0, y0, s, ZZZ_COLOR.alpha(1.0 - phase as f32 / 25.0));
            }
        }
    }

    fn draw_clones(&self, c: &mut Canvas, t: i32, bob: f32) {
        let (ox, oy) = (10.0, 26.0);
        let hover = self.mood == Mood::Hover;
        // the big one: hands up and eyes shut to concentrate, then proud and blushing when back
        if t <= BIG_POOF {
            let grid = sprite::render(0.0, Eyes::Closed, false, Screen::Listening { tick: 3 }, None);
            draw_grid(c, &grid, ox + if t % 2 == 0 { PX } else { 0.0 }, oy, PX);
        } else if t > BIG_BACK {
            let eyes = Eyes::Open { look_up: hover };
            let grid = sprite::render(self.tick as f64 * 0.18, eyes, true, Screen::Prompt, None);
            draw_grid(c, &grid, ox, oy + bob, PX);
        }

        for (i, &(x, y)) in CLONE_SPOTS.iter().enumerate() {
            let k = i as i32;
            let (born, gone) = (CLONES_IN + k, CLONES_OUT + k);
            if t > born && t < gone {
                // each clone has its own pace: hops, blinks and glances out of step with the others
                let life = t - born;
                let hop = (life + k * 7) % 22;
                let lift = if hop < 4 { [1.0, 3.0, 3.0, 1.0][hop as usize] * CLONE_PX } else { 0.0 };
                let blink = (life + k * 11) % 31 < 2;
                let eyes = if blink { Eyes::Closed } else { Eyes::Open { look_up: hover } };
                let screen = match k % 3 {
                    0 => Screen::Thinking { active: (life / 5 + k) % 3 },
                    1 => Screen::Listening { tick: life + k },
                    _ => Screen::Prompt,
                };
                let phase = self.tick as f64 * (0.5 + k as f64 * 0.12) + k as f64 * 1.3;
                let grid = sprite::render(phase, eyes, hover, screen, None);
                let bob = if (life / 4 + k) % 2 == 0 { CLONE_PX } else { 0.0 };
                draw_grid(c, &grid, x, y - lift + bob, CLONE_PX);
            }
            for start in [born - 1, gone - 1] {
                let s = t - start;
                if (0..6).contains(&s) {
                    let (cx, cy) = (x + 9.0 * CLONE_PX, y + (sprite::TOP as f32 + 5.0) * CLONE_PX);
                    puff(c, cx, cy, 9.0 + s as f32 * 2.0, 1.0 - s as f32 / 6.0, (i as i32 * 7 + start) as u32);
                }
            }
        }

        // the big puff over Pixel's body
        let (cx, cy) = (ox + 9.0 * PX, oy + (sprite::TOP as f32 + 5.0) * PX);
        for start in [BIG_POOF - 1, BIG_BACK - 1] {
            let s = t - start;
            if (0..9).contains(&s) {
                puff(c, cx, cy, 38.0 + s as f32 * 3.0, 1.0 - s as f32 / 9.0, start as u32);
            }
        }
    }
}

fn draw_grid(c: &mut Canvas, grid: &sprite::Grid, ox: f32, oy: f32, p: f32) {
    for (y, row) in grid.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            if let Some(color) = color {
                c.fill_rect(ox + x as f32 * p, oy + y as f32 * p, p, p, *color);
            }
        }
    }
}

/// A pixel-art smoke cloud: a ring of round puffs around a core, light inside with a grey rim.
/// It fades by crumbling: as `fade` drops, more and more cells drop out.
fn puff(c: &mut Canvas, cx: f32, cy: f32, r: f32, fade: f32, seed: u32) {
    let cell = 3.0;
    let mut blobs = vec![(cx, cy, r * 0.7)];
    for k in 0..6 {
        let a = seed as f32 * 0.9 + k as f32 * std::f32::consts::TAU / 6.0;
        blobs.push((cx + a.cos() * r * 0.6, cy + a.sin() * r * 0.45, r * (0.4 + 0.08 * (k % 3) as f32)));
    }
    // how deep a point sits inside the cloud: < 1 inside, < 0.75 in the light core
    let depth = |x: f32, y: f32| {
        blobs.iter().map(|&(bx, by, br)| ((x - bx).powi(2) + (y - by).powi(2)).sqrt() / br).fold(f32::MAX, f32::min)
    };
    let n = (r * 1.3 / cell).ceil() as i32;
    let (gx, gy) = ((cx / cell).round(), (cy / cell).round());
    for j in -n..=n {
        for i in -n..=n {
            let (x, y) = ((gx + i as f32) * cell, (gy + j as f32) * cell);
            let d = depth(x + cell / 2.0, y + cell / 2.0);
            // the rim crumbles first
            if d < 1.0 && noise(i, j, seed) < fade * (1.6 - d) {
                c.fill_rect(x, y, cell, cell, if d < 0.75 { SMOKE } else { SMOKE_SHADE });
            }
        }
    }
}

/// Stable per-cell noise in 0..1.
fn noise(i: i32, j: i32, seed: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(0x9E37_79B1) ^ (j as u32).wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

fn draw_pattern(c: &mut Canvas, pattern: &[&str], x0: f32, y0: f32, s: f32, color: Rgba) {
    for (r, line) in pattern.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == 'x' {
                c.fill_rect(x0 + col as f32 * s, y0 + r as f32 * s, s, s, color);
            }
        }
    }
}

/// xorshift: enough randomness for tricks and glitches, no dependency.
struct Rng(u64);

impl Rng {
    fn seeded() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Rng(nanos | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Inclusive range.
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u64) as i32
    }
}
