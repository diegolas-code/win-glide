# History Log: Boundary Distance Accumulation Fix

*   **Date:** 2026-07-22
*   **Feature:** Screen Boundary Distance Accumulation & Collision Velocity Fix
*   **Branch:** `fix/boundary-distance-accumulation`

---

## Technical Decisions & Rationale

### 1. Distance Accumulation at Off-Screen Parking Boundaries
- **Problem:** When moving a window towards the screen edges, the window stopped traveling at the 150px minimum visible margin constraint as intended. However, the high-precision float position accumulators (`self.pos_x` and `self.pos_y`) continued accumulating distance into negative or out-of-bounds numbers while thrusting. Additionally, the physics velocity component in the collision direction was not reset.
- **Consequence:** When applying thrust in the opposite direction (moving back towards the center of the screen), the user experienced significant input lag because `pos_x`/`pos_y` had to travel back all the phantom accumulated distance before the window bounds integer rect (`new_rect`) actually changed.
- **Decision:** Clamp `pos_x` and `pos_y` directly against the virtual screen boundary limits (`min_x`, `max_x`, `min_y`, `max_y`), and reset the velocity component in the direction of collision to `0.0`.
- **Implementation:**
  - In `App::apply_movement()` inside `src/app.rs`, compute `min_x`, `max_x`, `min_y`, `max_y` as floats.
  - If `pos_x < min_x`, clamp `pos_x = min_x` and if `velocity.x < 0.0`, set `velocity.x = 0.0`.
  - If `pos_x > max_x`, clamp `pos_x = max_x` and if `velocity.x > 0.0`, set `velocity.x = 0.0`.
  - Apply symmetric clamping logic for `pos_y` against `min_y` and `max_y`.

---

## Verification
- **Unit Tests:** All unit and integration tests compile and pass (`cargo test`).
- **Clippy:** Passed clean with zero warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
