# win-glide Pause State

## Current Status: Movement Improved (v0.1.5)
The application now features smooth 120Hz continuous movement and tuned physics.

### Completed Recently:
- **Continuous Thrust:** Implemented state-based key tracking to allow smooth 120Hz acceleration, bypassing OS repeat rates.
- **Diagonal Normalization:** Fixed issue where diagonal movement was faster than cardinal movement.
- **Physics Tuning:** Increased acceleration (10,000) and top speed (2,500) to match the "Snappy & Light" spec.
- **Overlay Refinement:** Added full-window blue tint and 10px top extension.

### Next Steps / Future Ideas:
- **Fluidity Polish:** Investigate potential jitter during boundary clamping or high-speed `SetWindowPos` calls.
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
