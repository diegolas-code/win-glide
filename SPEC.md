# `win-glide` Specification

## 1. Project Vision
win-glide is a high-performance Windows utility for rapid, momentum-based window repositioning using keyboard arrow keys and hybrid mouse control. It prioritizes tactile feedback and precision.

## 2. Movement Physics (Snappy & Light)
*   **Acceleration:** High; reaching top speed within ~100ms.
*   **Top Speed:** Calibrated to traverse a 1080p monitor in ~1.5s.
*   **Friction:** High; windows "glide" for 50-100ms after key release.
*   **Diagonal Normalization:** Vector addition is normalized to ensure diagonal movement isn't faster than cardinal movement.

## 3. Hybrid Interaction Model
*   **Keyboard:** Arrows apply thrust. Releasing keys triggers the friction/glide.
*   **Mouse:** Moving the mouse during an active session "grabs" the window via deltas.
*   **Handoff:** Mouse movement resets keyboard-driven velocity to zero. Keyboard input can resume from the window's new mouse-driven position.
*   **Active State:** The session remains active until an explicit exit or timeout occurs.

## 4. Multi-Monitor & DPI
*   **Virtual Coordinates:** Seamless movement across all monitors in the virtual desktop.
*   **DPI Awareness:** `PerMonitorV2`. Visual speed (thrust) and border thickness are normalized against the target window's monitor DPI, passed directly to the renderer.

## 5. Visuals & UI
*   **Overlay:** A 3px transparent layered window border rendered via `tiny-skia`.
*   **Tracking:** The border must follow the active window's bounding box in real-time.
*   **Resize Indicators:** Show DPI-scaled 36px bold white chevron indicators centered inside the window borders on the overlay at 80% opacity when `Shift` (Expand) or `Alt` (Shrink) is pressed, pointing in the direction of the border transformation.

## 6. Technical Stack
*   **Language:** Rust (Edition 2024).
*   **OS APIs:** `windows-rs` (Win32).
*   **Activation:** Global Hotkey (`RegisterHotKey`) set to `Ctrl + Alt + F10`.
*   **Hooks:** `WH_KEYBOARD_LL` and `WH_MOUSE_LL` (low-level, non-blocking).
*   **Movement:** `SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`.
*   **Configuration:** The configuration file `config.json` is loaded and saved relative to the directory containing the running executable.

## 7. Exit & Safety
*   **Trigger:** `Ctrl + Alt + F10` activates the movement session for the current foreground window.
*   **Explicit Exit:** `Esc` key, any alphanumeric key, or focus loss.

## 8. Instant Center Action (New)
*   **Trigger:** `Win + Alt + C`.
*   **Intent:** Instantly center the current foreground window on the active monitor.
*   **Mode:** One-shot action (does not start a glide session by itself).
*   **Target Monitor:** Use the monitor nearest to the target window (`MonitorFromWindow(..., MONITOR_DEFAULTTONEAREST)`).
*   **Reference Area:** Center inside the monitor `WorkArea` (not full monitor bounds), so taskbar-reserved space is respected.
*   **Position Formula:**
	*   `new_left = work.left + (work_width - window_width) / 2`
	*   `new_top  = work.top  + (work_height - window_height) / 2`
*   **Move API:** Use `SetWindowPos` (or `DeferWindowPos` when paired with overlay) with `SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE`.
*   **Session Interaction:**
	*   If glide session is inactive: reposition window only.
	*   If glide session is active: reposition both window and overlay, and zero current velocity to avoid immediate drift after centering.
*   **Safety Rules:**
	*   Skip maximized windows.
	*   Skip elevated windows when app is not elevated (same UIPI policy as glide activation).
	*   If work area query fails, abort gracefully without moving the window.

## 9. Engineering Standards & Workflow
*   **Test-Driven Development (TDD):** Every feature must be accompanied by unit or integration tests to ensure reliability and easy debugging.
*   **Code Quality:** Strictly adhere to Rust best practices (idiomatic code, clear ownership, robust error handling with `thiserror`).
*   **Incremental Development:** Work in small, logical steps. Commit frequently with descriptive messages.
*   **Branching Strategy:** Use feature branches for significant changes to keep the main branch stable.

## 10. Keyboard-Driven Window Resizing
*   **Control Scheme:** Modifiers held while pressing Arrow keys during an active glide session:
    *   **`Shift` + Arrow Key:** Expand (Grow) window outward from the matching edge.
    *   **`Alt` + Arrow Key:** Shrink (Pull In) window inward from the matching edge.
    *   **Arrow Key (No Modifiers):** Standard translation/movement physics.
*   **Speed Control:** Configurable via root-level `resize_speed` parameter in `config.json` (defaults to `600.0` pixels per second).
*   **Movement Model:** Continuous momentum-based resizing physics utilizing a dedicated `resize_physics` simulation configured around `resize_speed`. Coordinate and scale updates are accumulated frame-rate independently using `width_f32` and `height_f32` fields on the `App` struct.
*   **Safety Guards:**
    *   **Minimum Size Floor:** Hardcoded minimum dimensions of 350x350px, dynamically scaled by the target window's DPI factor.
    *   **Work Area Limits:** Resizing expansion cannot push the window bounds outside the nearest monitor's work area.
    *   **Off-screen Margins:** Resizing/shifting cannot push window beyond the virtual screen margins (keeping at least 150px of the window visible).
*   **Handoff Logic:** Zero out translation velocity `self.physics.velocity` immediately upon resize key combination processing to avoid momentum drift during resizing.
*   **Input Layer:** Keyboard hook `keyboard_proc` does not consume `VK_MENU` or `VK_SHIFT` to prevent stuck modifier keys. Main loop checks modifier states using `GetAsyncKeyState`.
*   **UI Synchronization:** Coordinated update of the window and overlay in a single `BeginDeferWindowPos` transaction.
*   **Optimization:** Redraw the overlay tint via `tiny-skia` only when target window's integer width or height changes.
