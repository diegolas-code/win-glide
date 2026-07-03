# Keyboard-Driven Window Resizing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement keyboard-driven window resizing in `win-glide` during an active glide session: Alt + Arrow expands the window, and Shift + Arrow shrinks the window.

**Architecture:** 
1. Update config structure and file serialization to read and defaults `resize_speed`.
2. Add high-precision size accumulators (`width_f32`, `height_f32`) to the `App` state.
3. Query modifier keys using `GetAsyncKeyState` in the main loop thread, resetting translation velocity when active.
4. Implement pure coordinate-based resizing math in `src/window.rs` with safety bounds (minimum scaled size, work area growth limits, visibility constraints).
5. Synchronously resize the window and overlay in a single Win32 `BeginDeferWindowPos` transaction, only calling `Overlay::redraw` if the dimensions change.

**Tech Stack:** Rust (2024 edition), Win32 API (`windows-rs`), `serde`, `tiny-skia`.

## Global Constraints

* Language: Rust (Edition 2024).
* OS APIs: `windows-rs` (Win32).
* Activation: Global Hotkey (`RegisterHotKey`) set to `Ctrl + Alt + F10`.
* Hooks: `WH_KEYBOARD_LL` and `WH_MOUSE_LL` (low-level, non-blocking).
* Movement: `SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`.

---

### Task 1: Add `resize_speed` to Configuration

**Files:**
- Modify: [src/config.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/config.rs)
- Modify: [config.json](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/config.json)

**Interfaces:**
- Produces: `Config::resize_speed: f32`

- [ ] **Step 1: Write the failing test**

Add to [src/config.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/config.rs):
```rust
    #[test]
    fn test_config_deserialization_with_resize_speed() {
        let json_data = r#"{
            "physics": {
                "acceleration": 4000.0,
                "friction": 10.0,
                "thrust_friction": 0.5,
                "top_speed": 4000.0
            },
            "resize_speed": 750.0,
            "hotkey": {
                "modifiers": 3,
                "vk": 121
            },
            "center_hotkey": {
                "modifiers": 9,
                "vk": 67
            }
        }"#;
        let config: Result<Config, _> = serde_json::from_str(json_data);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.resize_speed, 750.0);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test config::tests::test_config_deserialization_with_resize_speed`
Expected: FAIL due to missing field `resize_speed` on `Config`.

- [ ] **Step 3: Write minimal implementation**

Modify `Config` struct in [src/config.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/config.rs):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub physics: PhysicsConfig,
    #[serde(default = "default_resize_speed")]
    pub resize_speed: f32,
    pub hotkey: HotkeyConfig,
    pub center_hotkey: HotkeyConfig,
}

fn default_resize_speed() -> f32 {
    600.0
}
```
Update `Config::default()` in [src/config.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/config.rs):
```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            resize_speed: 600.0,
            hotkey: HotkeyConfig {
                modifiers: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
                vk: 0x79,                   // F10
            },
            center_hotkey: HotkeyConfig {
                modifiers: 0x0008 | 0x0001, // MOD_WIN | MOD_ALT
                vk: 0x43,                   // C
            },
        }
    }
}
```

Update [config.json](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/config.json) to include `resize_speed`:
```json
{
  "physics": {
    "acceleration": 4000.0,
    "friction": 10.0,
    "thrust_friction": 0.5,
    "top_speed": 4000.0
  },
  "resize_speed": 600.0,
  "hotkey": {
    "modifiers": 3,
    "vk": 121
  },
  "center_hotkey": {
    "modifiers": 9,
    "vk": 67
  }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: PASS (all tests pass)

- [ ] **Step 5: Commit**

```bash
git add src/config.rs config.json
git commit -m "feat: add resize_speed to configuration struct and file"
```

---

### Task 2: Implement Pure Resizing Coordinate Calculations and Constraints

**Files:**
- Modify: [src/window.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/window.rs)

**Interfaces:**
- Produces: `pub fn calculate_resized_rect(current_x: f32, current_y: f32, current_w: f32, current_h: f32, is_alt_down: bool, is_shift_down: bool, left_pressed: bool, right_pressed: bool, up_pressed: bool, down_pressed: bool, step: f32, dpi: u32, work_area: RECT, vs: RECT) -> (f32, f32, f32, f32)`

- [ ] **Step 1: Write the failing test**

Add to [src/window.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/window.rs)'s `tests` module:
```rust
    #[test]
    fn test_calculate_resized_rect() {
        let work_area = RECT { left: 0, top: 0, right: 1000, bottom: 1000 };
        let vs = RECT { left: -5000, top: -5000, right: 5000, bottom: 5000 };

        // Test Alt + Right (Expand Right)
        let (x, y, w, h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            true, false,
            false, true, false, false,
            50.0, 96, work_area, vs
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 350.0);

        // Test Alt + Left (Expand Left)
        let (x, y, w, h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            true, false,
            true, false, false, false,
            50.0, 96, work_area, vs
        );
        assert_eq!(x, 50.0);
        assert_eq!(w, 350.0);

        // Test Shift + Right (Shrink Right)
        let (x, y, w, h) = calculate_resized_rect(
            100.0, 100.0, 300.0, 300.0,
            false, true,
            false, true, false, false,
            50.0, 96, work_area, vs
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 250.0);

        // Test Shift + Right clamp at Min Size (250px)
        let (x, y, w, h) = calculate_resized_rect(
            100.0, 100.0, 260.0, 300.0,
            false, true,
            false, true, false, false,
            20.0, 96, work_area, vs
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 250.0);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test window::tests::test_calculate_resized_rect`
Expected: FAIL due to missing function `calculate_resized_rect`.

- [ ] **Step 3: Write minimal implementation**

Add `calculate_resized_rect` to [src/window.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/window.rs):
```rust
/// Computes the resized position and dimensions of a window based on modifier states, 
/// arrow keys, step size, and DPI, clamping to safety boundaries (min size, work area limits, visibility).
pub fn calculate_resized_rect(
    current_x: f32,
    current_y: f32,
    current_w: f32,
    current_h: f32,
    is_alt_down: bool,
    is_shift_down: bool,
    left_pressed: bool,
    right_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    step: f32,
    dpi: u32,
    work_area: RECT,
    vs: RECT,
) -> (f32, f32, f32, f32) {
    let mut new_x = current_x;
    let mut new_y = current_y;
    let mut new_w = current_w;
    let mut new_h = current_h;

    if is_alt_down && !is_shift_down {
        if left_pressed {
            new_x -= step;
            new_w += step;
        }
        if right_pressed {
            new_w += step;
        }
        if up_pressed {
            new_y -= step;
            new_h += step;
        }
        if down_pressed {
            new_h += step;
        }
    } else if is_shift_down && !is_alt_down {
        if left_pressed {
            new_x += step;
            new_w -= step;
        }
        if right_pressed {
            new_w -= step;
        }
        if up_pressed {
            new_y += step;
            new_h -= step;
        }
        if down_pressed {
            new_h -= step;
        }
    }

    // 1. Minimum Size Floor (DPI scaled)
    let scale_factor = dpi as f32 / 96.0;
    let min_w = 250.0 * scale_factor;
    let min_h = 250.0 * scale_factor;

    if new_w < min_w {
        if is_shift_down && left_pressed {
            new_x = current_x + current_w - min_w;
        }
        new_w = min_w;
    }
    if new_h < min_h {
        if is_shift_down && up_pressed {
            new_y = current_y + current_h - min_h;
        }
        new_h = min_h;
    }

    // 2. Monitor Work Area Boundary (Only for Alt-Expansion)
    if is_alt_down && !is_shift_down {
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
Expected: PASS (all tests pass)

- [ ] **Step 5: Commit**

```bash
git add src/window.rs
git commit -m "feat: implement pure window resizing coordinate maths and constraints with tests"
```

---

### Task 3: Add High-Precision Size Accumulators and Keyboard Modifiers Checking to App

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

**Interfaces:**
- Consumes: `calculate_resized_rect`
- Produces: `App::width_f32`, `App::height_f32` fields

- [ ] **Step 1: Write the failing test**

*(We will mock or check state initialization, but since `App` has no tests module and handles live HWNDs, we will add the fields to the struct and verify that it compiles first).*

- [ ] **Step 2: Run build to verify compilation**

Ensure `App` fields addition compiles successfully.

- [ ] **Step 3: Write minimal implementation**

Add `width_f32` and `height_f32` fields to `App` struct in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
pub struct App {
    physics: PhysicsState,
    event_rx: Receiver<InputEvent>,
    input_manager: Arc<InputManager>,
    last_update: Instant,
    last_input: Instant,
    active_window: Option<HWND>,
    window_rect: RECT,
    pos_x: f32,
    pos_y: f32,
    width_f32: f32,
    height_f32: f32,
    dpi: u32,
    overlay: Overlay,
    last_sent_rect: RECT,
    pressed_keys: HashSet<u32>,
    running: bool,
}
```
Update `App::new()` in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
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
            width_f32: 0.0,
            height_f32: 0.0,
            dpi: 96,
            overlay: Overlay::new().expect("Failed to create Overlay"),
            last_sent_rect: RECT::default(),
            pressed_keys: HashSet::new(),
            running: true,
        }
```

Update `App::activate_session()` in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
                    self.active_window = Some(hwnd);
                    self.window_rect = rect;
                    self.pos_x = rect.left as f32;
                    self.pos_y = rect.top as f32;
                    self.width_f32 = (rect.right - rect.left) as f32;
                    self.height_f32 = (rect.bottom - rect.top) as f32;
```

Update `App::center_window()` in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
                    self.pos_x = new_rect.left as f32;
                    self.pos_y = new_rect.top as f32;
                    self.width_f32 = (new_rect.right - new_rect.left) as f32;
                    self.height_f32 = (new_rect.bottom - new_rect.top) as f32;
                    self.window_rect = new_rect;
```

- [ ] **Step 4: Run build to verify it passes**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: add high-precision sizing fields to App state"
```

---

### Task 4: Integrate Resizing Logic into App Loop

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

**Interfaces:**
- Consumes: `Config::resize_speed`, `calculate_resized_rect`

- [ ] **Step 1: Write a failing compilation check**

We will implement the resizing check in `App::run` or a helper method `App::apply_resize`. Let's create the code directly.

- [ ] **Step 2: Write minimal implementation**

Add imports to [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_MENU, VK_SHIFT};
```

Update config load reference in `App` if needed, or add `resize_speed` field to `App`. Wait! In `App::new`, we receive `physics_config`, but we don't save `config.resize_speed` directly, or we can add it as `resize_speed: f32` to the `App` struct. Let's add `resize_speed: f32` to `App` struct.
Let's modify `App` in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
    /// Resize speed in pixels per second.
    resize_speed: f32,
```
Update `App::new()`:
```rust
    pub fn new(
        event_rx: Receiver<InputEvent>,
        physics_config: PhysicsConfig,
        resize_speed: f32,
        input_manager: Arc<InputManager>,
    ) -> Self {
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
Update [src/main.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/main.rs#L84):
```rust
    let mut app = App::new(rx, config.physics, config.resize_speed, input_manager);
```

Now, create the resize handling logic. In [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs), update `App::run()`:
```rust
            // If a session is active, perform the physics and movement update.
            if self.active_window.is_some() {
                let is_resizing = self.process_resize(dt);
                if !is_resizing {
                    let is_thrusting = self.apply_thrust(dt);
                    self.update(dt, is_thrusting);
                    self.apply_movement(dt);
                }
            }
```

Add `App::process_resize` to [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
```rust
    /// Checks for resize modifiers and active arrow keys.
    /// If resizing is active, computes the new coordinates, zeros momentum,
    /// performs safety bounds, and updates the window layout in a single transaction.
    fn process_resize(&mut self, dt: f32) -> bool {
        let hwnd = match self.active_window {
            Some(h) => h,
            None => return false,
        };

        let is_alt_down = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } as u16 & 0x8000 != 0;
        let is_shift_down = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } as u16 & 0x8000 != 0;

        let left_pressed = self.pressed_keys.contains(&0x25);
        let up_pressed = self.pressed_keys.contains(&0x26);
        let right_pressed = self.pressed_keys.contains(&0x27);
        let down_pressed = self.pressed_keys.contains(&0x28);

        let has_arrow_pressed = left_pressed || up_pressed || right_pressed || down_pressed;
        let is_resizing = (is_alt_down || is_shift_down) && has_arrow_pressed;

        if !is_resizing {
            return false;
        }

        // Handoff logic: zero out translation velocity immediately when resizing
        self.physics.velocity = Vector2D::default();

        let step = self.resize_speed * dt;
        let work_area = Platform::get_nearest_monitor_work_area(hwnd).unwrap_or_default();
        let vs = Platform::get_virtual_screen_rect();

        let (new_x, new_y, new_w, new_h) = crate::window::calculate_resized_rect(
            self.pos_x,
            self.pos_y,
            self.width_f32,
            self.height_f32,
            is_alt_down,
            is_shift_down,
            left_pressed,
            right_pressed,
            up_pressed,
            down_pressed,
            step,
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

        // Optimization: Only call Win32 movement APIs if the integer position/size changed.
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

                    // Move/Resize overlay to match
                    if let Ok(h) = self.overlay.defer_update_position(hdwp, self.window_rect) {
                        hdwp = h;
                    }

                    let _ = EndDeferWindowPos(hdwp);
                    self.last_sent_rect = new_rect;
                }
            }

            // Redraw overlay bitmap content only when integer size changes
            if size_changed {
                let _ = self.overlay.redraw(self.window_rect);
            }
        }

        true
    }
```

- [ ] **Step 3: Run cargo test to verify everything passes**

Run: `cargo test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: integrate resize checking, velocity handoff, and BeginDeferWindowPos update loop"
```
