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
*   **DPI Awareness:** `PerMonitorV2`. Visual speed (thrust) and border thickness are normalized against the current monitor's DPI.
*   **Boundary Handling:** Movement respects the `WorkArea` (excludes Taskbar).

## 5. Visuals & UI
*   **Overlay:** A 3px transparent layered window border rendered via `tiny-skia`.
*   **Tracking:** The border must follow the active window's bounding box in real-time.

## 6. Technical Stack
*   **Language:** Rust (Edition 2024).
*   **OS APIs:** `windows-rs` (Win32).
*   **Activation:** Global Hotkey (`RegisterHotKey`) set to `Ctrl + Shift + M`.
*   **Hooks:** `WH_KEYBOARD_LL` and `WH_MOUSE_LL` (low-level, non-blocking).
*   **Movement:** `SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`.

## 7. Exit & Safety
*   **Trigger:** `Ctrl + Shift + M` activates the movement session for the current foreground window.
*   **Explicit Exit:** `Esc` key, any alphanumeric key, or focus loss.
## 8. Engineering Standards & Workflow
*   **Test-Driven Development (TDD):** Every feature must be accompanied by unit or integration tests to ensure reliability and easy debugging.
*   **Code Quality:** Strictly adhere to Rust best practices (idiomatic code, clear ownership, robust error handling with `thiserror`).
*   **Incremental Development:** Work in small, logical steps. Commit frequently with descriptive messages.
*   **Branching Strategy:** Use feature branches for significant changes to keep the main branch stable.

