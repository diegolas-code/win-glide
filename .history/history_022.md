# History Log 022: Window Position Logging

## Context
The user requested a feature to log the window's screen position to the terminal when the overlay is shown (activation) and when it exits (deactivation). This provides immediate visual feedback for debugging and user awareness of window coordinates.

## Technical Decisions
- **Source of Truth**: Used the `window_rect` stored in the `App` state. For activation, this is populated immediately after `GetWindowRect` succeeds. For deactivation, it contains the final position updated by the physics loop.
- **Log Format**: Followed existing logging conventions: `App: Activating session for window: [HWND] at position: (x, y)` and `App: Deactivating session (Final position: x, y)`.

## Significant Changes
- **src/app.rs**: Updated `activate_session` to include `rect.left` and `rect.top` in the activation log.
- **src/app.rs**: Updated `deactivate_session` to include `self.window_rect.left` and `self.window_rect.top` in the deactivation log.

## Verification Results
- `cargo check` passed.
- Logic verified via manual inspection of code paths: `window_rect` is updated every frame in `App::run`, ensuring the deactivation log is accurate.
