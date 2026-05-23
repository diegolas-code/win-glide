# win-glide Project History: v0.1.0 Release

## Phase 1: Foundation
**Implemented Steps:**
- Initialized Rust project (Edition 2024).
- Configured `windows-rs` with `Win32_Foundation`, `Win32_UI_WindowsAndMessaging`, and `Win32_Graphics_Gdi`.
- Implemented `get_active_window` in `src/window.rs`.
- Created `Platform` module in `src/platform.rs` for DPI detection and Monitor enumeration.
- Verified logic with unit tests.

**Issues:**
- Initial `unsafe` usage needed adjustment for Rust 2024 strictness.
- Monitor enumeration required a system callback (`enum_monitor_callback`) with manual pointer casting for the `LPARAM` state.

## Phase 2: Input & Hooks
**Implemented Steps:**
- Added `Win32_UI_Input_KeyboardAndMouse` and `crossbeam-channel`.
- Implemented `HotkeyManager`, `KeyboardHook`, and `MouseHook` using RAII (`Drop` trait) for automatic cleanup.
- Created `InputManager` with a Win32 message loop (`GetMessageW`).
- Implemented a thread-safe global dispatcher using `OnceLock<Sender<InputEvent>>`.

**Issues:**
- **Hotkey Conflict:** The initial hotkey `Ctrl + Shift + M` caused `HRESULT(0x80070581)` (Already registered).
- **Troubleshooting:** Changed hotkey to `Ctrl + Alt + F10` (VK 0x79) to minimize collisions with IDEs and browsers.

## Phase 3 & 4: Physics, Movement & UI
**Implemented Steps:**
- Created `PhysicsState` with acceleration and friction logic.
- Implemented `App` loop (~120Hz) in `src/app.rs`.
- Integrated `SetWindowPos` for movement.
- Created `Overlay` in `src/ui.rs` using `WS_EX_LAYERED | WS_EX_TRANSPARENT`.
- Integrated `tiny-skia` for 2D border rendering.
- Synced overlay position with the moving window.

**Issues:**
- **Rendering Performance:** Initial attempts used standard GDI; switched to `UpdateLayeredWindow` with a BGRA pixmap from `tiny-skia` for clean transparency and 3px border visuals.
- **Type Mismatches:** Rust 2024 required explicit `extern "system"` for function pointers in Win32 callbacks.

## Phase 5: Configuration & Polish
**Implemented Steps:**
- Added `serde` and `serde_json`.
- Created `Config` module to manage `config.json`.
- Implemented **Idle Timeout (3s)** and **Focus Loss Detection**.
- Implemented **Boundary Clamping** using `MonitorFromWindow` and `GetMonitorInfoW` to respect work areas.

**Troubleshooting Advice:**
- If the hotkey fails to register, check for other background utilities (PowerToys, etc.) and update `config.json`.
- Windows movement requires the target window to not be running with higher privileges than `win-glide` (UIPI limitation).
