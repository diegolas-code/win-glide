# Current Status - win-glide

## Recent Achievements
- **Phase 7: Visual Polish & UX (Refinements)**
    - Reduced overlay "header" extension from 10px to 7px for a tighter fit.
    - Implemented rounded corners (8px radius) for the blue tinted overlay using `tiny-skia` paths.
- **Phase 8: Performance & Optimization**
    - **Zero-Copy Rendering Pipeline:** Refactored the overlay rendering to use `tiny-skia`'s `PixmapMut::from_bytes`. This allows rendering directly into the memory allocated by the Win32 GDI DIB section, completely eliminating a full-frame memory copy.
    - **Instant Activation:** Optimized the hotkey trigger path by forcing an immediate Win32 message pump, ensuring the overlay appears with zero perceived latency.

## Immediate Next Steps
- **Phase 8: Optimization Continued**
    - Implement "Sleep Mode" for the physics loop to reduce CPU usage when stationary.
    - Further optimize redraw logic to skip `UpdateLayeredWindow` if the window state hasn't changed.

## Technical Notes
- Zero-copy rendering uses `std::slice::from_raw_parts_mut` to wrap GDI memory.
- Pre-swapped color components (`BGRA` instead of `RGBA`) further optimize the pixel pipeline.
