# History Log - 026: Restricting Elevated Window Interaction

## Context
The application was failing or behaving unpredictably when the target window was a high-integrity process (like Task Manager) and `win-glide` was running as a standard user. This is due to Windows security (UIPI).

## Technical Findings
- **UIPI Restrictions:** Standard user processes cannot send most messages or change ownership of windows belonging to elevated processes.
- **Detection Method:** We can detect if a target window is elevated by trying to open its process token with `TOKEN_QUERY`. If this fails with `ERROR_ACCESS_DENIED` (5), it indicates the target has a higher integrity level than our process.

## Technical Decisions
- **Safety First:** Instead of attempting to interact and failing with OS errors, the application now proactively checks the elevation of the target window.
- **User Feedback:** Clear console messages are provided when a window is skipped due to privilege mismatches, instructing the user on how to resolve it (running as Admin).

## Changes

### `src/platform.rs`
- Added `is_admin()` helper to check if `win-glide` is elevated.

### `src/window.rs`
- Added `is_window_elevated(hwnd)` to detect the privilege level of a target window.

### `src/app.rs`
- Updated `activate_session` to call `is_window_elevated`.
- If the target is elevated and the app is not, it prints a message and aborts activation.

### `src/main.rs`
- Added a startup check and warning if the app is not running as Administrator.

## Impact
Improved stability and user awareness. The application no longer enters a broken state when encountering Task Manager, and the user is clearly informed about the cause of the restriction.
