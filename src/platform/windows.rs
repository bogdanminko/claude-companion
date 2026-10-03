//! Windows: Win32. Desktop units are physical pixels of the virtual desktop (winit makes the process
//! per-monitor DPI aware, so cursor, monitor and window coordinates all agree).

use super::{spawn_detached, which, Rect, WindowKind, CLAUDE_CODE_URL, CLAUDE_CODE_WEB, CLAUDE_WEB};
use claude_companion::hotkey::KeyState;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, INVALID_HANDLE_VALUE, POINT};
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, HKEY, HKEY_CLASSES_ROOT, KEY_READ};
use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CAPITAL, VK_LBUTTON, VK_LMENU,
    VK_MBUTTON, VK_MENU, VK_RBUTTON, VK_RMENU, VK_XBUTTON1, VK_XBUTTON2,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GWL_EXSTYLE, SW_SHOWNORMAL, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW};

pub const CAPS_IS_TOGGLE: bool = false;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn down(vk: u16) -> bool {
    unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 }
}

pub fn key_state() -> Option<KeyState> {
    let mouse = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2].into_iter().any(down);
    let alt = down(VK_LMENU) || down(VK_RMENU);
    // Alt + a key isn't a tap. Ordinary keys are only scanned while Alt is held, to keep polling cheap;
    // AltGr arrives as Ctrl + right Alt, so it never counts as a lone Alt
    let other_key = alt && (0x08u16..=0xFE).filter(|&vk| ![VK_MENU, VK_LMENU, VK_RMENU, VK_CAPITAL].contains(&vk)).any(down);
    Some(KeyState { alt, caps: down(VK_CAPITAL), other: mouse || other_key, mouse })
}

pub fn tap_caps_lock(_on: bool) {
    let key = |flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CAPITAL, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
    };
    let inputs = [key(0), key(KEYEVENTF_KEYUP)];
    unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
}

pub fn cursor_position() -> Option<(f64, f64)> {
    let mut p = POINT { x: 0, y: 0 };
    (unsafe { GetCursorPos(&mut p) } != 0).then_some((p.x as f64, p.y as f64))
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

/// Monitor under `point` without the taskbar.
pub fn work_area(point: (f64, f64), _monitors: &[Rect]) -> Option<Rect> {
    unsafe {
        let monitor = MonitorFromPoint(POINT { x: point.0 as i32, y: point.1 as i32 }, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return None;
        }
        let r = info.rcWork;
        Some(Rect { x: r.left as f64, y: r.top as f64, w: (r.right - r.left) as f64, h: (r.bottom - r.top) as f64 })
    }
}

/// Clicks on Pixel must not steal focus from the app you're working in; no Alt+Tab entry.
pub fn prepare_window(window: &Window, _kind: WindowKind) {
    let Ok(handle) = window.window_handle() else { return };
    let RawWindowHandle::Win32(h) = handle.as_raw() else { return };
    let hwnd = h.hwnd.get() as HWND;
    let extra = (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as isize;
    unsafe {
        #[cfg(target_pointer_width = "64")]
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW};
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, GetWindowLongPtrW(hwnd, GWL_EXSTYLE) | extra);
        }
        #[cfg(target_pointer_width = "32")]
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongW, SetWindowLongW};
            SetWindowLongW(hwnd, GWL_EXSTYLE, GetWindowLongW(hwnd, GWL_EXSTYLE) | extra as i32);
        }
    }
}

use winit::window::Window;

pub fn ensure_accessibility(_prompt: bool) -> bool {
    true
}

// MARK: Claude

/// Claude Desktop's per-user install (Squirrel). The Microsoft Store build has no fixed path: reached via `claude://`.
fn claude_exe() -> Option<PathBuf> {
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    [local.join("AnthropicClaude").join("claude.exe"), local.join("Programs").join("Claude").join("Claude.exe")]
        .into_iter()
        .find(|p| p.is_file())
}

fn has_url_handler() -> bool {
    let name = wide("claude");
    let mut key: HKEY = std::ptr::null_mut();
    unsafe {
        let ok = RegOpenKeyExW(HKEY_CLASSES_ROOT, name.as_ptr(), 0, KEY_READ, &mut key) == 0;
        if ok {
            RegCloseKey(key);
        }
        ok
    }
}

fn open_url(url: &str) -> bool {
    let (verb, url) = (wide("open"), wide(url));
    let r = unsafe {
        ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), url.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL)
    };
    r as isize > 32
}

/// A `claude.exe` from the desktop app (the Claude Code CLI is also called claude.exe, so check the folder).
pub fn claude_running() -> bool {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut more = Process32FirstW(snap, &mut entry) != 0;
        while more && !found {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..len]).eq_ignore_ascii_case("claude.exe") {
                found = process_path(entry.th32ProcessID)
                    .is_some_and(|p| p.contains("AnthropicClaude") || p.contains("WindowsApps") || p.contains("\\Claude\\"));
            }
            more = Process32NextW(snap, &mut entry) != 0;
        }
        CloseHandle(snap);
        found
    }
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len) != 0;
        CloseHandle(h);
        ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

pub fn open_claude() {
    if let Some(exe) = claude_exe() {
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
        open_terminal(&cli);
    } else {
        open_url(CLAUDE_CODE_WEB);
    }
}

/// Claude Code CLI in Windows Terminal, or a plain console window, in the home folder.
fn open_terminal(cli: &std::path::Path) {
    let home = dirs::home_dir().unwrap_or_else(|| "C:\\".into());
    if let Some(wt) = which("wt") {
        spawn_detached(Command::new(wt).arg("-d").arg(&home).args(["cmd", "/k"]).arg(cli));
        return;
    }
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    let _ = Command::new("cmd.exe").arg("/k").arg(cli).current_dir(&home).creation_flags(CREATE_NEW_CONSOLE).spawn();
}
