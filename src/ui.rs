//! UI and Rendering.
//!
//! This module handles the visual overlay that indicates an active session.
//! It uses a WS_EX_LAYERED window with per-pixel alpha transparency.
//! Rendering is performed using `tiny-skia` into a GDI DIB section,
//! which is then uploaded via `UpdateLayeredWindow`.

use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, PixmapMut, Stroke, Transform,
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DIB_RGB_COLORS, DT_CALCRECT, DT_CENTER,
    DT_WORDBREAK, DeleteDC, DeleteObject, DrawTextW, SelectObject, SetBkMode, SetTextColor,
    TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DeferWindowPos, HDWP, RegisterClassW, SW_HIDE,
    SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER, ULW_ALPHA, UpdateLayeredWindow, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::w;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

struct GdiBuffer {
    dc: windows::Win32::Graphics::Gdi::HDC,
    bitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    old_bitmap: windows::Win32::Graphics::Gdi::HGDIOBJ,
    bits: *mut u8,
}

impl GdiBuffer {
    fn clean(&self) {
        unsafe {
            if !self.dc.is_invalid() {
                if !self.old_bitmap.is_invalid() {
                    let _ = windows::Win32::Graphics::Gdi::SelectObject(self.dc, self.old_bitmap);
                }
                if !self.bitmap.is_invalid() {
                    let _ = windows::Win32::Graphics::Gdi::DeleteObject(self.bitmap);
                }
                let _ = windows::Win32::Graphics::Gdi::DeleteDC(self.dc);
            }
        }
    }
}

/// Manages a transparent overlay window.
pub struct Overlay {
    pub hwnd: HWND,
    buffer_a: std::cell::RefCell<Option<GdiBuffer>>,
    buffer_b: std::cell::RefCell<Option<GdiBuffer>>,
    use_buffer_b: std::cell::Cell<bool>,
    cached_width: std::cell::Cell<i32>,
    cached_height: std::cell::Cell<i32>,
    cached_arrow_size: std::cell::Cell<f32>,
    cached_path_up: std::cell::RefCell<Option<tiny_skia::Path>>,
    cached_path_down: std::cell::RefCell<Option<tiny_skia::Path>>,
    cached_path_left: std::cell::RefCell<Option<tiny_skia::Path>>,
    cached_path_right: std::cell::RefCell<Option<tiny_skia::Path>>,
}

impl Drop for Overlay {
    fn drop(&mut self) {
        if let Some(buf) = self.buffer_a.replace(None) {
            buf.clean();
        }
        if let Some(buf) = self.buffer_b.replace(None) {
            buf.clean();
        }
    }
}

/// Constant for the vertical extension above the window (the "header").
pub const OVERLAY_TOP_EXTENSION: i32 = 7;

/// Opacity value for indicators (arrows and text).
pub const INDICATOR_OPACITY: u8 = 204;

/// Helper struct containing GDI handles for a prepared overlay surface.
pub struct PreparedOverlaySurface {
    pub mem_dc: windows::Win32::Graphics::Gdi::HDC,
    pub width: i32,
    pub height: i32,
}

impl Overlay {
    /// Internal window procedure for the overlay.
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // The overlay is passive and doesn't handle inputs directly.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// Creates a new transparent, click-through overlay window.
    pub fn new() -> windows::core::Result<Self> {
        let instance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)? };
        let class_name = w!("WinGlideOverlay");

        // Optimized Window Class: No CS_HREDRAW/VREDRAW or CS_OWNDC to minimize RAM.
        let wnd_class = WNDCLASSW {
            style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
            lpfnWndProc: Some(Self::wnd_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        unsafe {
            let _ = RegisterClassW(&wnd_class);
        }

        // WS_EX_LAYERED: Enables per-pixel alpha via UpdateLayeredWindow.
        // WS_EX_TRANSPARENT: Makes the window "click-through".
        // WS_EX_NOACTIVATE: Prevents the window from stealing focus.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
                class_name,
                w!("win-glide overlay"),
                WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                None,
            )?
        };

        Ok(Self {
            hwnd,
            buffer_a: std::cell::RefCell::new(None),
            buffer_b: std::cell::RefCell::new(None),
            use_buffer_b: std::cell::Cell::new(false),
            cached_width: std::cell::Cell::new(0),
            cached_height: std::cell::Cell::new(0),
            cached_arrow_size: std::cell::Cell::new(0.0),
            cached_path_up: std::cell::RefCell::new(None),
            cached_path_down: std::cell::RefCell::new(None),
            cached_path_left: std::cell::RefCell::new(None),
            cached_path_right: std::cell::RefCell::new(None),
        })
    }

    /// Sets the target window as the "owner" of the overlay.
    /// This ensures the overlay stays on top of the target window.
    pub fn set_owner(&self, owner: HWND) {
        use windows::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GWLP_HWNDPARENT, GetWindowLongW, HWND_NOTOPMOST, HWND_TOPMOST,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos, WS_EX_TOPMOST,
        };
        unsafe {
            // Set parent/owner relationship
            SetWindowLongPtrW(self.hwnd, GWLP_HWNDPARENT, owner.0 as isize);

            // Check if the owner window is topmost
            let ex_style = GetWindowLongW(owner, GWL_EXSTYLE) as u32;
            let owner_is_topmost = (ex_style & WS_EX_TOPMOST.0) != 0;

            let insert_after = if owner_is_topmost {
                HWND_TOPMOST
            } else {
                HWND_NOTOPMOST
            };

            // Set overlay topmost status to match the owner
            let _ = SetWindowPos(
                self.hwnd,
                insert_after,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_arrow(
        &self,
        pixmap: &mut tiny_skia::PixmapMut,
        paint: &tiny_skia::Paint,
        center_x: f32,
        center_y: f32,
        size: f32,
        direction: ArrowDirection,
        dpi_scale: f32,
    ) {
        if self.cached_arrow_size.get() != size {
            self.cached_arrow_size.set(size);
            *self.cached_path_up.borrow_mut() = None;
            *self.cached_path_down.borrow_mut() = None;
            *self.cached_path_left.borrow_mut() = None;
            *self.cached_path_right.borrow_mut() = None;
        }

        let path_cell = match direction {
            ArrowDirection::Up => &self.cached_path_up,
            ArrowDirection::Down => &self.cached_path_down,
            ArrowDirection::Left => &self.cached_path_left,
            ArrowDirection::Right => &self.cached_path_right,
        };

        let mut path_borrow = path_cell.borrow_mut();
        if path_borrow.is_none() {
            let half_w = size / 2.0;
            let half_h = size / 4.0;
            let mut pb = tiny_skia::PathBuilder::new();
            match direction {
                ArrowDirection::Up => {
                    pb.move_to(-half_w, half_h);
                    pb.line_to(0.0, -half_h);
                    pb.line_to(half_w, half_h);
                }
                ArrowDirection::Down => {
                    pb.move_to(-half_w, -half_h);
                    pb.line_to(0.0, half_h);
                    pb.line_to(half_w, -half_h);
                }
                ArrowDirection::Left => {
                    pb.move_to(half_h, -half_w);
                    pb.line_to(-half_h, 0.0);
                    pb.line_to(half_h, half_w);
                }
                ArrowDirection::Right => {
                    pb.move_to(-half_h, -half_w);
                    pb.line_to(half_h, 0.0);
                    pb.line_to(-half_h, half_w);
                }
            }
            *path_borrow = pb.finish();
        }

        if let Some(ref path) = *path_borrow {
            let stroke = Stroke {
                width: 8.0 * dpi_scale,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Stroke::default()
            };

            let transform = tiny_skia::Transform::from_translate(center_x, center_y);

            pixmap.stroke_path(path, paint, &stroke, transform, None);
        }
    }

    /// Pre-allocates dual GDI buffers to the specified maximum dimensions.
    pub fn preallocate_buffers(&self, max_w: i32, max_h: i32) {
        unsafe {
            // Clean up any existing buffers first
            if let Some(buf) = self.buffer_a.replace(None) {
                buf.clean();
            }
            if let Some(buf) = self.buffer_b.replace(None) {
                buf.clean();
            }

            let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
            if screen_dc.is_invalid() {
                return;
            }

            let dc_a = CreateCompatibleDC(screen_dc);
            if dc_a.is_invalid() {
                windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                return;
            }

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: max_w,
                    biHeight: -max_h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits_a = std::ptr::null_mut();
            let bmp_a = match CreateDIBSection(dc_a, &bmi, DIB_RGB_COLORS, &mut bits_a, None, 0) {
                Ok(bmp) => bmp,
                Err(_) => {
                    let _ = DeleteDC(dc_a);
                    windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                    return;
                }
            };
            let old_bmp_a = SelectObject(dc_a, bmp_a);

            let dc_b = CreateCompatibleDC(screen_dc);
            if dc_b.is_invalid() {
                if !old_bmp_a.is_invalid() {
                    let _ = SelectObject(dc_a, old_bmp_a);
                }
                let _ = DeleteObject(bmp_a);
                let _ = DeleteDC(dc_a);
                windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                return;
            }

            let mut bits_b = std::ptr::null_mut();
            let bmp_b = match CreateDIBSection(dc_b, &bmi, DIB_RGB_COLORS, &mut bits_b, None, 0) {
                Ok(bmp) => bmp,
                Err(_) => {
                    if !old_bmp_a.is_invalid() {
                        let _ = SelectObject(dc_a, old_bmp_a);
                    }
                    let _ = DeleteObject(bmp_a);
                    let _ = DeleteDC(dc_a);
                    let _ = DeleteDC(dc_b);
                    windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                    return;
                }
            };
            let old_bmp_b = SelectObject(dc_b, bmp_b);

            windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);

            *self.buffer_a.borrow_mut() = Some(GdiBuffer {
                dc: dc_a,
                bitmap: bmp_a,
                old_bitmap: old_bmp_a,
                bits: bits_a as *mut u8,
            });

            *self.buffer_b.borrow_mut() = Some(GdiBuffer {
                dc: dc_b,
                bitmap: bmp_b,
                old_bitmap: old_bmp_b,
                bits: bits_b as *mut u8,
            });

            self.cached_width.set(max_w);
            self.cached_height.set(max_h);
        }
    }

    /// Frees both GDI buffers to return memory back to the baseline.
    pub fn free_buffers(&self) {
        if let Some(buf) = self.buffer_a.replace(None) {
            buf.clean();
        }
        if let Some(buf) = self.buffer_b.replace(None) {
            buf.clean();
        }
        self.cached_width.set(0);
        self.cached_height.set(0);
    }

    /// Prepares the overlay surface on the CPU by rendering into a GDI DIB section.
    /// Returns GDI handles wrapped in a RAII container.
    pub fn prepare_surface(
        &self,
        rect: RECT,
        is_shift_down: bool,
        is_alt_down: bool,
    ) -> Option<PreparedOverlaySurface> {
        let width = rect.right - rect.left;
        let height = (rect.bottom - rect.top) + OVERLAY_TOP_EXTENSION;

        if width <= 0 || height <= 0 {
            return None;
        }

        unsafe {
            let has_buffer_a = self.buffer_a.borrow().is_some();
            let has_buffer_b = self.buffer_b.borrow().is_some();
            let cache_w = self.cached_width.get();
            let cache_h = self.cached_height.get();

            let is_cache_valid =
                has_buffer_a && has_buffer_b && cache_w >= width && cache_h >= height;

            if !is_cache_valid {
                // Fallback allocation if pre-allocation was not called or is too small
                self.preallocate_buffers(width + 256, height + 256);
            }

            // Select the current back buffer
            let use_b = self.use_buffer_b.get();
            let (mem_dc, bits) = if use_b {
                let borrow = self.buffer_b.borrow();
                let buf = borrow.as_ref().unwrap();
                (buf.dc, buf.bits)
            } else {
                let borrow = self.buffer_a.borrow();
                let buf = borrow.as_ref().unwrap();
                (buf.dc, buf.bits)
            };

            let current_cache_w = self.cached_width.get();
            let current_cache_h = self.cached_height.get();

            // Wrap the DIB section's memory in a tiny-skia PixmapMut using cached buffer size.
            let slice = std::slice::from_raw_parts_mut(
                bits,
                (current_cache_w * current_cache_h * 4) as usize,
            );

            // Clear only the active region with transparency (GDI memory might be uninitialized)
            let stride = current_cache_w as usize * 4;
            let active_row_bytes = width as usize * 4;
            for y in 0..height as usize {
                let row_start = y * stride;
                let row_end = row_start + active_row_bytes;
                if row_end <= slice.len() {
                    slice[row_start..row_end].fill(0);
                }
            }

            if let Some(mut pixmap) =
                PixmapMut::from_bytes(slice, current_cache_w as u32, current_cache_h as u32)
            {
                let mut paint = Paint::default();
                // Optimization: Use pre-swapped color (BGRA) for direct compatibility.
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

                // Get actual DPI to scale arrows correctly
                let dpi = {
                    let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
                    let res = windows::Win32::Graphics::Gdi::GetDeviceCaps(
                        screen_dc,
                        windows::Win32::Graphics::Gdi::LOGPIXELSX,
                    );
                    windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                    res as u32
                };
                let dpi_scale = dpi as f32 / 96.0;
                let arrow_size = 36.0 * dpi_scale;
                let margin = 30.0 * dpi_scale;

                // Draw arrows if Alt or Shift is down and window is large enough
                if (is_shift_down || is_alt_down)
                    && w >= 3.0 * arrow_size
                    && (h - OVERLAY_TOP_EXTENSION as f32) >= 3.0 * arrow_size
                {
                    let mut white_paint = Paint::default();
                    white_paint.set_color(Color::from_rgba8(255, 255, 255, INDICATOR_OPACITY));
                    white_paint.anti_alias = true;

                    let top_direction = if is_shift_down {
                        ArrowDirection::Up
                    } else {
                        ArrowDirection::Down
                    };
                    let bottom_direction = if is_shift_down {
                        ArrowDirection::Down
                    } else {
                        ArrowDirection::Up
                    };
                    let left_direction = if is_shift_down {
                        ArrowDirection::Left
                    } else {
                        ArrowDirection::Right
                    };
                    let right_direction = if is_shift_down {
                        ArrowDirection::Right
                    } else {
                        ArrowDirection::Left
                    };

                    // Top Arrow
                    self.draw_arrow(
                        &mut pixmap,
                        &white_paint,
                        w / 2.0,
                        OVERLAY_TOP_EXTENSION as f32 + margin + arrow_size / 2.0,
                        arrow_size,
                        top_direction,
                        dpi_scale,
                    );

                    // Bottom Arrow
                    self.draw_arrow(
                        &mut pixmap,
                        &white_paint,
                        w / 2.0,
                        h - margin - arrow_size / 2.0,
                        arrow_size,
                        bottom_direction,
                        dpi_scale,
                    );

                    // Left Arrow
                    self.draw_arrow(
                        &mut pixmap,
                        &white_paint,
                        margin + arrow_size / 2.0,
                        OVERLAY_TOP_EXTENSION as f32 + (h - OVERLAY_TOP_EXTENSION as f32) / 2.0,
                        arrow_size,
                        left_direction,
                        dpi_scale,
                    );

                    // Right Arrow
                    self.draw_arrow(
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
            // Calculate margins & sizes for text safe bounding box
            let dpi = {
                let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
                let res = windows::Win32::Graphics::Gdi::GetDeviceCaps(
                    screen_dc,
                    windows::Win32::Graphics::Gdi::LOGPIXELSX,
                );
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
                    400, // FW_NORMAL (non-bold)
                    0,
                    0,
                    0,
                    0, // DEFAULT_CHARSET
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

                    // 1. Calculate height needed
                    let _ = DrawTextW(
                        mem_dc,
                        &mut text_utf16,
                        &mut rect,
                        DT_CENTER | DT_WORDBREAK | DT_CALCRECT,
                    );

                    // 2. Center vertically and horizontally
                    let text_width = rect.right - rect.left;
                    let text_height = rect.bottom - rect.top;

                    let available_width = target_right - target_left;
                    let x_offset = ((available_width - text_width) / 2).max(0);

                    let available_height = target_bottom - target_top;
                    let y_offset = ((available_height - text_height) / 2).max(0);

                    let mut draw_rect = RECT {
                        left: target_left + x_offset,
                        top: target_top + y_offset,
                        right: target_left + x_offset + text_width,
                        bottom: target_top + y_offset + text_height,
                    };

                    // Draw text background on pixmap
                    {
                        if let Some(mut pixmap) = PixmapMut::from_bytes(
                            slice,
                            current_cache_w as u32,
                            current_cache_h as u32,
                        ) {
                            let bg_r = 6.0f32; // Corner radius
                            let bg_pad_x = 12.0f32;
                            let bg_pad_y = 8.0f32;

                            let bg_left =
                                (draw_rect.left as f32 - bg_pad_x).max(target_left as f32);
                            let bg_right =
                                (draw_rect.right as f32 + bg_pad_x).min(target_right as f32);
                            let bg_top = (draw_rect.top as f32 - bg_pad_y).max(target_top as f32);
                            let bg_bottom =
                                (draw_rect.bottom as f32 + bg_pad_y).min(target_bottom as f32);

                            let mut bg_pb = PathBuilder::new();
                            bg_pb.move_to(bg_left + bg_r, bg_top);
                            bg_pb.line_to(bg_right - bg_r, bg_top);
                            bg_pb.quad_to(bg_right, bg_top, bg_right, bg_top + bg_r);
                            bg_pb.line_to(bg_right, bg_bottom - bg_r);
                            bg_pb.quad_to(bg_right, bg_bottom, bg_right - bg_r, bg_bottom);
                            bg_pb.line_to(bg_left + bg_r, bg_bottom);
                            bg_pb.quad_to(bg_left, bg_bottom, bg_left, bg_bottom - bg_r);
                            bg_pb.line_to(bg_left, bg_top + bg_r);
                            bg_pb.quad_to(bg_left, bg_top, bg_left + bg_r, bg_top);
                            bg_pb.close();

                            if let Some(bg_path) = bg_pb.finish() {
                                let mut bg_paint = Paint::default();
                                bg_paint.set_color(Color::from_rgba8(0, 0, 0, 76)); // 0.3 alpha (76/255)
                                bg_paint.anti_alias = true;
                                pixmap.fill_path(
                                    &bg_path,
                                    &bg_paint,
                                    FillRule::Winding,
                                    Transform::identity(),
                                    None,
                                );
                            }
                        }
                    }

                    // 3. Draw text
                    let _ = DrawTextW(
                        mem_dc,
                        &mut text_utf16,
                        &mut draw_rect,
                        DT_CENTER | DT_WORDBREAK,
                    );

                    let _ = SelectObject(mem_dc, old_font);
                    let _ = DeleteObject(font);

                    // --- Alpha Channel Post-Processing (Localized Scan) ---
                    let scan_top = draw_rect.top.max(0) as usize;
                    let scan_bottom = (draw_rect.bottom as usize).min(height as usize);
                    let scan_left = draw_rect.left.max(0) as usize;
                    let scan_right = (draw_rect.right as usize).min(width as usize);

                    let stride = self.cached_width.get() as usize * 4;
                    for y in scan_top..scan_bottom {
                        let row_offset = y * stride;
                        for x in scan_left..scan_right {
                            let offset = row_offset + x * 4;
                            let r_val = slice[offset + 2]; // Red channel (0 for background, >0 for text)
                            let a = &mut slice[offset + 3];
                            if r_val > 0 {
                                let intensity = r_val as f32 / 255.0;
                                let bg_alpha = *a;
                                *a = (bg_alpha as f32
                                    + (INDICATOR_OPACITY as f32 - bg_alpha as f32) * intensity)
                                    as u8;
                                slice[offset] = 255; // Blue
                                slice[offset + 1] = 255; // Green
                                slice[offset + 2] = 255; // Red
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

    /// Commits the prepared surface to the DWM/layered window using UpdateLayeredWindow.
    pub fn commit_surface(
        &self,
        prepared: PreparedOverlaySurface,
        rect: RECT,
    ) -> windows::core::Result<()> {
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

            if res.is_ok() {
                self.use_buffer_b.set(!self.use_buffer_b.get());
            }

            res
        }
    }

    /// Redraws the overlay based on the target window's dimensions.
    pub fn redraw(
        &self,
        rect: RECT,
        is_shift_down: bool,
        is_alt_down: bool,
    ) -> windows::core::Result<()> {
        if let Some(prepared) = self.prepare_surface(rect, is_shift_down, is_alt_down) {
            self.commit_surface(prepared, rect)
        } else {
            Ok(())
        }
    }

    /// Queues a position update for the overlay.
    ///
    /// Should be called within a BeginDeferWindowPos block for synchronization.
    pub fn defer_update_position(&self, hdwp: HDWP, rect: RECT) -> windows::core::Result<HDWP> {
        unsafe {
            DeferWindowPos(
                hdwp,
                self.hwnd,
                HWND::default(),
                rect.left,
                rect.top - OVERLAY_TOP_EXTENSION,
                rect.right - rect.left,
                (rect.bottom - rect.top) + OVERLAY_TOP_EXTENSION,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
        }
    }

    /// Directly updates the position of the overlay (no redraw).
    pub fn update_position(&self, rect: RECT) -> windows::core::Result<()> {
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                self.hwnd,
                HWND::default(),
                rect.left,
                rect.top - OVERLAY_TOP_EXTENSION,
                rect.right - rect.left,
                (rect.bottom - rect.top) + OVERLAY_TOP_EXTENSION,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
        }
    }

    /// Shows or hides the overlay.
    pub fn show(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd, cmd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GWL_EXSTYLE, GetWindowLongW, HWND_NOTOPMOST, HWND_TOPMOST,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos, WS_EX_TOPMOST, WS_POPUP,
    };
    use windows::core::w;

    #[test]
    fn test_overlay_topmost_sync() {
        let overlay = Overlay::new().unwrap();

        // Create a dummy window
        let instance =
            unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap() };
        let hwnd_test = unsafe {
            CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("Test Window"),
                WS_POPUP,
                0,
                0,
                100,
                100,
                None,
                None,
                instance,
                None,
            )
            .unwrap()
        };

        // Case 1: Test window is not topmost -> Overlay parent should not be topmost
        overlay.set_owner(hwnd_test);
        let ex_style = unsafe { GetWindowLongW(overlay.hwnd, GWL_EXSTYLE) as u32 };
        assert_eq!(ex_style & WS_EX_TOPMOST.0, 0);

        // Case 2: Make test window topmost -> Overlay should become topmost
        unsafe {
            let _ = SetWindowPos(
                hwnd_test,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        overlay.set_owner(hwnd_test);
        let ex_style_topmost = unsafe { GetWindowLongW(overlay.hwnd, GWL_EXSTYLE) as u32 };
        assert_ne!(ex_style_topmost & WS_EX_TOPMOST.0, 0);

        // Case 3: Make test window non-topmost -> Overlay should become non-topmost
        unsafe {
            let _ = SetWindowPos(
                hwnd_test,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        overlay.set_owner(hwnd_test);
        let ex_style_normal = unsafe { GetWindowLongW(overlay.hwnd, GWL_EXSTYLE) as u32 };
        assert_eq!(ex_style_normal & WS_EX_TOPMOST.0, 0);

        unsafe {
            let _ = DestroyWindow(hwnd_test);
        }
    }

    #[test]
    fn test_overlay_arrow_rendering() {
        let overlay = Overlay::new().unwrap();

        // Case 1: Window rect is too small to draw arrows (should still prepare surface successfully)
        let small_rect = RECT {
            left: 100,
            top: 100,
            right: 150,
            bottom: 150,
        };
        let prepared_small = overlay.prepare_surface(small_rect, true, false);
        assert!(prepared_small.is_some());
        let surf = prepared_small.unwrap();
        assert_eq!(surf.width, 50);
        assert_eq!(surf.height, 50 + OVERLAY_TOP_EXTENSION);

        // Case 2: Window rect is large enough to draw arrows
        let large_rect = RECT {
            left: 100,
            top: 100,
            right: 500,
            bottom: 500,
        };
        let prepared_large = overlay.prepare_surface(large_rect, true, false);
        assert!(prepared_large.is_some());
        let surf_large = prepared_large.unwrap();
        assert_eq!(surf_large.width, 400);
        assert_eq!(surf_large.height, 400 + OVERLAY_TOP_EXTENSION);

        // Case 3: Window rect is large enough to draw default help text (no modifiers)
        let prepared_default = overlay.prepare_surface(large_rect, false, false);
        assert!(prepared_default.is_some());
        let surf_default = prepared_default.unwrap();
        assert_eq!(surf_default.width, 400);
        assert_eq!(surf_default.height, 400 + OVERLAY_TOP_EXTENSION);
    }

    #[test]
    fn test_overlay_gdi_caching() {
        let overlay = Overlay::new().unwrap();

        let rect1 = RECT {
            left: 100,
            top: 100,
            right: 400,
            bottom: 400,
        };

        // Frame 1: uses Buffer A
        let prepared1 = overlay.prepare_surface(rect1, false, false).unwrap();
        let dc_a = prepared1.mem_dc;
        overlay.commit_surface(prepared1, rect1).unwrap(); // swaps to Buffer B

        // Frame 2: uses Buffer B
        let prepared2 = overlay.prepare_surface(rect1, false, false).unwrap();
        let dc_b = prepared2.mem_dc;
        assert_ne!(dc_a, dc_b, "Double buffering should return alternating DCs");
        overlay.commit_surface(prepared2, rect1).unwrap(); // swaps back to Buffer A

        // Frame 3: uses Buffer A again
        let prepared3 = overlay.prepare_surface(rect1, false, false).unwrap();
        assert_eq!(prepared3.mem_dc, dc_a, "Third frame should reuse Buffer A");

        // Smaller dimensions should hit capacity cache (not trigger reallocation)
        let rect_small = RECT {
            left: 100,
            top: 100,
            right: 350,
            bottom: 350,
        };
        // Currently we are on Buffer A. We commit it -> swaps to Buffer B.
        overlay.commit_surface(prepared3, rect1).unwrap();
        let prepared_small = overlay.prepare_surface(rect_small, false, false).unwrap();
        assert_eq!(
            prepared_small.mem_dc, dc_b,
            "DC should be re-used when dimensions are smaller than cache capacity"
        );
        overlay.commit_surface(prepared_small, rect_small).unwrap(); // swaps back to Buffer A

        // Large dimensions exceeding capacity -> triggers reallocation of both buffers
        let rect_large = RECT {
            left: 100,
            top: 100,
            right: 700,
            bottom: 700,
        };
        let prepared_large = overlay.prepare_surface(rect_large, false, false).unwrap();
        let dc_large_a = prepared_large.mem_dc;
        assert_ne!(
            dc_large_a, dc_a,
            "Reallocation must create fresh DC handles"
        );
        assert_ne!(
            dc_large_a, dc_b,
            "Reallocation must create fresh DC handles"
        );

        assert_eq!(overlay.cached_width.get(), 600 + 256);
        assert_eq!(
            overlay.cached_height.get(),
            600 + OVERLAY_TOP_EXTENSION + 256
        );
    }
}
