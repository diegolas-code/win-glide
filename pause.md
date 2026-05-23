# win-glide Pause State

## Current Status: Physics Refined (v0.1.9)
The application now features a dual-friction physics model for better acceleration and speed.

### Completed Recently:
- **Dual-Friction Model:** Implemented low friction during thrusting and high friction during coasting. This allows the window to reach the full 4,000 pixels/s top speed while still stopping quickly on release.
- **Physics Tuning:** Set acceleration to 4,000 pixels/s² to provide a clear 1-second build-up to max speed.
- **Graceful Shutdown:** Handled Ctrl+C via console control handler.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
