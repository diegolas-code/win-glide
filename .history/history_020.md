# History Log 020: Comprehensive Codebase Documentation

## Date: Saturday, May 23, 2026

## Changes
- **Source Code Documentation:** Visited all files in `src/` and added extensive explanatory comments.
    - `main.rs`: Documented the startup sequence, thread orchestration, and message pump requirements.
    - `app.rs`: Explained the high-frequency loop (120Hz), state machine transitions, and off-screen boundary handling.
    - `physics.rs`: Detailed the "Snappy & Light" momentum model, dual friction logic, and diagonal normalization.
    - `input.rs`: Documented low-level Win32 hook safety (avoiding stuck keys), thread-safe event delivery, and message loops.
    - `ui.rs`: Explained the layered window approach, `tiny-skia` to GDI conversion, and `UpdateLayeredWindow` usage.
    - `platform.rs`: Documented DPI awareness and monitor enumeration logic.
    - `config.rs`: Documented JSON serialization and fallback logic.
    - `window.rs`: Documented foreground window acquisition.

## Rationale
While the codebase was functional, many of the technical decisions (especially regarding Win32 API nuances like stuck modifiers or message pumps) were not documented. Adding these comments ensures that future developers (or the user) can understand the "why" behind the implementation, which is critical for maintaining a systems-level utility in Rust.

## Technical Notes
- No functional changes were made; only comments and docstrings were added.
- Verified that `cargo test` still passes across the entire project.
- Branch `commenting-code` was created to isolate these documentation changes.
