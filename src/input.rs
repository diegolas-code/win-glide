//! Low-level input handling.
//!
//! This module manages Windows hooks (WH_KEYBOARD_LL, WH_MOUSE_LL) and
//! global hotkeys (RegisterHotKey).
//!
//! Key design principles:
//! 1. **Non-blocking Hooks**: LL hooks must return as fast as possible to avoid
//!    system-wide lag. We use a thread-safe channel to send events to the main thread.
//! 2. **Input Interception**: When a session is active, we consume arrow key
//!    events so they don't reach the target window, but we ALWAYS allow
//!    modifiers and KeyUp events to prevent "stuck keys".
//! 3. **Dedicated Thread**: Hooks require a Win32 message loop (GetMessage) to
//!    function, so they run on their own thread.

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, RegisterHotKey, UnregisterHotKey,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, KBDLLHOOKSTRUCT, MSG, PostThreadMessageW,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_HOTKEY, WM_KEYDOWN,
    WM_KEYUP, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_QUIT, WM_RBUTTONDOWN, WM_SYSKEYDOWN, WM_SYSKEYUP,
    WM_XBUTTONDOWN,
};

/// Events sent from the input thread to the main application loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// The global activation hotkey was pressed.
    HotkeyTriggered(i32),
    /// A key was pressed.
    KeyDown(u32),
    /// A key was released.
    KeyUp(u32),
    /// A mouse button was clicked.
    MouseButtonDown,
    /// System shutdown or Ctrl+C signal.
    Shutdown,
}

use crossbeam_channel::Sender;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::System::Console::{CTRL_C_EVENT, SetConsoleCtrlHandler};

/// Global sender for events. Initialized once at startup.
static EVENT_SENDER: OnceLock<Sender<InputEvent>> = OnceLock::new();

/// Atomic flag used by hooks to know if they should intercept input.
static IS_SESSION_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_event_sender(sender: Sender<InputEvent>) {
    if EVENT_SENDER.set(sender).is_err() {
        eprintln!("Warning: EVENT_SENDER was already set and cannot be re-initialized.");
    }
}

/// Registers a handler for console signals like Ctrl+C.
pub fn register_shutdown_handler() -> windows::core::Result<()> {
    unsafe {
        SetConsoleCtrlHandler(Some(console_ctrl_handler), true)?;
    }
    Ok(())
}

/// Native callback for console control events.
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> windows::Win32::Foundation::BOOL {
    if ctrl_type == CTRL_C_EVENT {
        println!("\nShutdown signal received (Ctrl+C)");
        emit_event(InputEvent::Shutdown);
        return windows::Win32::Foundation::BOOL(1);
    }
    windows::Win32::Foundation::BOOL(0)
}

pub fn set_session_active(active: bool) {
    IS_SESSION_ACTIVE.store(active, Ordering::Relaxed);
}

fn emit_event(event: InputEvent) {
    if let Some(sender) = EVENT_SENDER.get()
        && let Err(e) = sender.send(event)
    {
        eprintln!("Failed to send event: {:?}", e);
    }
}

/// Returns true if the VK code corresponds to a modifier key (Shift, Ctrl, Alt, Win).
fn is_modifier(vk_code: u32) -> bool {
    matches!(vk_code, 0x10..=0x12 | 0x5B..=0x5C | 0xA0..=0xA5)
}

/// Low-level keyboard hook callback.
///
/// Intercepts keys system-wide. When a glide session is active, it blocks
/// arrow keys from reaching the target window while reporting them to the app.
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

            // CRITICAL SAFETY RULE:
            // Do NOT block modifier keys or KeyUp events.
            // Blocking these causes "stuck keys" where the OS thinks a key is still
            // down because it never saw the release event. This breaks the desktop.
            if is_modifier(vk_code) || !is_key_down {
                return unsafe { CallNextHookEx(None, code, wparam, lparam) };
            }

            // Consume the input so it doesn't reach the target window.
            // Returning LRESULT(1) tells Windows we handled the event.
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Low-level mouse hook callback.
///
/// Used to detect mouse clicks to automatically deactivate the session.
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

/// RAII wrapper for RegisterHotKey.
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

/// RAII wrapper for SetWindowsHookExW (WH_KEYBOARD_LL).
pub struct KeyboardHook {
    hhook: HHOOK,
}

impl KeyboardHook {
    pub fn new(
        callback: unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT,
    ) -> windows::core::Result<Self> {
        let hhook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), None, 0)? };
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

/// RAII wrapper for SetWindowsHookExW (WH_MOUSE_LL).
pub struct MouseHook {
    hhook: HHOOK,
}

impl MouseHook {
    pub fn new(
        callback: unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT,
    ) -> windows::core::Result<Self> {
        let hhook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(callback), None, 0)? };
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

/// Orchestrates input threads and hooks.
pub struct InputManager {
    _hotkey: HotkeyManager,
    _center_hotkey: Option<HotkeyManager>,
    _kbd_hook: KeyboardHook,
    _mouse_hook: MouseHook,
    thread_id: u32,
}

unsafe impl Send for InputManager {}
unsafe impl Sync for InputManager {}

impl InputManager {
    /// Initializes all hooks and hotkeys based on the provided configuration.
    pub fn new_with_config(
        sender: Sender<InputEvent>,
        hotkey_config: HotkeyConfig,
        center_hotkey_config: HotkeyConfig,
    ) -> windows::core::Result<Self> {
        set_event_sender(sender);

        let hotkey = HotkeyManager::new(1337, HOT_KEY_MODIFIERS(hotkey_config.modifiers), hotkey_config.vk)?;
        
        let center_hotkey = match HotkeyManager::new(1338, HOT_KEY_MODIFIERS(center_hotkey_config.modifiers), center_hotkey_config.vk) {
            Ok(hk) => Some(hk),
            Err(e) => {
                eprintln!("\nWARNING: Failed to register window center hotkey (Ctrl+Win+C): {}.", e);
                eprintln!("The centering feature will be disabled. You can disable Windows Color Filters in Settings, or configure a different hotkey in config.json.\n");
                None
            }
        };

        let kbd_hook = KeyboardHook::new(keyboard_proc)?;
        let mouse_hook = MouseHook::new(mouse_proc)?;
        let thread_id = unsafe { GetCurrentThreadId() };

        Ok(Self {
            _hotkey: hotkey,
            _center_hotkey: center_hotkey,
            _kbd_hook: kbd_hook,
            _mouse_hook: mouse_hook,
            thread_id,
        })
    }

    /// Runs the Win32 message loop required for hooks to process events.
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

    /// Signals the message loop to exit.
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
        assert!(
            res.is_ok(),
            "Keyboard hook registration failed: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_mouse_hook_registration() {
        let res = MouseHook::new(test_hook_proc);
        assert!(
            res.is_ok(),
            "Mouse hook registration failed: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_global_dispatcher() {
        let (tx, rx) = crossbeam_channel::unbounded();
        // This might fail if another test already set it, but we'll try.
        set_event_sender(tx);

        emit_event(InputEvent::KeyDown(0x5A)); // 'Z'
        let event = rx.recv().unwrap();
        assert_eq!(event, InputEvent::KeyDown(0x5A));
    }

    #[test]
    fn test_center_hotkey_registration() {
        // Register Ctrl + Win + Z (0x5A) as a test hotkey
        let res = HotkeyManager::new(998, MOD_CONTROL | HOT_KEY_MODIFIERS(0x0008), 0x5A);
        assert!(res.is_ok(), "Center hotkey registration failed: {:?}", res.err());
    }

    #[test]
    #[ignore]
    fn test_input_manager_initialization() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let config = crate::config::Config::default();
        let manager = InputManager::new_with_config(tx, config.hotkey, config.center_hotkey);
        assert!(
            manager.is_ok(),
            "InputManager creation failed: {:?}",
            manager.err()
        );
    }
}
