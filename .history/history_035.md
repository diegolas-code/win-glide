# History Log: Keyboard-Driven Window Resizing Implementation

*   **Date:** 2026-07-02
*   **Feature:** Keyboard-Driven Window Resizing (Implementation)
*   **Branch:** `feat/keyboard-resizing`

---

## Technical Decisions & Rationale

### 1. Unified Sizing & Position Integration in App Loop
*   *Decision:* Integrated checks for keyboard modifiers directly in `App::run` via a new helper `App::process_resize`. If active, it bypasses standard translation physics (`apply_thrust` and `apply_movement`) and performs resize updates instead.
*   *Why:* To avoid duplicate updates and simplify state handling, isolating the resizing path ensures that sizing changes and normal gliding do not interfere.

### 2. High-Precision Sizing Accumulators
*   *Decision:* Added `width_f32` and `height_f32` fields to the `App` struct, mirroring `pos_x` and `pos_y` to store fractional window dimensions. These are updated frame-by-frame and rounded only when applying to the final window boundaries.
*   *Why:* Prevents loss of precision and integer truncation drift during fine-grained updates at high frame rates (120Hz).

### 3. Pure Coordinate Resizing Math
*   *Decision:* Implemented `calculate_resized_rect` as a pure function in `src/window.rs`, handling all calculations (growth/shrinkage, DPI-scaled min size, work area limits, off-screen margin constraints) completely free of Win32 handle or thread-state dependencies.
*   *Why:* Ensures complete ease of testability. We verified multiple edge cases (expanding, shrinking, DPI scaling, and screen boundaries) via pure unit tests.

### 4. Coordinated OS Transactions and Optimization
*   *Decision:* Used `BeginDeferWindowPos` to resize both the target window and the overlay synchronously in one step. Only called `Overlay::redraw` when the integer width or height of the window actually changed.
*   *Why:* Prevents overlay tearing/lag and avoids GDI/DIB resource thrashing, keeping RAM usage to the stable 1.2MB target.

---

## Documentation Updated
*   **[TODO.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/TODO.md):** Marked all Phase 10 sub-tasks as completed.
*   **[PAUSE.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/PAUSE.md):** Moved Phase 11 (Keyboard-Driven Window Resizing) to Recent Achievements and updated Immediate Next Steps with Phase 12 (Productization).
