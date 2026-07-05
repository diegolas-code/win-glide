# History Log: Double-Buffered GDI Buffers for Tearing-Free Overlay

*   **Date:** 2026-07-04
*   **Feature:** Resizing and Redraw Smoothness Optimizations (Task 2)
*   **Branch:** `experiment/smoother-resize-redraw`

---

## Technical Decisions & Rationale

### 1. Dual GdiBuffer Implementation
- **Problem:** Writing rendering updates directly into the shared GDI DIB section selected by the target window can cause transient flickering or tearing. This happens if the DWM/OS composer accesses the source device context in the middle of our clearing, path filling, GDI text drawing, or alpha post-processing steps.
- **Decision:** Introduce a helper struct `GdiBuffer` containing GDI resources for a single rendering target:
  - `dc: HDC`
  - `bitmap: HBITMAP`
  - `old_bitmap: HGDIOBJ`
  - `bits: *mut u8`
- **Double Buffering:** Store two buffer instances (`buffer_a` and `buffer_b`) and a boolean toggle (`use_buffer_b: Cell<bool>`) inside `Overlay`.
- **Alternating Render-Present Logic:** 
  - `prepare_surface` selects the currently inactive back-buffer (based on `use_buffer_b`).
  - Drawing is completed entirely on the back-buffer's shared memory.
  - `commit_surface` invokes `UpdateLayeredWindow` using the back-buffer's GDI DC.
  - Upon successful return of `UpdateLayeredWindow`, `use_buffer_b` is toggled so the other buffer will be used as the back-buffer in the next frame.
- **Unified Deallocation & Reallocation**: If a capacity check fails, both buffers are cleaned up and reallocated in lock-step, ensuring they share the same dimensions.

---

## Verification
- **Unit Tests:** Updated `test_overlay_gdi_caching` to assert alternating DC allocations and verify that double-buffering behaves correctly. All 20 tests pass.
- **Lints:** Clean cargo clippy output.
