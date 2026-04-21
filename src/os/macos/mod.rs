use super::{OcrEngine, ScreenshotPlatform, ThumbnailProvider, WindowManager};
use crate::i18n::lang::Language;
use egui::{ColorImage, Rect};
use image::DynamicImage;
use std::path::Path;

pub struct MacosPlatform;

impl WindowManager for MacosPlatform {
    fn get_window_handle(&self, _cc: &eframe::CreationContext<'_>) -> usize {
        0 // Placeholder
    }

    fn show_window_restore(&self, _hwnd_usize: usize) {}
    fn show_window_restore_offscreen(&self, _hwnd_usize: usize) {}
    fn show_window_hide(&self, _hwnd_usize: usize) {}
    fn force_get_focus(&self, _hwnd_usize: usize) {}
    fn set_launch_on_startup(&self, _enabled: bool) -> Result<(), String> {
        Ok(())
    }
}

impl ScreenshotPlatform for MacosPlatform {
    fn lock_cursor_for_screenshot(&self) {}
    fn unlock_cursor(&self) {}
    fn get_taskbar_rects(&self) -> Vec<Rect> {
        Vec::new()
    }
}

impl ThumbnailProvider for MacosPlatform {
    fn load_thumbnail(&self, _path: &Path, _size: (u32, u32)) -> Result<ColorImage, String> {
        Err("Not implemented on macOS".to_string())
    }
}

impl OcrEngine for MacosPlatform {
    fn recognize_text(&self, _img: DynamicImage, _language: Language) -> Result<String, String> {
        Err("Not implemented on macOS".to_string())
    }
}
