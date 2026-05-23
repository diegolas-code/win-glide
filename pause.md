# win-glide Pause State

## Current Status: Physics Refined (v0.1.9)
The application now features a dual-friction physics model for better acceleration and speed.

### Completed Recently:
- **Advanced Flicker Reduction:** Implemented dynamic window ownership and added `SWP_DEFERERASE` / `SWP_NOSENDCHANGING` flags to eliminate lingering flicker during high-speed gliding.
- **Overlay Sync:** Switched to `DeferWindowPos` for atomic movement of the window and overlay.
- **Visibility & Timeout Refinement:** Increased minimum off-screen visibility to 150px and idle timeout to 5 seconds.

### Next Steps / Future Ideas:
- **Visual Effects:** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
