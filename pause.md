# Project Pause: `win-glide`

## Current Status
- **Phase:** Phase 2: Input & Hooks Complete. Moving to Phase 3: Physics & Movement.
- **Branch:** `feat/hotkey-registration`
- **Workflow:** TDD-driven, idiomatic Rust 2024.

## Last Actions
- Implemented `InputManager` with a Win32 message loop and a global thread-safe dispatcher.
- Added low-level hook callbacks (`keyboard_proc`, `mouse_proc`) to capture and emit `InputEvent`s.
- Verified all input registrations and dispatcher with unit tests.
- Handled `HotkeyManager`, `KeyboardHook`, and `MouseHook` with RAII.

## Next Steps
- Implement `Ctrl + Alt + F10` global hotkey.
- Implement low-level keyboard (`WH_KEYBOARD_LL`) and mouse (`WH_MOUSE_LL`) hooks.
- Set up thread-safe message queuing for input processing.

## Blocking Issues
- None. Ready for implementation.
