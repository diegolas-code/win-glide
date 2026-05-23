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

/// The main entry point for the win-glide application.
///
/// It orchestrates the startup sequence:
/// 1. Load configuration from `config.json` or use defaults.
/// 2. Initialize a thread-safe channel for input events.
/// 3. Register a console shutdown handler (Ctrl+C).
/// 4. Spawn a dedicated thread for low-level keyboard/mouse hooks to ensure zero latency.
/// 5. Initialize and run the main application state machine.
fn main() -> windows::core::Result<()> {
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
    let (input_ready_tx, input_ready_rx) = unbounded::<std::sync::Arc<InputManager>>();

    std::thread::spawn(move || {
        // Initialize the InputManager which sets up the Win32 hooks.
        let manager = std::sync::Arc::new(
            InputManager::new_with_config(tx, hotkey_config)
                .expect("Failed to initialize InputManager"),
        );

        // Signal that the input manager is ready.
        input_ready_tx.send(manager.clone()).unwrap();

        // Start the Win32 message loop required for hooks.
        manager.run_loop();
    });

    // Wait for the input thread to initialize before starting the app.
    let input_manager = input_ready_rx
        .recv()
        .expect("Failed to receive InputManager");

    // Initialize the main application with the receiver end of the event channel.
    let mut app = App::new(rx, config.physics, input_manager);

    println!("win-glide is running. Press Ctrl+Alt+F10 (or your configured hotkey) to start.");
    println!("Press Ctrl+C to exit.");

    // Run the main application loop (event processing and physics).
    app.run();

    Ok(())
}
