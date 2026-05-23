# win-glide

win-glide is a high-performance Windows utility designed for rapid, momentum-based window repositioning using keyboard arrow keys and hybrid mouse control. It prioritizes tactile feedback, precision, and a "snappy yet light" physics model.

## Features
- **Keyboard Thrust:** Move the active window using arrow keys with fluid acceleration and friction (~120Hz physics loop).
- **Global Activation:** Toggle movement sessions instantly using a customizable global hotkey (Default: `Ctrl + Alt + F10`).
- **Visual Feedback:** A sleek, transparent 3px blue border highlights the window currently being moved.
- **DPI & Multi-Monitor Aware:** Automatically scales movement speed and visuals based on monitor DPI; respects work areas and monitor boundaries.
- **Configurable:** Fine-tune acceleration, friction, top speed, and hotkeys via a simple `config.json` file.
- **Safe & Non-Blocking:** Built with Rust and native low-level Win32 hooks (`WH_KEYBOARD_LL`, `WH_MOUSE_LL`) for zero-latency interaction.

## How to Use
1.  **Launch:** Run the application (`cargo run` or the compiled binary).
2.  **Activate:** Press **`Ctrl + Alt + F10`** while any window is focused to start a "glide" session. A blue border will appear.
3.  **Move:** Use the **Arrow Keys** to apply thrust to the window.
4.  **Exit:** Press **`Esc`**, click away (focus loss), or wait 3 seconds for the **Idle Timeout** to automatically end the session.

## Configuration
Upon first run, `win-glide` generates a `config.json` in the application directory. You can customize the following:

```json
{
  "physics": {
    "acceleration": 2000.0,
    "friction": 10.0,
    "top_speed": 1500.0
  },
  "hotkey": {
    "modifiers": 3,
    "vk": 121
  }
}
```
*Note: Modifiers and Virtual Key (vk) codes follow Win32 standards.*

## Build & Run Requirements
This project targets Windows 10/11 on x86_64 and needs:

- A recent Rust toolchain with `cargo`, installed via [rustup.rs](https://rustup.rs/) for Windows.
- **Microsoft Visual C++ Build Tools (MSVC)** including the Windows 10/11 SDK, available from the [Visual Studio Build Tools installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

Install both prerequisites before running `cargo build` or `cargo run` so the Win32 bindings can link successfully on Windows.

## Installation & Development

### 1. Clone the Repository
```bash
git clone git@github.com:diegolas-code/win-glide.git
cd win-glide
```

### 2. Build and Run
```bash
cargo build --release
./target/release/win-glide.exe
```

### 3. Run Tests
```bash
cargo test
```

## Development Workflow
This project follows **Test-Driven Development (TDD)** and strict idiomatic Rust standards. The repository uses a `dev` branch for features and `master` for stable releases. CI is configured via GitHub Actions to run `fmt`, `clippy`, and `test` on every push.

## License
This project is licensed under the [MIT License](LICENSE).
