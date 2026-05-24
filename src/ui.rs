//! UI and Rendering.
//!
//! This module handles the visual overlay that indicates an active session.
//! It uses a WS_EX_LAYERED window with per-pixel alpha transparency.
//! Rendering is performed using `tiny-skia` into a GDI DIB section,
//! which is then uploaded via `UpdateLayeredWindow`.

use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Transform};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC,
    CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
    DeferWindowPos, HDWP, RegisterClassW, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER,
    ULW_ALPHA, UpdateLayeredWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
    WS_POPUP,
};
use windows::core::w;

/// Manages a transparent overlay window.
pub struct Overlay {
    pub hwnd: HWND,
}

/// Constant for the vertical extension above the window (the "header").
pub const OVERLAY_TOP_EXTENSION: i32 = 7;

impl Overlay {
    /// Internal window procedure for the overlay.
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // The overlay is mostly passive and doesn't handle inputs directly.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// Creates a new transparent, click-through overlay window.
    pub fn new() -> windows::core::Result<Self> {
        let instance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)? };
        let class_name = w!("WinGlideOverlay");

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
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

        Ok(Self { hwnd })
    }

    /// Sets the target window as the "owner" of the overlay.
    /// This ensures the overlay stays on top of the target window.
    pub fn set_owner(&self, owner: HWND) {
        use windows::Win32::UI::WindowsAndMessaging::{GWLP_HWNDPARENT, SetWindowLongPtrW};
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_HWNDPARENT, owner.0 as isize);
        }
    }

    /// Redraws the overlay based on the target window's dimensions.
    ///
    /// This uses `tiny-skia` for high-quality 2D rendering directly into
    /// a GDI DIB section to avoid unnecessary memory copies.
    pub fn redraw(&self, rect: RECT) -> windows::core::Result<()> {
        let width = rect.right - rect.left;
        let height = (rect.bottom - rect.top) + OVERLAY_TOP_EXTENSION;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        unsafe {
            let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
            if screen_dc.is_invalid() {
                return Ok(());
            }

            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_invalid() {
                windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
                return Ok(());
            }

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Negative height means top-down bitmap
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0, // BI_RGB
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits = std::ptr::null_mut();
            let result = CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0);

            match result {
                Ok(bitmap) => {
                    if !bits.is_null() {
                        // 1. Wrap the DIB section's memory in a tiny-skia PixmapMut.
                        // This allows rendering directly into GDI-managed memory, eliminating a copy.
                        let slice = std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize);
                        if let Some(mut pixmap) = PixmapMut::from_bytes(slice, width as u32, height as u32) {
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
                        }

                        // 2. Upload to GDI Layered Window
                        let old_obj = SelectObject(mem_dc, bitmap);

                        let pt_src = POINT { x: 0, y: 0 };
                        let pt_dst = POINT {
                            x: rect.left,
                            y: rect.top - OVERLAY_TOP_EXTENSION,
                        };
                        let size = SIZE {
                            cx: width,
                            cy: height,
                        };

                        let blend = BLENDFUNCTION {
                            BlendOp: AC_SRC_OVER as u8,
                            BlendFlags: 0,
                            SourceConstantAlpha: 255,
                            AlphaFormat: AC_SRC_ALPHA as u8,
                        };

                        if let Err(e) = UpdateLayeredWindow(
                            self.hwnd,
                            screen_dc,
                            Some(&pt_dst),
                            Some(&size),
                            mem_dc,
                            Some(&pt_src),
                            None,
                            Some(&blend),
                            ULW_ALPHA,
                        ) {
                            eprintln!("Overlay: UpdateLayeredWindow failed: {:?}", e);
                        }

                        let _ = SelectObject(mem_dc, old_obj);
                    }
                    let _ = DeleteObject(bitmap);
                }
                Err(e) => {
                    eprintln!("Overlay: CreateDIBSection failed: {:?}", e);
                }
            }

            let _ = DeleteDC(mem_dc);
            windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
        }

        Ok(())
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

    /// Shows or hides the overlay.
    pub fn show(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd, cmd);
        }
    }
}
