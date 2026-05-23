mod window;
mod platform;
mod input;
mod physics;
mod app;
mod ui;

use crossbeam_channel::unbounded;
use crate::input::{InputEvent, InputManager};
use crate::app::App;

fn main() -> windows::core::Result<()> {
    let (tx, rx) = unbounded::<InputEvent>();
    
    // Spawn Input Thread
    std::thread::spawn(move || {
        let manager = InputManager::new(tx).expect("Failed to initialize InputManager");
        manager.run_loop();
    });

    let mut app = App::new(rx);
    println!("win-glide is running. Press Ctrl+Alt+F10 to start (placeholder).");
    app.run();

    Ok(())
}
