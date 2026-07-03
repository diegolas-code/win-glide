# Discrete Resizing Steps and Overlay Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement snappy, discrete-step resizing (Shift = grow, Alt = shrink) on key down events with automatic overlay size synchronization from the actual window bounds.

---

### Task 1: Update App Struct and Constructor

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Modify App Struct Definition**
  - Add `resize_speed: f32` back to the `App` struct.
  - Remove `resize_physics` and `resize_accumulated_dt` from the `App` struct.

- [ ] **Step 2: Update App::new Constructor**
  - Initialize `resize_speed` field.
  - Remove `resize_physics` and `resize_accumulated_dt` initialization.

- [ ] **Step 3: Verify build compiles**
  - Run `cargo check` and verify it compiles (with warnings about unused variables like `resize_speed` or unused functions).

- [ ] **Step 4: Commit**
  - Commit Task 1 changes.

---

### Task 2: Implement Discrete Resizing Logic

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Write `perform_discrete_resize` Method**
  - Implement `perform_discrete_resize(&mut self, vk: u32, is_shift_down: bool, is_alt_down: bool)` in `App`.
  - Calculate `step = (self.resize_speed / 12.0).round().max(10.0)`.
  - Calculate `dx` and `dy` based on `vk` direction.
  - Invoke `calculate_resized_rect` with step.
  - Call `SetWindowPos` on target window.
  - Immediately query `GetWindowRect(hwnd)` to get `actual_rect`.
  - Sync internal coordinates (`pos_x`, `pos_y`, `width_f32`, `height_f32`, `window_rect`, `last_sent_rect`) to `actual_rect`.
  - Call `self.overlay.redraw(actual_rect)` to paint the overlay at the true size.

- [ ] **Step 2: Verify compilation**
  - Run `cargo check`.

- [ ] **Step 3: Commit**
  - Commit Task 2 changes.

---

### Task 3: Integrate with Event Loop and Clean Up Continuous Loop

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Intercept Arrow Keydowns in `process_events`**
  - Update `InputEvent::KeyDown(vk)` case for `0x25..=0x28` to check if `Shift` or `Alt` modifiers are down.
  - If a modifier is down, zero translational velocity and call `self.perform_discrete_resize(...)`.
  - Otherwise, insert the key into `self.pressed_keys` as usual.

- [ ] **Step 2: Prevent Translation While Resizing is Active**
  - Implement `is_resizing_active(&mut self) -> bool` to check modifier keys.
  - Update `App::run` to skip translational updates (`apply_thrust`, `update`, `apply_movement`) if `is_resizing_active()` is true.

- [ ] **Step 3: Remove Old `process_resize` Code**
  - Delete `process_resize` method from `src/app.rs`.

- [ ] **Step 4: Run all tests and verify**
  - Run `cargo test` and verify that all unit tests compile and pass.

- [ ] **Step 5: Commit**
  - Commit Task 3 changes.
