use windows::Win32::Foundation::HWND;

pub fn get_active_window() -> HWND {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    #[test]
    fn test_get_active_window() {
        let active_window = get_active_window();
        let expected_window = unsafe { GetForegroundWindow() };
        assert_eq!(active_window, expected_window);
    }
}
