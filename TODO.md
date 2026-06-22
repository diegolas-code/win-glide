# `win-glide` TODO

## General Workflow Rules
- Work in small steps (1-5 minutes per task).
- Write a test before or alongside implementation.
- Commit after every successful step/test pass.
- Use feature branches for each Phase.

## Phase 1: Foundation
- [x] Initialize Rust project (2024 edition) + Initial Commit
- [x] Implement basic Win32 window capture logic (get active window)
- [x] Add unit tests for window handle acquisition
- [x] Create `Platform` module for DPI and Monitor enumeration
- [x] Add tests for DPI scaling calculations
- [x] Set up GitHub Actions CI (fmt, clippy, test)
- [x] Create project README.md
- [x] Commit & Merge to dev

## Phase 2: Input & Hooks
- [x] Implement global hotkey registration (`Ctrl + Alt + F10`)
- [x] Implement `WH_KEYBOARD_LL` hook for arrow key detection
- [x] Implement `WH_MOUSE_LL` hook for delta tracking
- [x] Add integration tests for input event processing
- [x] Set up message queuing for thread-safe input processing
- [x] Commit & Merge to dev

## Phase 3: Physics & Movement
- [x] Implement the Physics Loop (60Hz/120Hz timer)
- [x] Define Physics State (velocity, thrust, friction)
- [x] Integrate thrust, friction, and velocity calculations
- [x] Implement `SetWindowPos` movement logic with DPI normalization
- [x] Commit & Merge to dev

## Phase 4: UI / Visuals
- [x] Create transparent layered window for the border
- [x] Implement `tiny-skia` rendering for the 3px border
- [x] Sync border position with the moving window
- [x] Commit & Merge to dev

## Phase 5: Configuration & Polish
- [x] Implement `serde` based JSON config loading
- [x] Add idle timeout and exit condition checks
- [x] Finalize multi-monitor edge case handling
- [x] Fix window hang (message pump) and movement (f32 accumulation)
- [x] Suppress keyboard input to target window during active session
- [x] Finalize project README.md with usage and config info
- [x] Fix keyboard interference (stuck modifiers) by allowing KeyUp/Modifiers
- [x] Replace mouse movement tracking with click-to-deactivate for safety
- [x] Refine overlay: replace 3px border with full-window solid tint
- [x] Extend overlay: add 10px top extension "header"
- [x] Improve movement: implement 120Hz continuous thrust and diagonal normalization
- [x] Tune physics: align acceleration and top speed with "Snappy & Light" model
- [x] Fix movement asymmetry: resolve positive direction lag caused by integer truncation
- [x] Implement graceful shutdown: handle Ctrl+C via SetConsoleCtrlHandler
- [x] Tune physics: increase top speed and decrease acceleration for better feel
- [x] Implement dual-friction model: allow reaching top speed while maintaining quick stop
- [x] Refine physics: reduce acceleration for 1.33s spin-up time
- [x] Remove monitor edge limits: allow free movement across virtual desktop
- [x] Limit off-screen movement: ensure at least 150px of the window remains visible
- [x] Fix overlay flicker and lag: implement DeferWindowPos and advanced flicker-reduction flags
- [x] Prevent glide on maximized windows: add IsZoomed check before activation
- [x] Implement exit on any key: any non-arrow key (except modifiers) stops the glide
- [x] Commit & Merge to dev

## Phase 6: Refinement & Hygiene
- [x] Implement graceful input-thread shutdown
- [x] Gate system-level tests to avoid CI flakiness
- [x] Improve error handling for event delivery and config persistence
- [x] Add defensive checks for UI resource allocation
- [x] Update README.md with developer notes on testing and shutdown
- [x] Add comprehensive explanatory comments to the entire codebase
- [x] Resolve Clippy errors and dead code warnings to satisfy CI
- [x] Implement window position logging on activation/deactivation
- [x] Commit & Merge to dev

## Phase 7: Visual Polish & UX
- [x] Reduce extra space above top window edge (10px -> 7px)
- [x] Implement rounded corners (8px radius) for the overlay
- [x] Commit & Merge to main

## Phase 8: Performance & Optimization
- [x] Implement zero-copy rendering (render directly into GDI DIB bits)
- [x] Restrict interaction with elevated windows (Task Manager) to avoid OS errors
- [x] Optimize overlay updates to only occur when window position actually changes
- [x] Profile and minimize GDI handle usage and memory allocations (Stable Polling approach)
- [x] Commit & Merge to dev

## Phase 9: Window Center Hotkey (`Win + Alt + C`)
- [x] Add a second global hotkey registration for center action (separate ID from glide hotkey)
- [x] Extend input event model with a dedicated center command event
- [x] Implement monitor work-area query helper for nearest monitor (`MonitorFromWindow` + `GetMonitorInfoW`)
- [x] Implement center-position calculation utility (window rect + monitor work area)
- [x] Implement one-shot centering path when glide session is inactive
- [x] Integrate centering while glide session is active (move overlay too; zero velocity)
- [x] Add safety guards: skip maximized/elevated windows under existing policy
- [x] Add unit tests for center calculation math and edge cases (odd/even dimensions)
- [x] Add integration/system test for hotkey event dispatch and action routing
- [x] Update README hotkey documentation and config notes if schema changes
- [ ] Commit & Merge to dev

## Phase 10: Productization (The Road to v1.0.0)
- [ ] Create a release-optimized build profile
- [ ] Implement a system tray icon for easy exit and status visibility
- [ ] Research and implement a simple installer (e.g., Inno Setup or a WiX-based MSI)
- [ ] Final audit of the `config.json` schema for long-term stability
- [ ] Finalize user documentation and version bump to v1.0.0
- [ ] Commit & Merge to dev
