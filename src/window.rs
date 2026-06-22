//! Window handling utilities.
//!
//! This module provides functions to interact with Win32 window handles (HWND),
//! focusing on identifying the current user-focused window.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, RECT};
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

/// Computes the centered position of a window inside the monitor work area,
/// resizing (shrinking) the window if its dimensions exceed the work area.
pub fn calculate_centered_rect(window_rect: RECT, work_area: RECT) -> RECT {
    let win_w = window_rect.right - window_rect.left;
    let win_h = window_rect.bottom - window_rect.top;
    let work_w = work_area.right - work_area.left;
    let work_h = work_area.bottom - work_area.top;

    let new_w = if win_w > work_w { work_w } else { win_w };
    let new_h = if win_h > work_h { work_h } else { win_h };

    let new_left = work_area.left + (work_w - new_w) / 2;
    let new_top = work_area.top + (work_h - new_h) / 2;

    RECT {
        left: new_left,
        top: new_top,
        right: new_left + new_w,
        bottom: new_top + new_h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    use windows::Win32::Foundation::RECT;

    #[test]
    fn test_get_active_window() {
        // Simple verification that our wrapper returns the same as the raw API.
        let active_window = get_active_window();
        let expected_window = unsafe { GetForegroundWindow() };
        assert_eq!(active_window, expected_window);
    }

    #[test]
    fn test_calculate_centered_rect() {
        // Normal sizing: fits in work area
        let window_rect = RECT { left: 100, top: 100, right: 300, bottom: 200 }; // 200x100
        let work_area = RECT { left: 0, top: 0, right: 1000, bottom: 1000 };
        let result = calculate_centered_rect(window_rect, work_area);
        assert_eq!(result.left, 400);
        assert_eq!(result.top, 450);
        assert_eq!(result.right - result.left, 200);
        assert_eq!(result.bottom - result.top, 100);

        // Oversized sizing: too large for work area (width and height shrunk)
        let window_rect_large = RECT { left: 100, top: 100, right: 1200, bottom: 1200 }; // 1100x1100
        let result_large = calculate_centered_rect(window_rect_large, work_area);
        assert_eq!(result_large.left, 0);
        assert_eq!(result_large.top, 0);
        assert_eq!(result_large.right, 1000);
        assert_eq!(result_large.bottom, 1000);

        // Oversized width, fitting height
        let window_rect_wide = RECT { left: 100, top: 100, right: 1500, bottom: 500 }; // 1400x400
        let result_wide = calculate_centered_rect(window_rect_wide, work_area);
        assert_eq!(result_wide.left, 0);
        assert_eq!(result_wide.top, 300);
        assert_eq!(result_wide.right, 1000);
        assert_eq!(result_wide.bottom, 700);
    }
}
