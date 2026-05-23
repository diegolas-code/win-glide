# Project Pause: `win-glide`

## Current Status
- **Phase:** Phase 5: Configuration & Polish (In Progress).
- **Branch:** `feat/config-system`
- **Workflow:** TDD-driven, idiomatic Rust 2024.

## Last Actions
- Implemented `Config` system in `src/config.rs` using `serde`.
- Updated `InputManager` and `App` to be fully configurable via `config.json`.
- Integrated `Overlay` with `tiny-skia` rendering and smooth movement.
- Verified that all components work together with the new configuration system.

## Next Steps
- Implement `serde` based JSON config loading.
- Add idle timeout and exit condition checks (Escape key, focus loss).
- Finalize multi-monitor edge case handling.

## Blocking Issues
- None. Ready for implementation.
