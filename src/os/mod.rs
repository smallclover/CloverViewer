use crate::i18n::lang::Language;
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
pub trait WindowManager {
    fn get_window_handle(&self, cc: &eframe::CreationContext<'_>) -> usize;
    fn show_window_restore(&self, hwnd_usize: usize);
    fn show_window_restore_offscreen(&self, hwnd_usize: usize);
    fn show_window_hide(&self, hwnd_usize: usize);
    fn force_get_focus(&self, hwnd_usize: usize);
    fn set_launch_on_startup(&self, enabled: bool) -> Result<(), String>;
}

pub trait ScreenshotPlatform {
    fn lock_cursor_for_screenshot(&self);
    fn unlock_cursor(&self);
    fn get_taskbar_rects(&self) -> Vec<Rect>;
}

pub trait ThumbnailProvider {
    fn load_thumbnail(&self, path: &Path, size: (u32, u32)) -> Result<ColorImage, String>;
}

pub trait OcrEngine {
    fn recognize_text(&self, img: DynamicImage, language: Language) -> Result<String, String>;
}

pub trait Platform:
    WindowManager + ScreenshotPlatform + ThumbnailProvider + OcrEngine
{
}

impl<T> Platform for T where T: WindowManager + ScreenshotPlatform + ThumbnailProvider + OcrEngine {}

// 获取当前平台的 Handler
pub fn current_platform() -> &'static dyn Platform {
    #[cfg(target_os = "windows")]
    {
        static PLATFORM: windows::WindowsPlatform = windows::WindowsPlatform;
        &PLATFORM
    }
    #[cfg(target_os = "macos")]
    {
        static PLATFORM: macos::MacosPlatform = macos::MacosPlatform;
        &PLATFORM
    }
    #[cfg(target_os = "linux")]
    {
        static PLATFORM: linux::LinuxPlatform = linux::LinuxPlatform;
        &PLATFORM
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        panic!("Unsupported platform");
    }
}
