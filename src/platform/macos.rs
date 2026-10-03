//! macOS: AppKit / CoreGraphics. Desktop units are points with a top-left origin on the main screen.

use super::{spawn_detached, which, Rect, WindowKind, CLAUDE_CODE_URL, CLAUDE_CODE_WEB, CLAUDE_WEB};
use claude_companion::hotkey::KeyState;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSRunningApplication, NSScreen, NSView, NSWindowCollectionBehavior, NSWorkspace};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};
use objc2_core_graphics::{
    CGEvent, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation, CGEventType, CGMouseButton,
};
use objc2_foundation::NSString;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::process::Command;
use winit::window::Window;

pub const CLAUDE_BUNDLE_ID: &str = "com.anthropic.claudefordesktop";
/// `KeyState::caps` is the Caps Lock flag here; each press flips it.
pub const CAPS_IS_TOGGLE: bool = true;

const CAPS_LOCK_KEY: u16 = 57; // kVK_CapsLock
const HID: CGEventSourceStateID = CGEventSourceStateID::HIDSystemState;

pub fn key_state() -> Option<KeyState> {
    let flags = CGEventSource::flags_state(HID);
    let mouse = [CGMouseButton::Left, CGMouseButton::Right, CGMouseButton::Center]
        .into_iter()
        .any(|b| CGEventSource::button_state(HID, b));
    let other_mods = flags.intersects(CGEventFlags::MaskShift | CGEventFlags::MaskControl | CGEventFlags::MaskCommand);
    let alt = flags.contains(CGEventFlags::MaskAlternate);
    // Option + a key isn't a tap. Ordinary keys are only scanned while Option is held, to keep polling cheap;
    // virtual key codes 54…63 are modifiers (incl. Caps Lock 57, Option 58 / 61)
    let other_key = alt && (0u16..=127).filter(|k| !(54..=63).contains(k)).any(|k| CGEventSource::key_state(HID, k));
    Some(KeyState { alt, caps: flags.contains(CGEventFlags::MaskAlphaShift), other: mouse || other_mods || other_key, mouse })
}

pub fn tap_caps_lock(on: bool) {
    let source = CGEventSource::new(HID);
    for down in [true, false] {
        if let Some(e) = CGEvent::new_keyboard_event(source.as_deref(), CAPS_LOCK_KEY, down) {
            CGEvent::set_type(Some(&e), CGEventType::FlagsChanged);
            CGEvent::set_flags(Some(&e), if on { CGEventFlags::MaskAlphaShift } else { CGEventFlags::empty() });
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&e));
        }
    }
}

/// Height of the main screen: AppKit's y axis grows up from its bottom edge.
fn main_height(mtm: MainThreadMarker) -> f64 {
    NSScreen::screens(mtm).firstObject().map(|s| s.frame().size.height).unwrap_or(0.0)
}

fn flip(r: objc2_foundation::NSRect, h0: f64) -> Rect {
    Rect { x: r.origin.x, y: h0 - r.origin.y - r.size.height, w: r.size.width, h: r.size.height }
}

pub fn cursor_position() -> Option<(f64, f64)> {
    let mtm = MainThreadMarker::new()?;
    let p = NSEvent::mouseLocation();
    Some((p.x, main_height(mtm) - p.y))
}

pub fn monitors(_event_loop: &winit::event_loop::ActiveEventLoop) -> Vec<Rect> {
    let Some(mtm) = MainThreadMarker::new() else { return Vec::new() };
    let h0 = main_height(mtm);
    NSScreen::screens(mtm).iter().map(|s| flip(s.frame(), h0)).collect()
}

/// Screen under `point` without the menu bar and the Dock.
pub fn work_area(point: (f64, f64), _monitors: &[Rect]) -> Option<Rect> {
    let mtm = MainThreadMarker::new()?;
    let h0 = main_height(mtm);
    let screens = NSScreen::screens(mtm);
    let screen = screens.iter().find(|s| flip(s.frame(), h0).contains(point)).or_else(|| screens.firstObject())?;
    Some(flip(screen.visibleFrame(), h0))
}

/// Float on every Space, also over full-screen apps; no shadow (the sprite has its own outline).
pub fn prepare_window(window: &Window, _kind: WindowKind) {
    let Ok(handle) = window.window_handle() else { return };
    let RawWindowHandle::AppKit(h) = handle.as_raw() else { return };
    let view: &NSView = unsafe { h.ns_view.cast().as_ref() };
    let Some(w) = view.window() else { return };
    w.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    w.setHidesOnDeactivate(false);
    w.setHasShadow(false);
}

/// Accessibility is needed to see global keys and to press Caps Lock for voice.
pub fn ensure_accessibility(prompt: bool) -> bool {
    use objc2_application_services::{kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions};
    let key: &CFString = unsafe { kAXTrustedCheckOptionPrompt };
    let options = CFDictionary::<CFString, CFBoolean>::from_slices(&[key], &[CFBoolean::new(prompt)]);
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) }
}

// MARK: Claude

pub fn claude_running() -> bool {
    let id = NSString::from_str(CLAUDE_BUNDLE_ID);
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&id).count() > 0
}

fn claude_installed() -> bool {
    let id = NSString::from_str(CLAUDE_BUNDLE_ID);
    NSWorkspace::sharedWorkspace().URLForApplicationWithBundleIdentifier(&id).is_some()
}

fn open(arg: &str) -> bool {
    spawn_detached(Command::new("/usr/bin/open").arg(arg))
}

pub fn open_claude() {
    if claude_installed() {
        spawn_detached(Command::new("/usr/bin/open").args(["-b", CLAUDE_BUNDLE_ID]));
    } else {
        open(CLAUDE_WEB);
    }
}

pub fn open_claude_code() {
    if claude_installed() {
        open(CLAUDE_CODE_URL);
    } else if let Some(cli) = which("claude") {
        // no desktop app: the Claude Code CLI in a new Terminal window
        spawn_detached(Command::new("/usr/bin/open").args(["-a", "Terminal"]).arg(cli));
    } else {
        open(CLAUDE_CODE_WEB);
    }
}
