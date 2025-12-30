use std::path::{Path, PathBuf};
use crate::constants;
/// 统一的判断逻辑
pub fn is_image(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| constants::SUPPORTED_IMAGE_EXTENSIONS.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn collect_images(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if is_image(&p) {
                v.push(p);
            }
        }
    }
    v
}