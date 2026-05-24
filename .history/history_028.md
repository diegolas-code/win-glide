# History Log - 028: Completing Phase 8 - Deep Performance Optimizations

## Context
With the core functionality and visual refinements stable, Phase 8 focused on making `win-glide` as efficient and "silent" as possible on system resources.

## Technical Decisions

### 1. Idle Sleep Mode (Blocking on Events)
Previously, the main loop ran at a constant ~120Hz, polling the event channel with `try_recv()`.
- **Change:** When `active_window` is `None`, the loop now uses `event_rx.recv()`.
- **Benefit:** This puts the application thread to sleep while idle. The OS scheduler won't wake it up until a hotkey or shutdown signal is sent by the input thread. CPU usage drops to 0% during idle.

### 2. API Churn Reduction (`last_sent_rect`)
The physics loop updates at ~120Hz. Even with small velocities, the rounded integer position of the window might not change every frame.
- **Change:** Added `last_sent_rect` to track the state of the last successful `DeferWindowPos` call.
- **Benefit:** Skips expensive Win32 API calls (`BeginDeferWindowPos`, `DeferWindowPos` x2, `EndDeferWindowPos`) if the window hasn't actually moved to a new pixel.

### 3. Integrated Diagnostics & Safety
The previous investigation into Task Manager compatibility was refined into a permanent safety feature.
- **Elevation Check:** Added `is_window_elevated` check.
- **Impact:** Prevents "Access Denied" errors when encountering high-integrity windows by proactively skipping them and informing the user.

## Changes

### `src/app.rs`
- Refactored `run()` to handle Active vs. Idle states.
- Implemented `handle_single_event()` to centralize state transitions.
- Added `last_sent_rect` tracking to `apply_movement()`.
- Ensured `pump_messages` and `check_exit_conditions` are preserved.

## Impact
`win-glide` is now optimized for long-term background operation. It consumes near-zero CPU when idle and minimizes OS API interaction during movement, all while maintaining its "instant" responsiveness.
