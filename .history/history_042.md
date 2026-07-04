# History Log: Overlay Resize Indicators (Arrows)

*   **Date:** 2026-07-04
*   **Feature:** Overlay Resize Indicators
*   **Branch:** `feat/overlay-resize-indicators`

---

## Technical Decisions & Rationale

### 1. Vector-Based Arrow Rendering
*   **Problem:** The user requested visual arrow indicators centered along each window edge to show the resize directions when `Shift` (Expand) or `Alt` (Shrink) modifiers are pressed. Drawing Unicode font glyphs requires loading external system fonts which can be unreliable across different Windows configurations and introduces unwanted dependencies.
*   **Decision:** Implement vector paths representing chevrons/triangles directly in `src/ui.rs`. When modifier states are active:
    *   Compute the DPI-scaled arrow size (`48.0 * scale_factor`).
    *   Position arrows centered vertically/horizontally along the inner borders.
    *   Construct paths using `tiny-skia` and paint them solid white (`Color::from_rgba8(255, 255, 255, 255)`).
*   **Safety Threshold:** Suppress drawing arrows if the window is too small (`width < 3.0 * arrow_size` or `height < 3.0 * arrow_size`) to prevent visual clutter and overlapping.

### 2. Symmetrical Modifier Mappings
*   **Expand Mode (Shift only):** Arrows point outward (Up, Down, Left, Right) relative to their respective edges.
*   **Shrink Mode (Alt only):** Arrows point inward (Down, Up, Right, Left) relative to their respective edges.

### 3. Reactive Event Polling
*   Rather than redrawing at 120Hz constantly (which wastes CPU), `App::run` checks the current modifier state `(is_shift_down, is_alt_down)` via `GetAsyncKeyState` at 120Hz. If it differs from the cached `last_modifiers_state`, a synchronous redraw is triggered immediately.

---

## Verification
*   **Unit Tests:** Added `test_overlay_arrow_rendering` in `src/ui.rs` to verify that `prepare_surface` correctly computes bounds and handles size thresholds. All 18 tests pass successfully.
