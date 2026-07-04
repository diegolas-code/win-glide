# High-Performance Overlay Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize overlay redrawing and resizing during active sessions to achieve microsecond CPU rendering latency, zero handle thrashing, and zero-tearing DWM positioning.

**Architecture:** 
* Introduce persistent `RefCell`/`Cell` GDI resource cache fields in the `Overlay` struct.
* Cache the compatible DC and DIB section, recreating them only on window dimension changes.
* Restrict the GDI alpha reconstruction loop to the exact bounding box (`draw_rect`) of the text calculated by `DrawTextW`.
* Clean up caching resources safely in `Overlay`'s `Drop` implementation.

**Tech Stack:** Rust (2024), GDI, `tiny-skia`.

## Global Constraints
* safety: safe wrappers around windows crate, FFI calls documented.
* layered architecture: `ui/` layer handles the border overlay.
* optimization: redraw overlay only on width/height change, and cache CPU resources.

---

### Task 1: Add Caching Fields and Implement Drop for Overlay

**Files:**
- Modify: [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs)

- [ ] **Step 1: Update Overlay struct definition**
  Add caching fields to the `Overlay` struct inside [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs):
  ```rust
  pub struct Overlay {
      pub hwnd: HWND,
      cached_dc: std::cell::RefCell<Option<windows::Win32::Graphics::Gdi::HDC>>,
      cached_bitmap: std::cell::RefCell<Option<windows::Win32::Graphics::Gdi::HBITMAP>>,
      cached_old_bitmap: std::cell::RefCell<Option<windows::Win32::Graphics::Gdi::HGDIOBJ>>,
      cached_width: std::cell::Cell<i32>,
      cached_height: std::cell::Cell<i32>,
      cached_bits: std::cell::Cell<*mut u8>,
  }
  ```

- [ ] **Step 2: Update Overlay::new constructor**
  Initialize the fields inside `Overlay::new()`:
  ```rust
          Ok(Self {
              hwnd,
              cached_dc: std::cell::RefCell::new(None),
              cached_bitmap: std::cell::RefCell::new(None),
              cached_old_bitmap: std::cell::RefCell::new(None),
              cached_width: std::cell::Cell::new(0),
              cached_height: std::cell::Cell::new(0),
              cached_bits: std::cell::Cell::new(std::ptr::null_mut()),
          })
  ```

- [ ] **Step 3: Implement Drop for Overlay**
  Add the GDI cleanup logic in the `Drop` implementation for `Overlay`:
  ```rust
  impl Drop for Overlay {
      fn drop(&mut self) {
          unsafe {
              let dc = self.cached_dc.replace(None);
              let old_bmp = self.cached_old_bitmap.replace(None);
              let bmp = self.cached_bitmap.replace(None);

              if let Some(mem_dc) = dc {
                  if !mem_dc.is_invalid() {
                      if let Some(old) = old_bmp {
                          if !old.is_invalid() {
                              let _ = windows::Win32::Graphics::Gdi::SelectObject(mem_dc, old);
                          }
                      }
                      if let Some(bitmap) = bmp {
                          if !bitmap.is_invalid() {
                              let _ = windows::Win32::Graphics::Gdi::DeleteObject(bitmap);
                          }
                      }
                      let _ = windows::Win32::Graphics::Gdi::DeleteDC(mem_dc);
                  }
              }
          }
      }
  }
  ```

- [ ] **Step 4: Update PreparedOverlaySurface struct**
  Simplify `PreparedOverlaySurface` inside [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs) to remove GDI ownership fields:
  ```rust
  pub struct PreparedOverlaySurface {
      pub mem_dc: windows::Win32::Graphics::Gdi::HDC,
      pub width: i32,
      pub height: i32,
  }
  ```
  Remove its `impl Drop for PreparedOverlaySurface` block completely since the cache now owns the handle lifetimes.

- [ ] **Step 5: Run tests and verify**
  Run: `cargo test`
  Expected: Compilation and test success.

- [ ] **Step 6: Commit**
  ```bash
  git add src/ui.rs
  git commit -m "feat: add persistent GDI cache fields and Drop implementation to Overlay"
  ```

---

### Task 2: Implement Persistent GDI Cache and Localized Alpha Scan

**Files:**
- Modify: [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs)

- [ ] **Step 1: Rewrite prepare_surface to utilize caching and localized alpha scan**
  Update the method signature and body of `prepare_surface`:
  ```rust
      pub fn prepare_surface(&self, rect: RECT, is_shift_down: bool, is_alt_down: bool) -> Option<PreparedOverlaySurface> {
          let width = rect.right - rect.left;
          let height = (rect.bottom - rect.top) + OVERLAY_TOP_EXTENSION;

          if width <= 0 || height <= 0 {
              return None;
          }

          unsafe {
              let mut cache_dc_opt = self.cached_dc.borrow().clone();
              let mut cache_bmp_opt = self.cached_bitmap.borrow().clone();
              let mut cache_old_opt = self.cached_old_bitmap.borrow().clone();
              let cache_w = self.cached_width.get();
              let cache_h = self.cached_height.get();
              let mut bits = self.cached_bits.get();

              let is_cache_valid = cache_dc_opt.is_some()
                  && cache_bmp_opt.is_some()
                  && cache_old_opt.is_some()
                  && cache_w == width
                  && cache_h == height
                  && !bits.is_null();

              let (mem_dc, bitmap, old_bitmap) = if is_cache_valid {
                  (cache_dc_opt.unwrap(), cache_bmp_opt.unwrap(), cache_old_opt.unwrap())
              } else {
                  // Cache Miss or resize: Clean up old cached resources first
                  if let Some(mem_dc) = cache_dc_opt {
                      if !mem_dc.is_invalid() {
                          if let Some(old) = cache_old_opt {
                              let _ = SelectObject(mem_dc, old);
                          }
                          if let Some(bmp) = cache_bmp_opt {
                              let _ = DeleteObject(bmp);
                          }
                          let _ = DeleteDC(mem_dc);
                      }
                  }

                  // Allocate new compatible DC and DIB section
                  let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
                  if screen_dc.is_invalid() {
                      return None;
                  }

                  let mem_dc = CreateCompatibleDC(screen_dc);
                  if mem_dc.is_invalid() {
                      windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                      return None;
                  }

                  let bmi = BITMAPINFO {
                      bmiHeader: BITMAPINFOHEADER {
                          biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                          biWidth: width,
                          biHeight: -height, // top-down
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
                  self.cached_width.set(width);
                  self.cached_height.set(height);
                  self.cached_bits.set(new_bits);
                  bits = new_bits;

                  (mem_dc, bitmap, old_bitmap)
              };

              // Wrap the DIB section memory in tiny-skia PixmapMut
              let slice = std::slice::from_raw_parts_mut(
                  bits as *mut u8,
                  (width * height * 4) as usize,
              );
              if let Some(mut pixmap) = PixmapMut::from_bytes(slice, width as u32, height as u32) {
                  pixmap.fill(Color::TRANSPARENT);

                  let mut paint = Paint::default();
                  paint.set_color(Color::from_rgba8(215, 120, 0, 50));
                  paint.anti_alias = true;

                  let mut pb = PathBuilder::new();
                  let r = 8.0f32; // Corner radius
                  let w = width as f32;
                  let h = height as f32;

                  pb.move_to(r, 0.0);
                  pb.line_to(w - r, 0.0);
                  pb.quad_to(w, 0.0, w, r);
                  pb.line_to(w, h - r);
                  pb.quad_to(w, h, w - r, h);
                  pb.line_to(r, h);
                  pb.quad_to(0.0, h, 0.0, h - r);
                  pb.line_to(0.0, r);
                  pb.quad_to(0.0, 0.0, r, 0.0);
                  pb.close();

                  if let Some(path) = pb.finish() {
                      pixmap.fill_path(
                          &path,
                          &paint,
                          FillRule::Winding,
                          Transform::identity(),
                          None,
                      );
                  }

                  let dpi = {
                      let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
                      let res = windows::Win32::Graphics::Gdi::GetDeviceCaps(screen_dc, windows::Win32::Graphics::Gdi::LOGPIXELSX);
                      windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                      res as u32
                  };
                  let dpi_scale = dpi as f32 / 96.0;
                  let arrow_size = 36.0 * dpi_scale;
                  let margin = 30.0 * dpi_scale;

                  if (is_shift_down || is_alt_down)
                      && w >= 3.0 * arrow_size
                      && (h - OVERLAY_TOP_EXTENSION as f32) >= 3.0 * arrow_size
                  {
                      let mut white_paint = Paint::default();
                      white_paint.set_color(Color::from_rgba8(255, 255, 255, INDICATOR_OPACITY));
                      white_paint.anti_alias = true;

                      let top_direction = if is_shift_down { ArrowDirection::Up } else { ArrowDirection::Down };
                      let bottom_direction = if is_shift_down { ArrowDirection::Down } else { ArrowDirection::Up };
                      let left_direction = if is_shift_down { ArrowDirection::Left } else { ArrowDirection::Right };
                      let right_direction = if is_shift_down { ArrowDirection::Right } else { ArrowDirection::Left };

                      draw_arrow(
                          &mut pixmap,
                          &white_paint,
                          w / 2.0,
                          OVERLAY_TOP_EXTENSION as f32 + margin + arrow_size / 2.0,
                          arrow_size,
                          top_direction,
                          dpi_scale,
                      );

                      draw_arrow(
                          &mut pixmap,
                          &white_paint,
                          w / 2.0,
                          h - margin - arrow_size / 2.0,
                          arrow_size,
                          bottom_direction,
                          dpi_scale,
                      );

                      draw_arrow(
                          &mut pixmap,
                          &white_paint,
                          margin + arrow_size / 2.0,
                          OVERLAY_TOP_EXTENSION as f32 + (h - OVERLAY_TOP_EXTENSION as f32) / 2.0,
                          arrow_size,
                          left_direction,
                          dpi_scale,
                      );

                      draw_arrow(
                          &mut pixmap,
                          &white_paint,
                          w - margin - arrow_size / 2.0,
                          OVERLAY_TOP_EXTENSION as f32 + (h - OVERLAY_TOP_EXTENSION as f32) / 2.0,
                          arrow_size,
                          right_direction,
                          dpi_scale,
                      );
                  }
              }

              // --- Draw Help Text ---
              let dpi = {
                  let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
                  let res = windows::Win32::Graphics::Gdi::GetDeviceCaps(screen_dc, windows::Win32::Graphics::Gdi::LOGPIXELSX);
                  windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                  res as u32
              };
              let dpi_scale = dpi as f32 / 96.0;
              let arrow_size = 36.0 * dpi_scale;
              let margin = 30.0 * dpi_scale;

              let target_left = (margin + arrow_size + 10.0) as i32;
              let target_right = (width as f32 - margin - arrow_size - 10.0) as i32;
              let target_top = (OVERLAY_TOP_EXTENSION as f32 + margin + arrow_size + 10.0) as i32;
              let target_bottom = (height as f32 - margin - arrow_size - 10.0) as i32;

              if target_right > target_left && target_bottom > target_top {
                  let text_str = if is_shift_down {
                      "Press the [Arrow keys] to push the window borders outwards."
                  } else if is_alt_down {
                      "Press the [Arrow keys] to pull the window borders inwards."
                  } else {
                      "Press [Arrow keys] to move the window around.\nPress [Shift] and [Arrow keys] to resize the window up.\nPress [Alt] and [Arrow keys] to resize the window down."
                  };

                  let mut text_utf16: Vec<u16> = text_str.encode_utf16().collect();

                  let font_height = -((18.0 * dpi_scale).round() as i32);
                  let font = CreateFontW(
                      font_height,
                      0,
                      0,
                      0,
                      400,
                      0,
                      0,
                      0,
                      0,
                      0,
                      0,
                      ANTIALIASED_QUALITY.0 as u32,
                      0,
                      windows::core::w!("Segoe UI"),
                  );

                  if !font.is_invalid() {
                      let old_font = SelectObject(mem_dc, font);
                      let _ = SetTextColor(mem_dc, windows::Win32::Foundation::COLORREF(0x00ffffff));
                      let _ = SetBkMode(mem_dc, TRANSPARENT);

                      let mut rect = RECT {
                          left: target_left,
                          top: target_top,
                          right: target_right,
                          bottom: target_bottom,
                      };

                      let _ = DrawTextW(
                          mem_dc,
                          &mut text_utf16,
                          &mut rect,
                          DT_CENTER | DT_WORDBREAK | DT_CALCRECT,
                      );

                      let text_height = rect.bottom - rect.top;
                      let available_height = target_bottom - target_top;
                      let y_offset = ((available_height - text_height) / 2).max(0);

                      let mut draw_rect = RECT {
                          left: target_left,
                          top: target_top + y_offset,
                          right: target_right,
                          bottom: target_top + y_offset + text_height,
                      };

                      let _ = DrawTextW(
                          mem_dc,
                          &mut text_utf16,
                          &mut draw_rect,
                          DT_CENTER | DT_WORDBREAK,
                      );

                      let _ = SelectObject(mem_dc, old_font);
                      let _ = DeleteObject(font);

                      // Restrict alpha scan post-processing to the bounding box of the drawn text
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
                  }
              }

              Some(PreparedOverlaySurface {
                  mem_dc,
                  width,
                  height,
              })
          }
      }
  ```

- [ ] **Step 2: Update commit_surface**
  Remove the deselect and delete cleanup inside `commit_surface` since those resources are persistently held in the `Overlay` cache fields:
  ```rust
      pub fn commit_surface(&self, prepared: PreparedOverlaySurface, rect: RECT) -> windows::core::Result<()> {
          unsafe {
              let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
              if screen_dc.is_invalid() {
                  return Ok(());
              }

              let pt_src = POINT { x: 0, y: 0 };
              let pt_dst = POINT {
                  x: rect.left,
                  y: rect.top - OVERLAY_TOP_EXTENSION,
              };
              let size = SIZE {
                  cx: prepared.width,
                  cy: prepared.height,
              };

              let blend = BLENDFUNCTION {
                  BlendOp: AC_SRC_OVER as u8,
                  BlendFlags: 0,
                  SourceConstantAlpha: 255,
                  AlphaFormat: AC_SRC_ALPHA as u8,
              };

              let res = UpdateLayeredWindow(
                  self.hwnd,
                  screen_dc,
                  Some(&pt_dst),
                  Some(&size),
                  prepared.mem_dc,
                  Some(&pt_src),
                  None,
                  Some(&blend),
                  ULW_ALPHA,
              );

              windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);

              res
          }
      }
  ```

- [ ] **Step 3: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 4: Commit**
  ```bash
  git add src/ui.rs
  git commit -m "perf: implement persistent DIB section caching and localized alpha scanning"
  ```
