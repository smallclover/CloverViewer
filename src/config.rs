use serde::{Deserialize, Serialize};
use crate::i18n::Language;
use std::{
    fs,
    path::PathBuf,
    env,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub language: Language,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
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
