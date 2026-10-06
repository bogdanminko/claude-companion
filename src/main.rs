#![cfg_attr(windows, windows_subsystem = "windows")] // no console window

//! Claude Companion: Pixel on the desktop, the Claude / Claude Code capsule, the right-click menu and the tray.

use claude_companion::bubble::{self, Button};
use claude_companion::canvas::Canvas;
use claude_companion::companion::{self, Companion};
use claude_companion::hotkey::{self, Detector, KeyEvent};
use claude_companion::menu::{self, Action};
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

mod autostart;
#[cfg_attr(target_os = "macos", path = "gfx_mac.rs")]
mod gfx;
mod instance;
mod platform;
mod settings;
mod supervisor;
mod tray;
mod voice;

use platform::{Rect, WindowKind};

#[derive(Debug)]
pub enum UserEvent {
    Action(Action),
}

const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const BUBBLE_HIDE_DELAY: Duration = Duration::from_millis(500);
const MENU_GRACE: Duration = Duration::from_millis(300);
const DRAG_THRESHOLD: f64 = 3.0;

struct Win {
    gl: gfx::GlWin,
    canvas: Canvas,
}

impl Win {
    fn window(&self) -> &Window {
        &self.gl.window
    }

    /// Canvas matching the window's current physical size and scale.
    fn canvas(&mut self) -> &mut Canvas {
        let size = self.gl.window.inner_size();
        let scale = self.gl.window.scale_factor() as f32;
        let (w, h) = (size.width.max(1) as usize, size.height.max(1) as usize);
        if self.canvas.width != w || self.canvas.height != h || self.canvas.scale != scale {
            self.canvas = Canvas::sized(w, h, scale);
        }
        &mut self.canvas
    }
}

struct Press {
    cursor: (f64, f64),
    origin: (f64, f64),
    double: bool,
    dragged: bool,
}

#[derive(Default)]
struct BubbleState {
    visible: bool,
    hover: bool,
    button: Option<Button>,
    cursor: (f32, f32),
    was_hovered: bool,
    left_at: Option<Instant>,
}

#[derive(Default)]
struct MenuState {
    visible: bool,
    item: Option<Action>,
    cursor: (f32, f32),
    opened_at: Option<Instant>,
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    gfx: Option<gfx::Gfx>,
    pixel: Option<Win>,
    bubble: Option<Win>,
    menu: Option<Win>,
    tray: Option<tray::Tray>,
    companion: Companion,
    detector: Detector,
    voice: voice::Voice,
    monitors: Vec<Rect>,
    monitors_at: Option<Instant>,
    next_tick: Instant,
    next_poll: Instant,
    press: Option<Press>,
    last_click: Option<Instant>,
    pixel_hover: bool,
    prev_mouse: bool,
    b: BubbleState,
    m: MenuState,
    autostart: bool,
    summons: instance::SummonWatch,
    ticks: u64,
}

// MARK: Desktop units (see `platform`)

fn du_scale(w: &Window) -> f64 {
    if cfg!(target_os = "macos") {
        1.0
    } else {
        w.scale_factor()
    }
}

fn win_pos(w: &Window) -> (f64, f64) {
    let p = w.outer_position().unwrap_or_default();
    if cfg!(target_os = "macos") {
        let l: LogicalPosition<f64> = p.to_logical(w.scale_factor());
        (l.x, l.y)
    } else {
        (p.x as f64, p.y as f64)
    }
}

fn set_win_pos(w: &Window, p: (f64, f64)) {
    if cfg!(target_os = "macos") {
        w.set_outer_position(LogicalPosition::new(p.0, p.1));
    } else {
        w.set_outer_position(PhysicalPosition::new(p.0.round() as i32, p.1.round() as i32));
    }
}

fn win_rect(w: &Window) -> Rect {
    let (x, y) = win_pos(w);
    let s = w.inner_size();
    let k = if cfg!(target_os = "macos") { w.scale_factor() } else { 1.0 };
    Rect { x, y, w: s.width as f64 / k, h: s.height as f64 / k }
}

fn attributes(kind: WindowKind, size: (f32, f32)) -> WindowAttributes {
    #[allow(unused_mut)]
    let mut a = Window::default_attributes()
        .with_title("Claude Companion")
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_inner_size(LogicalSize::new(size.0, size.1))
        .with_visible(false)
        .with_active(false);
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowAttributesExtWindows;
        a = a.with_skip_taskbar(true);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use winit::platform::x11::{WindowAttributesExtX11, WindowType};
        a = a.with_name("claude-companion", "Claude Companion");
        a = match kind {
            WindowKind::Pixel => a.with_x11_window_type(vec![WindowType::Utility]),
            WindowKind::Popup => a.with_override_redirect(true).with_x11_window_type(vec![WindowType::PopupMenu]),
        };
    }
    let _ = kind;
    a
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let now = Instant::now();
        App {
            proxy,
            gfx: None,
            pixel: None,
            bubble: None,
            menu: None,
            tray: None,
            companion: Companion::new(),
            detector: Detector::new(platform::CAPS_IS_TOGGLE),
            voice: voice::Voice::default(),
            monitors: Vec::new(),
            monitors_at: None,
            next_tick: now,
            next_poll: now,
            press: None,
            last_click: None,
            pixel_hover: false,
            prev_mouse: false,
            b: BubbleState::default(),
            m: MenuState::default(),
            autostart: false,
            summons: instance::SummonWatch::new(),
            ticks: 0,
        }
    }

    fn create_windows(&mut self, el: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let (gfx, pixel) = gfx::Gfx::new(el, attributes(WindowKind::Pixel, companion::SIZE))?;
        let bubble = gfx.add_window(el, attributes(WindowKind::Popup, bubble::SIZE))?;
        let menu = gfx.add_window(el, attributes(WindowKind::Popup, menu::size(1.0)))?;
        let wrap = |gl: gfx::GlWin| Win { gl, canvas: Canvas::sized(1, 1, 1.0) };
        self.pixel = Some(wrap(pixel));
        self.bubble = Some(wrap(bubble));
        self.menu = Some(wrap(menu));
        self.gfx = Some(gfx);
        for w in [&self.bubble, &self.menu].into_iter().flatten() {
            platform::prepare_window(w.window(), WindowKind::Popup);
        }

        self.refresh_monitors(el);
        self.restore_position();
        let pixel = self.pixel.as_ref().unwrap().window();
        pixel.set_visible(true);
        platform::prepare_window(pixel, WindowKind::Pixel);
        self.draw_pixel();
        Ok(())
    }

    fn refresh_monitors(&mut self, el: &ActiveEventLoop) {
        if self.monitors_at.is_some_and(|t| t.elapsed() < Duration::from_secs(2)) {
            return;
        }
        self.monitors = platform::monitors(el);
        self.monitors_at = Some(Instant::now());
    }

    fn pixel_window(&self) -> &Window {
        self.pixel.as_ref().expect("pixel window").window()
    }

    fn work_area(&self, at: (f64, f64)) -> Option<Rect> {
        platform::work_area(at, &self.monitors)
    }

    // MARK: Position

    fn restore_position(&mut self) {
        let w = self.pixel_window();
        let r = win_rect(w);
        if let Some(p) = settings::load_position() {
            let center = (p.0 + r.w / 2.0, p.1 + r.h / 2.0);
            if self.monitors.iter().any(|m| m.contains(center)) {
                set_win_pos(w, p);
                return;
            }
        }
        self.reset_position();
    }

    fn reset_position(&mut self) {
        let w = self.pixel_window();
        let r = win_rect(w);
        let s = du_scale(w);
        let center = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        let Some(work) = self.work_area(center).or_else(|| self.monitors.first().copied()) else { return };
        let p = (work.right() - r.w - 24.0 * s, work.bottom() - r.h - 24.0 * s);
        set_win_pos(w, p);
        settings::save_position(p);
    }

    fn save_position(&self) {
        settings::save_position(win_pos(self.pixel_window()));
    }

    // MARK: Ticks

    fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks % 10 == 0 && self.summons.check() {
            self.show_pixel(); // launched again while running
        }
        let step = self.companion.step();
        if let Some(dx) = step.walk_dx {
            let w = self.pixel_window();
            let s = du_scale(w);
            let r = win_rect(w);
            let mut x = r.x + dx as f64 * s;
            if let Some(work) = self.work_area((r.x + r.w / 2.0, r.y + r.h / 2.0)) {
                if x < work.x || x + r.w > work.right() {
                    x += self.companion.turn_around() as f64 * s; // bump into the edge, turn around
                }
            }
            set_win_pos(self.pixel_window(), (x, r.y));
        }
        if step.walk_ended {
            self.save_position();
        }
        self.draw_pixel();
    }

    fn poll(&mut self) {
        let now = Instant::now();
        if let Some(state) = platform::key_state() {
            match self.detector.feed(state, now) {
                Some(KeyEvent::DoubleAlt) => self.summon(),
                Some(KeyEvent::CapsLock) => {
                    if self.voice.caps_pressed() {
                        self.listening_changed();
                    }
                }
                None => {}
            }
            if state.mouse && !self.prev_mouse {
                self.click_anywhere();
            }
            self.prev_mouse = state.mouse;
        }
        if self.press.is_some() {
            self.drag_update(None);
        }

        // capsule: hide shortly after the mouse leaves both Pixel and the capsule
        if self.b.visible {
            if self.pixel_hover || self.b.hover {
                self.b.was_hovered = true;
                self.b.left_at = None;
            } else if self.b.was_hovered {
                let since = *self.b.left_at.get_or_insert(now);
                if now.duration_since(since) >= BUBBLE_HIDE_DELAY {
                    self.hide_bubble();
                }
            }
        }
    }

    /// A click anywhere on the desktop: close popups it missed.
    fn click_anywhere(&mut self) {
        let Some(c) = platform::cursor_position() else { return };
        if self.m.visible && self.m.opened_at.is_some_and(|t| t.elapsed() > MENU_GRACE) {
            if let Some(m) = &self.menu {
                if !win_rect(m.window()).contains(c) {
                    self.close_menu();
                }
            }
        }
        if self.b.visible {
            let in_bubble = self.bubble.as_ref().is_some_and(|b| win_rect(b.window()).contains(c));
            let in_pixel = win_rect(self.pixel_window()).contains(c);
            if !in_bubble && !in_pixel {
                self.hide_bubble();
            }
        }
    }

    // MARK: Drawing

    fn draw_pixel(&mut self) {
        let (Some(gfx), Some(w)) = (&self.gfx, &mut self.pixel) else { return };
        self.companion.draw(w.canvas());
        gfx.present(&mut w.gl, &w.canvas);
    }

    fn draw_bubble(&mut self) {
        let (Some(gfx), Some(w)) = (&self.gfx, &mut self.bubble) else { return };
        bubble::draw(w.canvas(), self.b.button);
        gfx.present(&mut w.gl, &w.canvas);
    }

    fn draw_menu(&mut self) {
        let (Some(gfx), Some(w)) = (&self.gfx, &mut self.menu) else { return };
        menu::draw(w.canvas(), self.m.item, self.voice.listening, self.autostart);
        gfx.present(&mut w.gl, &w.canvas);
    }

    // MARK: Capsule

    fn show_bubble(&mut self) {
        self.b.left_at = None;
        if self.b.visible {
            return;
        }
        let pixel = self.pixel_window();
        let anchor = win_rect(pixel);
        let s = du_scale(pixel);
        let (w, h) = (bubble::SIZE.0 as f64 * s, bubble::SIZE.1 as f64 * s);
        let mut origin = (anchor.x + anchor.w / 2.0 - w / 2.0, anchor.bottom() - 14.0 * s);
        if let Some(work) = self.work_area((anchor.x + anchor.w / 2.0, anchor.y + anchor.h / 2.0)) {
            if origin.1 + h > work.bottom() {
                origin.1 = anchor.y + 10.0 * s - h; // no room below — show above the head
            }
            origin.0 = origin.0.max(work.x + 4.0 * s).min(work.right() - w - 4.0 * s);
        }
        let Some(b) = &self.bubble else { return };
        set_win_pos(b.window(), origin);
        b.window().set_visible(true);
        b.window().set_window_level(WindowLevel::AlwaysOnTop);
        self.b.visible = true;
        self.b.hover = false;
        self.b.button = None;
        self.b.was_hovered = self.pixel_hover;
        self.draw_bubble();
    }

    fn hide_bubble(&mut self) {
        if let Some(b) = &self.bubble {
            b.window().set_visible(false);
        }
        self.b.visible = false;
        self.b.hover = false;
        self.b.left_at = None;
    }

    fn toggle_bubble(&mut self) {
        if self.b.visible {
            self.hide_bubble();
        } else {
            self.show_bubble();
        }
    }

    // MARK: Menu

    fn open_menu(&mut self) {
        let pixel = self.pixel_window();
        let s = du_scale(pixel);
        let scale = pixel.scale_factor() as f32;
        let size = menu::size(scale);
        let cursor = platform::cursor_position().unwrap_or_else(|| {
            let r = win_rect(pixel);
            (r.x + r.w / 2.0, r.y + r.h / 2.0)
        });
        let (w, h) = (size.0 as f64 * s, size.1 as f64 * s);
        let mut origin = cursor;
        if let Some(work) = self.work_area(cursor) {
            if origin.0 + w > work.right() {
                origin.0 = cursor.0 - w;
            }
            if origin.1 + h > work.bottom() {
                origin.1 = cursor.1 - h;
            }
        }
        let Some(m) = &self.menu else { return };
        let _ = m.window().request_inner_size(LogicalSize::new(size.0, size.1));
        set_win_pos(m.window(), origin);
        m.window().set_visible(true);
        m.window().set_window_level(WindowLevel::AlwaysOnTop);
        self.m.visible = true;
        self.m.item = None;
        self.m.opened_at = Some(Instant::now());
        self.draw_menu();
    }

    fn close_menu(&mut self) {
        if let Some(m) = &self.menu {
            m.window().set_visible(false);
        }
        self.m.visible = false;
        self.m.item = None;
    }

    // MARK: Actions

    /// Show Pixel (if hidden) and the capsule.
    fn show_pixel(&mut self) {
        let pixel = self.pixel_window();
        if !pixel.is_visible().unwrap_or(true) {
            pixel.set_visible(true);
            platform::prepare_window(pixel, WindowKind::Pixel);
        }
        self.companion.wake();
        self.show_bubble();
    }

    /// Double Alt / Option: show Pixel (if hidden) and the capsule; again — hide the capsule.
    fn summon(&mut self) {
        let pixel = self.pixel_window();
        if !pixel.is_visible().unwrap_or(true) {
            pixel.set_visible(true);
            platform::prepare_window(pixel, WindowKind::Pixel);
        }
        if self.voice.listening {
            self.toggle_voice();
        } else {
            self.toggle_bubble();
        }
    }

    /// Click on Pixel: while recording voice, stop it; otherwise show the capsule.
    fn handle_click(&mut self) {
        if self.voice.listening {
            self.toggle_voice();
        } else {
            self.toggle_bubble();
        }
    }

    fn toggle_voice(&mut self) {
        if self.voice.toggle() {
            self.listening_changed();
        }
    }

    fn listening_changed(&mut self) {
        self.companion.set_listening(self.voice.listening);
        if let Some(t) = &mut self.tray {
            t.set_listening(self.voice.listening);
        }
        if self.m.visible {
            self.draw_menu();
        }
    }

    fn perform(&mut self, action: Action, el: &ActiveEventLoop) {
        match action {
            Action::Voice => self.toggle_voice(),
            Action::OpenClaude => {
                self.companion.think();
                platform::open_claude();
            }
            Action::OpenCode => {
                self.companion.think();
                platform::open_claude_code();
            }
            Action::Toggle => {
                let pixel = self.pixel_window();
                if pixel.is_visible().unwrap_or(true) {
                    pixel.set_visible(false);
                    self.hide_bubble();
                } else {
                    pixel.set_visible(true);
                    platform::prepare_window(pixel, WindowKind::Pixel);
                }
            }
            Action::Sleep => self.companion.go_to_sleep(),
            Action::Fiesta => self.companion.fiesta(),
            Action::Clones => self.companion.clones(),
            Action::Smoke => self.companion.smoke(),
            Action::ResetPosition => {
                self.monitors_at = None;
                self.refresh_monitors(el);
                self.reset_position();
            }
            Action::Autostart => {
                autostart::set(!self.autostart);
                self.autostart = autostart::is_enabled();
                if let Some(t) = &mut self.tray {
                    t.set_autostart(self.autostart);
                }
            }
            Action::Quit => el.exit(), // exit 0 → the supervisor / launchd won't restart
        }
    }

    // MARK: Mouse on Pixel

    fn cursor_du(&self, local: Option<PhysicalPosition<f64>>) -> Option<(f64, f64)> {
        platform::cursor_position().or_else(|| {
            let w = self.pixel_window();
            let l = local?;
            let (x, y) = win_pos(w);
            let k = if cfg!(target_os = "macos") { w.scale_factor() } else { 1.0 };
            Some((x + l.x / k, y + l.y / k))
        })
    }

    fn drag_update(&mut self, local: Option<PhysicalPosition<f64>>) {
        let Some(cur) = self.cursor_du(local) else { return };
        let s = du_scale(self.pixel_window());
        let Some(p) = &mut self.press else { return };
        let (dx, dy) = (cur.0 - p.cursor.0, cur.1 - p.cursor.1);
        let mut started = false;
        if !p.dragged && dx.abs() + dy.abs() > DRAG_THRESHOLD * s {
            p.dragged = true;
            started = true;
        }
        if !p.dragged {
            return;
        }
        let target = (p.origin.0 + dx, p.origin.1 + dy);
        if started {
            self.companion.dragged = true;
            self.companion.wake();
            self.hide_bubble();
        }
        set_win_pos(self.pixel_window(), target);
    }

    fn pixel_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CursorEntered { .. } => {
                self.pixel_hover = true;
                self.companion.set_hover(true);
                if self.press.is_none() {
                    self.show_bubble();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.pixel_hover = false;
                self.companion.set_hover(false);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.press.is_some() {
                    self.drag_update(Some(position));
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if self.companion.stop_trick() {
                    self.save_position();
                }
                let cursor = self.cursor_du(None).unwrap_or_default();
                self.press = Some(Press {
                    cursor,
                    origin: win_pos(self.pixel_window()),
                    double: self.last_click.is_some_and(|t| t.elapsed() < DOUBLE_CLICK),
                    dragged: false,
                });
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                let Some(p) = self.press.take() else { return };
                self.companion.wake();
                if p.dragged {
                    self.companion.dragged = false;
                    self.save_position();
                } else if p.double {
                    self.last_click = None; // double click: glitch
                    self.companion.glitch();
                } else {
                    self.last_click = Some(Instant::now());
                    self.handle_click();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Right, .. } => {
                self.companion.wake();
                self.hide_bubble();
                self.open_menu();
            }
            WindowEvent::RedrawRequested | WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => self.draw_pixel(),
            _ => {}
        }
    }

    fn bubble_event(&mut self, event: WindowEvent, el: &ActiveEventLoop) {
        match event {
            WindowEvent::CursorEntered { .. } => self.b.hover = true,
            WindowEvent::CursorLeft { .. } => {
                self.b.hover = false;
                self.b.button = None;
                self.draw_bubble();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let k = self.bubble.as_ref().map_or(1.0, |b| b.window().scale_factor());
                self.b.cursor = ((position.x / k) as f32, (position.y / k) as f32);
                self.b.hover = true;
                let hovered = bubble::button_at(self.b.cursor.0, self.b.cursor.1);
                if hovered != self.b.button {
                    self.b.button = hovered;
                    self.draw_bubble();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if let Some(b) = bubble::button_at(self.b.cursor.0, self.b.cursor.1) {
                    self.hide_bubble();
                    self.perform(if b == Button::Claude { Action::OpenClaude } else { Action::OpenCode }, el);
                }
            }
            WindowEvent::RedrawRequested | WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if self.b.visible {
                    self.draw_bubble();
                }
            }
            _ => {}
        }
    }

    fn menu_event(&mut self, event: WindowEvent, el: &ActiveEventLoop) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let k = self.menu.as_ref().map_or(1.0, |m| m.window().scale_factor());
                self.m.cursor = ((position.x / k) as f32, (position.y / k) as f32);
                let item = menu::item_at(self.m.cursor.0, self.m.cursor.1, k as f32);
                if item != self.m.item {
                    self.m.item = item;
                    self.draw_menu();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.m.item = None;
                self.draw_menu();
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left | MouseButton::Right, .. } => {
                let k = self.menu.as_ref().map_or(1.0, |m| m.window().scale_factor()) as f32;
                if let Some(a) = menu::item_at(self.m.cursor.0, self.m.cursor.1, k) {
                    self.close_menu();
                    self.perform(a, el);
                }
            }
            WindowEvent::RedrawRequested | WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if self.m.visible {
                    self.draw_menu();
                }
            }
            _ => {}
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        if let Err(e) = self.create_windows(el) {
            eprintln!("claude-companion: can't create windows: {e}");
            std::process::exit(1);
        }
        autostart::sync();
        self.autostart = autostart::is_enabled();
        self.tray = tray::Tray::new(self.proxy.clone(), self.autostart);
        // Accessibility (macOS) is needed for the global double Option and to press Caps Lock for voice.
        platform::ensure_accessibility(true);
        let now = Instant::now();
        self.next_tick = now + companion::TICK;
        self.next_poll = now + hotkey::POLL;
    }

    fn user_event(&mut self, el: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Action(a) => self.perform(a, el),
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let is = |w: &Option<Win>| w.as_ref().is_some_and(|w| w.window().id() == id);
        if is(&self.pixel) {
            self.pixel_event(event);
        } else if is(&self.bubble) {
            self.bubble_event(event, el);
        } else if is(&self.menu) {
            self.menu_event(event, el);
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if self.gfx.is_none() {
            return;
        }
        while let Some(a) = self.tray.as_ref().and_then(|t| t.poll()) {
            self.perform(a, el);
        }
        let now = Instant::now();
        if now >= self.next_poll {
            self.poll();
            self.next_poll = (self.next_poll + hotkey::POLL).max(now);
        }
        if now >= self.next_tick {
            self.refresh_monitors(el);
            self.tick();
            // after a sleep / suspend, don't try to catch up
            self.next_tick =
                if now - self.next_tick > companion::TICK * 5 { now + companion::TICK } else { self.next_tick + companion::TICK };
        }
        el.set_control_flow(ControlFlow::WaitUntil(self.next_poll.min(self.next_tick)));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--supervise") => supervisor::run(),
        Some("--version" | "-V") => {
            println!("claude-companion {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("--help" | "-h") => {
            println!("claude-companion [--supervise]\n  --supervise  run Pixel and restart it if it crashes (used by autostart)");
            return;
        }
        _ => {}
    }

    let Some(_lock) = instance::acquire() else {
        instance::summon_running(); // already running: show that one
        return;
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    if std::env::var_os("DISPLAY").is_none() {
        eprintln!("claude-companion: needs X11 or XWayland (DISPLAY is not set)");
        std::process::exit(1);
    }

    #[allow(unused_mut)]
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Accessory); // no Dock icon
    }
    let event_loop = builder.build().expect("event loop");
    let mut app = App::new(event_loop.create_proxy());
    event_loop.run_app(&mut app).expect("event loop");
}
