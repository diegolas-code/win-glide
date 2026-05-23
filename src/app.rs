use crate::input::InputEvent;
use crate::physics::{PhysicsConfig, PhysicsState, Vector2D};
use crate::window::get_active_window;
use crossbeam_channel::Receiver;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

pub struct App {
    physics: PhysicsState,
    event_rx: Receiver<InputEvent>,
    last_update: Instant,
    active_window: Option<HWND>,
    window_rect: RECT,
}

impl App {
    pub fn new(event_rx: Receiver<InputEvent>) -> Self {
        Self {
            physics: PhysicsState::new(PhysicsConfig::default()),
            event_rx,
            last_update: Instant::now(),
            active_window: None,
            window_rect: RECT::default(),
        }
    }

    pub fn run(&mut self) {
        let frame_duration = Duration::from_millis(8); // ~120 FPS for smoother movement

        loop {
            let now = Instant::now();
            let dt = now.duration_since(self.last_update).as_secs_f32();
            self.last_update = now;

            self.process_events();
            self.update(dt);
            self.apply_movement(dt);

            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
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
                    self.physics.velocity = Vector2D::default();
                    println!("Session activated for window: {:?}", hwnd);
                }
            }
        }
    }

    fn deactivate_session(&mut self) {
        self.active_window = None;
        self.physics.velocity = Vector2D::default();
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
            _ => return true, // Ignore other keys for now
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
                    self.window_rect.left += dx;
                    self.window_rect.right += dx;
                    self.window_rect.top += dy;
                    self.window_rect.bottom += dy;

                    unsafe {
                        let _ = SetWindowPos(
                            hwnd,
                            HWND::default(),
                            self.window_rect.left,
                            self.window_rect.top,
                            0,
                            0,
                            SWP_NOACTIVATE | SWP_NOZORDER | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE,
                        );
                    }
                }
            }
        }
    }
}
