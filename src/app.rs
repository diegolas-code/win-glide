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
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetWindowRect, IsZoomed,
    SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
};

/// The central application controller.
pub struct App {
    /// Physics simulation state for translation.
    physics: PhysicsState,
    /// Physics simulation state for resizing.
    resize_physics: PhysicsState,
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
    /// High-precision width.
    width_f32: f32,
    /// High-precision height.
    height_f32: f32,

    /// DPI of the current monitor (for future scaling support).
    dpi: u32,
    /// The visual overlay (tinted window).
    overlay: Overlay,
    /// The last rect sent to DeferWindowPos (used to skip redundant updates).
    last_sent_rect: RECT,
    /// Set of keys currently held down.
    pressed_keys: HashSet<u32>,
    /// Dynamically detected minimum width constraint of the active target window.
    detected_min_w: Option<f32>,
    /// Dynamically detected minimum height constraint of the active target window.
    detected_min_h: Option<f32>,
    /// The modifier states of Shift and Alt during the last frame tick.
    last_modifiers_state: (bool, bool),
    /// Tracks if a continuous resizing operation is currently in progress.
    is_resizing_in_progress: bool,
    /// Flag to keep the main loop running.
    running: bool,
}

impl App {
    pub fn new(
        event_rx: Receiver<InputEvent>,
        config: &crate::config::Config,
        input_manager: Arc<InputManager>,
    ) -> Self {
        let resize_physics_config = config.resize_physics.unwrap_or(PhysicsConfig {
            acceleration: config.resize_speed * 5.0, // Snappy response
            friction: 15.0,                          // Balanced stop
            thrust_friction: 0.5,
            top_speed: config.resize_speed * 2.5, // Fluid glide speed
        });

        Self {
            physics: PhysicsState::new(config.physics),
            resize_physics: PhysicsState::new(resize_physics_config),
            event_rx,
            input_manager,
            last_update: Instant::now(),
            last_input: Instant::now(),
            active_window: None,
            window_rect: RECT::default(),
            pos_x: 0.0,
            pos_y: 0.0,
            width_f32: 0.0,
            height_f32: 0.0,

            dpi: 96,
            overlay: Overlay::new().expect("Failed to create Overlay"),
            last_sent_rect: RECT::default(),
            pressed_keys: HashSet::new(),
            detected_min_w: None,
            detected_min_h: None,
            last_modifiers_state: (false, false),
            is_resizing_in_progress: false,
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
                if !self.is_resizing_in_progress {
                    self.sync_overlay_to_actual_window();
                }

                // Query current modifier key states
                let is_ctrl_down =
                    unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) } as u16 & 0x8000 != 0;
                let is_lwin_down =
                    unsafe { GetAsyncKeyState(VK_LWIN.0 as i32) } as u16 & 0x8000 != 0;
                let is_rwin_down =
                    unsafe { GetAsyncKeyState(VK_RWIN.0 as i32) } as u16 & 0x8000 != 0;
                let is_win_down = is_lwin_down || is_rwin_down;

                let (is_shift_down, is_alt_down) = if is_ctrl_down || is_win_down {
                    (false, false)
                } else {
                    let shift = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } as u16 & 0x8000 != 0;
                    let alt = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } as u16 & 0x8000 != 0;
                    (shift, alt)
                };

                let current_modifiers = (is_shift_down, is_alt_down);
                if current_modifiers != self.last_modifiers_state {
                    self.last_modifiers_state = current_modifiers;
                    let _ = self
                        .overlay
                        .redraw(self.window_rect, is_shift_down, is_alt_down);
                }

                if self.is_resizing_active() {
                    // Zero out translation velocity to prevent drift during resizing actions
                    self.physics.velocity = Vector2D::default();
                    self.apply_continuous_resize(dt, is_shift_down, is_alt_down);

                    // If velocity decays to zero, commit the ghost bounds
                    if self.is_resizing_in_progress
                        && self.resize_physics.velocity.x == 0.0
                        && self.resize_physics.velocity.y == 0.0
                    {
                        self.commit_ghost_resize();
                    }
                } else {
                    // If transitioning from resizing in progress, commit final bounds
                    if self.is_resizing_in_progress {
                        self.commit_ghost_resize();
                    }

                    // Reset dynamic minimum bounds constraints when resize is inactive
                    if self.detected_min_w.is_some() || self.detected_min_h.is_some() {
                        self.detected_min_w = None;
                        self.detected_min_h = None;
                    }
                    // Reset resize velocity when resize is inactive
                    self.resize_physics.velocity = Vector2D::default();

                    let is_thrusting = self.apply_thrust(dt);
                    self.update(dt, is_thrusting);
                    self.apply_movement(dt);
                }
            }

            // Cap the frame rate.
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                let sleep_dur = frame_duration - elapsed;
                if sleep_dur.as_millis() > 2 {
                    std::thread::sleep(sleep_dur - std::time::Duration::from_millis(1));
                }
                while now.elapsed() < frame_duration {
                    std::thread::yield_now();
                }
            }
        }

        println!("App: Shutdown complete.");
    }

    /// Processes pending messages for the application's own windows.
    fn pump_messages(&mut self) {
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
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
            if let Some(active) = self.active_window
                && current_active != active
                && current_active != self.overlay.hwnd
            {
                println!(
                    "Focus lost, deactivating (Active: {:?}, Overlay: {:?}, Current: {:?})",
                    active, self.overlay.hwnd, current_active
                );
                self.deactivate_session();
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
                    if id == 1337 {
                        self.activate_session();
                        // Force a message pump to ensure the window shows up immediately
                        // without waiting for the next loop iteration.
                        self.pump_messages();
                    } else if id == 1338 {
                        let target_hwnd = if let Some(active) = self.active_window {
                            active
                        } else {
                            get_active_window()
                        };
                        if !target_hwnd.is_invalid() {
                            self.center_window(target_hwnd);
                        }
                    }
                }
                InputEvent::KeyDown(vk) => {
                    if self.active_window.is_some() {
                        match vk {
                            0x25..=0x28 => {
                                // Arrow keys: Left, Up, Right, Down
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

    /// Centers the specified window on the nearest monitor, adjusting size if needed,
    /// and updates physical state if a glide session is active.
    fn center_window(&mut self, hwnd: HWND) {
        unsafe {
            if IsZoomed(hwnd).as_bool() {
                println!("App: Cannot center a maximized window.");
                return;
            }

            if crate::window::is_window_elevated(hwnd) && !Platform::is_admin() {
                println!(
                    "App: Cannot center an elevated window (Access Denied). Run win-glide as Administrator."
                );
                return;
            }

            if crate::window::is_taskbar_or_start_menu(hwnd) {
                return;
            }

            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                let work_area = match Platform::get_nearest_monitor_work_area(hwnd) {
                    Ok(wa) => wa,
                    Err(e) => {
                        eprintln!("App: Failed to query nearest monitor work area: {:?}", e);
                        return;
                    }
                };

                let new_rect = crate::window::calculate_centered_rect(rect, work_area);
                let new_w = new_rect.right - new_rect.left;
                let new_h = new_rect.bottom - new_rect.top;
                let old_w = rect.right - rect.left;
                let old_h = rect.bottom - rect.top;

                let size_changed = new_w != old_w || new_h != old_h;
                let uflags = if size_changed {
                    SWP_NOACTIVATE | SWP_NOZORDER
                } else {
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE
                };

                if self.active_window == Some(hwnd) {
                    // Update internal state
                    self.pos_x = new_rect.left as f32;
                    self.pos_y = new_rect.top as f32;
                    self.width_f32 = new_w as f32;
                    self.height_f32 = new_h as f32;
                    self.window_rect = new_rect;
                    self.physics.velocity = Vector2D::default();

                    // If active, move window and overlay together
                    if let Ok(hdwp) = BeginDeferWindowPos(2) {
                        let mut hdwp = hdwp;
                        if let Ok(h) = DeferWindowPos(
                            hdwp,
                            hwnd,
                            HWND::default(),
                            new_rect.left,
                            new_rect.top,
                            new_w,
                            new_h,
                            uflags | SWP_NOCOPYBITS,
                        ) {
                            hdwp = h;
                        }
                        if let Ok(h) = self.overlay.defer_update_position(hdwp, new_rect) {
                            hdwp = h;
                        }
                        let _ = EndDeferWindowPos(hdwp);
                        self.last_sent_rect = new_rect;
                    }
                } else {
                    // Inactive: just move target window directly
                    let _ = SetWindowPos(
                        hwnd,
                        HWND::default(),
                        new_rect.left,
                        new_rect.top,
                        new_w,
                        new_h,
                        uflags,
                    );
                }
                println!(
                    "App: Centered window to ({}, {}) with size {}x{}",
                    new_rect.left, new_rect.top, new_w, new_h
                );
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
            // Security/Compatibility Check: Skip elevated windows if we are not elevated.
            // UIPI (User Interface Privilege Isolation) prevents a lower-integrity
            // process from interacting with higher-integrity windows.
            if crate::window::is_window_elevated(hwnd) && !Platform::is_admin() {
                println!(
                    "App: Cannot glide an elevated window (Access Denied). Run win-glide as Administrator to enable this."
                );
                return;
            }

            if crate::window::is_taskbar_or_start_menu(hwnd) {
                return;
            }

            unsafe {
                // Do not attempt to move maximized windows (physics wouldn't make sense).
                if IsZoomed(hwnd).as_bool() {
                    println!("App: Skipping activation for maximized window: {:?}", hwnd);
                    return;
                }

                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    println!(
                        "App: Activating session for window: {:?} at position: ({}, {})",
                        hwnd, rect.left, rect.top
                    );
                    self.active_window = Some(hwnd);
                    self.window_rect = rect;
                    // Seed high-precision position with current window position.
                    self.pos_x = rect.left as f32;
                    self.pos_y = rect.top as f32;
                    self.width_f32 = (rect.right - rect.left) as f32;
                    self.height_f32 = (rect.bottom - rect.top) as f32;
                    self.dpi = Platform::get_dpi_for_window(hwnd);
                    self.detected_min_w = None;
                    self.detected_min_h = None;
                    self.last_modifiers_state = (false, false);
                    self.is_resizing_in_progress = false;

                    // Tell the input hooks to start intercepting/modifying input.
                    crate::input::set_session_active(true);

                    // Setup the overlay to "tint" the target window.
                    self.overlay.set_owner(hwnd);
                    let _ = self.overlay.redraw(self.window_rect, false, false);
                    self.last_sent_rect = self.window_rect;
                    self.overlay.show(true);
                    // Force a message pump to ensure the window shows up immediately
                    // without waiting for the next loop iteration.
                    self.pump_messages();
                }
            }
        }
    }

    /// Ends the current movement session.
    fn deactivate_session(&mut self) {
        if self.active_window.is_none() {
            return;
        }
        println!(
            "App: Deactivating session (Final position: {}, {})",
            self.window_rect.left, self.window_rect.top
        );
        crate::input::set_session_active(false);
        self.active_window = None;
        self.physics.velocity = Vector2D::default();
        self.pressed_keys.clear();
        self.overlay.show(false);
        self.detected_min_w = None;
        self.detected_min_h = None;
        self.last_modifiers_state = (false, false);
        self.is_resizing_in_progress = false;
    }

    /// Converts held keys into a thrust vector.
    fn apply_thrust(&mut self, dt: f32) -> bool {
        let mut thrust = Vector2D::default();

        // Arrow keys: 0x25 (Left), 0x26 (Up), 0x27 (Right), 0x28 (Down)
        if self.pressed_keys.contains(&0x25) {
            thrust.x -= 1.0;
        }
        if self.pressed_keys.contains(&0x27) {
            thrust.x += 1.0;
        }
        if self.pressed_keys.contains(&0x26) {
            thrust.y -= 1.0;
        }
        if self.pressed_keys.contains(&0x28) {
            thrust.y += 1.0;
        }

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
            self.pos_x += self.physics.velocity.x * _dt;
            self.pos_y += self.physics.velocity.y * _dt;

            let mut new_rect = self.window_rect;
            let width = new_rect.right - new_rect.left;
            let height = new_rect.bottom - new_rect.top;

            new_rect.left = self.pos_x.round() as i32;
            new_rect.top = self.pos_y.round() as i32;

            // --- Boundary Handling ---
            let vs = Platform::get_virtual_screen_rect();
            let min_visible = 150;

            if new_rect.left < vs.left - width + min_visible {
                new_rect.left = vs.left - width + min_visible;
            } else if new_rect.left > vs.right - min_visible {
                new_rect.left = vs.right - min_visible;
            }

            if new_rect.top < vs.top - height + min_visible {
                new_rect.top = vs.top - height + min_visible;
            } else if new_rect.top > vs.bottom - min_visible {
                new_rect.top = vs.bottom - min_visible;
            }

            new_rect.right = new_rect.left + width;
            new_rect.bottom = new_rect.top + height;

            self.window_rect = new_rect;

            // Optimization: Only call Win32 movement APIs if the integer position has actually changed.
            if new_rect.left != self.last_sent_rect.left
                || new_rect.top != self.last_sent_rect.top
                || new_rect.right != self.last_sent_rect.right
                || new_rect.bottom != self.last_sent_rect.bottom
            {
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

                        // Move overlay to match
                        if let Ok(h) = self.overlay.defer_update_position(hdwp, self.window_rect) {
                            hdwp = h;
                        }

                        let _ = EndDeferWindowPos(hdwp);
                        self.last_sent_rect = new_rect;
                    }
                }
            }
        }
    }

    /// Checks if a resizing modifier is currently active.
    fn is_resizing_active(&mut self) -> bool {
        let is_ctrl_down = unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) } as u16 & 0x8000 != 0;
        let is_lwin_down = unsafe { GetAsyncKeyState(VK_LWIN.0 as i32) } as u16 & 0x8000 != 0;
        let is_rwin_down = unsafe { GetAsyncKeyState(VK_RWIN.0 as i32) } as u16 & 0x8000 != 0;
        if is_ctrl_down || is_lwin_down || is_rwin_down {
            return false;
        }

        let is_shift_down = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } as u16 & 0x8000 != 0;
        let is_alt_down = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } as u16 & 0x8000 != 0;
        is_shift_down || is_alt_down
    }

    /// Applies continuous resizing based on resize physics state.
    fn apply_continuous_resize(&mut self, dt: f32, is_shift_down: bool, is_alt_down: bool) {
        let hwnd = match self.active_window {
            Some(h) => h,
            None => return,
        };

        // 1. Calculate Resize Thrust
        let mut thrust = Vector2D::default();
        if self.pressed_keys.contains(&0x25) {
            thrust.x -= 1.0;
        }
        if self.pressed_keys.contains(&0x27) {
            thrust.x += 1.0;
        }
        if self.pressed_keys.contains(&0x26) {
            thrust.y -= 1.0;
        }
        if self.pressed_keys.contains(&0x28) {
            thrust.y += 1.0;
        }

        let is_thrusting = thrust.x != 0.0 || thrust.y != 0.0;
        if is_thrusting {
            // Normalize diagonal thrust
            let length = (thrust.x.powi(2) + thrust.y.powi(2)).sqrt();
            thrust.x /= length;
            thrust.y /= length;
            self.resize_physics.apply_thrust(thrust, dt);
        }

        // 2. Update Resize Physics State (friction decay)
        self.resize_physics.update(dt, is_thrusting);

        // 3. Compute continuous dx and dy sizing deltas
        let dx = self.resize_physics.velocity.x * dt;
        let dy = self.resize_physics.velocity.y * dt;

        if dx.abs() > 0.01 || dy.abs() > 0.01 {
            self.is_resizing_in_progress = true;

            let work_area = Platform::get_nearest_monitor_work_area(hwnd).unwrap_or_default();
            let vs = Platform::get_virtual_screen_rect();

            let (mut new_x, mut new_y, mut new_w, mut new_h) =
                crate::window::calculate_resized_rect(
                    self.pos_x,
                    self.pos_y,
                    self.width_f32,
                    self.height_f32,
                    is_shift_down,
                    is_alt_down,
                    dx,
                    dy,
                    self.dpi,
                    work_area,
                    vs,
                );

            // Clamp to dynamically detected application sizing limits
            if is_alt_down {
                if self.detected_min_w.is_some_and(|min_w| new_w < min_w) {
                    let min_w = self.detected_min_w.unwrap();
                    new_w = min_w;
                    if dx > 0.0 {
                        new_x = self.pos_x + self.width_f32 - new_w;
                    }
                }
                if self.detected_min_h.is_some_and(|min_h| new_h < min_h) {
                    let min_h = self.detected_min_h.unwrap();
                    new_h = min_h;
                    if dy > 0.0 {
                        new_y = self.pos_y + self.height_f32 - new_h;
                    }
                }
            }

            let new_rect = RECT {
                left: new_x.round() as i32,
                top: new_y.round() as i32,
                right: (new_x + new_w).round() as i32,
                bottom: (new_y + new_h).round() as i32,
            };

            // Pre-render the overlay content
            let prepared_surface =
                self.overlay
                    .prepare_surface(new_rect, is_shift_down, is_alt_down);

            // Apply changes ONLY to overlay window in a single transaction (deleting target DeferWindowPos)
            unsafe {
                if let Ok(hdwp) = BeginDeferWindowPos(1) {
                    let mut hdwp = hdwp;
                    if let Ok(h) = self.overlay.defer_update_position(hdwp, new_rect) {
                        hdwp = h;
                    }
                    let _ = EndDeferWindowPos(hdwp);
                }
            }

            // Update positions
            self.window_rect = new_rect;
            self.pos_x = new_x;
            self.pos_y = new_y;
            self.width_f32 = new_w;
            self.height_f32 = new_h;
            self.last_sent_rect = new_rect;

            // Upload prepared pixels immediately
            if let Some(prepared) = prepared_surface {
                let _ = self.overlay.commit_surface(prepared, new_rect);
            }
        }
    }

    /// Monitors the target window bounds at 120Hz. If the target window could not be resized
    /// to the requested size (due to minimum application bounds limits), it detects the mismatch,
    /// performs a corrective SetWindowPos to undo any position shift, and updates the overlay.
    fn sync_overlay_to_actual_window(&mut self) {
        let hwnd = match self.active_window {
            Some(h) => h,
            None => return,
        };

        let mut actual_rect = RECT::default();
        unsafe {
            if GetWindowRect(hwnd, &mut actual_rect).is_ok() {
                // If the target window size/position differs from what we are tracking:
                if actual_rect.left != self.window_rect.left
                    || actual_rect.top != self.window_rect.top
                    || actual_rect.right != self.window_rect.right
                    || actual_rect.bottom != self.window_rect.bottom
                {
                    let old_rect = self.window_rect;
                    let actual_w = actual_rect.right - actual_rect.left;
                    let actual_h = actual_rect.bottom - actual_rect.top;

                    // Query keyboard state directly to see if Alt is down (shrink mode)
                    let is_alt_down = GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000 != 0;

                    let mut corrected_rect = actual_rect;
                    let mut needs_correction = false;

                    if is_alt_down {
                        // Left/Right/Up/Down Arrow keys pressed?
                        let left_pressed = GetAsyncKeyState(0x25) as u16 & 0x8000 != 0;
                        let up_pressed = GetAsyncKeyState(0x26) as u16 & 0x8000 != 0;
                        let right_pressed = GetAsyncKeyState(0x27) as u16 & 0x8000 != 0;
                        let down_pressed = GetAsyncKeyState(0x28) as u16 & 0x8000 != 0;

                        if left_pressed {
                            // Shrink from Right: left edge should remain stationary at old_rect.left.
                            if actual_rect.left != old_rect.left {
                                corrected_rect.left = old_rect.left;
                                corrected_rect.right = old_rect.left + actual_w;
                                needs_correction = true;
                            }
                        } else if right_pressed {
                            // Shrink from Left: right edge should remain stationary at old_rect.right.
                            let corrected_left = old_rect.right - actual_w;
                            if actual_rect.left != corrected_left {
                                corrected_rect.left = corrected_left;
                                corrected_rect.right = old_rect.right;
                                needs_correction = true;
                            }
                        } else if up_pressed {
                            // Shrink from Bottom: top edge should remain stationary at old_rect.top.
                            if actual_rect.top != old_rect.top {
                                corrected_rect.top = old_rect.top;
                                corrected_rect.bottom = old_rect.top + actual_h;
                                needs_correction = true;
                            }
                        } else if down_pressed {
                            // Shrink from Top: bottom edge should remain stationary at old_rect.bottom.
                            let corrected_top = old_rect.bottom - actual_h;
                            if actual_rect.top != corrected_top {
                                corrected_rect.top = corrected_top;
                                corrected_rect.bottom = old_rect.bottom;
                                needs_correction = true;
                            }
                        }
                    }

                    // Dynamically cache detected minimum bounds constraints if actual size is larger than expected size
                    let expected_w = old_rect.right - old_rect.left;
                    let expected_h = old_rect.bottom - old_rect.top;
                    if actual_w > expected_w {
                        self.detected_min_w = Some(actual_w as f32);
                    }
                    if actual_h > expected_h {
                        self.detected_min_h = Some(actual_h as f32);
                    }

                    if needs_correction {
                        let _ = SetWindowPos(
                            hwnd,
                            HWND::default(),
                            corrected_rect.left,
                            corrected_rect.top,
                            corrected_rect.right - corrected_rect.left,
                            corrected_rect.bottom - corrected_rect.top,
                            SWP_NOACTIVATE | SWP_NOZORDER,
                        );
                        actual_rect = corrected_rect;
                    }

                    self.window_rect = actual_rect;
                    self.pos_x = actual_rect.left as f32;
                    self.pos_y = actual_rect.top as f32;
                    self.width_f32 = (actual_rect.right - actual_rect.left) as f32;
                    self.height_f32 = (actual_rect.bottom - actual_rect.top) as f32;
                    self.last_sent_rect = actual_rect;

                    let size_changed = (actual_rect.right - actual_rect.left)
                        != (old_rect.right - old_rect.left)
                        || (actual_rect.bottom - actual_rect.top)
                            != (old_rect.bottom - old_rect.top);

                    if size_changed {
                        let _ = self.overlay.redraw(actual_rect, false, false);
                    } else {
                        let _ = self.overlay.update_position(actual_rect);
                    }
                }
            }
        }
    }

    /// Commits the final overlay bounds to the physical target window.
    fn commit_ghost_resize(&mut self) {
        let hwnd = match self.active_window {
            Some(h) => h,
            None => return,
        };

        unsafe {
            let _ = SetWindowPos(
                hwnd,
                HWND::default(),
                self.window_rect.left,
                self.window_rect.top,
                self.window_rect.right - self.window_rect.left,
                self.window_rect.bottom - self.window_rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }

        let _ = self.overlay.update_position(self.window_rect);
        self.is_resizing_in_progress = false;
        println!(
            "App: Ghost resize committed (Size: {}x{})",
            self.window_rect.right - self.window_rect.left,
            self.window_rect.bottom - self.window_rect.top
        );
    }
}
