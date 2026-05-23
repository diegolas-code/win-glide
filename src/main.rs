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
    std::thread::spawn(move || {
        let manager = InputManager::new_with_config(tx, hotkey_config).expect("Failed to initialize InputManager");
        manager.run_loop();
    });

    let mut app = App::new(rx, config.physics);
    println!("win-glide is running. Press Ctrl+Alt+F10 (or your configured hotkey) to start.");
    println!("Press Ctrl+C to exit.");
    app.run();

    Ok(())
}
