use crate::i18n::lang::Language;
use egui::{ColorImage, Rect, pos2};
use image::DynamicImage;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::{iter, slice};
use windows::Win32::Foundation::{HWND, POINT, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BITMAP, DeleteObject, GetMonitorInfoW, GetObjectW, HGDIOBJ, MONITOR_DEFAULTTONEAREST,
    MONITORINFO, MonitorFromPoint,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Shell::{
    IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK,
    SIIGBF_RESIZETOFIT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, ClipCursor, FindWindowA, FindWindowExA, GetCursorPos, GetForegroundWindow,
    GetSystemMetrics, GetWindowRect, GetWindowThreadProcessId, HWND_TOP, SM_CXVIRTUALSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_HIDE, SW_RESTORE, SWP_NOSIZE,
    SWP_NOZORDER, SetForegroundWindow, SetWindowPos, ShowWindow,
};
use windows::core::{Interface, PCSTR, PCWSTR, s};

use super::Platform;

pub mod ocr;

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }

    fn get_window_handle(&self, hwnd_usize: usize) -> HWND {
        HWND(hwnd_usize as *mut std::ffi::c_void)
    }
}

impl Platform for WindowsPlatform {
    fn get_window_handle(&self, cc: &eframe::CreationContext<'_>) -> usize {
        let Ok(window_handle) = cc.window_handle() else {
            panic!("Failed to get window handle");
        };
        let RawWindowHandle::Win32(handle) = window_handle.as_raw() else {
            panic!("Unsupported platform");
        };

        handle.hwnd.get() as usize
    }

    fn show_window_restore(&self, hwnd_usize: usize) {
        let window_handle = self.get_window_handle(hwnd_usize);
        unsafe {
            let _ = ShowWindow(window_handle, SW_RESTORE);
        }
    }

    fn show_window_restore_offscreen(&self, hwnd_usize: usize) {
        let window_handle = self.get_window_handle(hwnd_usize);
        unsafe {
            let _ = SetWindowPos(
                window_handle,
                HWND_TOP.into(),
                -20000,
                -20000,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER,
            );
            let _ = ShowWindow(window_handle, SW_RESTORE);
        }
    }

    fn show_window_hide(&self, hwnd_usize: usize) {
        let window_handle = self.get_window_handle(hwnd_usize);
        unsafe {
            let _ = ShowWindow(window_handle, SW_HIDE);
        }
    }

    fn force_get_focus(&self, hwnd_usize: usize) {
        unsafe {
            let window_handle = self.get_window_handle(hwnd_usize);
            let fg_hwnd = GetForegroundWindow();

            if fg_hwnd == window_handle {
                return;
            }

            let fg_thread = GetWindowThreadProcessId(fg_hwnd, None);
            let current_thread = GetCurrentThreadId();

            if fg_thread != current_thread && fg_thread != 0 {
                let _ = AttachThreadInput(current_thread, fg_thread, true);
                if let Err(e) = BringWindowToTop(window_handle) {
                    tracing::error!("BringWindowToTop failed: {:?}", e);
                }
                let _ = SetForegroundWindow(window_handle);
                let _ = AttachThreadInput(current_thread, fg_thread, false);
            } else {
                if let Err(e) = BringWindowToTop(window_handle) {
                    tracing::error!("BringWindowToTop failed: {:?}", e);
                }
                let _ = SetForegroundWindow(window_handle);
            }
        }
    }

    fn lock_cursor_for_screenshot(&self) {
        unsafe {
            let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            let mut pt = POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);

            let hmonitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);

            let mut monitor_info: MONITORINFO = std::mem::zeroed();
            monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            let _ = GetMonitorInfoW(hmonitor, &mut monitor_info);

            let bottom_limit = if monitor_info.rcMonitor.bottom > 0 {
                monitor_info.rcMonitor.bottom - 2
            } else {
                vy + vh - 5
            };

            let rect = RECT {
                left: vx,
                top: vy,
                right: vx + vw,
                bottom: bottom_limit,
            };

            ClipCursor(Some(&rect as *const RECT)).expect("Calling Windows API failed!");
        }
    }

    fn unlock_cursor(&self) {
        unsafe {
            ClipCursor(None).expect("Calling Windows API failed!");
        }
    }

    fn get_taskbar_rects(&self) -> Vec<Rect> {
        let mut rects = Vec::new();
        unsafe {
            let mut push_rect_from_hwnd = |hwnd: HWND| {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    rects.push(Rect::from_min_max(
                        pos2(rect.left as f32, rect.top as f32),
                        pos2(rect.right as f32, rect.bottom as f32),
                    ));
                }
            };

            if let Ok(hwnd_main) = FindWindowA(s!("Shell_TrayWnd"), PCSTR::null())
                && !hwnd_main.0.is_null()
            {
                push_rect_from_hwnd(hwnd_main);
            }

            let mut current_hwnd = HWND::default();
            loop {
                match FindWindowExA(
                    HWND::default().into(),
                    current_hwnd.into(),
                    s!("Shell_SecondaryTrayWnd"),
                    PCSTR::null(),
                ) {
                    Ok(hwnd) if !hwnd.0.is_null() => {
                        push_rect_from_hwnd(hwnd);
                        current_hwnd = hwnd;
                    }
                    _ => break,
                }
            }
        }
        rects
    }

    fn load_thumbnail(&self, path: &Path, size: (u32, u32)) -> Result<ColorImage, String> {
        unsafe {
            let wide_path: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(iter::once(0))
                .collect();
            let shell_item: IShellItem =
                SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None)
                    .map_err(|e| e.to_string())?;
            let image_factory: IShellItemImageFactory =
                shell_item.cast().map_err(|e| e.to_string())?;

            let size_struct = SIZE {
                cx: size.0 as i32,
                cy: size.1 as i32,
            };
            let hbitmap = image_factory
                .GetImage(size_struct, SIIGBF_RESIZETOFIT | SIIGBF_BIGGERSIZEOK)
                .map_err(|e| e.to_string())?;

            let hgdiobj: HGDIOBJ = hbitmap.into();

            let mut bitmap: BITMAP = Default::default();
            if GetObjectW(
                hgdiobj,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut _ as *mut _),
            ) == 0
            {
                let _ = DeleteObject(hgdiobj);
                return Err("Failed to get bitmap object".to_string());
            }

            let width = usize::try_from(bitmap.bmWidth)
                .map_err(|_| "Bitmap width must be non-negative".to_string())?;
            let height = usize::try_from(bitmap.bmHeight)
                .map_err(|_| "Bitmap height must be non-negative".to_string())?;
            let stride = usize::try_from(bitmap.bmWidthBytes)
                .map_err(|_| "Bitmap stride must be non-negative".to_string())?;
            let bits_ptr = bitmap.bmBits as *const u8;

            if bits_ptr.is_null() {
                let _ = DeleteObject(hgdiobj);
                return Err("Bitmap bits are null".to_string());
            }

            let pixel_count = width
                .checked_mul(height)
                .ok_or_else(|| "Bitmap dimensions overflowed".to_string())?;
            let bits_len = stride
                .checked_mul(height)
                .ok_or_else(|| "Bitmap buffer length overflowed".to_string())?;
            let row_bytes = width
                .checked_mul(4)
                .ok_or_else(|| "Bitmap row width overflowed".to_string())?;

            let mut pixels = Vec::with_capacity(pixel_count);
            let bits = slice::from_raw_parts(bits_ptr, bits_len);

            if stride == row_bytes {
                pixels.extend(bits.chunks_exact(4).map(|chunk| {
                    egui::Color32::from_rgba_premultiplied(chunk[2], chunk[1], chunk[0], chunk[3])
                }));
            } else {
                for row in bits.chunks(stride).take(height) {
                    let row_end = row_bytes.min(row.len());
                    let row = &row[..row_end];
                    pixels.extend(row.chunks_exact(4).map(|chunk| {
                        egui::Color32::from_rgba_premultiplied(
                            chunk[2], chunk[1], chunk[0], chunk[3],
                        )
                    }));
                }
            }

            let _ = DeleteObject(hgdiobj);

            Ok(ColorImage {
                size: [width, height],
                source_size: egui::vec2(width as f32, height as f32),
                pixels,
            })
        }
    }

    fn recognize_text(&self, img: DynamicImage, language: Language) -> Result<String, String> {
        ocr::recognize_text_windows(img, language)
    }
}
