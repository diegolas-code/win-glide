//! # win-glide
//!
//! win-glide is a high-performance Windows utility for rapid, momentum-based
//! window repositioning using keyboard arrow keys.
//!
//! This entry point initializes the configuration, sets up the communication channels,
//! launches the low-level input hook thread, and starts the main application loop.

mod app;
mod config;
mod input;
mod physics;
mod platform;
mod ui;
mod window;

use crate::app::App;
use crate::config::Config;
use crate::input::{InputEvent, InputManager, register_shutdown_handler};
use crossbeam_channel::unbounded;

/// RAII guard to set Windows timer resolution to 1ms on startup and restore it on shutdown.
struct TimerResolutionGuard;

impl TimerResolutionGuard {
    fn new() -> Self {
        #[link(name = "winmm")]
        unsafe extern "system" {
            fn timeBeginPeriod(uPeriod: u32) -> u32;
        }
        unsafe {
            let _ = timeBeginPeriod(1);
        }
        Self
    }
}

impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        #[link(name = "winmm")]
        unsafe extern "system" {
            fn timeEndPeriod(uPeriod: u32) -> u32;
        }
        unsafe {
            let _ = timeEndPeriod(1);
        }
    }
}

/// The main entry point for the win-glide application.
///
/// It orchestrates the startup sequence:
/// 1. Load configuration from `config.json` or use defaults.
/// 2. Initialize a thread-safe channel for input events.
/// 3. Register a console shutdown handler (Ctrl+C).
/// 4. Spawn a dedicated thread for low-level keyboard/mouse hooks to ensure zero latency.
/// 5. Initialize and run the main application state machine.
fn main() -> windows::core::Result<()> {
    let _timer_guard = TimerResolutionGuard::new();

    // Load user preferences for physics and hotkeys.
    let config = Config::load();

    // Communication channel between the low-level input thread and the main application loop.
    let (tx, rx) = unbounded::<InputEvent>();

    // Register Ctrl+C handler to ensure clean hook unregistration on exit.
    register_shutdown_handler()?;

    // Spawn the Input Thread.
    // Low-level Win32 hooks (WH_KEYBOARD_LL) require a message pump and should
    // ideally run on a dedicated thread to avoid blocking or being blocked by
    // the main application logic.
    let hotkey_config = config.hotkey.clone();
    let center_hotkey_config = config.center_hotkey.clone();
    let (input_ready_tx, input_ready_rx) =
        unbounded::<Result<std::sync::Arc<InputManager>, windows::core::Error>>();

    std::thread::spawn(move || {
        // Initialize the InputManager which sets up the Win32 hooks.
        match InputManager::new_with_config(tx, hotkey_config, center_hotkey_config) {
            Ok(manager) => {
                let manager = std::sync::Arc::new(manager);
                // Signal that the input manager is ready.
                let _ = input_ready_tx.send(Ok(manager.clone()));
                // Start the Win32 message loop required for hooks.
                manager.run_loop();
            }
            Err(e) => {
                let _ = input_ready_tx.send(Err(e));
            }
        }
    });

    // Wait for the input thread to initialize before starting the app.
    let input_manager = match input_ready_rx
        .recv()
        .expect("Failed to receive InputManager initialization status")
    {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!(
                "\nCRITICAL ERROR: Failed to initialize win-glide input hooks: {}.",
                e
            );
            eprintln!(
                "Please make sure the activation hotkey is not already registered by another application.\n"
            );
            std::process::exit(1);
        }
    };

    // Initialize the main application with the receiver end of the event channel.
    let mut app = App::new(rx, &config, input_manager);

    if !crate::platform::Platform::is_admin() {
        println!("INFO: win-glide is running with standard user privileges.");
        println!(
            "Interaction with high-integrity windows (like Task Manager) will be restricted by Windows security."
        );
    }

    println!(
        "win-glide is running. Press {} to start glide, or {} to center the window.",
        config.hotkey, config.center_hotkey
    );
    println!("Press Ctrl+C to exit.");

    // Run the main application loop (event processing and physics).
    app.run();

    Ok(())
}
