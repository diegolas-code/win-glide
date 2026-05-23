# win-glide Pause State

## Current Status: Physics Tuned (v0.1.8)
The application now has a higher top speed and more weighted acceleration.

### Completed Recently:
- **Physics Tuning:** Increased top speed to 4,000 pixels/s and decreased acceleration to 6,000 pixels/s² for a more powerful feel and longer spin-up time.
- **Graceful Shutdown:** Added a Windows console control handler to catch Ctrl+C.
- **Asymmetry Fix:** Resolved issue where movement was faster left/up than right/down.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
