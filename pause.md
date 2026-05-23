# win-glide Pause State

## Current Status: Shutdown Improved (v0.1.7)
The application now exits gracefully on Ctrl+C.

### Completed Recently:
- **Graceful Shutdown:** Added a Windows console control handler to catch Ctrl+C and shut down the application cleanly.
- **Asymmetry Fix:** Resolved issue where movement was faster left/up than right/down.
- **Continuous Thrust:** Implemented state-based key tracking for smooth 120Hz acceleration.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
