# Project Pause: `win-glide`

## Current Status
- **Phase:** Phase 2: Input & Hooks (In Progress).
- **Branch:** `feat/hotkey-registration`
- **Workflow:** TDD-driven, idiomatic Rust 2024.

## Last Actions
- Created `dev` branch and `feat/hotkey-registration` feature branch.
- Added `Win32_UI_Input_KeyboardAndMouse` and `crossbeam-channel` features/dependencies.
- Implemented `HotkeyManager`, `KeyboardHook`, and `MouseHook` in `src/input.rs` with RAII support.
- Implemented `InputManager` with a Win32 message loop and a global thread-safe dispatcher.
- Added low-level hook callbacks (`keyboard_proc`, `mouse_proc`) to capture and emit `InputEvent`s.
- Verified input registrations and dispatcher with unit tests.

## Next Steps
- Implement `Ctrl + Shift + M` global hotkey.
- Implement low-level keyboard (`WH_KEYBOARD_LL`) and mouse (`WH_MOUSE_LL`) hooks.
- Set up thread-safe message queuing for input processing.

## Blocking Issues
- None. Ready for implementation.
