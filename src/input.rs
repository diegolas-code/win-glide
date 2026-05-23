use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_HOTKEY,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_QUIT, WM_RBUTTONDOWN, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WM_XBUTTONDOWN,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    HotkeyTriggered(i32),
    KeyDown(u32),
    KeyUp(u32),
    MouseButtonDown,
    Shutdown,
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use crossbeam_channel::Sender;
use windows::Win32::System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT};

static EVENT_SENDER: OnceLock<Sender<InputEvent>> = OnceLock::new();
static IS_SESSION_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_event_sender(sender: Sender<InputEvent>) {
    let _ = EVENT_SENDER.set(sender);
}

pub fn register_shutdown_handler() -> windows::core::Result<()> {
    unsafe {
        SetConsoleCtrlHandler(Some(console_ctrl_handler), true)?;
    }
    Ok(())
}

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> windows::Win32::Foundation::BOOL {
    if ctrl_type == CTRL_C_EVENT {
        println!("\nShutdown signal received (Ctrl+C)");
        emit_event(InputEvent::Shutdown);
        return windows::Win32::Foundation::BOOL(1); // Handle the event
    }
    windows::Win32::Foundation::BOOL(0)
}

pub fn set_session_active(active: bool) {
    IS_SESSION_ACTIVE.store(active, Ordering::Relaxed);
}

fn emit_event(event: InputEvent) {
    if let Some(sender) = EVENT_SENDER.get() {
        if let Err(e) = sender.send(event) {
            eprintln!("Failed to send event: {:?}", e);
        }
    }
}

fn is_modifier(vk_code: u32) -> bool {
    matches!(vk_code, 0x10..=0x12 | 0x5B..=0x5C | 0xA0..=0xA5)
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let is_active = IS_SESSION_ACTIVE.load(Ordering::Relaxed);
        
        if is_active {
            let kbd_struct = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
            let vk_code = kbd_struct.vkCode;
            
            let is_key_down = match wparam.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    emit_event(InputEvent::KeyDown(vk_code));
                    true
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    emit_event(InputEvent::KeyUp(vk_code));
                    false
                }
                _ => return unsafe { CallNextHookEx(None, code, wparam, lparam) },
            };

            // DO NOT block modifier keys or KeyUp events to avoid "stuck" state.
            // Stuck modifiers are especially bad as they change the meaning of subsequent keys
            // (e.g., stuck Ctrl makes Esc behave like the Windows key).
            if is_modifier(vk_code) || !is_key_down {
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // Consume the input so it doesn't reach the target window
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && IS_SESSION_ACTIVE.load(Ordering::Relaxed) {
        let msg = wparam.0 as u32;
        if msg == WM_LBUTTONDOWN
            || msg == WM_RBUTTONDOWN
            || msg == WM_MBUTTONDOWN
            || msg == WM_XBUTTONDOWN
        {
            emit_event(InputEvent::MouseButtonDown);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
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

use crate::config::HotkeyConfig;

pub struct InputManager {
    _hotkey: HotkeyManager,
    _kbd_hook: KeyboardHook,
    _mouse_hook: MouseHook,
    thread_id: u32,
}

unsafe impl Send for InputManager {}
unsafe impl Sync for InputManager {}

impl InputManager {
    pub fn new_with_config(sender: Sender<InputEvent>, config: HotkeyConfig) -> windows::core::Result<Self> {
        let _ = set_event_sender(sender);

        let hotkey = HotkeyManager::new(1337, HOT_KEY_MODIFIERS(config.modifiers), config.vk)?;
        let kbd_hook = KeyboardHook::new(keyboard_proc)?;
        let mouse_hook = MouseHook::new(mouse_proc)?;
        let thread_id = unsafe { GetCurrentThreadId() };

        Ok(Self {
            _hotkey: hotkey,
            _kbd_hook: kbd_hook,
            _mouse_hook: mouse_hook,
            thread_id,
        })
    }

    pub fn run_loop(&self) {
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                if msg.message == WM_HOTKEY {
                    println!("InputManager: Hotkey received (ID: {})", msg.wParam.0);
                    emit_event(InputEvent::HotkeyTriggered(msg.wParam.0 as i32));
                }
                let _ = DispatchMessageW(&msg);
            }
        }
        println!("InputManager: Message loop exited.");
    }

    pub fn request_stop(&self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_SHIFT};

    #[test]
    fn test_hotkey_registration() {
        // Use a less common hotkey for testing: Ctrl + Alt + K (0x4B)
        let res = HotkeyManager::new(999, MOD_CONTROL | MOD_SHIFT | MOD_ALT, 0x4B);
        assert!(res.is_ok(), "Hotkey registration failed: {:?}", res.err());
    }

    unsafe extern "system" fn test_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
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

    #[test]
    fn test_input_manager_initialization() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let manager = InputManager::new_with_config(tx, crate::config::Config::default().hotkey);
        assert!(manager.is_ok(), "InputManager creation failed: {:?}", manager.err());
    }
}
