# History Log: Overlay Resize Indicators (Chevrons)

*   **Date:** 2026-07-04
*   **Feature:** Overlay Resize Indicators
*   **Branch:** `feat/overlay-resize-indicators`

---

## Technical Decisions & Rationale

### 1. Vector-Based Chevron Rendering
*   **Problem:** The user requested visual arrow indicators centered along each window edge to show the resize directions when `Shift` (Expand) or `Alt` (Shrink) modifiers are pressed.
*   **Decision:** Implement vector V-shaped paths (chevrons) directly in `src/ui.rs`. When modifier states are active:
    *   Compute the DPI-scaled arrow size (`36.0 * scale_factor`).
    *   Position arrows centered vertically/horizontally along the inner borders.
    *   Construct flatter, wider open V-paths using `tiny-skia` by using separate horizontal and vertical offsets: `half_w = size / 2.0` and `half_h = size / 4.0` (producing a modern $120^{\circ}$ angle).
    *   Render them using `stroke_path` with a bold thickness of `8.0 * scale_factor`, `LineCap::Round`, and `LineJoin::Round` to ensure smooth ends.
    *   Use 80% opacity (`Color::from_rgba8(255, 255, 255, 204)`) to let underlying contents shine through.
    *   Increase the margin offset from borders from 15px to 30px (DPI-scaled) to give the chevrons more breathing room.
*   **Safety Threshold:** Suppress drawing arrows if the window is too small (`width < 3.0 * arrow_size` or `height < 3.0 * arrow_size`) to prevent visual clutter and overlapping.

### 2. Symmetrical Modifier Mappings
*   **Expand Mode (Shift only):** Chevrons point outward (Up, Down, Left, Right) relative to their respective edges.
*   **Shrink Mode (Alt only):** Chevrons point inward (Down, Up, Right, Left) relative to their respective edges.

### 3. Reactive Event Polling
*   Rather than redrawing at 120Hz constantly (which wastes CPU), `App::run` checks the current modifier state `(is_shift_down, is_alt_down)` via `GetAsyncKeyState` at 120Hz. If it differs from the cached `last_modifiers_state`, a synchronous redraw is triggered immediately.

---

## Verification
*   **Unit Tests:** Added `test_overlay_arrow_rendering` in `src/ui.rs` to verify that `prepare_surface` correctly computes bounds and handles size thresholds. All 18 tests pass successfully.
