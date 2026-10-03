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
}

impl Trick {
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
        let keeps_on_hover = self.mood == Mood::Hover && matches!(self.trick, Some(Trick::Fiesta | Trick::Glitch));
        if self.mood != Mood::Idle && !keeps_on_hover {
            return Step { walk_dx: None, walk_ended: self.stop_trick() };
        }
        if self.trick.is_none() && self.tick >= self.next_trick {
            let t = Trick::ALL[self.rng.range(0, Trick::ALL.len() as i32 - 1) as usize];
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
