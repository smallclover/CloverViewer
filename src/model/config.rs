use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    env,
};
use crate::i18n::lang::Language;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub language: Language,
    #[serde(default = "default_zoom_sensitivity")]
    pub zoom_sensitivity: f32,
}

fn default_zoom_sensitivity() -> f32 {
    1.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            zoom_sensitivity: default_zoom_sensitivity(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = env::current_exe().unwrap_or_default();
    path.set_file_name("config.json");
    path
}

pub fn load_config() -> Config {
    let path = get_config_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}
