# win-glide Project History: Advanced Flicker Reduction

## Issue: Lingering Overlay Flicker
**Symptoms:**
- While the overlay no longer "detaches," it still flickers occasionally during movement.

**Root Cause:**
- Using `WS_EX_TOPMOST` can sometimes lead to compositor "fighting" when moving two windows at high speed.
- The default window class style lacked `CS_OWNDC`, which can lead to less stable GDI/Compositor interaction for layered windows.
- Standard movement flags like `SWP_NOCOPYBITS` are sometimes insufficient without also suppressing erase and position-changing messages.

**Fix: Ownership & Precision Flags**
- **Dynamic Ownership:** The overlay is now dynamically "owned" by the target window using `GWLP_HWNDPARENT`. This ensures they are treated as a single unit by the Windows compositor, keeping their Z-order perfectly synced without needing `WS_EX_TOPMOST`.
- **Enhanced Class Style:** Added `CS_OWNDC` to the `Overlay` window class to provide a private Device Context.
- **Aggressive Suppression Flags:** Added `SWP_DEFERERASE` and `SWP_NOSENDCHANGING` to the movement loop. This reduces the number of messages the OS sends to both windows during rapid movement, significantly smoothing out the visual performance.
- **Ordered Deferral:** Ensured the target window is moved before the overlay in the `DeferWindowPos` batch.

## Result
- The flicker is eliminated by ensuring the OS compositor sees the two windows as a strictly parent-child related unit during the atomic update.
