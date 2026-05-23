mod window;
mod platform;
mod input;
mod physics;
mod app;
mod ui;
mod config;

use crossbeam_channel::unbounded;
use crate::input::{InputEvent, InputManager, register_shutdown_handler};
use crate::app::App;
use crate::config::Config;

fn main() -> windows::core::Result<()> {
    let config = Config::load();
    let (tx, rx) = unbounded::<InputEvent>();
    
    // Register Ctrl+C handler
    register_shutdown_handler()?;

    // Spawn Input Thread
    let hotkey_config = config.hotkey.clone();
    let (input_ready_tx, input_ready_rx) = unbounded::<std::sync::Arc<InputManager>>();
    std::thread::spawn(move || {
        let manager = std::sync::Arc::new(InputManager::new_with_config(tx, hotkey_config).expect("Failed to initialize InputManager"));
        input_ready_tx.send(manager.clone()).unwrap();
        manager.run_loop();
    });

    let input_manager = input_ready_rx.recv().expect("Failed to receive InputManager");
    let mut app = App::new(rx, config.physics, input_manager);
    println!("win-glide is running. Press Ctrl+Alt+F10 (or your configured hotkey) to start.");
    println!("Press Ctrl+C to exit.");
    app.run();

    Ok(())
}
