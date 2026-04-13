use crate::model::config::{Config, ConfigStore, save_config};
use egui::Context;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ConfigManager {
    config_store: Arc<ConfigStore>,
    pending_save: Option<Instant>,
    last_saved: Instant,
}

impl ConfigManager {
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);
    const MIN_SAVE_INTERVAL: Duration = Duration::from_secs(1);

    pub fn new(config: Config) -> Self {
        Self {
            config_store: Arc::new(ConfigStore::new(config)),
            pending_save: None,
            last_saved: Instant::now(),
        }
    }

    pub fn config_store(&self) -> Arc<ConfigStore> {
        Arc::clone(&self.config_store)
    }

    /// 供外部读取当前最新的配置
    pub fn config(&self) -> Arc<Config> {
        self.config_store.snapshot()
    }

    /// 直接在这里更新配置，并自动触发防抖保存
    pub fn update_and_save(&mut self, new_config: Config) {
        self.config_store.replace(new_config);
        self.pending_save = Some(Instant::now());
    }

    pub fn update(&mut self, ctx: &Context) {
        if let Some(request_time) = self.pending_save {
            let elapsed = request_time.elapsed();
            if elapsed >= Self::DEBOUNCE_DURATION {
                if self.last_saved.elapsed() >= Self::MIN_SAVE_INTERVAL {
                    self.save_async();
                }
                self.pending_save = None;
            } else {
                let wait_time = Self::DEBOUNCE_DURATION.saturating_sub(elapsed);
                ctx.request_repaint_after(wait_time);
            }
        }
    }

    pub fn save_now(&mut self) {
        let config = self.config_store.snapshot();
        save_config(config.as_ref());
        self.pending_save = None;
    }

    fn save_async(&mut self) {
        let config = self.config_store.snapshot();
        rayon::spawn(move || {
            save_config(config.as_ref());
        });
        self.last_saved = Instant::now();
    }
}
