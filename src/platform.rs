//! Windows platform-specific utilities.
//!
//! Provides helpers for DPI awareness, monitor enumeration, and
//! virtual desktop coordinate math.

use windows::Win32::Foundation::{HWND, RECT};
// use windows::Win32::Graphics::Gdi::HMONITOR;

/*
/// Information about a physical or virtual monitor.
pub struct Monitor {
    pub hmonitor: HMONITOR,
    /// The usable area of the monitor (excluding taskbars).
    pub work_area: RECT,
}
*/

pub struct Platform;

impl Platform {
    /// Returns the logical DPI for a specific window.
    ///
    /// This is used to scale UI elements (like the overlay border)
    /// correctly on high-DPI displays.
    pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
        unsafe { windows::Win32::UI::HiDpi::GetDpiForWindow(hwnd) }
    }

    /*
    /// Enumerates all currently active monitors.
    pub fn get_monitors() -> Vec<Monitor> {
        use windows::Win32::Foundation::LPARAM;
        use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC};

        let mut monitors = Vec::new();

        unsafe {
            // EnumDisplayMonitors calls our callback for every monitor detected.
            let _ = EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(enum_monitor_callback),
                LPARAM(&mut monitors as *mut Vec<Monitor> as isize),
            );
        }

        monitors
    }
    */

    /// Returns the bounding box of the entire virtual desktop.
    ///
    /// This spans all monitors and is used for boundary checking
    /// during window movement.
    pub fn get_virtual_screen_rect() -> RECT {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        };
        unsafe {
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let cx = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let cy = GetSystemMetrics(SM_CYVIRTUALSCREEN);
            RECT {
                left: x,
                top: y,
                right: x + cx,
                bottom: y + cy,
            }
        }
    }

    /// Retrieves the work area (excluding taskbars) of the monitor nearest to the given window.
    pub fn get_nearest_monitor_work_area(hwnd: HWND) -> Result<RECT, windows::core::Error> {
        use windows::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
        };

        unsafe {
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            if hmonitor.is_invalid() {
                return Err(windows::core::Error::from_win32());
            }

            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            GetMonitorInfoW(hmonitor, &mut info).ok()?;
            Ok(info.rcWork)
        }
    }

    /// Checks if the current process is running with Administrative privileges.
    pub fn is_admin() -> bool {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token: HANDLE = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

            let result = windows::Win32::Security::GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                size,
                &mut size,
            );

            let _ = CloseHandle(token);

            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }
}

/*
/// Callback for EnumDisplayMonitors.
unsafe extern "system" fn enum_monitor_callback(
    hmonitor: HMONITOR,
    _: windows::Win32::Graphics::Gdi::HDC,
    _: *mut windows::Win32::Foundation::RECT,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO};

    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<Monitor>) };
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    if unsafe { GetMonitorInfoW(hmonitor, &mut info).as_bool() } {
        monitors.push(Monitor {
            hmonitor,
            work_area: info.rcWork,
        });
    }

    true.into()
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    #[test]
    fn test_get_dpi_for_window() {
        let hwnd = unsafe { GetForegroundWindow() };
        let dpi = Platform::get_dpi_for_window(hwnd);
        if hwnd.0.is_null() {
            assert_eq!(dpi, 0);
        } else {
            assert!(dpi > 0);
        }
    }

    #[test]
    fn test_get_nearest_monitor_work_area() {
        let hwnd = unsafe { GetForegroundWindow() };
        let work_area = Platform::get_nearest_monitor_work_area(hwnd);
        assert!(work_area.is_ok());
        let rect = work_area.unwrap();
        assert!(rect.right > rect.left);
        assert!(rect.bottom > rect.top);
    }

    #[test]
    #[ignore]
    fn test_get_monitors() {
        // let monitors = Platform::get_monitors();
        // assert!(!monitors.is_empty());
    }
}
