# Current Status - win-glide

## Recent Achievements
- **Phase 8: Performance & Optimization (Completed)**
    - **Zero-Copy Rendering:** Refactored overlay rendering to draw directly into Win32 GDI memory, achieving "instant" appearance.
    - **Lean RAM Footprint:** Achieved and stabilized a **1.2MB Working Set** by using on-demand GDI resource allocation and slimming the window class.
    - **API Churn Reduction:** Implemented integer-based rectangle tracking to skip redundant Win32 `DeferWindowPos` calls when stationary.
    - **Elevated Window Safety:** Proactively detects and skips high-integrity windows (like Task Manager) to prevent OS access errors.
    - **Stability Focus:** Chose a stable 120Hz polling loop over blocking "Sleep Mode" to ensure perfect UI responsiveness and message pumping.

- **Phase 9: Window Center Hotkey (Completed)**
    - **Dual Hotkey Registration:** Integrated a second global hotkey (`Win + Alt + C`) routed through the application message loop.
    - **Work-Area Aware Math:** Queries the nearest monitor work area and centers the window inside it, respecting taskbar bounds.
    - **Oversized Windows:** Automatically resizes (shrinks) the window if it is larger than the work area before centering.
    - **Glide Integration:** Safely centers both the target window and the overlay synchronously while resetting glide velocity to zero.
    - **Hook Interception Fix:** Suppressed deactivating KeyDown events and allowed registered global hotkeys to pass through keyboard hooks to the OS while a glide session is active.
    - **Overlay Topmost Sync:** Dynamically sets the overlay window's Z-order style (`HWND_TOPMOST` / `HWND_NOTOPMOST`) to match the target window's topmost status, ensuring it renders on top of pinned topmost windows.
    - **Dynamic Console & Formatting Fix:** Added `Display` formatting to `HotkeyConfig` for printing active hotkeys dynamically. Configured main app console to output the centering hotkey (`Win+Alt+C` default) dynamically on startup. Resolved formatting-check errors by formatting files with `cargo fmt`.

## Immediate Next Steps
- **Phase 10: Productization (Queued Next)**
    - Create a release-optimized build profile.
    - Implement a system tray icon for status visibility and graceful exit.
    - Research and implement a simple installer.
    - Final audit of the `config.json` schema for long-term stability.

## Technical Notes
- The 1.2MB RAM usage is the stable "warmed-up" baseline after the first activation.
- The app is optimized for "silence" - consuming near-zero CPU and minimizing OS API interaction while backgrounded.
- Center-window design handles both active glide sessions (stops glide drift and updates overlay) and inactive sessions (moves foreground window directly) with safety policies in place.
