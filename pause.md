# win-glide Pause State

## Current Status: Physics Refined (v0.1.9)
The application now features a dual-friction physics model for better acceleration and speed.

### Completed Recently:
- **Limited Off-Screen Movement:** Windows can now move partially off-screen, but are capped such that 50px always remains visible on the virtual desktop.
- **No Border Limits:** Removed monitor boundary clamping.
- **Physics Refinement:** Reduced acceleration to 3,000 pixels/s² for a smoother 1.33s spin-up time.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
