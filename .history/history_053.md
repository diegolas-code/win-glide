# History Log: Pre-allocated GDI DIB sections on Activation

*   **Date:** 2026-07-04
*   **Feature:** Resizing and Redraw Smoothness Optimizations (Task 3)
*   **Branch:** `experiment/smoother-resize-redraw`

---

## Technical Decisions & Rationale

### 1. Pre-allocation to Virtual Screen Size on Activation
- **Problem:** Even with double-buffering, if the target window size exceeds the capacity of the GDI buffers during continuous resizing, the application must pause to recreate two new `CreateDIBSection` surfaces. This kernel-level allocation latency drops frames and causes visual "strobe" stuttering during fast resize expansions.
- **Decision:** Introduce a public `preallocate_buffers(max_w, max_h)` method on `Overlay`. On session activation, the virtual screen boundary is queried and the dual GDI buffers are immediately allocated to these maximum possible boundaries.
- **Benefits:** Since the overlay can never expand beyond the virtual screen dimensions, the buffers are guaranteed to never require reallocation during the active resize loop. `CreateDIBSection` calls are completely eliminated from the hot path.

### 2. On-Demand Cleanups (Memory Footprint Hygiene)
- **Decision:** To keep win-glide's idle working set memory footprint at the ~1.2MB baseline, we introduce a `free_buffers()` method on `Overlay` that drops both GdiBuffer instances. This is called automatically when deactivating the glide session.

### 3. Robust Fallback in prepare_surface
- **Decision:** Keep a fallback allocation call to `preallocate_buffers` inside `prepare_surface` in case a caller invokes it without prior activation (such as in unit tests).

---

## Verification
- **Unit Tests:** Run `cargo test`. All 20 tests pass.
- **Lints:** Clean clippy checks.
