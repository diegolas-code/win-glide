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
- [x] Commit & Merge to main

## Phase 2: Input & Hooks
- [x] Implement global hotkey registration (`Ctrl + Alt + F10`)
- [x] Implement `WH_KEYBOARD_LL` hook for arrow key detection
- [x] Implement `WH_MOUSE_LL` hook for delta tracking
- [x] Add integration tests for input event processing
- [x] Set up message queuing for thread-safe input processing
- [x] Commit & Merge to main

## Phase 3: Physics & Movement
- [ ] Implement the Physics Loop (60Hz/120Hz timer)
- [ ] Integrate thrust, friction, and velocity calculations
- [ ] Implement `SetWindowPos` movement logic with DPI normalization

## Phase 4: UI / Visuals
- [ ] Create transparent layered window for the border
- [ ] Implement `tiny-skia` rendering for the 3px border
- [ ] Sync border position with the moving window

## Phase 5: Configuration & Polish
- [ ] Implement `serde` based JSON config loading
- [ ] Add idle timeout and exit condition checks
- [ ] Finalize multi-monitor edge case handling
