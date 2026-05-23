//! Main application state machine and loop.
//! 
//! This module coordinates input events, physics updates, and window movement.
//! It maintains the "active session" state and ensures the overlay stays 
//! synchronized with the target window.

use crate::input::{InputEvent, InputManager};
use crate::physics::{PhysicsConfig, PhysicsState, Vector2D};
use crate::platform::Platform;
use crate::ui::Overlay;
use crate::window::get_active_window;
use crossbeam_channel::Receiver;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetWindowRect, IsZoomed,
    SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSIZE, SWP_NOZORDER,
};

/// The central application controller.
pub struct App {
    /// Physics simulation state.
    physics: PhysicsState,
    /// Channel for receiving events from the input thread.
    event_rx: Receiver<InputEvent>,
    /// Handle to the input manager for coordination (e.g., shutdown).
    input_manager: Arc<InputManager>,
    /// Timestamp of the last frame update.
    last_update: Instant,
    /// Timestamp of the last user input (used for idle timeout).
    last_input: Instant,
    /// The window currently being moved, if any.
    active_window: Option<HWND>,
    /// The current bounding box of the active window.
    window_rect: RECT,
    /// High-precision horizontal position (avoids integer truncation issues).
    pos_x: f32,
    /// High-precision vertical position.
    pos_y: f32,
    /// DPI of the current monitor (for future scaling support).
    dpi: u32,
    /// The visual overlay (tinted window).
    overlay: Overlay,
    /// Set of keys currently held down.
    pressed_keys: HashSet<u32>,
    /// Flag to keep the main loop running.
    running: bool,
}

impl App {
    pub fn new(event_rx: Receiver<InputEvent>, physics_config: PhysicsConfig, input_manager: Arc<InputManager>) -> Self {
        Self {
            physics: PhysicsState::new(physics_config),
            event_rx,
            input_manager,
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

    /// The main application loop.
    /// 
    /// Runs at ~120Hz to provide smooth, high-refresh-rate window movement.
    pub fn run(&mut self) {
        let frame_duration = Duration::from_millis(8); 

        while self.running {
            let now = Instant::now();
            let dt = now.duration_since(self.last_update).as_secs_f32();
            self.last_update = now;

            // Process internal Win32 messages (required for the overlay window).
            self.pump_messages();
            
            // Process events from the low-level input thread.
            self.process_events();
            
            // Handle automatic deactivation (timeout, focus loss).
            self.check_exit_conditions(now);
            
            // If a session is active, perform the physics and movement update.
            if self.active_window.is_some() {
                let is_thrusting = self.apply_thrust(dt);
                self.update(dt, is_thrusting);
                self.apply_movement(dt);
            }

            // Cap the frame rate.
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        
        println!("App: Shutdown complete.");
    }

    /// Processes pending messages for the application's own windows.
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

    /// Automatically stops the session under certain conditions.
    fn check_exit_conditions(&mut self, now: Instant) {
        if self.active_window.is_some() {
            // Idle timeout: stop moving if no keys/mouse events for 5 seconds.
            if now.duration_since(self.last_input) > Duration::from_secs(5) {
                println!("Idle timeout reached");
                self.deactivate_session();
                return;
            }

            // Focus loss: if the user clicks away or switches apps, stop movement.
            // We allow either the target window OR our overlay to be in focus.
            let current_active = get_active_window();
            if let Some(active) = self.active_window {
                if current_active != active && current_active != self.overlay.hwnd {
                    println!("Focus lost, deactivating (Active: {:?}, Overlay: {:?}, Current: {:?})", 
                        active, self.overlay.hwnd, current_active);
                    self.deactivate_session();
                }
            }
        }
    }

    /// Translates raw input events into application state changes.
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
                        match vk {
                            0x25..=0x28 => { // Arrow keys: Left, Up, Right, Down
                                self.pressed_keys.insert(vk);
                            }
                            0x10..=0x12 | 0x5B..=0x5C | 0xA0..=0xA5 => {
                                // Ignore modifiers (Shift, Ctrl, Alt, Win) to avoid
                                // immediate deactivation from hotkey release/repeat.
                            }
                            _ => {
                                // Any other key press acts as a "Stop" command.
                                println!("App: Non-arrow key pressed (0x{:X}), deactivating", vk);
                                self.deactivate_session();
                            }
                        }
                    }
                }
                InputEvent::KeyUp(vk) => {
                    self.pressed_keys.remove(&vk);
                }
                InputEvent::MouseButtonDown => {
                    // Clicking deactivates for safety.
                    println!("Mouse click detected, deactivating");
                    self.deactivate_session();
                }
                InputEvent::Shutdown => {
                    println!("App: Shutdown event received");
                    self.deactivate_session();
                    self.input_manager.request_stop();
                    self.running = false;
                }
            }
        }
    }

    /// Starts a window movement session for the current foreground window.
    fn activate_session(&mut self) {
        if self.active_window.is_some() {
            return;
        }
        let hwnd = get_active_window();
        if !hwnd.is_invalid() {
            unsafe {
                // Do not attempt to move maximized windows (physics wouldn't make sense).
                if IsZoomed(hwnd).as_bool() {
                    println!("App: Skipping activation for maximized window: {:?}", hwnd);
                    return;
                }

                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    println!("App: Activating session for window: {:?}", hwnd);
                    self.active_window = Some(hwnd);
                    self.window_rect = rect;
                    // Seed high-precision position with current window position.
                    self.pos_x = rect.left as f32;
                    self.pos_y = rect.top as f32;
                    self.dpi = Platform::get_dpi_for_window(hwnd);

                    // Tell the input hooks to start intercepting/modifying input.
                    crate::input::set_session_active(true);

                    // Setup the overlay to "tint" the target window.
                    self.overlay.set_owner(hwnd);
                    let _ = self.overlay.redraw(self.window_rect);
                    self.overlay.show(true);
                }
            }
        }
    }

    /// Ends the current movement session.
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

    /// Converts held keys into a thrust vector.
    fn apply_thrust(&mut self, dt: f32) -> bool {
        let mut thrust = Vector2D::default();
        
        // Arrow keys: 0x25 (Left), 0x26 (Up), 0x27 (Right), 0x28 (Down)
        if self.pressed_keys.contains(&0x25) { thrust.x -= 1.0; }
        if self.pressed_keys.contains(&0x27) { thrust.x += 1.0; }
        if self.pressed_keys.contains(&0x26) { thrust.y -= 1.0; }
        if self.pressed_keys.contains(&0x28) { thrust.y += 1.0; }

        if thrust.x != 0.0 || thrust.y != 0.0 {
            // Normalize diagonal thrust to ensure consistent speed.
            let length = (thrust.x.powi(2) + thrust.y.powi(2)).sqrt();
            thrust.x /= length;
            thrust.y /= length;
            
            self.physics.apply_thrust(thrust, dt);
            true
        } else {
            false
        }
    }

    /// Advances the physics simulation by one step.
    fn update(&mut self, dt: f32, is_thrusting: bool) {
        self.physics.update(dt, is_thrusting);
    }

    /// Applies the calculated velocity to the window position.
    /// 
    /// Includes collision detection with the virtual desktop boundaries
    /// to prevent windows from being lost off-screen.
    fn apply_movement(&mut self, _dt: f32) {
        if let Some(hwnd) = self.active_window {
            // Only perform updates if there is significant movement.
            if self.physics.velocity.x.abs() > 0.1 || self.physics.velocity.y.abs() > 0.1 {
                self.pos_x += self.physics.velocity.x * _dt;
                self.pos_y += self.physics.velocity.y * _dt;

                let mut new_rect = self.window_rect;
                let width = new_rect.right - new_rect.left;
                let height = new_rect.bottom - new_rect.top;

                new_rect.left = self.pos_x.round() as i32;
                new_rect.top = self.pos_y.round() as i32;

                // --- Boundary Handling ---
                // Limit off-screen movement: ensure at least 150px of the window 
                // remains visible on the virtual desktop.
                let vs = Platform::get_virtual_screen_rect();
                let min_visible = 150;

                // Clamp horizontal position
                if new_rect.left < vs.left - width + min_visible {
                    new_rect.left = vs.left - width + min_visible;
                } else if new_rect.left > vs.right - min_visible {
                    new_rect.left = vs.right - min_visible;
                }

                // Clamp vertical position
                if new_rect.top < vs.top - height + min_visible {
                    new_rect.top = vs.top - height + min_visible;
                } else if new_rect.top > vs.bottom - min_visible {
                    new_rect.top = vs.bottom - min_visible;
                }

                new_rect.right = new_rect.left + width;
                new_rect.bottom = new_rect.top + height;

                // Sync internal floats with the clamped/rounded integer values
                // to avoid "drift" between the simulation and actual window position.
                self.pos_x = new_rect.left as f32;
                self.pos_y = new_rect.top as f32;
                self.window_rect = new_rect;

                unsafe {
                    // Use BeginDeferWindowPos for atomic movement of multiple windows.
                    // This reduces flickering and ensures the overlay and target window
                    // move in the same screen refresh.
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

                        // Move overlay to match
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
