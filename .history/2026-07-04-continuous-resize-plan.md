# Continuous Gliding Window Resizing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement continuous momentum-based window resizing via arrow keys with modifiers, leveraging the same physics simulation model used for gliding window translation.

**Architecture:**
* Add `resize_physics: PhysicsState` to the `App` struct.
* Initialize it with a `PhysicsConfig` that has `top_speed = resize_speed` and `acceleration = resize_speed * 1.5` to match translation dynamics.
* Route arrow key down/up events directly to `pressed_keys` to accumulate arrow presses, omitting the discrete resize invocation.
* Inside the 120Hz main loop, check modifier states. If resizing is active, apply continuous thrust to `resize_physics`, update velocity, and calculate real-time resizing deltas (`dx = vx * dt`, `dy = vy * dt`).
* Apply continuous coordinates calculation, transaction update, and split-phase rendering.

**Tech Stack:** Rust (2024), Win32 GDI/DWM APIs, `tiny-skia`.

---

### Task 1: Add and Initialize resize_physics in App

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Add resize_physics field to App struct**
  Locate the `App` struct definition and add `resize_physics`:
  ```rust
      /// Physics simulation state for translation.
      physics: PhysicsState,
      /// Physics simulation state for resizing.
      resize_physics: PhysicsState,
  ```

- [ ] **Step 2: Initialize resize_physics in App::new**
  In the `App::new` constructor, initialize `resize_physics` using a scaled configuration based on `resize_speed`:
  ```rust
          let resize_physics_config = PhysicsConfig {
              acceleration: resize_speed * 1.5,
              friction: 10.0,
              thrust_friction: 0.5,
              top_speed: resize_speed,
          };
  ```
  And instantiate the struct field:
  ```rust
              physics: PhysicsState::new(physics_config),
              resize_physics: PhysicsState::new(resize_physics_config),
  ```

- [ ] **Step 3: Run tests and verify**
  Run: `cargo test`
  Expected: Success

- [ ] **Step 4: Commit**
  ```bash
  git add src/app.rs
  git commit -m "feat: add and initialize resize_physics in App"
  ```

---

### Task 2: Update Input Routing for Resizing Modifiers

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Route arrow keys with modifiers to pressed_keys**
  In `process_events`'s `InputEvent::KeyDown(vk)` branch, insert arrow keys into `pressed_keys` regardless of whether modifiers are down, and remove the discrete resize invocation:
  ```rust
                              0x25..=0x28 => {
                                  // Arrow keys: Left, Up, Right, Down
                                  self.pressed_keys.insert(vk);
                              }
  ```

- [ ] **Step 2: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 3: Commit**
  ```bash
  git add src/app.rs
  git commit -m "feat: route modifier-pressed arrow keys to pressed_keys for continuous tracking"
  ```

---

### Task 3: Implement Continuous Resize Integration in the Frame Loop

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Update main loop resizing check**
  In `App::run`, replace the resizing handling condition with continuous resize integration:
  ```rust
                  if self.is_resizing_active() {
                      // Zero out translation velocity to prevent drift during resizing actions
                      self.physics.velocity = Vector2D::default();
                      self.apply_continuous_resize(dt, is_shift_down, is_alt_down);
                  } else {
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
  ```

- [ ] **Step 2: Add apply_continuous_resize helper method**
  Implement the `apply_continuous_resize` helper method inside `impl App` to compute thrust, decay velocity, integrate resize delta, and reposition both window and overlay:
  ```rust
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
              let work_area = Platform::get_nearest_monitor_work_area(hwnd).unwrap_or_default();
              let vs = Platform::get_virtual_screen_rect();

              let (mut new_x, mut new_y, mut new_w, mut new_h) = crate::window::calculate_resized_rect(
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
              let prepared_surface = self
                  .overlay
                  .prepare_surface(new_rect, is_shift_down, is_alt_down);

              // Apply changes in a single atomic transaction
              unsafe {
                  if let Ok(hdwp) = BeginDeferWindowPos(2) {
                      let mut hdwp = hdwp;
                      if let Ok(h) = DeferWindowPos(
                          hdwp,
                          hwnd,
                          HWND::default(),
                          new_rect.left,
                          new_rect.top,
                          new_rect.right - new_rect.left,
                          new_rect.bottom - new_rect.top,
                          SWP_NOACTIVATE | SWP_NOZORDER,
                      ) {
                          hdwp = h;
                      }
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
  ```

- [ ] **Step 3: Remove obsolete perform_discrete_resize method**
  Delete the definition of `fn perform_discrete_resize` entirely from [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs) to avoid dead code warnings.

- [ ] **Step 4: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/app.rs
  git commit -m "feat: implement continuous gliding resizing in the frame loop"
  ```
