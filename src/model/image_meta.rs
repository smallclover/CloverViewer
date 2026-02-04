use std::path::PathBuf;
/// 图片EXIF信息
#[derive(Clone, Debug)]
pub struct ImageProperties {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub name: String,
    pub date: String,
    pub bits: Option<u32>,
    pub make: String,
    pub model: String,
    pub focal_length: String,
    pub f_number: String,
    pub iso: Option<u32>,

}

impl Default for ImageProperties {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            width: 0,
            height: 0,
            size: 0,
            name: "".to_string(),
            date: "".to_string(),
            bits: None,
            make: "".to_string(),
            model: "".to_string(),
            focal_length: "".to_string(),
            f_number: "".to_string(),
            iso: None,
        }
    }
}
