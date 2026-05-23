//! Window handling utilities.
//!
//! This module provides functions to interact with Win32 window handles (HWND),
//! focusing on identifying the current user-focused window.

use windows::Win32::Foundation::HWND;

/// Returns the handle of the window currently in the foreground (focused by the user).
///
/// This is used at the start of a glide session to determine which window
/// should be moved.
pub fn get_active_window() -> HWND {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    #[test]
    fn test_get_active_window() {
        // Simple verification that our wrapper returns the same as the raw API.
        let active_window = get_active_window();
        let expected_window = unsafe { GetForegroundWindow() };
        assert_eq!(active_window, expected_window);
    }
}
