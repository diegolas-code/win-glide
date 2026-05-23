use crate::input::InputEvent;
use crate::physics::{PhysicsConfig, PhysicsState, Vector2D};
use crate::platform::Platform;
use crate::ui::Overlay;
use crate::window::get_active_window;
use crossbeam_channel::Receiver;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetWindowRect, SWP_NOACTIVATE,
    SWP_NOCOPYBITS, SWP_NOSIZE, SWP_NOZORDER,
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
    pressed_keys: HashSet<u32>,
    running: bool,
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
            pressed_keys: HashSet::new(),
            running: true,
        }
    }

    pub fn run(&mut self) {
        let frame_duration = Duration::from_millis(8); // ~120 FPS for smoother movement

        while self.running {
            let now = Instant::now();
            let dt = now.duration_since(self.last_update).as_secs_f32();
            self.last_update = now;

            self.pump_messages();
            self.process_events();
            self.check_exit_conditions(now);
            
            if self.active_window.is_some() {
                let is_thrusting = self.apply_thrust(dt);
                self.update(dt, is_thrusting);
                self.apply_movement(dt);
            }

            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        
        println!("App: Graceful shutdown complete.");
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
            // Idle timeout: 5 seconds
            if now.duration_since(self.last_input) > Duration::from_secs(5) {
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
                InputEvent::HotkeyTriggered(id) => {
                    println!("Hotkey triggered (ID: {})", id);
                    self.activate_session();
                }
                InputEvent::KeyDown(vk) => {
                    if self.active_window.is_some() {
                        if vk == 0x1B { // ESC
                            self.deactivate_session();
                        } else {
                            self.pressed_keys.insert(vk);
                        }
                    }
                }
                InputEvent::KeyUp(vk) => {
                    self.pressed_keys.remove(&vk);
                }
                InputEvent::MouseButtonDown => {
                    println!("Mouse click detected, deactivating");
                    self.deactivate_session();
                }
                InputEvent::Shutdown => {
                    println!("App: Shutdown event received");
                    self.deactivate_session();
                    self.running = false;
                }
            }
        }
    }

    fn activate_session(&mut self) {
        if self.active_window.is_some() {
            return;
        }
        let hwnd = get_active_window();
        if !hwnd.is_invalid() {
            let mut rect = RECT::default();
            unsafe {
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    println!("App: Activating session for window: {:?}", hwnd);
                    self.active_window = Some(hwnd);
                    self.window_rect = rect;
                    self.pos_x = rect.left as f32;
                    self.pos_y = rect.top as f32;
                    self.dpi = Platform::get_dpi_for_window(hwnd);

                    crate::input::set_session_active(true);

                    let _ = self.overlay.redraw(self.window_rect);
                    self.overlay.show(true);
                }
            }
        }
    }

    fn deactivate_session(&mut self) {
        if self.active_window.is_none() {
            return;
        }
        println!("App: Deactivating session");
        crate::input::set_session_active(false);
        self.active_window = None;
        self.physics.velocity = Vector2D::default();
        self.pressed_keys.clear();
        self.overlay.show(false);
    }

    fn apply_thrust(&mut self, dt: f32) -> bool {
        let mut thrust = Vector2D::default();
        
        // Arrow keys: 0x25 (Left), 0x26 (Up), 0x27 (Right), 0x28 (Down)
        if self.pressed_keys.contains(&0x25) { thrust.x -= 1.0; }
        if self.pressed_keys.contains(&0x27) { thrust.x += 1.0; }
        if self.pressed_keys.contains(&0x26) { thrust.y -= 1.0; }
        if self.pressed_keys.contains(&0x28) { thrust.y += 1.0; }

        if thrust.x != 0.0 || thrust.y != 0.0 {
            // Normalize diagonal thrust
            let length = (thrust.x.powi(2) + thrust.y.powi(2)).sqrt();
            thrust.x /= length;
            thrust.y /= length;
            
            self.physics.apply_thrust(thrust, dt);
            true
        } else {
            false
        }
    }

    fn update(&mut self, dt: f32, is_thrusting: bool) {
        self.physics.update(dt, is_thrusting);
    }

    fn apply_movement(&mut self, _dt: f32) {
        if let Some(hwnd) = self.active_window {
            if self.physics.velocity.x.abs() > 0.1 || self.physics.velocity.y.abs() > 0.1 {
                self.pos_x += self.physics.velocity.x * _dt;
                self.pos_y += self.physics.velocity.y * _dt;

                let mut new_rect = self.window_rect;
                let width = new_rect.right - new_rect.left;
                let height = new_rect.bottom - new_rect.top;

                new_rect.left = self.pos_x.round() as i32;
                new_rect.top = self.pos_y.round() as i32;

                // Limit off-screen movement: at least 150px must stay visible on the virtual desktop
                let vs = Platform::get_virtual_screen_rect();
                let min_visible = 150;

                // Clamp horizontal
                if new_rect.left < vs.left - width + min_visible {
                    new_rect.left = vs.left - width + min_visible;
                } else if new_rect.left > vs.right - min_visible {
                    new_rect.left = vs.right - min_visible;
                }

                // Clamp vertical
                if new_rect.top < vs.top - height + min_visible {
                    new_rect.top = vs.top - height + min_visible;
                } else if new_rect.top > vs.bottom - min_visible {
                    new_rect.top = vs.bottom - min_visible;
                }

                new_rect.right = new_rect.left + width;
                new_rect.bottom = new_rect.top + height;

                // Update our internal floats to match rounded/clamped ints
                self.pos_x = new_rect.left as f32;
                self.pos_y = new_rect.top as f32;
                self.window_rect = new_rect;

                unsafe {
                    if let Ok(hdwp) = BeginDeferWindowPos(2) {
                        let mut hdwp = hdwp;
                        
                        // Move target window
                        if let Ok(h) = DeferWindowPos(
                            hdwp,
                            hwnd,
                            HWND::default(),
                            new_rect.left,
                            new_rect.top,
                            0,
                            0,
                            SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE | SWP_NOCOPYBITS,
                        ) {
                            hdwp = h;
                        }

                        // Move overlay
                        if let Ok(h) = self.overlay.defer_update_position(hdwp, self.window_rect) {
                            hdwp = h;
                        }

                        let _ = EndDeferWindowPos(hdwp);
                    }
                }
            }
        }
    }
}
