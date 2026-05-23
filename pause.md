# win-glide Pause State

## Current Status: Physics Refined (v0.1.9)
The application now features a dual-friction physics model for better acceleration and speed.

### Completed Recently:
- **Overlay Sync & Flicker Fix:** Implemented `DeferWindowPos` to ensure the target window and overlay move atomically. Added `SWP_NOCOPYBITS` to eliminate flicker.
- **Visibility & Timeout Refinement:** Increased minimum off-screen visibility to 150px and idle timeout to 5 seconds.
- **Limited Off-Screen Movement:** Windows can now move partially off-screen.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
