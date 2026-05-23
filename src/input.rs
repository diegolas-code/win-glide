use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT,
};

pub struct HotkeyManager {
    id: i32,
}

impl HotkeyManager {
    pub fn new(id: i32, modifiers: HOT_KEY_MODIFIERS, vk: u32) -> windows::core::Result<Self> {
        unsafe {
            RegisterHotKey(HWND::default(), id, modifiers, vk)?;
        }
        Ok(Self { id })
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterHotKey(HWND::default(), self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_registration() {
        // Use a less common hotkey for testing: Ctrl + Alt + K (0x4B)
        let res = HotkeyManager::new(999, MOD_CONTROL | MOD_SHIFT | MOD_ALT, 0x4B);
        assert!(res.is_ok(), "Hotkey registration failed: {:?}", res.err());
    }
}
