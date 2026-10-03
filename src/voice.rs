//! Claude voice input via Caps Lock: press, talk, press again. Pixel flaps its arms meanwhile.

use crate::platform;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Voice {
    pub listening: bool,
    last_synthetic: Option<Instant>,
    /// The physical Caps Lock is tracked only once voice has been used, so plain typing doesn't make Pixel flap.
    tracking: bool,
}

impl Voice {
    /// Returns true if the listening state changed.
    pub fn toggle(&mut self) -> bool {
        if !platform::claude_running() {
            platform::open_claude();
            return false;
        }
        if !platform::ensure_accessibility(true) {
            return false;
        }
        self.tracking = true;
        self.listening = !self.listening;
        self.last_synthetic = Some(Instant::now());
        platform::tap_caps_lock(self.listening);
        true
    }

    /// Caps Lock pressed by hand: keep Pixel in sync with Claude. Returns true if the state changed.
    pub fn caps_pressed(&mut self) -> bool {
        // skip our own press
        if !self.tracking || self.last_synthetic.is_some_and(|t| t.elapsed() < Duration::from_millis(300)) {
            return false;
        }
        self.listening = !self.listening;
        true
    }
}
