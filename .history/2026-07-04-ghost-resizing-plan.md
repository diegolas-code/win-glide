# Ghost Resizing Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Option A (Ghost Resizing) to update only the overlay bounds during continuous resizing, and commit the final bounds to the target window when the resizing session stops.

**Architecture:**
* Add `is_resizing_in_progress: bool` to track active resizing phases.
* Skip `sync_overlay_to_actual_window` while `is_resizing_in_progress` is true to prevent layout snapping.
* In `apply_continuous_resize`, defer layout updates only for the overlay window.
* Implement `commit_ghost_resize` to apply the final bounds to the target window and reset resizing tracking state.

**Tech Stack:** Rust (2024), Win32 GDI/DWM APIs.

---

### Task 1: Add is_resizing_in_progress State to App

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Add field to App struct**
  Add `is_resizing_in_progress: bool` to `App` struct in [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
  ```rust
      /// The modifier states of Shift and Alt during the last frame tick.
      last_modifiers_state: (bool, bool),
      /// Tracks if a continuous resizing operation is currently in progress.
      is_resizing_in_progress: bool,
      /// Flag to keep the main loop running.
      running: bool,
  ```

- [ ] **Step 2: Initialize in App::new constructor**
  Set `is_resizing_in_progress` to `false` in `App::new`:
  ```rust
              last_modifiers_state: (false, false),
              is_resizing_in_progress: false,
              running: true,
  ```

- [ ] **Step 3: Reset in activate_session and deactivate_session**
  Set `is_resizing_in_progress` to `false` on session transitions:
  * In `activate_session` right before `crate::input::set_session_active(true);`:
    ```rust
                      self.is_resizing_in_progress = false;
    ```
  * In `deactivate_session`:
    ```rust
          self.is_resizing_in_progress = false;
    ```

- [ ] **Step 4: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/app.rs
  git commit -m "feat: add is_resizing_in_progress state to App"
  ```

---

### Task 2: Implement Overlay-Only Resizing

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Update apply_continuous_resize to resize only the overlay**
  Modify `apply_continuous_resize` inside [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs) to defer positioning only on the overlay window and set `is_resizing_in_progress = true` when active sizing deltas are calculated:
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
              self.is_resizing_in_progress = true;

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
  ```

- [ ] **Step 2: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 3: Commit**
  ```bash
  git add src/app.rs
  git commit -m "feat: implement overlay-only bounds updates during continuous resizing"
  ```

---

### Task 3: Implement Commit Transition and Exclude Sync during Resize

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Skip sync_overlay_to_actual_window during resizing**
  Modify `App::run` to skip sync while `is_resizing_in_progress` is true:
  ```rust
              // If a session is active, perform the physics and movement update.
              if self.active_window.is_some() {
                  if !self.is_resizing_in_progress {
                      self.sync_overlay_to_actual_window();
                  }
  ```

- [ ] **Step 2: Update App::run main loop resizing branches to detect stop and commit**
  Modify `App::run` layout branch to check for resize-to-stop transitions and call `commit_ghost_resize()`:
  ```rust
                  if self.is_resizing_active() {
                      // Zero out translation velocity to prevent drift during resizing actions
                      self.physics.velocity = Vector2D::default();
                      self.apply_continuous_resize(dt, is_shift_down, is_alt_down);

                      // If velocity decays to zero, commit the ghost bounds
                      if self.is_resizing_in_progress && self.resize_physics.velocity.x == 0.0 && self.resize_physics.velocity.y == 0.0 {
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
  ```

- [ ] **Step 3: Implement commit_ghost_resize helper**
  Add the `commit_ghost_resize` helper method inside `impl App` in `src/app.rs`:
  ```rust
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
  ```

- [ ] **Step 4: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/app.rs
  git commit -m "feat: implement ghost resize commit logic and bypass sync during resize"
  ```
