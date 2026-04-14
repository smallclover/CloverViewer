use crate::model::image_meta::SUPPORTED_IMAGE_EXTENSIONS;
use crate::ui::resources::APP_IMG;
use std::path::{Path, PathBuf};
use tray_icon::Icon;

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
        .map(|e| e.path())
        .filter(|p| is_image(p))
        .collect();
    result.sort();
    result
}

pub fn load_icon() -> Option<egui::IconData> {
    let img = match image::load_from_memory(APP_IMG) {
        Ok(img) => img.into_rgba8(),
        Err(err) => {
            tracing::error!("Failed to read embedded app icon: {}", err);
            return None;
        }
    };
    let (w, h) = img.dimensions();

    Some(egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    })
}

pub fn load_tray_icon() -> Result<Icon, String> {
    let img = image::load_from_memory(APP_IMG)
        .map_err(|err| format!("Failed to read embedded app icon: {err}"))?
        .resize_exact(16, 16, image::imageops::FilterType::Lanczos3)
        .into_rgba8();

    let (w, h) = img.dimensions();

    Icon::from_rgba(img.into_raw(), w, h)
        .map_err(|err| format!("Failed to create tray icon: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{collect_images, is_image};
    use std::{
        env, fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after UNIX_EPOCH")
            .as_nanos();
        env::temp_dir().join(format!("cloverviewer-images-{timestamp}"))
    }

    #[test]
    fn is_image_matches_supported_extensions_case_insensitively() {
        assert!(is_image(Path::new("sample.PNG")));
        assert!(is_image(Path::new("photo.JpEg")));
        assert!(!is_image(Path::new("notes.txt")));
        assert!(!is_image(Path::new("no_extension")));
    }

    #[test]
    fn collect_images_filters_and_sorts_image_files() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).expect("Test directory should be created");
        fs::write(dir.join("b.jpg"), []).expect("Image placeholder should be written");
        fs::write(dir.join("a.png"), []).expect("Image placeholder should be written");
        fs::write(dir.join("notes.txt"), []).expect("Non-image placeholder should be written");

        let files = collect_images(&dir);

        let names: Vec<String> = files
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .collect();
        assert_eq!(names, vec!["a.png", "b.jpg"]);

        let _ = fs::remove_dir_all(dir);
    }
}
