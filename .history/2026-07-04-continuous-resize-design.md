# Continuous Gliding Window Resizing Spec

## Overview
This document describes the design for migrating from discrete-step keyboard resizing to a continuous, momentum-based gliding resizing model. It brings the physics characteristics of the translation glide mode to the resizing mechanics.

## Architecture
1. **Resize Physics State:**
   Introduce a dedicated `resize_physics: PhysicsState` to the `App` struct.
   
2. **Resizing Configuration:**
   Configure the resize physics to cap at `resize_speed` as the top speed, with acceleration scaled proportionally to achieve the snappy "100ms spin-up" response.
   
3. **Event Routing:**
   Update the keyboard hook handling in `src/app.rs` so that arrow keys are added to the `pressed_keys` set regardless of modifier states. Remove the instantaneous `perform_discrete_resize` call from the keypress handler.
   
4. **Frame Loop Execution (120Hz):**
   * Check if resizing is active using `is_resizing_active()`.
   * **If active:**
     * Zero out translation velocity to prevent translation drift.
     * Calculate continuous thrust from arrow keys and apply to `resize_physics`.
     * Update `resize_physics` using standard time-delta integration and dual friction.
     * Apply size deltas (`dx`, `dy`) using the calculated resize velocity scaled by `dt`.
     * Co-position the target window and overlay using the client-predicted coordinates, letting the 120Hz background monitoring system heal any min-bound limits.
   * **If inactive:**
     * Zero out resize velocity to halt resizing momentum immediately when modifiers are released.
     * Fall back to normal translation physics and movement.
