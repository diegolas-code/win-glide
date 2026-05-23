# win-glide Project History: Phase 5 Completion & Keyboard Fix

## Issue: Keyboard Mapping Interference (Stuck Modifiers)
**Symptoms:**
- After using the application, the `Esc` key would trigger the Windows Start Menu (acting like `Ctrl+Esc`).
- System behavior felt "shifted" as if modifier keys were held down.

**Root Cause:**
- The `WH_KEYBOARD_LL` hook was consuming all keyboard events during an active session, including `KeyUp` events for the hotkey modifiers (`Ctrl`, `Alt`).
- This left the OS in a state where it believed modifiers were still pressed even after the user released them.

**Fix:**
- Modified `keyboard_proc` in `src/input.rs` to always allow modifier keys and all `KeyUp` events to pass through.
- Only specific `KeyDown` events for non-modifier keys are now suppressed during an active session.

## Change: Mouse Input Refactoring
**Decision:** Removed mouse movement tracking for window movement.
**Reasoning:** 
- Tracking high-frequency mouse movement via a low-level hook can introduce latency or "jitter" if not handled extremely carefully.
- For the initial release, keyboard-based "gliding" is the primary focus.
- **New Feature:** Added `WH_MOUSE_LL` click detection. Any mouse button click (Left, Right, Middle, X) now immediately deactivates the glide session. This acts as a reliable "panic button" for the user to regain control of the window.

## Final Phase 5 Polish
- **Precision:** Finalized the switch to `f32` for position accumulation to prevent integer truncation at high refresh rates.
- **Responsiveness:** Ensured the main thread pumps Win32 messages to keep the overlay window responsive.
- **Diagnostics:** Refined logging to be less verbose but more informative regarding session transitions.
