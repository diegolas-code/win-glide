# Project Pause: `win-glide`

## Current Status
- **Phase:** Phase 2: Input & Hooks (In Progress).
- **Branch:** `feat/hotkey-registration`
- **Workflow:** TDD-driven, idiomatic Rust 2024.

## Last Actions
- Created `dev` branch and `feat/hotkey-registration` feature branch.
- Added `Win32_UI_Input_KeyboardAndMouse` feature to `Cargo.toml`.
- Initialized Phase 2 implementation.

## Next Steps
- Implement `Ctrl + Shift + M` global hotkey.
- Implement low-level keyboard (`WH_KEYBOARD_LL`) and mouse (`WH_MOUSE_LL`) hooks.
- Set up thread-safe message queuing for input processing.

## Blocking Issues
- None. Ready for implementation.
