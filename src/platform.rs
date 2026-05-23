use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::HMONITOR;

pub struct Monitor {
    pub hmonitor: HMONITOR,
    pub work_area: RECT,
}

pub struct Platform;

impl Platform {
    pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
        unsafe { windows::Win32::UI::HiDpi::GetDpiForWindow(hwnd) }
    }

    pub fn get_monitors() -> Vec<Monitor> {
        use windows::Win32::Foundation::LPARAM;
        use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC};

        let mut monitors = Vec::new();

        unsafe {
            let _ = EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(enum_monitor_callback),
                LPARAM(&mut monitors as *mut Vec<Monitor> as isize),
            );
        }

        monitors
    }

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
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    #[test]
    fn test_get_dpi_for_window() {
        let hwnd = unsafe { GetForegroundWindow() };
        let dpi = Platform::get_dpi_for_window(hwnd);
        assert!(dpi > 0);
    }

    #[test]
    fn test_get_monitors() {
        let monitors = Platform::get_monitors();
        assert!(!monitors.is_empty());
    }
}
