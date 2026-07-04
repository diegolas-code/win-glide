# History Log: Continuous Resize Cache Optimization

*   **Date:** 2026-07-04
*   **Feature:** High-Performance Continuous Resize Cache Optimization
*   **Branch:** `feat/optimize-continuous-resize`

---

## Technical Decisions & Rationale

### 1. Capacity-Based Cache Validation
*   **Problem:** Standard overlay rendering checked for exact dimension matches (`cache_w == width`). When continuous resizing changed window dimensions on every frame, the GDI cache validation failed on every frame, forcing a complete recreation of the compatible DC and DIB section at 120Hz.
*   **Decision:** Change cache validation to a capacity check (`cache_w >= width && cache_h >= height`). If the cached DIB section is already larger than or equal to the requested size, it is immediately reused.

### 2. Grow Padding Buffer (+256px Growth)
*   **Problem:** If the window grew continuously, capacity validation would fail on every frame of expansion.
*   **Decision:** On a cache miss (i.e. window grows larger than capacity), allocate a DIB section with `+256px` padding along both dimensions (`width + 256` by `height + 256`). This ensures subsequent growing operations of up to 256px are absorbed without re-allocating any GDI handles.

### 3. Stride-Safe Buffer Matching & Sub-rect Commits
*   **Decision:** Instantiate the `tiny-skia` `PixmapMut` using the over-allocated buffer size (`current_cache_w` and `current_cache_h`) to maintain byte alignment and pitch constraints. Scale the row stride in post-processing to `current_cache_w * 4`. When committing the surface, call `UpdateLayeredWindow` using only the actual window bounds (`width` and `height`) as the destination size to copy only the active top-left portion.

---

## Verification
*   **Unit Tests:** Updated `test_overlay_gdi_caching` in `src/ui.rs` to verify that smaller window sizes reuse the cached DC under the capacity model, and that larger window sizes trigger a padded reallocation. All 19 tests pass successfully.
