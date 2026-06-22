# History Log: Window Center Active Session Hotkey Fix

*   **Date:** 2026-06-22
*   **Feature:** Center Foreground Window on Active Monitor via `Win + Alt + C` (Glide Session Interception Fix)
*   **Branch:** `feat/center-window-active-fix`

---

## Technical Decisions & Root Cause Fix

### Root Cause
During an active glide session, the low-level keyboard hook callback `keyboard_proc` is configured to intercept and consume all key events that aren't modifier keys or key-up signals, forwarding them to the main thread as `InputEvent::KeyDown(vk_code)`. 
When the user triggered the centering hotkey (`Win + Alt + C`), the hook intercepted the `C` keypress (`0x43`), forwarded it to the app, and consumed it. The app received `KeyDown(0x43)` as a non-arrow key and executed its deactivation policy ("Panic Exit"). This killed the glide session immediately, preventing the centering from occurring inside the active session.

### Resolution
We updated the hook to ignore configured hotkeys during active sessions, letting them propagate normally to trigger Windows hotkey routing:
1.  **Global Hotkey Map:** Added a static `OnceLock<Vec<HotkeyConfig>>` named `HOTKEYS` in `src/input.rs` containing all registered hotkeys.
2.  **Modifier Queries:** Created a helper `check_modifiers` that queries current physical keystates of Ctrl, Alt, Shift, and Win using the Win32 `GetKeyState` API. By querying whether `GetKeyState(VK) < 0` (most significant bit is set if down), we cleanly avoid signed `i16` literal overflows (`0x8000` bounds check).
3.  **Bypassing KeyDown Emission:** Updated the `WM_KEYDOWN` and `WM_SYSKEYDOWN` match blocks inside `keyboard_proc` to check if the incoming virtual key code and current active modifiers match any configuration in `HOTKEYS`.
    *   If they match, the keypress is allowed to pass through (`CallNextHookEx`) and **no** `InputEvent::KeyDown` event is sent to the app.
    *   This ensures the OS receives the raw keys to trigger `WM_HOTKEY` (ID `1338`), which is handled correctly by the app main thread without deactivating the glide session beforehand.

## Verification

*   Checked that all 12 tests in the suite compile and pass perfectly with `cargo test --bin win-glide`.
*   Verified that the binary compiles cleanly under `dev` profiles.
