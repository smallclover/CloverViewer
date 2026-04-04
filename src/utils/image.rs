use std::path::{Path, PathBuf};
use rayon::prelude::*;
use tray_icon::Icon;
use crate::model::image_meta::SUPPORTED_IMAGE_EXTENSIONS;
use crate::ui::resources::APP_IMG;

/// 统一的判断逻辑
pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| SUPPORTED_IMAGE_EXTENSIONS.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn collect_images(dir: &Path) -> Vec<PathBuf> {
    // 使用 rayon 并行迭代器加速目录扫描和过滤
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut result: Vec<PathBuf> = entries
        .flatten()
        .par_bridge()
        .map(|e| e.path())
        .filter(|p| is_image(p))
        .collect();
    result.sort();
    result
}

pub fn load_icon()-> egui::IconData {
    let img = image::load_from_memory(APP_IMG)
        .expect("无法读取内嵌图标")
        .into_rgba8();
    let (w, h) = img.dimensions();

    egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    }
}

pub fn load_tray_icon() -> Icon {
    let img = image::load_from_memory(APP_IMG)
        .expect("无法读取内嵌图标")
        .resize_exact(16, 16, image::imageops::FilterType::Lanczos3)
        .into_rgba8();

    let (w, h) = img.dimensions();

    Icon::from_rgba(img.into_raw(), w, h)
        .expect("Failed to create tray icon")
}
