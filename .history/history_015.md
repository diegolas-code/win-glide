# win-glide Project History: Overlay Sync & Flicker Fix

## Issue: Overlay Flicker and Lag
**Symptoms:**
- The blue overlay flicker occasionally during window movement.
- Sometimes the overlay "detaches" and stays fixed while the window moves.

**Root Cause:**
- The application was calling `SetWindowPos` twice per frame: once for the target window and once for the overlay.
- These calls were not atomic, leading to a visual "gap" or lag between the two windows.
- High-frequency updates on layered windows can cause flickering if the OS attempts to preserve window bits (`SWP_NOCOPYBITS` was missing).

**Fix: Synchronized Batched Updates**
- Refactored `apply_movement` to use the Win32 `DeferWindowPos` API.
- **Atomic Operation:** By using `BeginDeferWindowPos(2)`, both the target window and the overlay are moved in a single system refresh cycle.
- **Reduced Flicker:** Added the `SWP_NOCOPYBITS` flag to the target window movement, which prevents the OS from trying to copy the old window content to the new position, a common cause of flicker during rapid movement.
- **Code Cleanup:** Refactored `Overlay` to expose `defer_update_position` and removed the unused `update_position` method.

## Result
- The overlay now stays perfectly locked to the target window without any perceptible lag or "detachment".
- Movement is significantly smoother and flicker-free.
