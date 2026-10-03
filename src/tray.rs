//! Tray / menu bar icon with the same actions as the right-click menu.
//! macOS and Windows: native menu (tray-icon). Linux: StatusNotifierItem over D-Bus (ksni) — KDE, Xfce, Cinnamon,
//! GNOME with the AppIndicator extension; without a tray the right-click menu on Pixel still has everything.

use crate::UserEvent;
use claude_companion::menu::{self, Action, Entry};
use claude_companion::sprite;
use winit::event_loop::EventLoopProxy;

/// Clawd at `p` pixels per sprite pixel, centred in a `w`×`h` RGBA image.
pub fn icon_rgba(w: usize, h: usize, p: usize) -> Vec<u8> {
    let grid = sprite::standing();
    let mut img = vec![0u8; w * h * 4];
    let ox = (w - sprite::W * p) / 2;
    let oy = (h - grid.len() * p) / 2;
    for (y, row) in grid.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            let Some(c) = c else { continue };
            for dy in 0..p {
                for dx in 0..p {
                    let i = ((oy + y * p + dy) * w + ox + x * p + dx) * 4;
                    img[i..i + 4].copy_from_slice(&c.0);
                }
            }
        }
    }
    img
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct Tray {
    _icon: tray_icon::TrayIcon,
    voice: tray_icon::menu::MenuItem,
    ids: Vec<(tray_icon::menu::MenuId, Action)>,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl Tray {
    pub fn new(_proxy: EventLoopProxy<UserEvent>) -> Option<Tray> {
        use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
        let menu = Menu::new();
        let mut ids = Vec::new();
        let mut voice = None;
        for e in &menu::ENTRIES {
            match e {
                Entry::Item(a) => {
                    let item = MenuItem::new(menu::title(*a, false), true, None);
                    menu.append(&item).ok()?;
                    ids.push((item.id().clone(), *a));
                    if *a == Action::Voice {
                        voice = Some(item);
                    }
                }
                Entry::Separator => menu.append(&PredefinedMenuItem::separator()).ok()?,
            }
        }
        // menu bar: wide and short, scaled to the bar height; Windows tray: square
        let (w, h) = if cfg!(target_os = "macos") { (40, 22) } else { (36, 36) };
        let icon = tray_icon::Icon::from_rgba(icon_rgba(w, h, 2), w as u32, h as u32).ok()?;
        let tray = tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_tooltip("Claude Companion")
            .build()
            .map_err(|e| eprintln!("claude-companion: no tray icon: {e}"))
            .ok()?;
        Some(Tray { _icon: tray, voice: voice?, ids })
    }

    pub fn set_listening(&mut self, on: bool) {
        self.voice.set_text(menu::title(Action::Voice, on));
    }

    pub fn poll(&self) -> Option<Action> {
        let event = tray_icon::menu::MenuEvent::receiver().try_recv().ok()?;
        self.ids.iter().find(|(id, _)| *id == event.id).map(|(_, a)| *a)
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub struct Tray {
    handle: ksni::blocking::Handle<Sni>,
}

#[cfg(all(unix, not(target_os = "macos")))]
pub struct Sni {
    proxy: EventLoopProxy<UserEvent>,
    listening: bool,
}

#[cfg(all(unix, not(target_os = "macos")))]
impl ksni::Tray for Sni {
    fn id(&self) -> String {
        "claude-companion".into()
    }
    fn title(&self) -> String {
        "Claude Companion".into()
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let (w, h) = (36, 36);
        let mut data = icon_rgba(w, h, 2);
        for px in data.chunks_exact_mut(4) {
            px.rotate_right(1); // RGBA → ARGB
        }
        vec![ksni::Icon { width: w as i32, height: h as i32, data }]
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        menu::ENTRIES
            .iter()
            .map(|e| match e {
                Entry::Item(a) => {
                    let a = *a;
                    ksni::menu::StandardItem {
                        label: menu::title(a, self.listening).into(),
                        activate: Box::new(move |t: &mut Sni| {
                            let _ = t.proxy.send_event(UserEvent::Action(a));
                        }),
                        ..Default::default()
                    }
                    .into()
                }
                Entry::Separator => ksni::MenuItem::Separator,
            })
            .collect()
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
impl Tray {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Option<Tray> {
        use ksni::blocking::TrayMethods;
        let handle =
            Sni { proxy, listening: false }.spawn().map_err(|e| eprintln!("claude-companion: no tray icon: {e}")).ok()?;
        Some(Tray { handle })
    }

    pub fn set_listening(&mut self, on: bool) {
        self.handle.update(|t| t.listening = on);
    }

    pub fn poll(&self) -> Option<Action> {
        None // actions arrive as user events
    }
}
