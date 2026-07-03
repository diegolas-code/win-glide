# Specification: Continuous Resizing Physics & Swapped Modifiers

This specification defines the behavior and mathematics for continuous, momentum-based window resizing in `win-glide` using the swapped modifier layout and corrected shrink directions.

## 1. Modifiers & Control Scheme

The keyboard modifiers for resizing are swapped:
*   **`Shift` + Arrow Key:** Expand (Grow) window outward.
*   **`Alt` + Arrow Key:** Shrink (Pull In) window inward.

### A. Growth Behavior (`Shift` held, `Alt` not held)
Arrow keys move the corresponding outer border outward to expand the window:
*   **`Shift` + `Left`:** Moves Left edge left (grows width, shifts position left).
*   **`Shift` + `Right`:** Moves Right edge right (grows width, position stable).
*   **`Shift` + `Up`:** Moves Top edge up (grows height, shifts position up).
*   **`Shift` + `Down`:** Moves Bottom edge down (grows height, position stable).

### B. Shrinkage Behavior (`Alt` held, `Shift` not held)
Arrow keys move the corresponding opposite border in the arrow's direction to shrink the window:
*   **`Alt` + `Left`:** Moves Right edge left (shrinks width, position stable).
*   **`Alt` + `Right`:** Moves Left edge right (shrinks width, shifts position right).
*   **`Alt` + `Up`:** Moves Bottom edge up (shrinks height, position stable).
*   **`Alt` + `Down`:** Moves Top edge down (shrinks height, shifts position down).

---

## 2. Resizing Physics Model

Continuous resizing utilizes a dedicated `PhysicsState` to track momentum:

### A. Initialization
We add `resize_physics: PhysicsState` to the `App` struct. On session activation, the resize physics configuration is scaled proportionally to `resize_speed`:
*   `resize_top_speed = resize_speed`
*   `resize_acceleration = resize_speed * (translation_acceleration / translation_top_speed)`
*   `resize_friction = translation_friction`
*   `resize_thrust_friction = translation_thrust_friction`

### B. Integration Loop
When a resize modifier is held:
1.  **Thrust:** Arrow keys apply unit thrust to `self.resize_physics.velocity`.
2.  **Velocity Updates:** `self.resize_physics.apply_thrust(thrust, dt)` and `self.resize_physics.update(dt, is_thrusting)` are called.
3.  **Application:** The velocity components `dx = self.resize_physics.velocity.x * dt` and `dy = self.resize_physics.velocity.y * dt` drive the layout changes.

### C. Math Formulas
*   **`is_shift_down` (Expand):**
    *   `dx > 0` (Right): `width_f32 += dx`
    *   `dx < 0` (Left): `pos_x += dx; width_f32 -= dx`
    *   `dy > 0` (Down): `height_f32 += dy`
    *   `dy < 0` (Up): `pos_y += dy; height_f32 -= dy`
*   **`is_alt_down` (Shrink):**
    *   `dx > 0` (Right): `pos_x += dx; width_f32 -= dx`
    *   `dx < 0` (Left): `width_f32 += dx`
    *   `dy > 0` (Down): `pos_y += dy; height_f32 -= dy`
    *   `dy < 0` (Up): `height_f32 += dy`

### D. Safety Handoff
If `is_resizing` becomes false (e.g. modifiers are released), `self.resize_physics.velocity` is immediately reset to `0.0` to prevent any ghost-resizing slide.
Similarly, while `is_resizing` is active, translation velocity `self.physics.velocity` is set to `0.0`.
