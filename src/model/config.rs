use crate::i18n::lang::Language;
use egui::{Context, Id};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;
use std::{fs, path::PathBuf};

/// 获取配置目录
/// 优先使用系统配置目录，如果不存在则使用exe目录（向后兼容）
fn get_config_dir() -> PathBuf {
    // 首先尝试系统配置目录
    if let Some(config_dir) = dirs::config_dir() {
        let app_config_dir = config_dir.join("CloverViewer");
        // 如果目录已存在或可以创建，使用它
        if app_config_dir.exists() || fs::create_dir_all(&app_config_dir).is_ok() {
            return app_config_dir;
        }
    }

    // 回退到exe目录（向后兼容/便携模式）
    std::env::current_exe()
        .map(|mut p| {
            p.pop();
            p
        })
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// 获取旧配置路径（exe目录，用于迁移）
fn get_legacy_config_path() -> Option<PathBuf> {
    std::env::current_exe().ok().map(|mut p| {
        p.set_file_name("config.json");
        p
    })
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct HotkeysConfig {
    pub show_screenshot: String,
    #[serde(alias = "copy_screenshot")]
    pub copy_color: String,
}

impl Default for HotkeysConfig {
    fn default() -> Self {
        Self {
            show_screenshot: "Alt+S".to_string(),
            copy_color: "Alt+C".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    #[serde(default)]
    pub language: Language,
    #[serde(default = "default_zoom_sensitivity")]
    pub zoom_sensitivity: f32,
    #[serde(default)]
    pub hotkeys: HotkeysConfig,
    #[serde(default = "default_minimize_on_close")]
    pub minimize_on_close: bool,
    #[serde(default = "default_magnifier_enabled")]
    pub magnifier_enabled: bool,
    #[serde(default)]
    pub screenshot_hides_main_window: bool,
    #[serde(default = "default_launch_on_startup")]
    pub launch_on_startup: bool,

    #[serde(default)]
    pub window_pos: Option<(f32, f32)>,
    #[serde(default)]
    pub window_size: Option<(f32, f32)>,
}

fn default_zoom_sensitivity() -> f32 {
    1.0
}

fn default_minimize_on_close() -> bool {
    true
}

fn default_magnifier_enabled() -> bool {
    true
}

fn default_launch_on_startup() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            zoom_sensitivity: default_zoom_sensitivity(),
            hotkeys: HotkeysConfig::default(),
            minimize_on_close: default_minimize_on_close(),
            magnifier_enabled: default_magnifier_enabled(),
            screenshot_hides_main_window: false,
            launch_on_startup: default_launch_on_startup(),
            window_pos: None,
            window_size: None,
        }
    }
}

fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

/// 尝试从旧位置迁移配置
fn try_migrate_legacy_config() -> Option<Config> {
    let legacy_path = get_legacy_config_path()?;
    let new_path = get_config_path();

    // 如果新位置已存在配置，不进行迁移
    if new_path.exists() {
        return None;
    }

    // 尝试读取旧配置
    let content = fs::read_to_string(&legacy_path).ok()?;
    let config: Config = serde_json::from_str(&content).ok()?;

    // 尝试保存到新位置
    if save_config_internal(&new_path, &config) {
        tracing::info!("配置已从 {:?} 迁移到 {:?}", legacy_path, new_path);
        Some(config)
    } else {
        None
    }
}

fn save_config_internal(path: &std::path::Path, config: &Config) -> bool {
    if let Ok(content) = serde_json::to_string_pretty(config) {
        fs::write(path, content).is_ok()
    } else {
        false
    }
}

pub fn load_config() -> Config {
    let path = get_config_path();

    // 先尝试从标准位置加载
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Config>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    tracing::warn!("配置文件格式错误: {}, 尝试备份", e);
                    // 备份错误配置
                    let backup_path = path.with_extension("json.backup");
                    let _ = fs::rename(&path, &backup_path);
                }
            },
            Err(e) => {
                tracing::error!("无法读取配置文件: {}", e);
            }
        }
    }

    // 尝试从旧位置迁移
    if let Some(config) = try_migrate_legacy_config() {
        return config;
    }

    // 返回默认配置
    Config::default()
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    if !save_config_internal(&path, config) {
        tracing::error!("无法保存配置到: {:?}", path);
    }
}

pub struct ConfigStore {
    config: RwLock<Config>,
}

impl ConfigStore {
    pub fn new(config: Config) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    pub fn snapshot(&self) -> Arc<Config> {
        let config = self.config.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::new(config.clone())
    }

    pub fn replace(&self, new_config: Config) {
        let mut config = self
            .config
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *config = new_config;
    }
}

fn config_store_id() -> Id {
    Id::new("config_store")
}

pub fn get_context_config_store(ctx: &Context) -> Option<Arc<ConfigStore>> {
    ctx.data(|d| d.get_temp::<Arc<ConfigStore>>(config_store_id()))
}

pub fn get_context_config(ctx: &Context) -> Arc<Config> {
    get_context_config_store(ctx)
        .map(|store| store.snapshot())
        .unwrap_or_else(|| Arc::new(Config::default()))
}

pub fn init_context_config_store(ctx: &Context, config_store: &Arc<ConfigStore>) {
    ctx.data_mut(|data| data.insert_temp(config_store_id(), Arc::clone(config_store)));
}
