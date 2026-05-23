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
    pos_x: f32,
    pos_y: f32,
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
            pos_x: 0.0,
            pos_y: 0.0,
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

            self.pump_messages();
            self.process_events(dt);
            self.check_exit_conditions(now);
            self.update(dt);
            self.apply_movement(dt);

            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }

    fn pump_messages(&mut self) {
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        };

        let mut msg = MSG::default();
        unsafe {
            while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
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

    fn process_events(&mut self, dt: f32) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.last_input = Instant::now();
            match event {
                InputEvent::HotkeyTriggered(id) => {
                    println!("Hotkey triggered (ID: {})", id);
                    self.activate_session();
                }
                InputEvent::KeyDown(vk) => {
                    if self.active_window.is_some() {
                        println!("Key down: 0x{:X}", vk);
                        if !self.handle_key_down(vk, dt) {
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
                    self.pos_x = rect.left as f32;
                    self.pos_y = rect.top as f32;
                    self.dpi = Platform::get_dpi_for_window(hwnd);

                    // Note: We should probably keep the base config and apply scaling to a runtime state
                    // For now, let's just log it.
                    println!("Session activated for window: {:?} (DPI: {})", hwnd, self.dpi);

                    let _ = self.overlay.redraw(self.window_rect);
                    self.overlay.show(true);
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

    fn handle_key_down(&mut self, vk: u32, dt: f32) -> bool {
        // Arrow keys: 0x25 (Left), 0x26 (Up), 0x27 (Right), 0x28 (Down)
        let thrust = match vk {
            0x25 => Vector2D { x: -1.0, y: 0.0 },
            0x26 => Vector2D { x: 0.0, y: -1.0 },
            0x27 => Vector2D { x: 1.0, y: 0.0 },
            0x28 => Vector2D { x: 0.0, y: 1.0 },
            0x1B => return false, // ESC
            _ => return true,      // Ignore other keys for now
        };
        self.physics.apply_thrust(thrust, dt);
        true
    }

    fn handle_key_up(&mut self, _vk: u32) {}

    fn update(&mut self, dt: f32) {
        self.physics.update(dt);
    }

    fn apply_movement(&mut self, dt: f32) {
        if let Some(hwnd) = self.active_window {
            if self.physics.velocity.x.abs() > 0.1 || self.physics.velocity.y.abs() > 0.1 {
                self.pos_x += self.physics.velocity.x * dt;
                self.pos_y += self.physics.velocity.y * dt;

                let mut new_rect = self.window_rect;
                let width = new_rect.right - new_rect.left;
                let height = new_rect.bottom - new_rect.top;

                new_rect.left = self.pos_x as i32;
                new_rect.top = self.pos_y as i32;
                new_rect.right = new_rect.left + width;
                new_rect.bottom = new_rect.top + height;

                // Clamp to work area
                self.clamp_to_work_area(hwnd, &mut new_rect);

                if new_rect.left != self.window_rect.left || new_rect.top != self.window_rect.top {
                    // Update our internal floats to match clamped ints if clamping happened
                    self.pos_x = new_rect.left as f32;
                    self.pos_y = new_rect.top as f32;
                    self.window_rect = new_rect;

                    unsafe {
                        let _ = SetWindowPos(
                            hwnd,
                            HWND::default(),
                            new_rect.left,
                            new_rect.top,
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
