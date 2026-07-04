# History Log: Continuous Gliding Window Resizing

*   **Date:** 2026-07-04
*   **Feature:** Continuous Gliding Window Resizing
*   **Branch:** `feat/continuous-resize`

---

## Technical Decisions & Rationale

### 1. Dedicated Resizing Physics
*   **Problem:** Window translation uses a fluid, momentum-based "gliding" physics simulation, but window resizing used discrete, instant steps on key presses. This created a jarring transition when switching between translation and resizing.
*   **Decision:** Integrate a dedicated `resize_physics` simulation state to `App`. The physics config is scaled using the user's `resize_speed` as the top speed, with acceleration scaled proportionally (`resize_speed * 1.5`) to match translation feel.

### 2. Modifiers-Aware Event Routing
*   **Problem:** The keyboard hook previously intercepted and processed modifier + arrow combinations instantly for discrete steps.
*   **Decision:** Simplify key routing in the low-level hook. All arrow key events are placed into `pressed_keys` regardless of modifier states. The 120Hz physics tick reads the active modifier states and pressed arrows to apply thrust to the appropriate simulation state.

### 3. Handoff & Boundary Handling
*   **Decision:** When resizing modifiers are held down, translation velocity is zeroed out to prevent drift. When modifiers are released, resizing velocity is zeroed out to immediately halt resizing momentum. Resizing limits (minimum sizes, monitor work area limits, off-screen margin rules) are enforced continuously via `calculate_resized_rect`.

---

## Verification
*   **Unit Tests:** Verified that `test_calculate_resized_rect` passes and covers continuous delta sizing. All unit tests pass successfully.
