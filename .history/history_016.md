# win-glide Project History: Advanced Flicker Reduction & Focus Fix

## Issue: Lingering Overlay Flicker
**Symptoms:**
- While the overlay no longer "detaches," it still flickers occasionally during movement.

**Root Cause:**
- High-frequency updates on layered windows can cause flickering if the OS attempts to preserve window bits.
- Using `WS_EX_TOPMOST` can lead to compositor fighting.

**Fix: Ownership & Precision Flags**
- **Dynamic Ownership:** The overlay is now "owned" by the target window. This locks their Z-order and composition layer.
- **Aggressive Suppression Flags:** Added `SWP_DEFERERASE` and `SWP_NOSENDCHANGING` to the movement loop.
- **Synchronized Updates:** Used `DeferWindowPos` for atomic movement of both windows.

## Issue: Immediate Session Deactivation
**Symptoms:**
- After implementing window ownership, the session would deactivate immediately upon start.

**Root Cause:**
- The OS was reporting a focus change when the owned overlay was shown, triggering the "Focus lost" safety check in `App`.

**Fix: Focus Resilience**
- **WS_EX_NOACTIVATE:** Added this style to the overlay to strictly prevent it from ever taking focus.
- **Focus Check Update:** Updated `check_exit_conditions` in `src/app.rs` to allow the overlay itself to be the "foreground" window without deactivating the session.
