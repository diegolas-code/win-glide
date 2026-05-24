# Current Status - win-glide

## Recent Achievements
- **Phase 7: Visual Polish & UX (Refinements)**
    - Reduced overlay "header" extension from 10px to 7px for a tighter fit.
    - Implemented rounded corners (8px radius) for the blue tinted overlay using `tiny-skia` paths.
- **Phase 8: Performance & Optimization**
    - **Zero-Copy Rendering Pipeline:** Refactored the overlay rendering to use `tiny-skia`'s `PixmapMut::from_bytes`, eliminating a full-frame memory copy.
    - **Instant Activation:** Optimized the hotkey trigger path by forcing an immediate Win32 message pump.
    - **Elevated Window Protection:** Implemented a security check to detect and skip high-integrity windows (like Task Manager) if the app is not elevated. This prevents "Access Denied" errors and provides clear console feedback.

## Immediate Next Steps
- **Optimization:** Implement "Sleep Mode" for the physics loop to reduce CPU usage when stationary.
- **Further Performance:** Optimize redraw logic to skip `UpdateLayeredWindow` if the window state hasn't changed.

## Technical Notes
- `is_window_elevated` uses process token checking and handles `ERROR_ACCESS_DENIED` as a signal of a higher-integrity target.
- UIPI (User Interface Privilege Isolation) is the primary driver for these restrictions.
