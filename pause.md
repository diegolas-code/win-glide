# Project Pause: `win-glide`

## Current Status
- **Phase:** Troubleshooting & Refinement (v0.1.1).
- **Branch:** `fix/window-hang-and-movement`
- **Workflow:** Systematic Debugging, TDD-driven.

## Last Actions
- Identified root cause of window hang: main thread was not pumping Win32 messages for the overlay window.
- Identified root cause of movement failure: `i32` truncation of sub-pixel movement.
- Implemented `pump_messages` in `App::run`.
- Refactored `App` to use `f32` for position accumulation (`pos_x`, `pos_y`).
- Implemented keyboard input suppression using a global atomic flag in the `WH_KEYBOARD_LL` hook.
- Added diagnostic logging for input events.

## Next Steps
- Merge `dev` into `master`.
- Perform a final end-to-end manual verification on Windows.
- Tag v0.1.0 release.

## Blocking Issues
- None. Ready for implementation.
