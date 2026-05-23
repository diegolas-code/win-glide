# win-glide Pause State

## Current Status: UX Refined (v0.1.10)
The application now provides more intuitive control with "any key to stop" and optimized window guards.

### Completed Recently:
- **Exit on Any Key:** Any key press other than arrow keys (and modifiers) now immediately terminates the glide session.
- **Maximized Window Constraint:** Added a check to prevent gliding functionality when the active window is maximized.
- **Advanced Flicker Reduction:** Implemented dynamic window ownership and added aggressive suppression flags.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
