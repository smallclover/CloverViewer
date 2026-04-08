use super::Platform;
use egui::{ColorImage, Rect};
use image::DynamicImage;
use std::path::Path;

pub struct MacosPlatform;

impl MacosPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Platform for MacosPlatform {
    fn get_window_handle(&self, _cc: &eframe::CreationContext<'_>) -> usize {
        0 // Placeholder
    }

    fn show_window_restore(&self, _hwnd_usize: usize) {}
    fn show_window_restore_offscreen(&self, _hwnd_usize: usize) {}
    fn show_window_hide(&self, _hwnd_usize: usize) {}
    fn force_get_focus(&self, _hwnd_usize: usize) {}

    fn lock_cursor_for_screenshot(&self) {}
    fn unlock_cursor(&self) {}
    fn get_taskbar_rects(&self) -> Vec<Rect> {
        Vec::new()
    }

    fn load_thumbnail(&self, _path: &Path, _size: (u32, u32)) -> Result<ColorImage, String> {
        Err("Not implemented on macOS".to_string())
    }

    fn recognize_text(&self, _img: DynamicImage) -> Result<String, String> {
        Err("Not implemented on macOS".to_string())
    }
}
