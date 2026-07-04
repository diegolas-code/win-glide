//! Window handling utilities.
//!
//! This module provides functions to interact with Win32 window handles (HWND),
//! focusing on identifying the current user-focused window.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, RECT};
use windows::Win32::Security::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

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
    use windows::Win32::Foundation::WIN32_ERROR;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

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

/// Finds the absolute root window by climbing parent and owner windows.
pub fn get_root_window(hwnd: HWND) -> HWND {
    use windows::Win32::UI::WindowsAndMessaging::{GA_ROOTOWNER, GW_OWNER, GetAncestor, GetWindow};
    unsafe {
        let mut current = hwnd;
        loop {
            let ancestor = GetAncestor(current, GA_ROOTOWNER);
            if !ancestor.is_invalid() && !ancestor.0.is_null() && ancestor != current {
                current = ancestor;
                continue;
            }
            let owner = GetWindow(current, GW_OWNER).unwrap_or_default();
            if !owner.is_invalid() && !owner.0.is_null() && owner != current {
                current = owner;
                continue;
            }
            break;
        }
        current
    }
}

/// Retrieves the class name of the given window.
pub fn get_window_class_name(hwnd: HWND) -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut buffer = [0u16; 256];
    unsafe {
        let len = GetClassNameW(hwnd, &mut buffer);
        if len > 0 {
            Some(String::from_utf16_lossy(&buffer[..len as usize]))
        } else {
            None
        }
    }
}

/// Retrieves the image name of the process that owns the given window.
pub fn get_window_process_name(hwnd: HWND) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let res = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);

        if res.is_ok() && size > 0 {
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            if let Some(file_name) = std::path::Path::new(&path).file_name() {
                return file_name.to_str().map(|s| s.to_string());
            }
        }
        None
    }
}

/// Checks if a window is a Windows taskbar, Start Menu, or other core shell UI component
/// that should be excluded from glide or centering operations.
pub fn is_taskbar_or_start_menu(hwnd: HWND) -> bool {
    if hwnd.is_invalid() || hwnd.0.is_null() {
        return false;
    }

    let root_hwnd = get_root_window(hwnd);

    let check_window = |h: HWND| -> Option<(&'static str, String, String)> {
        let class_name = get_window_class_name(h)?;
        let process_name = get_window_process_name(h).unwrap_or_default();

        let class_lower = class_name.to_lowercase();
        let proc_lower = process_name.to_lowercase();

        // 1. Excluded Processes
        let is_proc_excluded = matches!(
            proc_lower.as_str(),
            "startmenuexperiencehost.exe" | "searchhost.exe" | "shellexperiencehost.exe"
        );
        if is_proc_excluded {
            return Some(("process", class_name, process_name));
        }

        // 2. Excluded Class Names
        let is_class_excluded = matches!(
            class_lower.as_str(),
            "shell_traywnd"
                | "shell_secondarytraywnd"
                | "traynotifywnd"
                | "notifyiconoverflowwindow"
                | "trayclockwclass"
                | "clockflyoutwindow"
                | "controlcenterwindow"
                | "shell_lightdismissoverlay"
                | "progman"
                | "workerw"
                | "classicshell.cmenucontainer"
                | "openshell.cmenucontainer"
                | "dv2controlhost"
                | "xamlexplorerhostislandwindow"
        );
        if is_class_excluded {
            return Some(("class", class_name, process_name));
        }

        // 3. Explorer Modern UI Containers (explorer.exe + specific class)
        if proc_lower == "explorer.exe"
            && (class_lower == "windows.ui.core.corewindow" || class_lower == "nativehwndhost")
        {
            return Some(("explorer_ui", class_name, process_name));
        }

        None
    };

    // Evaluate active window
    if let Some((reason, class_name, process_name)) = check_window(hwnd) {
        println!(
            "[Win-Glide] [Warning] Ignoring action because target window is System UI: HWND={:?}, Class={:?}, Process={:?}, Root={:?}, Reason={}",
            hwnd, class_name, process_name, root_hwnd, reason
        );
        return true;
    }

    // Evaluate root window if different
    if root_hwnd != hwnd {
        let root_check = check_window(root_hwnd);
        if let Some((reason, class_name, process_name)) = root_check {
            println!(
                "[Win-Glide] [Warning] Ignoring action because target root window is System UI: HWND={:?} (Root={:?}), Class={:?}, Process={:?}, Reason={}",
                hwnd, root_hwnd, class_name, process_name, reason
            );
            return true;
        }
    }

    false
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

/// Computes the resized position and dimensions of a window based on modifier states,
/// continuous physics deltas (dx, dy), and DPI, clamping to safety boundaries (min size, work area limits, visibility).
pub fn calculate_resized_rect(
    current_x: f32,
    current_y: f32,
    current_w: f32,
    current_h: f32,
    is_shift_down: bool,
    is_alt_down: bool,
    dx: f32,
    dy: f32,
    dpi: u32,
    work_area: RECT,
    vs: RECT,
) -> (f32, f32, f32, f32) {
    let mut new_x = current_x;
    let mut new_y = current_y;
    let mut new_w = current_w;
    let mut new_h = current_h;

    if is_shift_down && !is_alt_down {
        // Expand (Grow)
        if dx > 0.0 {
            new_w += dx;
        } else if dx < 0.0 {
            new_x += dx;
            new_w -= dx;
        }

        if dy > 0.0 {
            new_h += dy;
        } else if dy < 0.0 {
            new_y += dy;
            new_h -= dy;
        }
    } else if is_alt_down && !is_shift_down {
        // Shrink (Reduce)
        if dx > 0.0 {
            new_x += dx;
            new_w -= dx;
        } else if dx < 0.0 {
            new_w += dx;
        }

        if dy > 0.0 {
            new_y += dy;
            new_h -= dy;
        } else if dy < 0.0 {
            new_h += dy;
        }
    }

    // 1. Minimum Size Floor (DPI scaled)
    let scale_factor = dpi as f32 / 96.0;
    let min_w = 350.0 * scale_factor;
    let min_h = 350.0 * scale_factor;

    if new_w < min_w {
        if is_alt_down && dx > 0.0 {
            // Shrunk from Left, adjust pos_x to preserve right edge
            new_x = current_x + current_w - min_w;
        }
        new_w = min_w;
    }
    if new_h < min_h {
        if is_alt_down && dy > 0.0 {
            // Shrunk from Top, adjust pos_y to preserve bottom edge
            new_y = current_y + current_h - min_h;
        }
        new_h = min_h;
    }

    // 2. Monitor Work Area Boundary (Only for Shift-Expansion)
    if is_shift_down && !is_alt_down {
        if new_x < work_area.left as f32 {
            new_x = work_area.left as f32;
            new_w = (current_x + current_w) - new_x;
        }
        if new_x + new_w > work_area.right as f32 {
            new_w = work_area.right as f32 - new_x;
        }
        if new_y < work_area.top as f32 {
            new_y = work_area.top as f32;
            new_h = (current_y + current_h) - new_y;
        }
        if new_y + new_h > work_area.bottom as f32 {
            new_h = work_area.bottom as f32 - new_y;
        }
    }

    // 3. Off-Screen Parking Constraints (Minimum 150px visible)
    let min_visible = 150.0;
    if new_x < vs.left as f32 - new_w + min_visible {
        new_x = vs.left as f32 - new_w + min_visible;
    } else if new_x > vs.right as f32 - min_visible {
        new_x = vs.right as f32 - min_visible;
    }

    if new_y < vs.top as f32 - new_h + min_visible {
        new_y = vs.top as f32 - new_h + min_visible;
    } else if new_y > vs.bottom as f32 - min_visible {
        new_y = vs.bottom as f32 - min_visible;
    }

    (new_x, new_y, new_w, new_h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

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
        let window_rect = RECT {
            left: 100,
            top: 100,
            right: 300,
            bottom: 200,
        }; // 200x100
        let work_area = RECT {
            left: 0,
            top: 0,
            right: 1000,
            bottom: 1000,
        };
        let result = calculate_centered_rect(window_rect, work_area);
        assert_eq!(result.left, 400);
        assert_eq!(result.top, 450);
        assert_eq!(result.right - result.left, 200);
        assert_eq!(result.bottom - result.top, 100);

        // Oversized sizing: too large for work area (width and height shrunk)
        let window_rect_large = RECT {
            left: 100,
            top: 100,
            right: 1200,
            bottom: 1200,
        }; // 1100x1100
        let result_large = calculate_centered_rect(window_rect_large, work_area);
        assert_eq!(result_large.left, 0);
        assert_eq!(result_large.top, 0);
        assert_eq!(result_large.right, 1000);
        assert_eq!(result_large.bottom, 1000);

        // Oversized width, fitting height
        let window_rect_wide = RECT {
            left: 100,
            top: 100,
            right: 1500,
            bottom: 500,
        }; // 1400x400
        let result_wide = calculate_centered_rect(window_rect_wide, work_area);
        assert_eq!(result_wide.left, 0);
        assert_eq!(result_wide.top, 300);
        assert_eq!(result_wide.right, 1000);
        assert_eq!(result_wide.bottom, 700);
    }

    #[test]
    fn test_live_window_manager_is_taskbar_or_start_menu() {
        use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
        use windows::core::w;

        // Verify invalid/null handles are not classified as taskbar/startmenu
        assert!(!is_taskbar_or_start_menu(HWND::default()));

        // Try to find the system taskbar
        let taskbar_hwnd = unsafe { FindWindowW(w!("Shell_TrayWnd"), None).unwrap_or_default() };
        if !taskbar_hwnd.is_invalid() && !taskbar_hwnd.0.is_null() {
            assert!(is_taskbar_or_start_menu(taskbar_hwnd));
        }

        // Try to find the desktop window (Progman or WorkerW)
        let progman_hwnd = unsafe { FindWindowW(w!("Progman"), None).unwrap_or_default() };
        if !progman_hwnd.is_invalid() && !progman_hwnd.0.is_null() {
            assert!(is_taskbar_or_start_menu(progman_hwnd));
        }
    }

    #[test]
    fn test_calculate_resized_rect() {
        let work_area = RECT {
            left: 0,
            top: 0,
            right: 1000,
            bottom: 1000,
        };
        let vs = RECT {
            left: -5000,
            top: -5000,
            right: 5000,
            bottom: 5000,
        };

        // Test Shift + Right (Expand Right, dx > 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 400.0, 400.0, true, false, // is_shift_down, is_alt_down
            50.0, 0.0, // dx, dy
            96, work_area, vs,
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 450.0);

        // Test Shift + Left (Expand Left, dx < 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 400.0, 400.0, true, false, -50.0, 0.0, 96, work_area, vs,
        );
        assert_eq!(x, 50.0);
        assert_eq!(w, 450.0);

        // Test Alt + Right (Shrink Left edge rightwards, dx > 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 400.0, 400.0, false, true, 50.0, 0.0, 96, work_area, vs,
        );
        assert_eq!(x, 150.0);
        assert_eq!(w, 350.0);

        // Test Alt + Left (Shrink Right edge leftwards, dx < 0)
        let (x, _y, w, _h) = calculate_resized_rect(
            100.0, 100.0, 400.0, 400.0, false, true, -50.0, 0.0, 96, work_area, vs,
        );
        assert_eq!(x, 100.0);
        assert_eq!(w, 350.0);
    }
}
