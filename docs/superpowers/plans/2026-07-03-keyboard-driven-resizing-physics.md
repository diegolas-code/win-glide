# Continuous Resizing Physics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement smooth, momentum-based window resizing (Shift to expand, Alt to shrink) driven by a dedicated resize physics simulation with corrected edge movement logic.

**Architecture:**
1. Initialize a separate `resize_physics: PhysicsState` in `App` that has its `acceleration` and `top_speed` scaled proportionally to the user's `resize_speed` configuration.
2. Update `calculate_resized_rect` to accept `dx` and `dy` from the resize physics engine, applying the swapped modifier controls (Shift = Expand, Alt = Shrink) and the corrected border shrink rules.
3. Integrate the resize physics updates into `process_resize`, applying continuous thrust and friction, and handling clean velocity handoffs.

**Tech Stack:** Rust (2024 edition), Win32 API (`windows-rs`), `serde`.

## Global Constraints

* Language: Rust (Edition 2024).
* OS APIs: `windows-rs` (Win32).
* Activation: Global Hotkey (`RegisterHotKey`) set to `Ctrl + Alt + F10`.
* Hooks: `WH_KEYBOARD_LL` and `WH_MOUSE_LL` (low-level, non-blocking).
* Movement: `SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`.

---

### Task 1: Add Resizing Physics State to App

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

**Interfaces:**
- Produces: `App::resize_physics: PhysicsState`

- [ ] **Step 1: Write a failing compilation check**

We will add a compilation verification or test that fails if `resize_physics` is not defined in `App`.

- [ ] **Step 2: Run build to verify compilation**

Ensure `App` struct field addition fails compilation first because of missing field `resize_physics`.

- [ ] **Step 3: Write minimal implementation**

Add `resize_physics` field to `App` struct in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
pub struct App {
    physics: PhysicsState,
    /// Physics simulation state for resizing.
    resize_physics: PhysicsState,
    event_rx: Receiver<InputEvent>,
    input_manager: Arc<InputManager>,
    // ...
```

In `App::new`, construct `resize_physics` by scaling the physics config:
```rust
    pub fn new(
        event_rx: Receiver<InputEvent>,
        physics_config: PhysicsConfig,
        resize_speed: f32,
        input_manager: Arc<InputManager>,
    ) -> Self {
        // Scale physics properties proportionally for resizing speed
        let resize_scale = if physics_config.top_speed > 0.0 {
            resize_speed / physics_config.top_speed
        } else {
            1.0
        };

        let resize_physics_config = PhysicsConfig {
            acceleration: physics_config.acceleration * resize_scale,
            friction: physics_config.friction,
            thrust_friction: physics_config.thrust_friction,
            top_speed: resize_speed,
        };

        Self {
            physics: PhysicsState::new(physics_config),
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
            resize_speed,
            dpi: 96,
            overlay: Overlay::new().expect("Failed to create Overlay"),
            last_sent_rect: RECT::default(),
            pressed_keys: HashSet::new(),
            running: true,
        }
    }
```

- [ ] **Step 4: Run build to verify it compiles**

Run: `cargo check`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: initialize scaled resize_physics state in App"
```

---

### Task 2: Refactor Coordinate Calculations to Use Deltas and Swapped Modifiers

**Files:**
- Modify: [src/window.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/window.rs)

**Interfaces:**
- Produces: `pub fn calculate_resized_rect(current_x: f32, current_y: f32, current_w: f32, current_h: f32, is_shift_down: bool, is_alt_down: bool, dx: f32, dy: f32, dpi: u32, work_area: RECT, vs: RECT) -> (f32, f32, f32, f32)`

- [ ] **Step 1: Write the failing test**

Modify the test `test_calculate_resized_rect` in [src/window.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/window.rs) to verify the new physics deltas and border directions:
```rust
    #[test]
    fn test_calculate_resized_rect() {
        let work_area = RECT { left: 0, top: 0, right: 1000, bottom: 1000 };
        let vs = RECT { left: -5000, top: -5000, right: 5000, bottom: 5000 };

        // Test Shift + Right (Expand Right, dx > 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            true, false, // is_shift_down, is_alt_down
            50.0, 0.0, // dx, dy
            96, work_area, vs
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 350.0);

        // Test Shift + Left (Expand Left, dx < 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            true, false,
            -50.0, 0.0,
            96, work_area, vs
        );
        assert_eq!(x, 50.0);
        assert_eq!(w, 350.0);

        // Test Alt + Right (Shrink Left edge rightwards, dx > 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            false, true,
            50.0, 0.0,
            96, work_area, vs
        );
        assert_eq!(x, 150.0);
        assert_eq!(w, 250.0);

        // Test Alt + Left (Shrink Right edge leftwards, dx < 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            false, true,
            -50.0, 0.0,
            96, work_area, vs
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 250.0);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test`
Expected: FAIL due to compilation error in `src/window.rs` (mismatched signatures or test assertion failures).

- [ ] **Step 3: Write minimal implementation**

Refactor `calculate_resized_rect` in [src/window.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/window.rs):
```rust
/// Computes the resized position and dimensions of a window based on modifier states,
/// continuous physics deltas (dx, dy), and DPI, clamping to safety boundaries.
pub fn calculate_resized_rect(
    current_x: f32,
    current_y: f32,
    current_w: f32,
    current_h: f32,
    is_shift_down: bool,
    is_alt_down: bool,
    dx: f32,
    dy: f32,
    dpi: u32,
    work_area: RECT,
    vs: RECT,
) -> (f32, f32, f32, f32) {
    let mut new_x = current_x;
    let mut new_y = current_y;
    let mut new_w = current_w;
    let mut new_h = current_h;

    if is_shift_down && !is_alt_down {
        // Expand (Grow)
        if dx > 0.0 {
            new_w += dx;
        } else if dx < 0.0 {
            new_x += dx;
            new_w -= dx;
        }

        if dy > 0.0 {
            new_h += dy;
        } else if dy < 0.0 {
            new_y += dy;
            new_h -= dy;
        }
    } else if is_alt_down && !is_shift_down {
        // Shrink (Reduce)
        if dx > 0.0 {
            new_x += dx;
            new_w -= dx;
        } else if dx < 0.0 {
            new_w += dx;
        }

        if dy > 0.0 {
            new_y += dy;
            new_h -= dy;
        } else if dy < 0.0 {
            new_h += dy;
        }
    }

    // 1. Minimum Size Floor (DPI scaled)
    let scale_factor = dpi as f32 / 96.0;
    let min_w = 250.0 * scale_factor;
    let min_h = 250.0 * scale_factor;

    if new_w < min_w {
        if is_alt_down && dx > 0.0 {
            // Shrunk from Left, adjust pos_x to preserve right edge
            new_x = current_x + current_w - min_w;
        }
        new_w = min_w;
    }
    if new_h < min_h {
        if is_alt_down && dy > 0.0 {
            // Shrunk from Top, adjust pos_y to preserve bottom edge
            new_y = current_y + current_h - min_h;
        }
        new_h = min_h;
    }

    // 2. Monitor Work Area Boundary (Only for Shift-Expansion)
    if is_shift_down && !is_alt_down {
        if new_x < work_area.left as f32 {
            new_x = work_area.left as f32;
            new_w = (current_x + current_w) - new_x;
        }
        if new_x + new_w > work_area.right as f32 {
            new_w = work_area.right as f32 - new_x;
        }
        if new_y < work_area.top as f32 {
            new_y = work_area.top as f32;
            new_h = (current_y + current_h) - new_y;
        }
        if new_y + new_h > work_area.bottom as f32 {
            new_h = work_area.bottom as f32 - new_y;
        }
    }

    // 3. Off-Screen Parking Constraints (Minimum 150px visible)
    let min_visible = 150.0;
    if new_x < vs.left as f32 - new_w + min_visible {
        new_x = vs.left as f32 - new_w + min_visible;
    } else if new_x > vs.right as f32 - min_visible {
        new_x = vs.right as f32 - min_visible;
    }

    if new_y < vs.top as f32 - new_h + min_visible {
        new_y = vs.top as f32 - new_h + min_visible;
    } else if new_y > vs.bottom as f32 - min_visible {
        new_y = vs.bottom as f32 - min_visible;
    }

    (new_x, new_y, new_w, new_h)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/window.rs
git commit -m "feat: refactor calculate_resized_rect to use continuous deltas and updated bounds checks"
```

---

### Task 3: Integrate Continuous Physics Loop and Modifiers Into App

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

**Interfaces:**
- Consumes: `calculate_resized_rect`, `App::resize_physics`

- [ ] **Step 1: Write a failing compilation check**

We will update `process_resize` to match the new continuous loop design. If we change it, the old step-based call compile will fail first.

- [ ] **Step 2: Run cargo check to verify compile failure**

Verify that compiler flags the old implementation.

- [ ] **Step 3: Write minimal implementation**

Modify `process_resize` in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
    /// Checks for resize modifiers and active arrow keys.
    /// If resizing is active, applies continuous physics thrust and updates window sizes.
    /// Otherwise, zeroes out resize physics velocity.
    fn process_resize(&mut self, dt: f32) -> bool {
        let hwnd = match self.active_window {
            Some(h) => h,
            None => return false,
        };

        // Swapped modifiers: Shift is expand, Alt (Menu) is shrink
        let is_shift_down = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } as u16 & 0x8000 != 0;
        let is_alt_down = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } as u16 & 0x8000 != 0;

        let left_pressed = self.pressed_keys.contains(&0x25);
        let up_pressed = self.pressed_keys.contains(&0x26);
        let right_pressed = self.pressed_keys.contains(&0x27);
        let down_pressed = self.pressed_keys.contains(&0x28);

        let has_arrow_pressed = left_pressed || up_pressed || right_pressed || down_pressed;
        let is_resizing = (is_shift_down || is_alt_down) && has_arrow_pressed;

        if !is_resizing {
            // Reset resize physics velocity immediately when resizing is inactive to prevent sliding
            self.resize_physics.velocity = Vector2D::default();
            return false;
        }

        // Handoff logic: zero out translation velocity immediately when resizing
        self.physics.velocity = Vector2D::default();

        // Calculate continuous thrust vector from arrow keys
        let mut thrust = Vector2D::default();
        if left_pressed {
            thrust.x -= 1.0;
        }
        if right_pressed {
            thrust.x += 1.0;
        }
        if up_pressed {
            thrust.y -= 1.0;
        }
        if down_pressed {
            thrust.y += 1.0;
        }

        if thrust.x != 0.0 || thrust.y != 0.0 {
            // Normalize diagonal thrust
            let length = (thrust.x.powi(2) + thrust.y.powi(2)).sqrt();
            thrust.x /= length;
            thrust.y /= length;

            self.resize_physics.apply_thrust(thrust, dt);
        }

        // Update resize physics (apply friction)
        let is_thrusting = thrust.x != 0.0 || thrust.y != 0.0;
        self.resize_physics.update(dt, is_thrusting);

        let dx = self.resize_physics.velocity.x * dt;
        let dy = self.resize_physics.velocity.y * dt;

        let work_area = Platform::get_nearest_monitor_work_area(hwnd).unwrap_or_default();
        let vs = Platform::get_virtual_screen_rect();

        let (new_x, new_y, new_w, new_h) = crate::window::calculate_resized_rect(
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

        self.pos_x = new_x;
        self.pos_y = new_y;
        self.width_f32 = new_w;
        self.height_f32 = new_h;

        let new_rect = RECT {
            left: new_x.round() as i32,
            top: new_y.round() as i32,
            right: (new_x + new_w).round() as i32,
            bottom: (new_y + new_h).round() as i32,
        };

        let old_rect = self.window_rect;
        self.window_rect = new_rect;

        let width_changed = (new_rect.right - new_rect.left) != (old_rect.right - old_rect.left);
        let height_changed = (new_rect.bottom - new_rect.top) != (old_rect.bottom - old_rect.top);
        let size_changed = width_changed || height_changed;

        if new_rect.left != self.last_sent_rect.left
            || new_rect.top != self.last_sent_rect.top
            || new_rect.right != self.last_sent_rect.right
            || new_rect.bottom != self.last_sent_rect.bottom
        {
            unsafe {
                if let Ok(hdwp) = BeginDeferWindowPos(2) {
                    let mut hdwp = hdwp;

                    // Move/Resize target window
                    if let Ok(h) = DeferWindowPos(
                        hdwp,
                        hwnd,
                        HWND::default(),
                        new_rect.left,
                        new_rect.top,
                        new_rect.right - new_rect.left,
                        new_rect.bottom - new_rect.top,
                        SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOCOPYBITS,
                    ) {
                        hdwp = h;
                    }

                    // Move/Resize overlay
                    if let Ok(h) = self.overlay.defer_update_position(hdwp, self.window_rect) {
                        hdwp = h;
                    }

                    let _ = EndDeferWindowPos(hdwp);
                    self.last_sent_rect = new_rect;
                }
            }

            if size_changed {
                let _ = self.overlay.redraw(self.window_rect);
            }
        }

        true
    }
```

- [ ] **Step 4: Run test to verify it compiles and runs successfully**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: integrate resize physics calculation loop and swapped modifier controls"
```
