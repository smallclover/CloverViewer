use std::path::PathBuf;
use egui::TextureHandle;

pub struct ImageEntry {
    pub path: PathBuf, // 文件路径
    pub name: String, // 文件名
}

pub struct BusinessData {
    pub image_list: Vec<ImageEntry>,
    pub current_index: Option<usize>,
    pub current_texture: Option<TextureHandle>,
}

impl BusinessData {
    pub fn new() -> Self {
        Self {
            image_list: Vec::new(),
            current_index: None,
            current_texture: None,
        }
    }

    // 切换下一张图片
    pub fn next_image(&mut self) {
        if let Some(idx) = self.current_index {
            if !self.image_list.is_empty() {    
                self.current_index = Some((idx + 1) % self.image_list.len());
            }
        }
    }
}