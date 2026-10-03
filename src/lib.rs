//! Claude Companion: a tiny pixel Clawd that lives on your desktop and opens Claude or Claude Code.
//! The library holds everything that doesn't touch the OS (drawing, moods, menus, hotkey logic),
//! so tools and tests can drive the real thing offscreen. The app itself is `main.rs`.

pub mod bubble;
pub mod canvas;
pub mod companion;
pub mod hotkey;
pub mod menu;
pub mod sprite;
