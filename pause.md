# win-glide Pause State

## Current Status: Movement Improved (v0.1.6)
The application now features smooth 120Hz continuous movement, tuned physics, and perfectly symmetric speed.

### Completed Recently:
- **Asymmetry Fix:** Resolved issue where movement was faster left/up than right/down by switching to rounded integer conversion and removing redundant position checks.
- **Continuous Thrust:** Implemented state-based key tracking to allow smooth 120Hz acceleration.
- **Diagonal Normalization:** Fixed issue where diagonal movement was faster than cardinal movement.
- **Physics Tuning:** Increased acceleration (10,000) and top speed (2,500) to match the "Snappy & Light" spec.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
