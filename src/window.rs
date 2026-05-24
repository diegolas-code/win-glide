//! Window handling utilities.
//!
//! This module provides functions to interact with Win32 window handles (HWND),
//! focusing on identifying the current user-focused window.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::System::Threading::{OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::Security::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};

/// Returns the handle of the window currently in the foreground (focused by the user).
///
/// This is used at the start of a glide session to determine which window
/// should be moved.
pub fn get_active_window() -> HWND {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }
}

/// Checks if a window belongs to a process with higher privileges (Administrator).
///
/// This is used to prevent the application from attempting to interact with
/// high-integrity windows (like Task Manager) if the application itself
/// is not elevated, as UIPI (User Interface Privilege Isolation) would block it.
pub fn is_window_elevated(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
    use windows::Win32::Foundation::WIN32_ERROR;

    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        if pid == 0 {
            return false;
        }

        let process = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(p) => p,
            Err(e) => {
                // If we can't even open the process for limited info, it's likely very protected.
                if e.code().0 as u32 == WIN32_ERROR(5).0 {
                    return true;
                }
                return false;
            }
        };

        let mut token: HANDLE = HANDLE::default();
        let result = OpenProcessToken(process, TOKEN_QUERY, &mut token);
        let _ = CloseHandle(process);

        if let Err(e) = result {
            // ERROR_ACCESS_DENIED (5) means the target process has a higher integrity level than us.
            if e.code().0 as u32 == WIN32_ERROR(5).0 {
                return true;
            }
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let res = windows::Win32::Security::GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );

        let _ = CloseHandle(token);

        res.is_ok() && elevation.TokenIsElevated != 0
    }
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
