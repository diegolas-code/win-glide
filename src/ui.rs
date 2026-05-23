use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, AC_SRC_ALPHA,
    AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, DIB_RGB_COLORS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, SW_HIDE,
    SW_SHOW, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    UpdateLayeredWindow, ULW_ALPHA,
};
use tiny_skia::{Color, Pixmap};

pub struct Overlay {
    pub hwnd: HWND,
}

const TOP_EXTENSION: i32 = 10;

impl Overlay {
    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
    pub fn new() -> windows::core::Result<Self> {
        let instance = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)? };
        let class_name = w!("WinGlideOverlay");

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        unsafe {
            let _ = RegisterClassW(&wnd_class);
        }

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT,
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

    pub fn redraw(&self, rect: RECT) -> windows::core::Result<()> {
        let width = rect.right - rect.left;
        let height = (rect.bottom - rect.top) + TOP_EXTENSION;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();
        // Fill the entire pixmap with a semi-transparent tint
        // Color: Win-glide blue (0, 120, 215) at ~20% opacity (alpha: 50)
        pixmap.fill(Color::from_rgba8(0, 120, 215, 50));

        // Convert RGBA to BGRA for Win32
        let mut bgra = pixmap.data().to_vec();
        for chunk in bgra.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        unsafe {
            let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
            let mem_dc = CreateCompatibleDC(screen_dc);

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0, // BI_RGB
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits = std::ptr::null_mut();
            let bitmap = CreateDIBSection(
                mem_dc,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            )?;

            std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits as *mut u8, bgra.len());

            let old_obj = SelectObject(mem_dc, bitmap);

            let pt_src = POINT { x: 0, y: 0 };
            let pt_dst = POINT { x: rect.left, y: rect.top - TOP_EXTENSION };
            let size = SIZE { cx: width, cy: height };

            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            UpdateLayeredWindow(
                self.hwnd,
                screen_dc,
                Some(&pt_dst),
                Some(&size),
                mem_dc,
                Some(&pt_src),
                None,
                Some(&blend),
                ULW_ALPHA,
            )?;

            let _ = SelectObject(mem_dc, old_obj);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
        }

        Ok(())
    }

    pub fn update_position(&self, rect: RECT) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                self.hwnd,
                HWND::default(),
                rect.left,
                rect.top - TOP_EXTENSION,
                rect.right - rect.left,
                (rect.bottom - rect.top) + TOP_EXTENSION,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE | windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER,
            );
        }
    }

    pub fn show(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd, cmd);
        }
    }
}
