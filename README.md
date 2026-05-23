# win-glide

win-glide is a high-performance Windows utility designed for rapid, momentum-based window repositioning using keyboard arrow keys and hybrid mouse control. It prioritizes tactile feedback, precision, and a "snappy yet light" physics model.


## Features
- **Keyboard Control:** Move the active window using arrow keys with fluid acceleration and friction.
- **Hybrid Interaction:** Seamlessly transition between keyboard thrust and mouse-driven "grabbing."
- **High Performance:** Built with Rust and the native Win32 API for minimal latency.
- **DPI & Multi-Monitor Aware:** Scales movement and visuals correctly across different monitors and DPI settings.
- **Visual Overlay:** (Planned) A sleek, transparent 3px border to indicate the active movement session.

## Current State
The project has completed **Phase 1: Foundation**.
- [x] Win32 window capture (Active window handle).
- [x] Platform module (DPI detection & Monitor enumeration).
- [x] Automated CI pipeline (GitHub Actions).
- [ ] Phase 2: Input & Hooks (In Progress).

## Build & Run Requirements
This project targets Windows 10/11 on x86_64 and needs:

- A recent Rust toolchain with `cargo`, installed via [rustup.rs](https://rustup.rs/) for Windows.
- **Microsoft Visual C++ Build Tools (MSVC)** including the Windows 10/11 SDK, available from the [Visual Studio Build Tools installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

Install both prerequisites before running `cargo build` or `cargo run` so the Win32 bindings can link successfully on Windows.

## Building and Running

### 1. Clone the Repository
```bash
git clone git@github.com:diegolas-code/`win-glide`.git
cd `win-glide`
```

### 2. Build the Project
```bash
cargo build
```

### 3. Run the Project
```bash
cargo run
```
*Note: Currently, the application prints "Hello, world!" as the core interaction loop is scheduled for Phase 2.*

### 4. Running Tests
```bash
cargo test
```

## Development Workflow
This project follows **Test-Driven Development (TDD)** and strict idiomatic Rust standards. CI is configured to run `fmt`, `clippy`, and `test` on every push.

## License
[MIT](LICENSE) (Or specify your preferred license)
