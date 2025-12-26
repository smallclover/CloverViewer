use std::path::{Path, PathBuf};

pub fn is_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase()
            .as_str(),
        "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp" | "avif" | "heic" | "heif"
    )
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