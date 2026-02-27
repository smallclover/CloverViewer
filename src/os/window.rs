use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    path::PathBuf,
    os::windows::ffi::OsStrExt,
    mem,slice,iter
};
use egui::ColorImage;
use windows::{
    core::{Interface},
    Win32::{
        System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
        Graphics::Gdi::{GetObjectW, BITMAP, DeleteObject, HGDIOBJ},
        Foundation::{SIZE, HWND},
        UI::{
            Shell::{
                SHCreateItemFromParsingName, IShellItem, IShellItemImageFactory,
                SIIGBF_RESIZETOFIT, SIIGBF_BIGGERSIZEOK,
            },
            WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_MINIMIZE, SW_RESTORE}
        },
    },
    core::PCWSTR,
};


/// 操作窗口
/// /// ============================================================================================
/// egui 的程序从托盘状态唤醒
/// 参照 github的讨论https://github.com/emilk/egui/discussions/737
/// 关键在于使用ShowWindow方法，将主程序真正唤醒
/// 如果没有唤醒egui的update方法是不会有任何响应
/// 除了ShowWindow之外还要配合一个全局的变量来控制当前
/// 窗口是隐藏还是表示的状态Arc<Mutex<bool>>
pub fn show_window_mini(hwnd_isize: isize){
    let window_handle = get_window_handle(hwnd_isize);
    unsafe { let _ = ShowWindow(window_handle, SW_MINIMIZE); }
}

pub fn show_window_restore(hwnd_isize: isize){
    let window_handle = get_window_handle(hwnd_isize);
    unsafe { let _ = ShowWindow(window_handle, SW_RESTORE); }
}

pub fn show_window_hide(hwnd_isize: isize){
    let window_handle = get_window_handle(hwnd_isize);
    unsafe { let _ = ShowWindow(window_handle, SW_HIDE); }
}

pub fn get_window_handle(hwnd_isize: isize) -> HWND {
    HWND(hwnd_isize as *mut std::ffi::c_void)
}

pub fn get_hwnd_isize(cc: &eframe::CreationContext<'_>) -> isize {

    // 获取原生句柄并转为 isize 以支持跨线程
    let RawWindowHandle::Win32(handle) = cc.window_handle().unwrap().as_raw() else {
        panic!("Unsupported platform");
    };
    
    handle.hwnd.get()
}

/// 获取win缩略图
/// ================================================================================================

// 直接调用Win的缩略图，来提高加载速度

struct CoUninitializeOnDrop;

impl Drop for CoUninitializeOnDrop {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub fn load_thumbnail_windows(path: &PathBuf, size: (u32, u32)) -> Result<ColorImage, String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().map_err(|e| e.to_string())?;
        let _ = CoUninitializeOnDrop;

        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(iter::once(0)).collect();
        let shell_item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None).map_err(|e| e.to_string())?;
        let image_factory: IShellItemImageFactory = shell_item.cast().map_err(|e| e.to_string())?;

        let size_struct = SIZE { cx: size.0 as i32, cy: size.1 as i32 };
        let hbitmap = image_factory.GetImage(size_struct, SIIGBF_RESIZETOFIT | SIIGBF_BIGGERSIZEOK).map_err(|e| e.to_string())?;

        let hgdiobj: HGDIOBJ = mem::transmute(hbitmap);

        let mut bitmap: BITMAP = mem::zeroed();
        if GetObjectW(hgdiobj, size_of::<BITMAP>() as i32, Some(&mut bitmap as *mut _ as *mut _)) == 0 {
            let _ = DeleteObject(hgdiobj);
            return Err("Failed to get bitmap object".to_string());
        }

        let width = bitmap.bmWidth as usize;
        let height = bitmap.bmHeight as usize;
        let stride = bitmap.bmWidthBytes as usize;
        let bits_ptr = bitmap.bmBits as *const u8;

        if bits_ptr.is_null() {
            let _ = DeleteObject(hgdiobj);
            return Err("Bitmap bits are null".to_string());
        }

        let mut pixels = Vec::with_capacity(width * height);
        let bits = slice::from_raw_parts(bits_ptr, stride * height);

        //Windows GDI位图(BGRA (Blue-Green-Red-Alpha))转egui RGBA
        for y in 0..height {
            for x in 0..width {
                let offset = y * stride + x * 4;
                if offset + 3 < bits.len() {
                    let b = bits[offset];
                    let g = bits[offset + 1];
                    let r = bits[offset + 2];
                    let a = bits[offset + 3];
                    pixels.push(egui::Color32::from_rgba_premultiplied(r, g, b, a));
                } else {
                    pixels.push(egui::Color32::BLACK);
                }
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