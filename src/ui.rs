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

fn draw_arrow(
    pixmap: &mut tiny_skia::PixmapMut,
    paint: &tiny_skia::Paint,
    center_x: f32,
    center_y: f32,
    size: f32,
    direction: ArrowDirection,
    dpi_scale: f32,
) {
    let half_w = size / 2.0;
    let half_h = size / 4.0;
    let mut pb = tiny_skia::PathBuilder::new();
    match direction {
        ArrowDirection::Up => {
            pb.move_to(center_x - half_w, center_y + half_h);
            pb.line_to(center_x, center_y - half_h);
            pb.line_to(center_x + half_w, center_y + half_h);
        }
        ArrowDirection::Down => {
            pb.move_to(center_x - half_w, center_y - half_h);
            pb.line_to(center_x, center_y + half_h);
            pb.line_to(center_x + half_w, center_y - half_h);
        }
        ArrowDirection::Left => {
            pb.move_to(center_x + half_h, center_y - half_w);
            pb.line_to(center_x - half_h, center_y);
            pb.line_to(center_x + half_h, center_y + half_w);
        }
        ArrowDirection::Right => {
            pb.move_to(center_x - half_h, center_y - half_w);
            pb.line_to(center_x + half_h, center_y);
            pb.line_to(center_x - half_h, center_y + half_w);
        }
    }
    if let Some(path) = pb.finish() {
        let stroke = Stroke {
            width: 8.0 * dpi_scale,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };

        pixmap.stroke_path(
            &path,
            paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}

/// Manages a transparent overlay window.
pub struct Overlay {
    pub hwnd: HWND,
    cached_dc: std::cell::RefCell<Option<windows::Win32::Graphics::Gdi::HDC>>,
    cached_bitmap: std::cell::RefCell<Option<windows::Win32::Graphics::Gdi::HBITMAP>>,
    cached_old_bitmap: std::cell::RefCell<Option<windows::Win32::Graphics::Gdi::HGDIOBJ>>,
    cached_width: std::cell::Cell<i32>,
    cached_height: std::cell::Cell<i32>,
    cached_bits: std::cell::Cell<*mut u8>,
}

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            let dc = self.cached_dc.replace(None);
            let old_bmp = self.cached_old_bitmap.replace(None);
            let bmp = self.cached_bitmap.replace(None);

            if let Some(mem_dc) = dc.filter(|d| !d.is_invalid()) {
                if let Some(old) = old_bmp.filter(|o| !o.is_invalid()) {
                    let _ = windows::Win32::Graphics::Gdi::SelectObject(mem_dc, old);
                }
                if let Some(bitmap) = bmp.filter(|b| !b.is_invalid()) {
                    let _ = windows::Win32::Graphics::Gdi::DeleteObject(bitmap);
                }
                let _ = windows::Win32::Graphics::Gdi::DeleteDC(mem_dc);
            }
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
            cached_dc: std::cell::RefCell::new(None),
            cached_bitmap: std::cell::RefCell::new(None),
            cached_old_bitmap: std::cell::RefCell::new(None),
            cached_width: std::cell::Cell::new(0),
            cached_height: std::cell::Cell::new(0),
            cached_bits: std::cell::Cell::new(std::ptr::null_mut()),
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
            let cache_dc_opt = *self.cached_dc.borrow();
            let cache_bmp_opt = *self.cached_bitmap.borrow();
            let cache_old_opt = *self.cached_old_bitmap.borrow();
            let cache_w = self.cached_width.get();
            let cache_h = self.cached_height.get();
            let mut bits = self.cached_bits.get();

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
                let bitmap =
                    match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut new_bits, None, 0) {
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
                // Clear with transparent (GDI memory might be uninitialized)
                pixmap.fill(Color::TRANSPARENT);

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
                    draw_arrow(
                        &mut pixmap,
                        &white_paint,
                        w / 2.0,
                        OVERLAY_TOP_EXTENSION as f32 + margin + arrow_size / 2.0,
                        arrow_size,
                        top_direction,
                        dpi_scale,
                    );

                    // Bottom Arrow
                    draw_arrow(
                        &mut pixmap,
                        &white_paint,
                        w / 2.0,
                        h - margin - arrow_size / 2.0,
                        arrow_size,
                        bottom_direction,
                        dpi_scale,
                    );

                    // Left Arrow
                    draw_arrow(
                        &mut pixmap,
                        &white_paint,
                        margin + arrow_size / 2.0,
                        OVERLAY_TOP_EXTENSION as f32 + (h - OVERLAY_TOP_EXTENSION as f32) / 2.0,
                        arrow_size,
                        left_direction,
                        dpi_scale,
                    );

                    // Right Arrow
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

                    // 2. Center vertically
                    let text_height = rect.bottom - rect.top;
                    let available_height = target_bottom - target_top;
                    let y_offset = ((available_height - text_height) / 2).max(0);

                    let mut draw_rect = RECT {
                        left: target_left,
                        top: target_top + y_offset,
                        right: target_right,
                        bottom: target_top + y_offset + text_height,
                    };

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

        // Call prepare_surface for the first time (allocates 300+256 by 307+256)
        let prepared1 = overlay.prepare_surface(rect1, false, false).unwrap();
        let dc1 = prepared1.mem_dc;

        // Call prepare_surface again with the same dimensions (should hit capacity cache)
        let prepared2 = overlay.prepare_surface(rect1, false, false).unwrap();
        let dc2 = prepared2.mem_dc;

        assert_eq!(
            dc1, dc2,
            "DC should be cached and re-used for identical dimensions"
        );

        // Call prepare_surface with smaller dimensions (should hit capacity cache and re-use)
        let rect_small = RECT {
            left: 100,
            top: 100,
            right: 350,
            bottom: 350,
        };
        let prepared_small = overlay.prepare_surface(rect_small, false, false).unwrap();
        assert_eq!(
            prepared_small.mem_dc, dc1,
            "DC should be re-used when dimensions are smaller than cache capacity"
        );

        // Call prepare_surface with dimensions exceeding capacity (should trigger reallocation)
        let rect_large = RECT {
            left: 100,
            top: 100,
            right: 700,
            bottom: 700,
        };
        let prepared3 = overlay.prepare_surface(rect_large, false, false).unwrap();
        let _dc3 = prepared3.mem_dc;

        assert_eq!(overlay.cached_width.get(), 600 + 256);
        assert_eq!(
            overlay.cached_height.get(),
            600 + OVERLAY_TOP_EXTENSION + 256
        );
    }
}
