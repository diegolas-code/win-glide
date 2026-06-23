# History Log: System UI Exclusions Implementation

*   **Date:** 2026-06-23
*   **Feature:** System UI Exclusions
*   **Branch:** `feature/system-ui-exclusions`

---

## Technical Decisions & Root Cause Fix

### Root Cause
1. **System UI Safety/Stability:** Previously, win-glide could target and trigger glide/movement or centering actions on Windows system UI components—such as the Taskbar (`Shell_TrayWnd`), Start Menu, System Tray, Action Center, and Desktop (`Progman`/`WorkerW`). Modifying the positioning or drawing overlays around these windows can disrupt OS layout systems, compromise stability, and degrade the user experience.

### Resolution
1. **Window Hierarchy & Owner Resolution (`get_root_window`):** Added a helper function to climb parent/owner chains recursively using `GetAncestor(hwnd, GA_ROOTOWNER)` and `GetWindow(hwnd, GW_OWNER)` (for owned popups/flyouts) to identify the ultimate top-level owner window.
2. **Exclusion Checking (`is_taskbar_or_start_menu`):** Implemented a layered Win32 exclusion checker in `src/window.rs`:
    * **Processes:** Case-insensitively blocks windows owned by `startmenuexperiencehost.exe`, `searchhost.exe`, and `shellexperiencehost.exe`.
    * **Class Names:** Case-insensitively blocks well-known system class names (e.g. `Shell_TrayWnd`, `Shell_SecondaryTrayWnd`, `TrayNotifyWnd`, `NotifyIconOverflowWindow`, `TrayClockWClass`, `ClockFlyoutWindow`, `ControlCenterWindow`, `Shell_LightDismissOverlay`, `Progman`, `WorkerW`, `ClassicShell.CMenuContainer`, `OpenShell.CMenuContainer`, `DV2ControlHost`, `XamlExplorerHostIslandWindow`).
    * **Explorer modern UI containers:** Filters `explorer.exe` windows matching `Windows.UI.Core.CoreWindow` or `NativeHWNDHost`.
    * **Hierarchy Check:** Verifies both the active window handle and its resolved root owner window.
3. **Application Control Integration:**
    * In `src/app.rs::activate_session()`, blocks activation if `is_taskbar_or_start_menu` evaluates to `true`.
    * In `src/app.rs::center_window()`, blocks centering if `is_taskbar_or_start_menu` evaluates to `true`.
4. **Integration Test:** Added `test_live_window_manager_is_taskbar_or_start_menu` in `src/window.rs` which dynamically queries system UI elements (like `Shell_TrayWnd` and `Progman`) using `FindWindowW` and asserts that they are correctly identified as excluded, while verifying default/invalid handles are not.

## Verification

*   Added integration test `window::tests::test_live_window_manager_is_taskbar_or_start_menu` verifying the checks against actual live Windows elements.
*   Verified that all 15 tests compile and pass successfully with `cargo test`.
