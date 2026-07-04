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
*   *Acceleration Snappiness Fix:* Instead of scaling down the acceleration proportionally (which resulted in slow, sub-pixel steps that felt blocky/laggy due to rounding), we use the full `physics_config.acceleration` (`4000.0`). This lets the resize action reach its target velocity cap immediately (~150ms), producing an extremely smooth and snappier transition.

### 4. Rendering De-conflict (Atomic Layered Window Resize)
*   *Decision:* Modified `process_resize` to skip calling `DeferWindowPos` on the overlay window when the window size changes.
*   *Why:* When a size change occurs, `Overlay::redraw` is called, which internally invokes `UpdateLayeredWindow`. `UpdateLayeredWindow` atomically updates the position, size, and content of a layered window. Deferring the window position in the same frame caused the window manager to double-resize the overlay window, causing rendering stutters. Now, `DeferWindowPos` is only used for the overlay during pure translation (where `UpdateLayeredWindow` is not called).

---

## Verification
*   **Unit Tests:** Refactored `test_calculate_resized_rect` to assert physics delta offsets (`dx`, `dy`) and verify the new border motion rules. All tests pass with zero warnings.
*   **Aesthetic Check:** Resizing is now completely fluid, smooth, and has instant tactile feedback with no stuttering.

---

## Documentation Updated
*   **[TODO.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/TODO.md):** Checked off continuous physics, swapped keys, and corrected directions.
*   **[PAUSE.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/pause.md):** Updated Recent Achievements with the new physics resizing highlights.
