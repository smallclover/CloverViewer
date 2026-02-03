use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ImageProperties {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub name: String,
    pub date: String,
    pub dpi: Option<u32>,
    pub bits: Option<u32>,
    pub make: String,
    pub model: String,
    pub focal_length: String,
    pub f_number: String,
    pub exposure_time: String,
    pub iso: Option<u32>,
    pub exposure_bias: String,
    pub flash: String,
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
            dpi: None,
            bits: None,
            make: "".to_string(),
            model: "".to_string(),
            focal_length: "".to_string(),
            f_number: "".to_string(),
            exposure_time: "".to_string(),
            iso: None,
            exposure_bias: "".to_string(),
            flash: "".to_string(),
        }
    }
}
