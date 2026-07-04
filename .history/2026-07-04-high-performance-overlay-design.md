# Design Specification: High-Performance Overlay Rendering

*   **Date:** 2026-07-04
*   **Feature:** High-Performance Overlay Rendering (Redraw Optimization)
*   **Status:** APPROVED

---

## 1. Context & Problem Statement

When the user resizes a window during an active `win-glide` session (at 120Hz), both visual tearing (separation of the overlay from the target window) and UI stutter (frame rate drops) occur.

There are three primary bottlenecks causing this lag:
1.  **Pixel Scan Overhead:** GDI text rendering is alpha-unaware, requiring an alpha recovery loop. Currently, this loop iterates over every single pixel of the overlay's DIB section (over 2 million pixels for a 1080p window), converting and calculating values on the CPU. This takes 5–15ms and blocks the main thread.
2.  **Heap / GDI Allocation Thrashing:** A new GDI DIB section and `tiny-skia` canvas are created and destroyed on every repeat key-press, adding heap allocation and kernel handle churn.
3.  **Synchronization Lag:** Because CPU rendering takes several milliseconds, the pixel upload (`UpdateLayeredWindow`) is delayed, mismatching the window resize transaction committed by the DWM.

---

## 2. Design Goals

*   Reduce CPU rendering latency from 5-15ms to **under 0.2ms**.
*   Eliminate heap and GDI handle allocation thrashing during active resizing.
*   Ensure the overlay window and target window bounds remain synchronized under the DWM with zero tearing.

---

## 3. Detailed Architecture

### 3.1 GDI Resource Caching
We introduce persistent GDI handle caching directly inside the `Overlay` struct using `RefCell` to allow interior mutability:

```rust
pub struct Overlay {
    pub hwnd: HWND,
    cached_dc: RefCell<Option<windows::Win32::Graphics::Gdi::HDC>>,
    cached_bitmap: RefCell<Option<windows::Win32::Graphics::Gdi::HBITMAP>>,
    cached_old_bitmap: RefCell<Option<windows::Win32::Graphics::Gdi::HGDIOBJ>>,
    cached_width: RefCell<i32>,
    cached_height: RefCell<i32>,
    cached_bits: RefCell<*mut u8>,
}
```

*   **Cache Hit:** If the requested width/height matches `cached_width`/`cached_height`, we skip all GDI bitmap/DC allocations, re-using the existing buffer.
*   **Cache Miss / Window Size Change:** We clean up existing cached handles, allocate a new compatible DC and DIB section, and cache them.
*   **RAII Cleanup:** We implement `Drop` for `Overlay` to release cached resources.

### 3.2 Localized Alpha Scan
The alpha recovery post-processing loop will be restricted to the exact bounding box (`draw_rect`) of the text calculated by `DrawTextW`:

```rust
let scan_top = draw_rect.top.max(0) as usize;
let scan_bottom = (draw_rect.bottom as usize).min(height as usize);
let scan_left = draw_rect.left.max(0) as usize;
let scan_right = (draw_rect.right as usize).min(width as usize);

let stride = width as usize * 4;
for y in scan_top..scan_bottom {
    let row_offset = y * stride;
    for x in scan_left..scan_right {
        let offset = row_offset + x * 4;
        let b = slice[offset];
        let a = &mut slice[offset + 3];
        if b > 0 {
            let intensity = b as f32 / 255.0;
            let bg_alpha = *a;
            *a = (bg_alpha as f32 + (INDICATOR_OPACITY as f32 - bg_alpha as f32) * intensity) as u8;
        }
    }
}
```
If GDI text drawing is skipped (e.g. on tiny windows), the loop is skipped entirely.

---

## 4. Testing & Verification

*   **Unit Tests:** Verify that `prepare_surface` and `redraw` function correctly with caching when dimensions are unchanged vs when dimensions change.
*   **Drop Safety:** Confirm that dropping the `Overlay` does not double-free or leak GDI handles.
