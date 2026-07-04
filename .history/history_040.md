# History Log: Split-Phase Rendering for Drag-Free Resizing

*   **Date:** 2026-07-03
*   **Feature:** Keyboard-Driven Window Resizing (Split-Phase Rendering)
*   **Branch:** `feat/keyboard-resizing-sync`

---

## Technical Decisions & Rationale

### 1. Split-Phase Overlay Rendering
*   **Problem:** During window expansion (`Shift + Arrows`), even with atomic positioning (`DeferWindowPos`), the overlay visual content lagged by 1 frame. This was because CPU-bound rendering (tiny-skia canvas rendering, GDI DIB section creation) took 2-3ms to execute *after* the window coordinates had already been updated by the OS. This delayed the pixel update (`UpdateLayeredWindow`) past the DWM's next frame composition boundary, causing a visible lag/dragging effect.
*   **Decision:** Split the rendering flow of the overlay window into two distinct execution phases:
    1.  **Preparation (CPU-bound):** Call a new method `Overlay::prepare_surface` *before* the coordinate transaction is committed. This performs all GDI allocations and tiny-skia rendering into a temporary memory buffer. Returns GDI handles wrapped in a RAII container (`PreparedOverlaySurface`) to guarantee leak-free cleanup on drop.
    2.  **Commit (GPU/Upload):** Call `Overlay::commit_surface` *after* the `DeferWindowPos` transaction completes. This instantly uploads the prepared DIB section via `UpdateLayeredWindow`.
*   **Result:** Reduces the post-layout transaction delay from ~3ms down to under 100 microseconds (just a texture upload/BitBlt). The overlay expands synchronously with the target window with no perceptible lag.

---

## Verification
*   **Unit Tests:** All unit tests pass cleanly.
*   **Aesthetics:** Resizing up (expanding) and resizing down (shrinking) are both visually instant, with the orange border locked to the target window frame.
