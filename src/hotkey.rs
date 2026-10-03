//! System-wide double tap of Alt / Option, and Caps Lock presses (to keep Pixel in sync with Claude's voice input).
//! Works on polled keyboard snapshots, so every OS only has to answer "what is held right now".
//! A tap counts only if Alt is pressed and released alone, with no other keys or clicks.

use std::time::{Duration, Instant};

const MAX_TAP: Duration = Duration::from_millis(350);
const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(400);
/// How often to poll: short enough to never miss a quick tap.
pub const POLL: Duration = Duration::from_millis(15);

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct KeyState {
    /// Left or right Alt / Option is held.
    pub alt: bool,
    /// Any other key, modifier or mouse button is held.
    pub other: bool,
    /// Caps Lock: the key itself (Windows, Linux) or the lock flag (macOS, see `CAPS_IS_TOGGLE`).
    pub caps: bool,
    /// A mouse button is held (also sets `other`): closes popups on a click elsewhere.
    pub mouse: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyEvent {
    DoubleAlt,
    CapsLock,
}

pub struct Detector {
    prev: KeyState,
    alt_down_at: Option<Instant>,
    last_tap: Option<Instant>,
    caps_is_toggle: bool,
}

impl Detector {
    /// `caps_is_toggle`: `KeyState::caps` is the lock flag, so any change is a press.
    pub fn new(caps_is_toggle: bool) -> Self {
        Detector { prev: KeyState::default(), alt_down_at: None, last_tap: None, caps_is_toggle }
    }

    pub fn feed(&mut self, cur: KeyState, now: Instant) -> Option<KeyEvent> {
        let prev = std::mem::replace(&mut self.prev, cur);

        let caps_pressed = if self.caps_is_toggle { cur.caps != prev.caps } else { cur.caps && !prev.caps };
        if caps_pressed {
            self.reset();
            return Some(KeyEvent::CapsLock);
        }
        if cur.other {
            self.reset(); // any key or click resets the sequence
            return None;
        }
        if cur.alt && !prev.alt {
            self.alt_down_at = Some(now); // Alt pressed alone
        } else if !cur.alt && prev.alt {
            let Some(down) = self.alt_down_at.take() else { return None };
            if now.duration_since(down) >= MAX_TAP {
                self.reset();
            } else if self.last_tap.is_some_and(|t| now.duration_since(t) < DOUBLE_TAP_WINDOW) {
                self.last_tap = None;
                return Some(KeyEvent::DoubleAlt);
            } else {
                self.last_tap = Some(now);
            }
        }
        None
    }

    fn reset(&mut self) {
        self.alt_down_at = None;
        self.last_tap = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(seq: &[(u64, KeyState)]) -> Vec<KeyEvent> {
        let start = Instant::now();
        let mut d = Detector::new(false);
        seq.iter().filter_map(|(ms, s)| d.feed(*s, start + Duration::from_millis(*ms))).collect()
    }

    const UP: KeyState = KeyState { alt: false, other: false, caps: false, mouse: false };
    const ALT: KeyState = KeyState { alt: true, other: false, caps: false, mouse: false };
    const ALT_X: KeyState = KeyState { alt: true, other: true, caps: false, mouse: false };
    const CAPS: KeyState = KeyState { alt: false, other: false, caps: true, mouse: false };

    #[test]
    fn double_tap() {
        assert_eq!(run(&[(0, ALT), (100, UP), (200, ALT), (300, UP)]), vec![KeyEvent::DoubleAlt]);
    }

    #[test]
    fn too_slow() {
        assert!(run(&[(0, ALT), (100, UP), (700, ALT), (800, UP)]).is_empty());
        assert!(run(&[(0, ALT), (500, UP), (600, ALT), (700, UP)]).is_empty());
    }

    #[test]
    fn combo_is_not_a_tap() {
        assert!(run(&[(0, ALT), (50, ALT_X), (100, ALT), (120, UP), (200, ALT), (300, UP)]).is_empty());
    }

    #[test]
    fn triple_tap_fires_once() {
        let s = [(0, ALT), (80, UP), (160, ALT), (240, UP), (320, ALT), (400, UP)];
        assert_eq!(run(&s), vec![KeyEvent::DoubleAlt]);
    }

    #[test]
    fn caps() {
        assert_eq!(run(&[(0, CAPS), (50, CAPS), (100, UP)]), vec![KeyEvent::CapsLock]);
        let start = Instant::now();
        let mut d = Detector::new(true);
        assert_eq!(d.feed(CAPS, start), Some(KeyEvent::CapsLock));
        assert_eq!(d.feed(UP, start), Some(KeyEvent::CapsLock));
    }
}
