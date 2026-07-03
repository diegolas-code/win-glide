# History Log: Discrete Step Resizing and Overlay Sync

*   **Date:** 2026-07-03
*   **Feature:** Keyboard-Driven Window Resizing (Discrete Steps & Overlay Sync)
*   **Branch:** `feat/keyboard-resizing-discrete`

---

## Technical Decisions & Rationale

### 1. Discrete Step-Based Resizing
*   **Problem:** Continuous resizing, even with 60Hz layout throttling and optimized Win32 flags, felt stuttery and blocky because foreign target windows could not repaint their client areas fast enough.
*   **Decision:** Replaced continuous physics simulation for resizing with discrete-step sizing triggered directly by `InputEvent::KeyDown(vk)` events in the event loop:
    *   The step size is computed dynamically: `step = (self.resize_speed / 12.0).round().max(10.0)` (exactly `50px` for the default `600` speed).
    *   By executing on keydown events, holding the key down automatically utilizes the OS-native keyboard repeat rate and delay setting, yielding a highly responsive, snappy experience with no frame rate lag.
    *   Completely removed `resize_physics` and `resize_accumulated_dt` fields to simplify the codebase.

### 2. Overlay Synchronization & Position Correction via 120Hz Monitoring
*   **Problem 1 (Overlay Lag):** Because target windows are owned by separate threads, `SetWindowPos` is asynchronous. Querying `GetWindowRect` immediately after calling `SetWindowPos` returned stale dimensions because the target thread hadn't processed the resize message yet. This caused the overlay to lag behind by one step, creating a visible redraw delay.
*   **Problem 2 (Position Shifting):** When a window is shrunk from the left (`Alt + Right`) or top (`Alt + Down`), the position changes. If the target window refuses to shrink below its application-internal minimum size (e.g. 400px), it ignores the size change but accepts the position change. This caused the whole window to shift ("pull to the side") without resizing.
*   **Decision:** Split the synchronization and correction into a two-pronged system:
    1.  **Immediate Prediction:** In `perform_discrete_resize`, immediately update the overlay to the calculated `new_rect` for instant, zero-latency visual feedback.
    2.  **120Hz Background Sync:** In `App::run`, call a new background method `sync_overlay_to_actual_window` at 120Hz. If a mismatch is detected between the actual OS window bounds and our tracked state (which happens 8-16ms after a constrained resize), it updates the tracked state and syncs the overlay.
    3.  **Position Shift Recovery:** If a mismatch is detected during a shrink operation where the actual width/height is larger than expected, the sync code calculates the corrected coordinates needed to keep the opposite edge (right or bottom) stationary. It then invokes a corrective `SetWindowPos` to undo the position shift.
*   **Result:** The overlay behaves with zero latency, the window never drifts or shifts when hitting sizing limits, and the overlay perfectly matches the final actual boundaries.

---

## Verification
*   **Unit Tests:** All unit tests pass cleanly.
*   **Aesthetics:** Resizing is instantly snappy with zero blockiness, and the overlay bounds perfectly track target window boundaries even at sizing limits.
