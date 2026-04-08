use egui::{ColorImage, Rect};
use image::DynamicImage;
use std::path::Path;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

// 定义跨平台统一接口
pub trait Platform {
    // 窗口操作
    fn get_window_handle(&self, cc: &eframe::CreationContext<'_>) -> usize;
    fn show_window_restore(&self, hwnd_usize: usize);
    fn show_window_restore_offscreen(&self, hwnd_usize: usize);
    fn show_window_hide(&self, hwnd_usize: usize);
    fn force_get_focus(&self, hwnd_usize: usize);

    // 截图与光标
    fn lock_cursor_for_screenshot(&self);
    fn unlock_cursor(&self);
    fn get_taskbar_rects(&self) -> Vec<Rect>;

    // 图像与缩略图
    fn load_thumbnail(&self, path: &Path, size: (u32, u32)) -> Result<ColorImage, String>;

    // OCR 识别
    fn recognize_text(&self, img: DynamicImage) -> Result<String, String>;
}

// 获取当前平台的 Handler
pub fn current_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacosPlatform::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        panic!("Unsupported platform");
    }
}
