use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::model::config::{Config, save_config};

pub struct ConfigManager {
    config: Arc<Config>,
    pending_save: Option<Instant>,
    last_saved: Instant,
}

impl ConfigManager {
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);
    const MIN_SAVE_INTERVAL: Duration = Duration::from_secs(1);

    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            pending_save: None,
            last_saved: Instant::now(),
        }
    }

    /// 供外部读取当前最新的配置
    pub fn config(&self) -> Arc<Config> {
        Arc::clone(&self.config)
    }

    /// 直接在这里更新配置，并自动触发防抖保存
    pub fn update_and_save(&mut self, new_config: Arc<Config>) {
        self.config = new_config; // 永远保持唯一的数据源是最新的
        self.pending_save = Some(Instant::now());
    }

    pub fn update(&mut self) {
        if let Some(request_time) = self.pending_save {
            let elapsed = request_time.elapsed();
            if elapsed >= Self::DEBOUNCE_DURATION {
                if self.last_saved.elapsed() >= Self::MIN_SAVE_INTERVAL {
                    self.save_async();
                }
                self.pending_save = None;
            }
        }
    }

    pub fn save_now(&mut self) {
        save_config(&self.config);
        self.pending_save = None;
    }

    fn save_async(&mut self) {
        let config = Arc::clone(&self.config);
        std::thread::spawn(move || {
            save_config(&config);
        });
        self.last_saved = Instant::now();
    }
}