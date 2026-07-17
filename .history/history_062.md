# History Log: Overlay Resizing Performance & Smoothness Optimization

*   **Date:** 2026-07-16
*   **Feature:** Resizing Smoothness & 10% Speed Increase
*   **Branch:** `dev` (via `perf/smooth-resize`)

---

## Technical Decisions & Rationale

### 1. Overlay Resizing Jitter & Blockiness
- **Problem:** Resizing the overlay felt blocky. Even though target window repainting was deferred (ghost-resize), the overlay itself stuttered.
- **Root Cause:**
  - **Sub-pixel Jitter:** The physics engine updates floats at 120Hz. During slow/decelerating phases, size changes are sub-pixel, meaning the integer bounds (`new_rect`) do not change. The code still cleared, re-drew, and committed the layered window via GDI/UpdateLayeredWindow on every frame, causing frame pacing issues.
  - **API Conflict:** The loop called both `DeferWindowPos` (to resize the overlay window) and `UpdateLayeredWindow` (which also moves/resizes the layered window structure). This triggered double Win32 message loops (`WM_SIZE`) and DWM compositing overhead.
- **Decision:** Gate the updates by integer boundaries, and remove the redundant `DeferWindowPos` call during active resizing.
- **Implementation:**
  - In `App::apply_continuous_resize()` in `src/app.rs`, gate the redraw and `commit_surface` code by checking if `new_rect` differs from `self.last_sent_rect`.
  - Remove the `BeginDeferWindowPos` transaction loop entirely from `apply_continuous_resize`. Let the single atomic `UpdateLayeredWindow` call inside `commit_surface` handle both moving, resizing, and uploading the bitmap.
  - Ensure float coordinates continue to accumulate on every tick for high-precision physics.

### 2. 10% Speed & Acceleration Increase
- **Problem:** The user requested both window translation movement and window resizing glide to be 10% faster.
- **Decision:** Increase top speed and acceleration parameters in `config.json` by 10%.
- **Implementation:**
  - Update `physics` block: `acceleration` `2600.0` ➡️ `2860.0`, `top_speed` `2250.0` ➡️ `2475.0`.
  - Update `resize_speed`: `600.0` ➡️ `660.0`.
  - Update `resize_physics` block: `acceleration` `3000.0` ➡️ `3300.0`, `top_speed` `1500.0` ➡️ `1650.0`.

---

## Verification
- **Unit Tests:** All tests compile and pass.
- **Manual Verification:** Tested translation and resizing to confirm both are visually fluid and 10% faster.
