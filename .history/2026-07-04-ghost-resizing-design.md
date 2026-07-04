# Ghost Resizing (Overlay-Only Resizing) Design Spec

## Overview
This document describes the design for "Ghost Resizing", an optimization to make window resizing fluid and lag-free. Instead of physically resizing the target window on every frame (which forces synchronous layout passes and repaints on the target application thread), only the overlay window is resized in real time at 120Hz. The target window is physically resized only once when the resizing session stops or decays to zero velocity.

## State Machine
1. **Resizing In Progress Tracker:**
   Add `is_resizing_in_progress: bool` to the `App` struct.
   
2. **Active Resizing Phase:**
   When `self.is_resizing_active()` is true:
   * Set `is_resizing_in_progress = true`.
   * Apply thrust and friction to `resize_physics`.
   * Calculate size deltas `dx` and `dy` based on the resize velocity.
   * Update the overlay position/size (`new_rect`) and internal tracked bounds (`window_rect`, `pos_x`, `pos_y`, etc.).
   * **Do NOT resize the target window.** Use `DeferWindowPos` or `SetWindowPos` only on the overlay window.

3. **Commit Transition Phase:**
   We commit the final bounds to the target window when `is_resizing_in_progress` is true and either of the following occurs:
   * The user releases the modifier keys (`self.is_resizing_active()` becomes false).
   * Resizing velocity decays to zero (`self.resize_physics.velocity` is zero).
   
   **Actions during Commit:**
   * Perform a single physical `SetWindowPos` on the target window to size/position it to `self.window_rect`.
   * Set `is_resizing_in_progress = false`.
   * Sync overlay position and topmost Z-order to match.
