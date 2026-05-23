# win-glide

win-glide is a high-performance Windows utility designed for rapid, momentum-based window repositioning using keyboard arrow keys. It prioritizes tactile feedback, precision, and a "snappy yet light" physics model, allowing you to "glide" windows across your desktop with ease.

## Core Features
- **Fluid Keyboard Movement:** Move active windows using arrow keys with high-precision acceleration and friction, powered by a 120Hz physics loop.
- **Snappy & Light Physics:** Uses a **Dual-Friction Model** for slow, deliberate acceleration to high speeds while maintaining a nearly instant "glide stop" upon release.
- **Modern Visual Feedback:** A full-window semi-transparent blue tint clearly identifies the active glide target.
- **Free Multi-Monitor Movement:** Glide windows seamlessly across your entire virtual desktop. Windows can be "parked" partially off-screen while maintaining a safe 150px visible margin.
- **Safe & Responsive:**
    - **Maximized Window Guard:** Prevents accidental movement of maximized windows.
    - **Panic Exit:** Any keyboard input or mouse click immediately deactivates the glide session for instant control recovery.
    - **Shutdown:** Cleanly exits and releases system hooks on `Ctrl+C`.

## How to Use
1.  **Launch:** Run the application (`cargo run` or the compiled binary).
2.  **Activate:** Press **`Ctrl + Alt + F10`** while any window is focused to start a "glide" session. A blue tint will appear over the window.
3.  **Move:** Use the **Arrow Keys** to apply thrust. Acceleration is continuous; hold the keys to reach top speed (~1.3s spin-up).
4.  **Exit:** 
    - **Keys:** Press **`Esc`** or **any non-arrow key** to simply let the window glide to a stop.
    - **Mouse:** **Click anywhere** to instantly deactivate the session.
    - **Timeout:** The session automatically ends after **5 seconds** of inactivity.
    - **Focus Loss:** Switching windows or losing focus will also end the session.

## Configuration
Upon first run, `win-glide` generates a `config.json` in the application directory. You can customize the physics and hotkeys:

```json
{
  "physics": {
    "acceleration": 3000.0,
    "friction": 10.0,
    "thrust_friction": 0.5,
    "top_speed": 4000.0
  },
  "hotkey": {
    "modifiers": 3,
    "vk": 121
  }
}
```
*Note: Acceleration and top speed are in pixels per second. Modifiers (Default: Ctrl+Alt) and Virtual Key (vk) codes follow Win32 standards.*

## Build & Run Requirements
This project targets Windows 10/11 on x86_64 and needs:

- A recent Rust toolchain with `cargo` (installed via [rustup.rs](https://rustup.rs/)).
- **Microsoft Visual C++ Build Tools (MSVC)** including the Windows 10/11 SDK.

### Build and Run
```bash
cargo build --release
./target/release/win-glide.exe
```

### Run Tests
```bash
cargo test
```

## Development Workflow
This project follows **strict idiomatic Rust standards** and mandatory documentation habits. Detailed technical decisions and bug fixes are recorded in the `.history/` directory.

## License
This project is licensed under the [MIT License](LICENSE).
