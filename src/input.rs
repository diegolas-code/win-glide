use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_KEYBOARD_LL, WH_MOUSE_LL,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    HotkeyTriggered(i32),
    KeyDown(u32),
    KeyUp(u32),
    MouseMove { dx: i32, dy: i32 },
}

use std::sync::OnceLock;
use crossbeam_channel::Sender;

static EVENT_SENDER: OnceLock<Sender<InputEvent>> = OnceLock::new();

pub fn set_event_sender(sender: Sender<InputEvent>) -> Result<(), Sender<InputEvent>> {
    EVENT_SENDER.set(sender)
}

fn emit_event(event: InputEvent) {
    if let Some(sender) = EVENT_SENDER.get() {
        let _ = sender.send(event);
    }
}

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

pub struct KeyboardHook {
    hhook: HHOOK,
}

impl KeyboardHook {
    pub fn new(callback: unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT) -> windows::core::Result<Self> {
        let hhook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), None, 0)?
        };
        Ok(Self { hhook })
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWindowsHookEx(self.hhook);
        }
    }
}

pub struct MouseHook {
    hhook: HHOOK,
}

impl MouseHook {
    pub fn new(callback: unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT) -> windows::core::Result<Self> {
        let hhook = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(callback), None, 0)?
        };
        Ok(Self { hhook })
    }
}

impl Drop for MouseHook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWindowsHookEx(self.hhook);
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

    unsafe extern "system" fn test_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
    }

    #[test]
    fn test_keyboard_hook_registration() {
        let res = KeyboardHook::new(test_hook_proc);
        assert!(res.is_ok(), "Keyboard hook registration failed: {:?}", res.err());
    }

    #[test]
    fn test_mouse_hook_registration() {
        let res = MouseHook::new(test_hook_proc);
        assert!(res.is_ok(), "Mouse hook registration failed: {:?}", res.err());
    }

    #[test]
    fn test_global_dispatcher() {
        let (tx, rx) = crossbeam_channel::unbounded();
        // This might fail if another test already set it, but we'll try.
        let _ = set_event_sender(tx);
        
        emit_event(InputEvent::KeyDown(0x5A)); // 'Z'
        let event = rx.recv().unwrap();
        assert_eq!(event, InputEvent::KeyDown(0x5A));
    }
}
