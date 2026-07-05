# History Log: Active Region Clearing / Dirty-Rect Clipping

*   **Date:** 2026-07-04
*   **Feature:** Resizing and Redraw Smoothness Optimizations (Task 4)
*   **Branch:** `experiment/smoother-resize-redraw`

---

## Technical Decisions & Rationale

### 1. Bypassing Full-Screen Pixmap Clearing
- **Problem:** With GDI double-buffers pre-allocated to the virtual screen dimensions, calling `pixmap.fill(Color::TRANSPARENT)` writes zeros to the entire buffer (e.g. 1920x1080 * 4 bytes = 8.3 MB per frame). This creates massive memory bandwidth overhead (~1 GB/s) and severe cache thrashing, leading to the "strobe-like" stuttering during resizes.
- **Decision:** Remove `pixmap.fill()` and clear only the active window rectangle sub-region `(0, 0, width, height)` of the flat slice buffer before passing it to `PixmapMut::from_bytes`.
- **Row-by-Row Memory Zeroing:**
  ```rust
  let stride = current_cache_w as usize * 4;
  let active_row_bytes = width as usize * 4;
  for y in 0..height as usize {
      let row_start = y * stride;
      let row_end = row_start + active_row_bytes;
      if row_end <= slice.len() {
          slice[row_start..row_end].fill(0);
      }
  }
  ```
  This reduces memory writes by up to 90% (for a typical 500x500 window size), vastly improving frame times and rendering speed.
- **Borrow Checker Solution:** To satisfy Rust's borrow checker, this clearing loop operates on `slice` *before* the slice is mutably borrowed by the `PixmapMut::from_bytes` wrapping constructor.

---

## Verification
- **Unit Tests:** Run `cargo test`. All 20 tests pass.
- **Lints:** Clean clippy checks.
