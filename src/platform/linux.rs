//! Linux: X11 (Wayland sessions run Pixel through XWayland, see `main`). libX11 / libXtst are loaded at
//! runtime, so the binary has no build-time X11 dependency.

use super::{spawn_detached, which, Rect, WindowKind, CLAUDE_CODE_URL, CLAUDE_CODE_WEB, CLAUDE_WEB};
use claude_companion::hotkey::KeyState;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::{c_int, c_long, c_uchar, c_uint, c_ulong};
use std::process::Command;
use winit::window::Window;
use x11_dl::xlib::{self, Xlib};
use x11_dl::xtest::Xf86vmode as Xtst;

pub const CAPS_IS_TOGGLE: bool = false;

struct X {
    xlib: Xlib,
    xtst: Option<Xtst>,
    dpy: *mut xlib::Display,
    root: xlib::Window,
    alt: Vec<u8>,
    caps: u8,
}

thread_local! {
    static X11: RefCell<Option<X>> = RefCell::new(X::open());
}

impl X {
    fn open() -> Option<X> {
        let xlib = Xlib::open().ok()?;
        let dpy = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
        if dpy.is_null() {
            return None;
        }
        let root = unsafe { (xlib.XDefaultRootWindow)(dpy) };
        let code = |sym: u32| unsafe { (xlib.XKeysymToKeycode)(dpy, sym as c_ulong) };
        let alt = [x11_dl::keysym::XK_Alt_L, x11_dl::keysym::XK_Alt_R].into_iter().map(code).filter(|&c| c != 0).collect();
        let caps = code(x11_dl::keysym::XK_Caps_Lock);
        Some(X { xtst: Xtst::open().ok(), xlib, dpy, root, alt, caps })
    }

    fn atom(&self, name: &str) -> xlib::Atom {
        let name = CString::new(name).unwrap();
        unsafe { (self.xlib.XInternAtom)(self.dpy, name.as_ptr(), xlib::False) }
    }

    /// Root coordinates and button mask.
    fn pointer(&self) -> Option<((f64, f64), c_uint)> {
        let (mut root, mut child) = (0, 0);
        let (mut rx, mut ry, mut wx, mut wy) = (0, 0, 0, 0);
        let mut mask: c_uint = 0;
        let ok = unsafe {
            (self.xlib.XQueryPointer)(self.dpy, self.root, &mut root, &mut child, &mut rx, &mut ry, &mut wx, &mut wy, &mut mask)
        };
        (ok != 0).then_some(((rx as f64, ry as f64), mask))
    }
}

fn with_x<R>(f: impl FnOnce(&X) -> R) -> Option<R> {
    X11.with(|x| x.borrow().as_ref().map(f))
}

pub fn key_state() -> Option<KeyState> {
    with_x(|x| {
        let mut keys = [0 as std::os::raw::c_char; 32];
        unsafe { (x.xlib.XQueryKeymap)(x.dpy, keys.as_mut_ptr()) };
        let down = |code: u8| (keys[code as usize / 8] as u8 >> (code % 8)) & 1 == 1;
        let mut s = KeyState::default();
        for code in 8..=255u8 {
            if !down(code) {
                continue;
            }
            if x.alt.contains(&code) {
                s.alt = true;
            } else if code == x.caps {
                s.caps = true;
            } else {
                s.other = true;
            }
        }
        let buttons = xlib::Button1Mask | xlib::Button2Mask | xlib::Button3Mask | xlib::Button4Mask | xlib::Button5Mask;
        if x.pointer().is_some_and(|(_, mask)| mask & buttons != 0) {
            s.other = true;
            s.mouse = true;
        }
        s
    })
}

pub fn tap_caps_lock(_on: bool) {
    with_x(|x| {
        let Some(xtst) = &x.xtst else {
            eprintln!("claude-companion: libXtst not found, can't press Caps Lock");
            return;
        };
        unsafe {
            (xtst.XTestFakeKeyEvent)(x.dpy, x.caps as c_uint, xlib::True, 0);
            (xtst.XTestFakeKeyEvent)(x.dpy, x.caps as c_uint, xlib::False, 0);
            (x.xlib.XFlush)(x.dpy);
        }
    });
}

pub fn cursor_position() -> Option<(f64, f64)> {
    with_x(|x| x.pointer().map(|(p, _)| p)).flatten()
}

pub fn monitors(event_loop: &winit::event_loop::ActiveEventLoop) -> Vec<Rect> {
    event_loop
        .available_monitors()
        .map(|m| {
            let (p, s) = (m.position(), m.size());
            Rect { x: p.x as f64, y: p.y as f64, w: s.width as f64, h: s.height as f64 }
        })
        .collect()
}

/// The monitor under `point`, minus panels (`_NET_WORKAREA` spans all monitors, so intersect).
pub fn work_area(point: (f64, f64), monitors: &[Rect]) -> Option<Rect> {
    let monitor = *monitors.iter().find(|m| m.contains(point)).or(monitors.first())?;
    let area = with_x(|x| unsafe {
        let (mut ty, mut format, mut n, mut after) = (0, 0 as c_int, 0, 0);
        let mut data: *mut c_uchar = std::ptr::null_mut();
        let ok = (x.xlib.XGetWindowProperty)(
            x.dpy,
            x.root,
            x.atom("_NET_WORKAREA"),
            0,
            4,
            xlib::False,
            xlib::XA_CARDINAL,
            &mut ty,
            &mut format,
            &mut n,
            &mut after,
            &mut data,
        );
        let mut r = None;
        if ok == 0 && !data.is_null() && format == 32 && n >= 4 {
            let v = std::slice::from_raw_parts(data as *const c_long, 4);
            r = Some(Rect { x: v[0] as f64, y: v[1] as f64, w: v[2] as f64, h: v[3] as f64 });
        }
        if !data.is_null() {
            (x.xlib.XFree)(data as *mut _);
        }
        r
    })
    .flatten();
    Some(area.and_then(|a| a.intersect(&monitor)).unwrap_or(monitor))
}

/// Pixel: on every desktop, above other windows, out of the taskbar and pager.
/// Popups are override-redirect (set at creation), nothing to do.
pub fn prepare_window(window: &Window, kind: WindowKind) {
    if kind != WindowKind::Pixel {
        return;
    }
    let Ok(handle) = window.window_handle() else { return };
    let id: xlib::Window = match handle.as_raw() {
        RawWindowHandle::Xlib(h) => h.window,
        RawWindowHandle::Xcb(h) => h.window.get() as xlib::Window,
        _ => return,
    };
    with_x(|x| unsafe {
        let state = x.atom("_NET_WM_STATE");
        let atoms = ["_NET_WM_STATE_STICKY", "_NET_WM_STATE_SKIP_TASKBAR", "_NET_WM_STATE_SKIP_PAGER", "_NET_WM_STATE_ABOVE"]
            .map(|a| x.atom(a));
        // before the window manager picks the window up…
        (x.xlib.XChangeProperty)(
            x.dpy,
            id,
            state,
            xlib::XA_ATOM,
            32,
            xlib::PropModeAppend,
            atoms.as_ptr() as *const c_uchar,
            atoms.len() as c_int,
        );
        let all_desktops: c_long = 0xFFFF_FFFF;
        (x.xlib.XChangeProperty)(
            x.dpy,
            id,
            x.atom("_NET_WM_DESKTOP"),
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            &all_desktops as *const c_long as *const c_uchar,
            1,
        );
        // …and after, for one that already manages it
        for pair in atoms.chunks(2) {
            let mut ev: xlib::XEvent = std::mem::zeroed();
            ev.client_message = xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: xlib::True,
                display: x.dpy,
                window: id,
                message_type: state,
                format: 32,
                data: xlib::ClientMessageData::from([1, pair[0] as c_long, pair[1] as c_long, 1, 0]),
            };
            (x.xlib.XSendEvent)(
                x.dpy,
                x.root,
                xlib::False,
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
                &mut ev,
            );
        }
        (x.xlib.XFlush)(x.dpy);
    });
}

pub fn ensure_accessibility(_prompt: bool) -> bool {
    true
}

// MARK: Claude

/// Community builds of Claude Desktop for Linux install a `claude-desktop` launcher.
fn claude_desktop() -> Option<std::path::PathBuf> {
    which("claude-desktop")
}

/// Is there an app registered for `claude://` links?
fn has_url_handler() -> bool {
    Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/claude"])
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}

fn open_url(url: &str) -> bool {
    spawn_detached(Command::new("xdg-open").arg(url))
}

pub fn claude_running() -> bool {
    let Ok(dir) = std::fs::read_dir("/proc") else { return false };
    dir.flatten().any(|e| {
        std::fs::read(e.path().join("cmdline")).map(|c| String::from_utf8_lossy(&c).contains("claude-desktop")).unwrap_or(false)
    })
}

pub fn open_claude() {
    if let Some(exe) = claude_desktop() {
        spawn_detached(&mut Command::new(exe));
    } else if has_url_handler() {
        open_url("claude://");
    } else {
        open_url(CLAUDE_WEB);
    }
}

pub fn open_claude_code() {
    if has_url_handler() {
        open_url(CLAUDE_CODE_URL);
    } else if let Some(cli) = which("claude") {
        if !open_terminal(&cli) {
            open_url(CLAUDE_CODE_WEB);
        }
    } else {
        open_url(CLAUDE_CODE_WEB);
    }
}

/// Claude Code CLI in a new terminal window, in the home folder.
fn open_terminal(cli: &std::path::Path) -> bool {
    let cli = cli.to_string_lossy().into_owned();
    let mut candidates: Vec<(String, Vec<&str>)> = Vec::new();
    if let Ok(t) = std::env::var("TERMINAL") {
        candidates.push((t, vec!["-e"]));
    }
    for (term, args) in [
        ("x-terminal-emulator", vec!["-e"]),
        ("gnome-terminal", vec!["--"]),
        ("ptyxis", vec!["--"]),
        ("konsole", vec!["-e"]),
        ("xfce4-terminal", vec!["-x"]),
        ("kitty", vec![]),
        ("alacritty", vec!["-e"]),
        ("wezterm", vec!["start", "--"]),
        ("foot", vec![]),
        ("xterm", vec!["-e"]),
    ] {
        candidates.push((term.into(), args));
    }
    let home = dirs::home_dir().unwrap_or_else(|| "/".into());
    candidates.into_iter().any(|(term, args)| {
        which(&term).is_some_and(|path| spawn_detached(Command::new(path).args(args).arg(&cli).current_dir(&home)))
    })
}
