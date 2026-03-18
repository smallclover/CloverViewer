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
            WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_RESTORE}
        },
    },
    core::PCWSTR,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow, SetWindowPos, HWND_TOP, SWP_NOSIZE, SWP_NOZORDER};

/// 操作窗口
/// /// ============================================================================================
/// egui 的程序从托盘状态唤醒
/// 参照 github的讨论https://github.com/emilk/egui/discussions/737
/// 关键在于使用ShowWindow方法，将主程序真正唤醒
/// 如果没有唤醒egui的update方法是不会有任何响应
/// 除了ShowWindow之外还要配合一个全局的变量来控制当前
/// 窗口是隐藏还是表示的状态Arc<Mutex<bool>>
// pub fn show_window_mini(hwnd_isize: isize){
//     let window_handle = get_window_handle(hwnd_isize);
//     unsafe { let _ = ShowWindow(window_handle, SW_MINIMIZE); }
// }

pub fn show_window_restore(hwnd_isize: isize){
    let window_handle = get_window_handle(hwnd_isize);
    unsafe { let _ = ShowWindow(window_handle, SW_RESTORE); }
}
// pub fn show_window_show(hwnd_isize: isize){
//     let window_handle = get_window_handle(hwnd_isize);
//     unsafe { let _ = ShowWindow(window_handle, SW_SHOW); }
// }

pub fn show_window_restore_offscreen(hwnd_isize: isize) {
    let window_handle = get_window_handle(hwnd_isize);

    unsafe {
        // 1. 在 Restore 之前，强制同步把窗口移到十万八千里外
        // SWP_NOSIZE: 不改变大小
        // SWP_NOZORDER: 不改变 Z 轴层级顺序
        let _ = SetWindowPos(
            window_handle,
            HWND_TOP.into(),
            -20000,
            -20000,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER,
        );

        // 2. 现在再调用恢复，它会在 -20000 的位置被唤醒，实现真正的无缝隐形唤醒
        let _ = ShowWindow(window_handle, SW_RESTORE);
    }
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

/// 强制获取焦点，否则最小化状态下无法退出截图状态
pub fn force_get_focus(hwnd_isize: isize) {
    unsafe {
        let window_handle = get_window_handle(hwnd_isize);
        let fg_hwnd = GetForegroundWindow();

        // 如果焦点已经是我们的窗口，直接返回
        if fg_hwnd == window_handle { return; }

        // 获取当前占据焦点的窗口线程 ID
        let fg_thread = GetWindowThreadProcessId(fg_hwnd, None);
        // 获取我们自己程序的线程 ID
        let current_thread = GetCurrentThreadId();

        if fg_thread != current_thread && fg_thread != 0 {
            // 核心魔法：将我们的线程输入附加到当前前台线程上
            let _ = AttachThreadInput(current_thread, fg_thread, true.into());

            // 此时我们有了和前台窗口一样的权限，趁机把窗口推到最前并设置焦点
            BringWindowToTop(window_handle).expect("Win32系统调用报错");
            let _ = SetForegroundWindow(window_handle);

            // 抢夺完毕，解除附加
            let _ = AttachThreadInput(current_thread, fg_thread, false.into());
        } else {
            // 如果是在同一个线程或者找不到前台线程，直接尝试
            BringWindowToTop(window_handle).expect("Win32系统调用报错");
            let _ = SetForegroundWindow(window_handle);
        }
    }
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