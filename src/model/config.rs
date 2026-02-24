use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    env,
};
use std::sync::Arc;
use egui::{Context, Id};
use crate::i18n::lang::Language;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct HotkeysConfig {
    pub show_screenshot: String,
    pub copy_screenshot: String,
}

impl Default for HotkeysConfig {
    fn default() -> Self {
        Self {
            show_screenshot: "Alt+S".to_string(),
            copy_screenshot: "Ctrl+C".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub language: Language,
    #[serde(default = "default_zoom_sensitivity")]
    pub zoom_sensitivity: f32,
    #[serde(default)]
    pub hotkeys: HotkeysConfig,
}

fn default_zoom_sensitivity() -> f32 {
    1.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            zoom_sensitivity: default_zoom_sensitivity(),
            hotkeys: HotkeysConfig::default(),
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

pub fn get_context_config(ctx: &Context) -> Arc<Config>{
    let config = ctx.data(|d| d.get_temp::<Arc<Config>>(Id::new("config")).unwrap());
    config
}

pub fn update_context_config(ctx:&Context, config: &Arc<Config>){
    // 保持 Config 在 context 中更新
    ctx.data_mut(|data| data.insert_temp(Id::new("config"), Arc::clone(config)));
}