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

### 2. Overlay Synchronization via GetWindowRect
*   **Problem:** Target windows (like VS Code, Task Manager, Slack, etc.) enforce their own application-internal minimum sizing limits that are larger than our custom 250px floor. Our previous code would shrink the overlay past these limits, causing a size mismatch.
*   **Decision:** Added a sync loop:
    1.  Perform `SetWindowPos` on the target window.
    2.  Immediately call `GetWindowRect(hwnd)` to query the **actual** bounds the OS applied.
    3.  Update the internal `App` state (`pos_x`, `pos_y`, `width_f32`, `height_f32`, `window_rect`) to match these actual bounds.
    4.  Call `Overlay::redraw` using the synced actual bounds.
*   **Result:** The overlay is guaranteed to never detach or shrink past the target window's physical limit.

---

## Verification
*   **Unit Tests:** All unit tests pass cleanly.
*   **Aesthetics:** Resizing is instantly snappy with zero blockiness, and the overlay bounds perfectly track target window boundaries even at sizing limits.
