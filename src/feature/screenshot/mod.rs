pub mod canvas;
pub mod capture;
pub mod color_picker;
pub mod draw;
pub mod help_box;
pub mod magnifier;
pub mod ocr;
pub mod state;
pub mod toolbar;

use self::state::{ScreenshotState, WindowPrevState};
use crate::core::hotkeys::HotkeyAction;
use crate::feature::Feature;
use crate::feature::screenshot::capture::handle_screenshot_system;
use crate::model::mode::AppMode;
use crate::model::state::CommonState;
use eframe::egui::Context;

/// ScreenshotFeature - 截图功能模块
pub struct ScreenshotFeature {
    pub state: ScreenshotState,
    /// 内部状态：是否处于截图模式
    is_active: bool,
}

impl ScreenshotFeature {
    pub fn new() -> Self {
        Self {
            state: ScreenshotState::default(),
            is_active: false,
        }
    }

    /// 进入截图模式，初始化状态
    pub fn enter_screenshot_mode(&mut self, prev_state: WindowPrevState) {
        self.state = ScreenshotState::new(prev_state);
        self.is_active = true;
    }
}

impl Default for ScreenshotFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenshotFeature {
    /// 清理超过24小时的临时文件
    fn clean_temp_files(temp_dir: &std::path::Path) {
        use std::time::{Duration, SystemTime};

        let now = SystemTime::now();
        let max_age = Duration::from_secs(24 * 60 * 60); // 24小时

        if let Ok(entries) = std::fs::read_dir(temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // 只清理 ocr_temp_ 开头的文件
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with("ocr_temp_") {
                        continue;
                    }
                } else {
                    continue;
                }

                // 检查文件修改时间
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > max_age {
                                let _ = std::fs::remove_file(&path);
                                tracing::debug!("清理过期OCR临时文件: {:?}", path);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Feature for ScreenshotFeature {
    fn update(&mut self, ctx: &Context, common: &mut CommonState, mode: &mut AppMode) {
        // 检测是否刚进入截图模式
        if *mode == AppMode::Screenshot && !self.is_active {
            self.enter_screenshot_mode(crate::feature::screenshot::state::WindowPrevState::Normal);
        }

        // 只在 Screenshot 模式下处理
        if *mode != AppMode::Screenshot {
            return;
        }

        // 同步 copy_requested 标志
        // 注意：这个标志由热键设置，需要外部传入
        // 这里假设 copy_requested 已经在 app.rs 中同步到 self.state

        // 调用截图系统处理逻辑
        let ocr_image_opt =
            handle_screenshot_system(ctx, &mut self.is_active, &mut self.state, common);

        if let Some(image) = ocr_image_opt {
            // 1. 开启右侧面板状态
            common.ocr_state.is_panel_open = true;
            common.ocr_state.is_processing = true;
            common.ocr_state.text = None;

            // ==========================================
            // 将图片存入临时目录，伪装成打开本地文件
            // ==========================================
            // 获取系统临时目录 (如 AppData/Local/Temp)
            let temp_dir = std::env::temp_dir().join("CloverViewer");
            let _ = std::fs::create_dir_all(&temp_dir); // 确保干净的专属目录存在

            // 清理超过24小时的临时文件
            Self::clean_temp_files(&temp_dir);

            // 加上时间戳，否则每次都从LRU里面的缓存读取
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let temp_path = temp_dir.join(format!("ocr_temp_{}.png", timestamp));

            // 保存图片并发送给 Viewer
            if let Err(e) = image.save(&temp_path) {
                tracing::error!("无法保存 OCR 临时图片: {}", e);
            } else {
                // 利用现有的消息通道，通知 Viewer 加载这张图！
                let _ = common.path_sender.send(temp_path);
            }

            // 2. 建立多线程通道，启动系统 OCR 引擎
            let (tx, rx) = std::sync::mpsc::channel();
            common.ocr_state.receiver = Some(rx);

            std::thread::spawn(move || {
                let platform = crate::os::current_platform();
                let result = platform.recognize_text(image);
                let _ = tx.send(result);
            });
        }

        // 检测是否退出截图模式
        if !self.is_active {
            *mode = AppMode::Viewer;
        }
    }

    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode> {
        match action {
            // 截图模式
            HotkeyAction::RequestScreenshotCopy => {
                self.state.copy_requested = true;
                None
            }
            // 复制截图
            HotkeyAction::SetScreenshotMode { prev_state } => {
                // 初始化截图状态
                self.enter_screenshot_mode(prev_state);
                // 告诉主应用：你需要把全局模式切换为截图
                Some(AppMode::Screenshot)
            }
        }
    }
}
