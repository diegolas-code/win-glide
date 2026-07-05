# History Log: Cache Arrow Paths (Chevron rendering optimization)

*   **Date:** 2026-07-04
*   **Feature:** Resizing and Redraw Smoothness Optimizations (Task 1)
*   **Branch:** `experiment/smoother-resize-redraw`

---

## Technical Decisions & Rationale

### 1. Cached Path Geometries in Overlay Struct
- **Problem:** When resizes are active, arrow indicators/chevrons pointing in the resizing directions are drawn onto the overlay. Previously, these path geometries were rebuilt dynamically on every single frame inside a free-standing `draw_arrow` helper. This caused continuous heap allocations/deallocations of points/verbs vectors at 120Hz.
- **Decision:** Implement private fields inside `Overlay` to cache the compiled `tiny_skia::Path` representations:
  - `cached_arrow_size: Cell<f32>` (for invalidating the cached paths if DPI/monitor scale factor shifts)
  - `cached_path_up`, `cached_path_down`, `cached_path_left`, `cached_path_right` (as `RefCell<Option<tiny_skia::Path>>`)
- **Origin Alignment & Transforms:** The paths are now built relative to `(0, 0)` rather than absolute coordinates. At draw time, we apply a translation transform:
  ```rust
  let transform = tiny_skia::Transform::from_translate(center_x, center_y);
  ```
  This keeps the cached path invariant to screen position changes, making it reusable across all frames.

### 2. Lint Suppressions
- **Decision:** The `draw_arrow` method consumes 8 parameters (`&self`, `pixmap`, `paint`, `center_x`, `center_y`, `size`, `direction`, `dpi_scale`). Added `#[allow(clippy::too_many_arguments)]` to avoid compiler warnings since all of these parameters are highly relevant for configuring path rendering, transforms, and scaling.

---

## Verification
- **Clippy:** Inspected by running `cargo clippy --all-targets --all-features -- -D warnings`. Clean.
- **Unit Tests:** Run `cargo test`. All 20 tests pass successfully.
