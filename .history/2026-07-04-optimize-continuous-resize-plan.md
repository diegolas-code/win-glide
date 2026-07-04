# High-Performance Resize Cache Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement capacity-based GDI caching and over-allocated growth to achieve zero handle thrashing and microsecond latency during continuous resizing.

**Architecture:**
* In `prepare_surface`, change cache validation from exact dimensions (`cache_w == width`) to capacity check (`cache_w >= width`).
* On cache miss, allocate a DIB section with extra padding: `width + 256` by `height + 256`.
* Instantiate the `tiny-skia` `PixmapMut` using the over-allocated bounds while rendering graphics within actual window bounds (`width`, `height`).
* Update unit tests to reflect the new over-allocated capacity bounds.

**Tech Stack:** Rust (2024), Win32 GDI/DWM APIs, `tiny-skia`.

---

### Task 1: Refactor prepare_surface to use Capacity Caching and Over-Allocation

**Files:**
- Modify: [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs)

- [ ] **Step 1: Update cache validity check and miss allocation size**
  Modify the validation condition and cache-miss DIB allocation inside `prepare_surface` in [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs):
  ```rust
              let is_cache_valid = cache_dc_opt.is_some()
                  && cache_bmp_opt.is_some()
                  && cache_old_opt.is_some()
                  && cache_w >= width
                  && cache_h >= height
                  && !bits.is_null();

              let mem_dc = if is_cache_valid {
                  cache_dc_opt.unwrap()
              } else {
                  // Cache Miss or resize: Clean up old cached resources first
                  if let Some(mem_dc) = cache_dc_opt.filter(|d| !d.is_invalid()) {
                      if let Some(old) = cache_old_opt.filter(|o| !o.is_invalid()) {
                          let _ = SelectObject(mem_dc, old);
                      }
                      if let Some(bmp) = cache_bmp_opt.filter(|b| !b.is_invalid()) {
                          let _ = DeleteObject(bmp);
                      }
                      let _ = DeleteDC(mem_dc);
                  }

                  // Allocate new compatible DC and DIB section with 256px growth padding
                  let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
                  if screen_dc.is_invalid() {
                      return None;
                  }

                  let mem_dc = CreateCompatibleDC(screen_dc);
                  if mem_dc.is_invalid() {
                      windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                      return None;
                  }

                  let alloc_w = width + 256;
                  let alloc_h = height + 256;

                  let bmi = BITMAPINFO {
                      bmiHeader: BITMAPINFOHEADER {
                          biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                          biWidth: alloc_w,
                          biHeight: -alloc_h, // top-down
                          biPlanes: 1,
                          biBitCount: 32,
                          biCompression: 0,
                          ..Default::default()
                      },
                      ..Default::default()
                  };

                  let mut new_bits = std::ptr::null_mut();
                  let bitmap = match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut new_bits, None, 0) {
                      Ok(bmp) => bmp,
                      Err(_) => {
                          let _ = DeleteDC(mem_dc);
                          windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                          return None;
                      }
                  };

                  if new_bits.is_null() {
                      let _ = DeleteObject(bitmap);
                      let _ = DeleteDC(mem_dc);
                      windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                      return None;
                  }

                  let old_bitmap = SelectObject(mem_dc, bitmap);
                  windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);

                  // Update cache
                  *self.cached_dc.borrow_mut() = Some(mem_dc);
                  *self.cached_bitmap.borrow_mut() = Some(bitmap);
                  *self.cached_old_bitmap.borrow_mut() = Some(old_bitmap);
                  self.cached_width.set(alloc_w);
                  self.cached_height.set(alloc_h);
                  self.cached_bits.set(new_bits as *mut u8);
                  bits = new_bits as *mut u8;

                  mem_dc
              };
  ```

- [ ] **Step 2: Update PixmapMut dimensions and rendering bounds**
  Instantiate the `PixmapMut` using the current cached over-allocated dimensions, while drawing paths and layout boundaries within the actual `width` and `height`:
  ```rust
              let current_cache_w = self.cached_width.get();
              let current_cache_h = self.cached_height.get();

              // Wrap the DIB section's memory in a tiny-skia PixmapMut using cached buffer size.
              let slice = std::slice::from_raw_parts_mut(
                  bits,
                  (current_cache_w * current_cache_h * 4) as usize,
              );
              if let Some(mut pixmap) =
                  PixmapMut::from_bytes(slice, current_cache_w as u32, current_cache_h as u32)
              {
                  // Clear with transparent
                  pixmap.fill(Color::TRANSPARENT);

                  let mut paint = Paint::default();
                  paint.set_color(Color::from_rgba8(215, 120, 0, 50));
                  paint.anti_alias = true;

                  let mut pb = PathBuilder::new();
                  let r = 8.0f32; // Corner radius
                  let w = width as f32;
                  let h = height as f32;
  ```

- [ ] **Step 3: Run tests and verify**
  Run: `cargo test`
  Expected: Success or test compilation checks.

- [ ] **Step 4: Commit**
  ```bash
  git add src/ui.rs
  git commit -m "perf: implement over-allocated DIB sections and capacity-based caching"
  ```

---

### Task 2: Update GDI Caching Unit Assertions

**Files:**
- Modify: [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs)

- [ ] **Step 1: Update assertions in test_overlay_gdi_caching**
  Adjust the expected cached width and height in the unit test to match the `+256` growth padding rule:
  ```rust
      #[test]
      fn test_overlay_gdi_caching() {
          let overlay = Overlay::new().unwrap();

          let rect1 = RECT {
              left: 100,
              top: 100,
              right: 400,
              bottom: 400,
          };

          // Call prepare_surface for the first time
          let prepared1 = overlay.prepare_surface(rect1, false, false).unwrap();
          let dc1 = prepared1.mem_dc;

          // Call prepare_surface again with the same dimensions (should hit cache)
          let prepared2 = overlay.prepare_surface(rect1, false, false).unwrap();
          let dc2 = prepared2.mem_dc;

          assert_eq!(dc1, dc2, "DC should be cached and re-used for identical dimensions");

          // Call prepare_surface with smaller dimensions (should hit capacity cache)
          let rect_small = RECT {
              left: 100,
              top: 100,
              right: 350,
              bottom: 350,
          };
          let prepared_small = overlay.prepare_surface(rect_small, false, false).unwrap();
          assert_eq!(prepared_small.mem_dc, dc1, "DC should be re-used when dimensions are smaller than cache capacity");

          // Call prepare_surface with larger dimensions (should miss cache and allocate new DC)
          let rect2 = RECT {
              left: 100,
              top: 100,
              right: 600,
              bottom: 600,
          };
          let prepared3 = overlay.prepare_surface(rect2, false, false).unwrap();
          let _dc3 = prepared3.mem_dc;

          assert_eq!(overlay.cached_width.get(), 500 + 256);
          assert_eq!(overlay.cached_height.get(), 500 + OVERLAY_TOP_EXTENSION + 256);
      }
  ```

- [ ] **Step 2: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 3: Commit**
  ```bash
  git add src/ui.rs
  git commit -m "test: update GDI caching assertions for capacity and padding checks"
  ```
