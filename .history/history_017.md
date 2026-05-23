# win-glide Project History: No Glide for Maximized Windows

## Change: Maximized Window Constraint
**Decision:** Prevented the application from activating its gliding functionality when the active window is maximized.
**Reasoning:** 
- Gliding a maximized window is generally not useful as it already occupies the entire monitor (or a fixed portion of it).
- This prevents accidental activation and potential visual glitches when trying to move windows that the OS expects to remain fixed.

## Implementation Details
- Added `IsZoomed` from `Win32_UI_WindowsAndMessaging` to the Win32 imports.
- Updated `activate_session` in `src/app.rs` to call `IsZoomed(hwnd)`.
- If the window is maximized, the session activation is skipped and a message is printed to the console.
- Cleaned up unused Win32 imports (`SWP_DEFERERASE`, `SWP_NOSENDCHANGING`).
