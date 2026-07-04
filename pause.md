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

- **Phase 9b: System UI Exclusions (Completed)**
    - **Recursive Ancestry climbing:** Traverses ancestors and owners to identify root window.
    - **Process & Class Exclusion Whitelist:** Excludes Windows Taskbars, Start Menu, Desktop backgrounds, calendars, clocks, Action center, Quick Settings, and XAML containers.
    - **Explorer UI Container Filtering:** Specifically blocks explorer-owned modern UI containers.
    - **Glide and Center Protection:** Blocks both glide activation and one-shot centering operations on system UI elements, outputting warning logs.
    - **Integration Test Coverage:** Validated the live exclusion checking mechanism using real system UI controls.

- **Phase 11: Keyboard-Driven Window Resizing (Completed)**
    - **Control Scheme Implemented:** Swapped modifiers to use `Shift` to grow/expand, and `Alt` to shrink/reduce.
    - **Discrete Resize Steps:** Replaced continuous resizing with discrete step changes triggered directly on KeyDown events, leveraging OS-native keyboard repeat rates for high snappiness and zero layout stutters.
    - **Corrected Shrink Border Directions:** Corrected opposite edges to pull inward, moving the active border in the direction of the arrow key pressed (e.g. Alt + Down pulls the top border down).
    - **Overlay Bounds Sync & 4-Way Position Correction:** Performs client-side prediction, coupled with a 120Hz background monitoring thread that queries `GetWindowRect` to self-heal size mismatches. Implements dynamic limit caching (`detected_min_w`/`detected_min_h`) and 4-way corrective `SetWindowPos` calls to prevent position shifting and overlay mismatch in all directions when target windows hit internal minimum limits.
    - **Coordinated Layout Transaction:** Uses an atomic `BeginDeferWindowPos(2)` block during the resizing keypress event, committing both target and overlay boundary updates in the exact same DWM refresh frame.
    - **Split-Phase Rendering:** Implemented `prepare_surface` and `commit_surface` to render the GDI/tiny-skia bitmap on the CPU *before* committing the layout transaction, reducing the post-transaction upload delay to under 100 microseconds and eliminating 1-frame expansion drag.
    - **Configuration Integrated:** Added `resize_speed` support mapping to `config.json`.
    - **Safety Boundaries Clamped:** Enforces DPI-scaled $350\text{px}$ floor, active monitor work area bounds, and $150\text{px}$ off-screen margin rules.
    - **Clean Transition Physics:** Instantly zeros translational velocity when resizing begins to prevent window drifting.
    - **Unit Tests Written:** Fully validated resizing coordinate math, deltas, and clamping limits under multiple mock monitors and modifiers.
    - **Overlay Resize Indicators:** Draws DPI-scaled bold white arrow indicators centered inside borders on the overlay. Displays outward arrows (↑, ↓, ←, →) for Shift-Expansion and inward arrows (↓, ↑, →, ←) for Alt-Shrinking, with automatic suppression if the window is too small. Redraws dynamically using 120Hz key state polling to respond immediately on modifier key press/release.

## Immediate Next Steps
- **Phase 12: Productization (The Road to v1.0.0)**
    - Create a release-optimized build profile.
    - Implement a system tray icon for easy exit and status visibility.
    - Research and implement a simple installer.
    - Final audit of the `config.json` schema for long-term stability.
    - Finalize user documentation and version bump to v1.0.0.

## Technical Notes
- The 1.2MB RAM usage is the stable "warmed-up" baseline after the first activation.
- The app is optimized for "silence" - consuming near-zero CPU and minimizing OS API interaction while backgrounded.
- Center-window design handles both active glide sessions (stops glide drift and updates overlay) and inactive sessions (moves foreground window directly) with safety policies in place.
- Resizing operates on step-based accumulator logic to keep dimensions robust and frame-rate independent.

