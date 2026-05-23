# win-glide Pause State

## Current Status: Overlay Refined (v0.1.3)
The application visuals have been updated to use a modern full-window tint instead of a simple border.

### Completed Recently:
- **Overlay Refinement:** Replaced the 3px blue border with a 20% opacity solid blue tint covering the entire active window.
- **Code Cleanup:** Removed unused rendering imports and simplified `src/ui.rs`.
- **Keyboard Hook Fix:** Resolved the "stuck modifier" bug by ensuring `KeyUp` and modifier events are never suppressed.
- **Mouse Input:** Switched from movement tracking to click-to-deactivate.

### Next Steps / Future Ideas:
- **Phase 6 (Potential):** Add subtle pulsing or "fade-in" animation for the tint.
- **Optimization:** Reduce CPU usage of the physics loop when idle.
- **Packaging:** Create a release build and potentially a simple installer.

## System Context:
- Operating System: Windows 10/11
- Language: Rust 2024
- Key Dependencies: `windows-rs`, `tiny-skia`, `crossbeam-channel`
