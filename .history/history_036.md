# History Log: Continuous Resize Physics and Swapped Modifiers

*   **Date:** 2026-07-03
*   **Feature:** Keyboard-Driven Window Resizing Refinement (Continuous Physics & Corrected Borders)
*   **Branch:** `feat/keyboard-resizing-refinement`

---

## Technical Decisions & Rationale

### 1. Swapped Modifiers (Shift = Expand, Alt = Shrink)
*   *Decision:* Replaced Alt-growth/Shift-shrink with **Shift-growth/Alt-shrink**.
*   *Why:* Matches user muscle memory and preferences, offering a more intuitive layout.

### 2. Corrected Opposite Edge Shrinking Direction
*   *Decision:* Refactored resizing direction mapping:
    *   **Expand:** Border moves in the direction of the arrow key.
    *   **Shrink:** Inward/opposite border moves in the direction of the arrow key.
    *   *Alt + Down:* Moves Top border down (shrinks height, window shifts down).
    *   *Alt + Up:* Moves Bottom border up (shrinks height, window position stable).
    *   *Alt + Right:* Moves Left border right (shrinks width, window shifts right).
    *   *Alt + Left:* Moves Right border left (shrinks width, window position stable).
*   *Why:* Ensures that the border in motion always shifts in the direction of the arrow key pressed, creating a highly tactile, cohesive physical feel.

### 3. Continuous Resize Physics Simulation
*   *Decision:* Replaced discrete step-based updates (`step = resize_speed * dt`) with a dedicated `resize_physics: PhysicsState` simulation run frame-by-frame.
*   *Why:* Smooths out resizing by applying responsive acceleration and friction curves, matching the translation glide feel. 
*   *Proportional Scaling:* The resizing physics engine's acceleration is scaled proportionally to `resize_speed` relative to translation parameters (`resize_acceleration = resize_speed * (accel / top_speed)`), maintaining responsiveness.

---

## Verification
*   **Unit Tests:** Refactored `test_calculate_resized_rect` to assert physics delta offsets (`dx`, `dy`) and verify the new border motion rules. All tests pass with zero warnings.

---

## Documentation Updated
*   **[TODO.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/TODO.md):** Checked off continuous physics, swapped keys, and corrected directions.
*   **[PAUSE.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/pause.md):** UpdatedRecent Achievements with the new physics resizing highlights.
