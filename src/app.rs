use crate::input::InputEvent;
use crate::physics::{PhysicsConfig, PhysicsState, Vector2D};
use crate::platform::Platform;
use crate::ui::Overlay;
use crate::window::get_active_window;
use crossbeam_channel::Receiver;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
};

pub struct App {
    physics: PhysicsState,
    event_rx: Receiver<InputEvent>,
    last_update: Instant,
    last_input: Instant,
    active_window: Option<HWND>,
    window_rect: RECT,
    dpi: u32,
    overlay: Overlay,
}

impl App {
    pub fn new(event_rx: Receiver<InputEvent>, physics_config: PhysicsConfig) -> Self {
        Self {
            physics: PhysicsState::new(physics_config),
            event_rx,
            last_update: Instant::now(),
            last_input: Instant::now(),
            active_window: None,
            window_rect: RECT::default(),
            dpi: 96,
            overlay: Overlay::new().expect("Failed to create Overlay"),
        }
    }

    pub fn run(&mut self) {
        let frame_duration = Duration::from_millis(8); // ~120 FPS for smoother movement

        loop {
            let now = Instant::now();
            let dt = now.duration_since(self.last_update).as_secs_f32();
            self.last_update = now;

            self.process_events();
            self.check_exit_conditions(now);
            self.update(dt);
            self.apply_movement(dt);

            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }

    fn check_exit_conditions(&mut self, now: Instant) {
        if self.active_window.is_some() {
            // Idle timeout: 3 seconds
            if now.duration_since(self.last_input) > Duration::from_secs(3) {
                println!("Idle timeout reached");
                self.deactivate_session();
                return;
            }

            // Focus loss
            let current_active = get_active_window();
            if let Some(active) = self.active_window {
                if current_active != active {
                    println!("Focus lost, deactivating");
                    self.deactivate_session();
                }
            }
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.last_input = Instant::now();
            match event {
                InputEvent::HotkeyTriggered(_) => {
                    self.activate_session();
                }
                InputEvent::KeyDown(vk) => {
                    if self.active_window.is_some() {
                        if !self.handle_key_down(vk) {
                            self.deactivate_session();
                        }
                    }
                }
                InputEvent::KeyUp(vk) => {
                    self.handle_key_up(vk);
                }
                InputEvent::MouseMove { dx: _, dy: _ } => {
                    // TODO: Handle mouse handoff
                }
            }
        }
    }

    fn activate_session(&mut self) {
        let hwnd = get_active_window();
        if !hwnd.is_invalid() {
            let mut rect = RECT::default();
            unsafe {
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    self.active_window = Some(hwnd);
                    self.window_rect = rect;
                    self.dpi = Platform::get_dpi_for_window(hwnd);

                    // Scale physics config based on DPI (96 is standard)
                    let scale = self.dpi as f32 / 96.0;
                    self.physics.config.acceleration = 2000.0 * scale;
                    self.physics.config.top_speed = 1500.0 * scale;

                    self.physics.velocity = Vector2D::default();

                    let _ = self.overlay.redraw(self.window_rect);
                    self.overlay.show(true);

                    println!(
                        "Session activated for window: {:?} (DPI: {})",
                        hwnd, self.dpi
                    );
                }
            }
        }
    }

    fn deactivate_session(&mut self) {
        self.active_window = None;
        self.physics.velocity = Vector2D::default();
        self.overlay.show(false);
        println!("Session deactivated");
    }

    fn handle_key_down(&mut self, vk: u32) -> bool {
        // Arrow keys: 0x25 (Left), 0x26 (Up), 0x27 (Right), 0x28 (Down)
        let thrust = match vk {
            0x25 => Vector2D { x: -1.0, y: 0.0 },
            0x26 => Vector2D { x: 0.0, y: -1.0 },
            0x27 => Vector2D { x: 1.0, y: 0.0 },
            0x28 => Vector2D { x: 0.0, y: 1.0 },
            0x1B => return false, // ESC
            _ => return true,      // Ignore other keys for now
        };
        self.physics.apply_thrust(thrust, 0.008);
        true
    }

    fn handle_key_up(&mut self, _vk: u32) {}

    fn update(&mut self, dt: f32) {
        self.physics.update(dt);
    }

    fn apply_movement(&mut self, dt: f32) {
        if let Some(hwnd) = self.active_window {
            if self.physics.velocity.x.abs() > 0.0 || self.physics.velocity.y.abs() > 0.0 {
                let dx = (self.physics.velocity.x * dt) as i32;
                let dy = (self.physics.velocity.y * dt) as i32;

                if dx != 0 || dy != 0 {
                    let mut new_rect = self.window_rect;
                    new_rect.left += dx;
                    new_rect.right += dx;
                    new_rect.top += dy;
                    new_rect.bottom += dy;

                    // Clamp to work area
                    self.clamp_to_work_area(hwnd, &mut new_rect);

                    if new_rect.left != self.window_rect.left || new_rect.top != self.window_rect.top {
                        self.window_rect = new_rect;
                        unsafe {
                            let _ = SetWindowPos(
                                hwnd,
                                HWND::default(),
                                self.window_rect.left,
                                self.window_rect.top,
                                0,
                                0,
                                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE,
                            );
                        }
                        self.overlay.update_position(self.window_rect);
                    }
                }
            }
        }
    }

    fn clamp_to_work_area(&self, hwnd: HWND, rect: &mut RECT) {
        unsafe {
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
                let work = info.rcWork;
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;

                if rect.left < work.left {
                    rect.left = work.left;
                    rect.right = rect.left + width;
                }
                if rect.right > work.right {
                    rect.right = work.right;
                    rect.left = rect.right - width;
                }
                if rect.top < work.top {
                    rect.top = work.top;
                    rect.bottom = rect.top + height;
                }
                if rect.bottom > work.bottom {
                    rect.bottom = work.bottom;
                    rect.top = rect.bottom - height;
                }
            }
        }
    }
}
