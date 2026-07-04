# History Log: Ghost Resizing (Overlay-Only Resizing)

*   **Date:** 2026-07-04
*   **Feature:** Ghost Resizing (Overlay-Only Resizing)
*   **Branch:** `feat/optimize-continuous-resize`

---

## Technical Decisions & Rationale

### 1. Overlay-Only Resizing during Active Glide
*   **Problem:** Physically resizing the target window on every frame during continuous resizing forces the target application to perform layout and paint cycles. If the target application has a heavy layout thread (e.g., browsers, IDEs, Electron apps), this causes significant visual stutter and frame drops.
*   **Decision:** Implement Ghost Resizing. When resizing is active, only the overlay window is resized/moved in real time at 120Hz. The target window remains at its original size, serving as a fluid preview bounding box.

### 2. State Machine & Deferring window pos
*   **Decision:** Add `is_resizing_in_progress: bool` to the `App` struct. In `apply_continuous_resize`, we set this flag to `true` and update bounds.
*   **Defer Pos:** The `BeginDeferWindowPos` call inside `apply_continuous_resize` is reduced to size `1` and targets *only* the overlay window. The target window handle is omitted, eliminating all layout messages to the target window.

### 3. Exclude bounds sync during resizing
*   **Problem:** The 120Hz monitoring loop `sync_overlay_to_actual_window` automatically pulls the actual target window bounds and corrects self-healing drift. If left active during ghost resizing, it would detect that the target window has not resized and overwrite the overlay position back to the target's original size, breaking the ghost resizing.
*   **Decision:** In `App::run`, bypass calling `sync_overlay_to_actual_window` while `is_resizing_in_progress` is true.

### 4. Committing Ghost Bounds
*   **Decision:** Implement the `commit_ghost_resize` helper. When `is_resizing_in_progress` is true and either (a) the user releases the modifier keys or (b) the resizing physics velocity decays to zero, perform a single physical `SetWindowPos` call on the target window to size it to the final overlay size, and set `is_resizing_in_progress` to `false`.

---

## Verification
*   **Unit Tests:** Verified that all 19 tests compile and pass successfully with zero warnings.
