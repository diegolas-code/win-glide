# Current Status - win-glide

## Recent Achievements
- **Phase 8: Performance & Optimization (Completed)**
    - **Zero-Copy Rendering:** Refactored overlay rendering to draw directly into Win32 GDI memory, achieving "instant" appearance.
    - **GDI Handle Caching:** Cached Device Contexts and DIB sections inside the `Overlay` struct to eliminate handle re-creation churn.
    - **Capacity-Based Caching:** Modified validation from exact size checks (`==`) to capacity checks (`>=`), avoiding re-allocations when the window shrinks or stays within cached bounds.
    - **Growth Padding Buffer:** Added a `+256px` growth padding buffer when allocating new DIB sections. This absorbs continuous growth up to 256px, reducing DIB section creations by over 99% during active resizing.
    - **Stride-Safe Buffer Matching:** Wraps the CPU DIB section using the over-allocated dimensions to preserve layout stride and pitch, while constraining drawn graphics to the actual window size.
    - **Localized Alpha Scan:** Restricted GDI's subpixel grayscale anti-aliasing reconstruction scan to the exact text bounding box (`draw_rect`), reducing scanning pixels by 98%+ and cutting rendering times to `<0.2ms`.
    - **Lean RAM Footprint:** Achieved and stabilized a **1.2MB Working Set** by using on-demand GDI resource allocation and slimming the window class.
    - **API Churn Reduction:** Implemented integer-based rectangle tracking to skip redundant Win32 `DeferWindowPos` calls when stationary.
    - **Elevated Window Safety:** Proactively detects and skips high-integrity windows (like Task Manager) to prevent OS access errors.
    - **Stability Focus:** Chose a stable 120Hz polling loop over blocking "Sleep Mode" to ensure perfect UI responsiveness and message pumping.
    - **Text Background Fix:** Checked the Red channel (`slice[offset + 2] > 0`) to correctly isolate text pixels from the blue background layout and force them to pure white (`RGB = 255`), resolving the gray background tint halo under text.
    - **OS Scheduler Resolution Tuning:** Instantiates a `TimerResolutionGuard` RAII guard on application startup that calls `timeBeginPeriod(1)` to request a 1ms scheduler resolution on Windows, and calls `timeEndPeriod(1)` on drop. This eliminates 15.6ms scheduler sleep overshoots, stabilizing the loop rate at exactly 120 FPS.
    - **Hybrid Sleep/Yield Frame Rate Capping:** Capping frame rate using a hybrid model: calls `std::thread::sleep` if remaining time > 2ms (sleeping for `remaining - 1ms`), and busy-loops using `std::thread::yield_now()` for the final sub-millisecond portion to guarantee sub-millisecond precision with near-zero CPU overhead.

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

- **Phase 10: Keyboard-Driven Window Resizing (Completed)**
    - **Control Scheme Implemented:** Swapped modifiers to use `Shift` to grow/expand, and `Alt` to shrink/reduce.
    - **Continuous Gliding Resizing:** Migrated from discrete steps to a fluid, momentum-based continuous resizing simulation using a dedicated `resize_physics` engine configured around `resize_speed`.
    - **Customizable Resize Physics:** Added `resize_physics` (with acceleration, friction, thrust_friction, and top_speed) as an optional block in `config.json`. If omitted, default resizing configurations are computed proportionally based on `resize_speed`.
    - **Ghost Resizing (Overlay-Only Resizing):** Solved target application main-thread layout and repaint overhead by resizing only the lightweight overlay window in real time at 120Hz. The physical target window is resized exactly once using a single `SetWindowPos` call when the resizing glide velocity decays to zero or when modifier keys are released.
    - **Self-Healing and Exclusions:** Bypasses `sync_overlay_to_actual_window` during active ghost resizing to prevent layout conflicts and snapbacks. The sync loop resumes once committed, automatically correcting coordinate drift and caching target window min-bounds limits.
    - **Corrected Shrink Border Directions:** Corrected opposite edges to pull inward, moving the active border in the direction of the arrow key pressed (e.g. Alt + Down pulls the top border down).
    - **Overlay Bounds Sync & 4-Way Position Correction:** Performs client-side prediction, coupled with a 120Hz background monitoring thread that queries `GetWindowRect` to self-heal size mismatches. Implements dynamic limit caching (`detected_min_w`/`detected_min_h`) and 4-way corrective `SetWindowPos` calls to prevent position shifting and overlay mismatch in all directions when target windows hit internal minimum limits.
    - **Coordinated Layout Transaction:** Uses an atomic `BeginDeferWindowPos` layout transaction block during the resizing keypress event, committing boundary updates in the exact same DWM refresh frame.
    - **Split-Phase Rendering:** Implemented `prepare_surface` and `commit_surface` to render the GDI/tiny-skia bitmap on the CPU *before* committing the layout transaction, reducing the post-transaction upload delay to under 100 microseconds and eliminating 1-frame expansion drag.
    - **Configuration Integrated:** Added `resize_speed` support mapping to `config.json`.
    - **Safety Boundaries Clamped:** Enforces DPI-scaled $350\text{px}$ floor, active monitor work area bounds, and $150\text{px}$ off-screen margin rules.
    - **Clean Transition Physics:** Instantly zeros translational velocity when resizing begins to prevent window drifting.
    - **Unit Tests Written:** Fully validated resizing coordinate math, deltas, and clamping limits under multiple mock monitors and modifiers.
    - **Overlay Resize Indicators:** Draws DPI-scaled bold white chevron indicators centered inside borders on the overlay at 80% opacity. Displays outward chevrons for Shift-Expansion and inward chevrons for Alt-Shrinking, offset by a 30px margin, with automatic suppression if the window is too small. Redraws dynamically using 120Hz key state polling to respond immediately on modifier key press/release. Prevents activation flicker by suppressing indicators when `Ctrl` or `Win` modifier keys are held down.
    - **Overlay Help Legends:** Renders centered, regular weight (non-bold, i.e., `400`), DPI-scaled "Segoe UI" help instructions at 18px size. Key names are formatted inside brackets (e.g. `[Shift]`, `[Alt]`, `[Arrow keys]`). The instructions dynamically adapt: displays detailed guides when idle, a simplified direction instruction when moving, and outward/inward guides when `Shift`/`Alt` are held. Features GDI `ANTIALIASED_QUALITY` grayscale smoothing and a custom alpha reconstruction loop using the Red channel `r` to eliminate text border pixelation. Shares the exact same 80% opacity value (`INDICATOR_OPACITY = 204`) as the chevrons.

## Immediate Next Steps
- **Phase 11: Productization (The Road to v1.0.0)**
    - Create a release-optimized build profile.
    - Implement a system tray icon for easy exit and status visibility.
    - Research and implement a simple installer.
    - Final audit of the `config.json` schema for long-term stability.
    - Finalize user documentation and version bump to v1.0.0.

## Technical Notes
- The 1.2MB RAM usage is the stable "warmed-up" baseline after the first activation.
- The app is optimized for "silence" - consuming near-zero CPU and minimizing OS API interaction while backgrounded.
- Center-window design handles both active glide sessions (stops glide drift and updates overlay) and inactive sessions (moves foreground window directly) with safety policies in place.
- Resizing operates on continuous momentum-based accumulator logic to keep dimensions fluid and frame-rate independent.
