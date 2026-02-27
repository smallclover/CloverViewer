use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_MINIMIZE, SW_RESTORE};

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