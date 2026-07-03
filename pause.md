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
    - **Continuous Resize Physics:** Replaced step-based resizing with a dedicated scaled `resize_physics` simulation that applies continuous thrust and decay.
    - **Corrected Shrink Border Directions:** Corrected opposite edges to pull inward, moving the active border in the direction of the arrow key pressed (e.g. Alt + Down pulls the top border down).
    - **Resizing Smoothness & 60Hz Throttling:** Throttles external layout calls to 60Hz while running physics at 120Hz, preventing paint backlogs in third-party window queues.
    - **Sizing Flag Optimizations:** Removed `SWP_NOCOPYBITS` to leverage native BitBlt copy-bits, maintaining compatibility with WPF, Chromium/Electron, and XAML windows.
    - **Configuration Integrated:** Added `resize_speed` support mapping to `config.json`.
    - **High-Precision Sizing State:** Implemented fractional accumulators (`width_f32`, `height_f32`) to keep dimensions robust and frame-rate independent.
    - **Glide Handoff:** Instantly zeros translational velocity when a resize combination starts to prevent drift.
    - **Safety Boundaries Clamped:** Enforces DPI-scaled $250\text{px}$ floor, active monitor work area bounds, and $150\text{px}$ off-screen margin rules.
    - **Overlay Synchronization Optimization:** Coordinated single-transaction layout update (`BeginDeferWindowPos`) and conditional redraw optimization to avoid GDI/DIB reallocation overhead.
    - **Unit Tests Written:** Fully validated resizing coordinate math, deltas, and clamping limits under multiple mock monitors and modifiers.

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

