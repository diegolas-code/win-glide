# History Log: Window Center Hotkey (Phase 9 Completion)

*   **Date:** 2026-06-22
*   **Feature:** Center Foreground Window on Active Monitor via `Win + Alt + C`
*   **Branch:** `feat/center-window-hotkey`

---

## Technical Decisions & Architecture

To support the instant window centering action via `Win + Alt + C`, we extended the Win32 global hotkeys and coordinate-handling architecture in a modular, thread-safe manner:

1.  **Multiple Hotkey Support:**
    *   Refactored `InputManager` to register and track multiple global hotkeys dynamically. We now register ID `1337` for the Glide activation hotkey and ID `1338` for the Center Window hotkey.
    *   Since the input event loop forwards all registered hotkeys with their respective IDs, `WM_HOTKEY` triggers `InputEvent::HotkeyTriggered(id)` which cleanly conveys the action type to the main loop without thread safety concerns.

2.  **Monitor Work Area Querying:**
    *   Implemented `Platform::get_nearest_monitor_work_area` using raw Win32 APIs:
        *   `MonitorFromWindow` (with `MONITOR_DEFAULTTONEAREST`) to find the monitor nearest to the target window.
        *   `GetMonitorInfoW` to extract the `rcWork` rectangle, which excludes system-reserved areas like taskbars.

3.  **Sizing and Math Calculations:**
    *   Extracted the centering layout logic into a pure, testable function `calculate_centered_rect` in `window.rs`.
    *   Ensured that if a window's dimensions are larger than the work area, they are shrunk to fit the work area boundaries before computing the center. This satisfies user specifications for large window handling.

4.  **Glide Session Interlock:**
    *   **Inactive Session:** Directly reposition the foreground window using `SetWindowPos` without activating glide or spawning overlays.
    *   **Active Session:** Update the internal tracking position coordinates (`pos_x`, `pos_y`) and bounding rectangle (`window_rect`), reset the physics velocity to zero to avoid immediate drift post-centering, and defer window/overlay positions together using `DeferWindowPos` to avoid stuttering or tearing.

## Verifications & Test Coverage

*   Added pure unit tests in `src/window.rs` (`test_calculate_centered_rect`) to check:
    *   Standard centering (fitting window).
    *   Oversized window handling (shrinking width and height to fit).
    *   Partial oversized handling (shrinking only one dimension).
*   Added integration check in `src/platform.rs` (`test_get_nearest_monitor_work_area`) to confirm API queries resolve correctly for active windows.
*   Verified clean compilation and correct build behavior with `cargo build`.
