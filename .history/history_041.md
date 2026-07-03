# History Log: Dynamic Minimum Size Limits Caching and 4-Way Position Correction

*   **Date:** 2026-07-03
*   **Feature:** Keyboard-Driven Window Resizing (Minimum Size Limits Refinement)
*   **Branch:** `feat/keyboard-resizing-min-fix`

---

## Technical Decisions & Rationale

### 1. The Root Cause of Overlay Mismatch and Shifting
*   **Overlay Mismatch (Continuous Shrink):** During keyboard repeat, events are queued and processed rapidly. The overlay would continue shrinking to the win-glide minimum (250px) because:
    1.  Calculations were based on predicted values (`self.width_f32`, etc.).
    2.  `GetWindowRect` temporarily returned the requested size while the target thread was busy, fooling the sync loop.
    3.  Once the key was released, the target thread finally clamped the size to its limit (e.g. 400px), allowing the background sync loop to correct the overlay.
*   **Window Shifting (Left/Top Borders):** When shrinking from the Left (`Alt + Right`) or Top (`Alt + Down`), the window position changes. If the target window refuses to shrink below its limit, it ignores the size change but accepts the position change, causing the whole window to shift/move.

### 2. Solution: Dynamic Limit Caching & Symmetrical 4-Way Correction
*   **Dynamic Limits Caching:**
    *   Added `detected_min_w` and `detected_min_h` to `App` struct.
    *   In `sync_overlay_to_actual_window`, if the actual size of the window after a resize is larger than requested, we cache the actual size as the dynamic min limit.
    *   These cached limits are reset when resizing is inactive (modifiers released) to accommodate dynamic target layout changes.
    *   In `perform_discrete_resize`, we (1) query `GetWindowRect` at the start to baseline our calculations on true OS coordinates, and (2) clamp `new_w` and `new_h` using these cached limits, adjusting position coordinates to preserve stationary edges when limits are hit.
*   **Symmetrical 4-Way Correction:**
    *   Refactored the background sync loop to implement position shift recovery checks for all four arrow keys (VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN), ensuring the window stays completely static when hitting limits in any direction.

---

## Verification
*   **Unit Tests:** All unit tests pass cleanly.
*   **Aesthetics:** Overlay never shrinks below target window limits, and the window remains completely static with no shifting or vibration when limits are reached.
