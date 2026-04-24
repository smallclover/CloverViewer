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
use crate::feature::screenshot::capture::{
    draw_screenshot_ui_inside, finalize_screenshot_action, prepare_screenshot_frame,
};
use crate::model::config::get_context_config;
use crate::model::mode::AppMode;
use crate::model::state::CommonState;
use eframe::egui::{Context, Frame, Ui, Key, Modifiers};

/// 将快捷键字符串（如 "Alt+C"）解析为 egui 的 Modifiers + Key
fn parse_egui_hotkey(hotkey_str: &str) -> Option<(Modifiers, Key)> {
    let parts: Vec<&str> = hotkey_str.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::default();
    let mut key = None;

    for part in &parts {
        match *part {
            "Ctrl" => modifiers.ctrl = true,
            "Alt" => modifiers.alt = true,
            "Shift" => modifiers.shift = true,
            "Cmd" | "Super" => modifiers.mac_cmd = true,
            k => {
                key = str_to_egui_key(k);
            }
        }
    }

    key.map(|k| (modifiers, k))
}

fn str_to_egui_key(s: &str) -> Option<Key> {
    match s {
        "A" => Some(Key::A),
        "B" => Some(Key::B),
        "C" => Some(Key::C),
        "D" => Some(Key::D),
        "E" => Some(Key::E),
        "F" => Some(Key::F),
        "G" => Some(Key::G),
        "H" => Some(Key::H),
        "I" => Some(Key::I),
        "J" => Some(Key::J),
        "K" => Some(Key::K),
        "L" => Some(Key::L),
        "M" => Some(Key::M),
        "N" => Some(Key::N),
        "O" => Some(Key::O),
        "P" => Some(Key::P),
        "Q" => Some(Key::Q),
        "R" => Some(Key::R),
        "S" => Some(Key::S),
        "T" => Some(Key::T),
        "U" => Some(Key::U),
        "V" => Some(Key::V),
        "W" => Some(Key::W),
        "X" => Some(Key::X),
        "Y" => Some(Key::Y),
        "Z" => Some(Key::Z),
        "Num0" | "0" => Some(Key::Num0),
        "Num1" | "1" => Some(Key::Num1),
        "Num2" | "2" => Some(Key::Num2),
        "Num3" | "3" => Some(Key::Num3),
        "Num4" | "4" => Some(Key::Num4),
        "Num5" | "5" => Some(Key::Num5),
        "Num6" | "6" => Some(Key::Num6),
        "Num7" | "7" => Some(Key::Num7),
        "Num8" | "8" => Some(Key::Num8),
        "Num9" | "9" => Some(Key::Num9),
        "Enter" => Some(Key::Enter),
        "Space" => Some(Key::Space),
        "Tab" => Some(Key::Tab),
        "Backspace" => Some(Key::Backspace),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        _ => None,
    }
}

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
                if let Ok(metadata) = entry.metadata()
                    && let Ok(modified) = metadata.modified()
                    && let Ok(age) = now.duration_since(modified)
                    && age > max_age
                {
                    let _ = std::fs::remove_file(&path);
                    tracing::debug!("清理过期OCR临时文件: {:?}", path);
                }
            }
        }
    }
}

impl Feature for ScreenshotFeature {
    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode> {
        match action {
            HotkeyAction::SetScreenshotMode { prev_state } => {
                // 初始化截图状态
                self.enter_screenshot_mode(prev_state);
                // 告诉主应用：你需要把全局模式切换为截图
                Some(AppMode::Screenshot)
            }
        }
    }
}

impl ScreenshotFeature {
    pub fn logic(&mut self, ctx: &Context, _common: &mut CommonState, mode: &mut AppMode) {
        // 检测是否刚进入截图模式
        if *mode == AppMode::Screenshot && !self.is_active {
            self.enter_screenshot_mode(crate::feature::screenshot::state::WindowPrevState::Normal);
        }

        // 只在 Screenshot 模式下处理
        if *mode != AppMode::Screenshot {
            return;
        }

        // 检测是否退出截图模式
        if !self.is_active {
            *mode = AppMode::Viewer;
            return;
        }

        // [关键] 在 logic() 中驱动截图准备流程（截屏捕获 + 窗口配置）
        // egui 0.34 在窗口不可见/被遮挡时会跳过 ui()，
        // 但截图流程需要在 ui() 之前就启动（发送 ViewportCommand、启动后台线程），
        // 否则窗口会卡在屏幕外无法恢复。
        if !prepare_screenshot_frame(ctx, &mut self.is_active, &mut self.state, _common)
            && !self.is_active
        {
            *mode = AppMode::Viewer;
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, common: &mut CommonState, mode: &mut AppMode) {
        if *mode != AppMode::Screenshot || !self.is_active {
            return;
        }

        // 截屏尚未完成或窗口尚未配置好，跳过绘制
        if self.state.capture.captures.is_empty() || !self.state.runtime.window_configured {
            return;
        }

        let ctx = ui.ctx().clone();

        // 通过 egui 本地按键检测复制快捷键（替代全局热键）
        let config = get_context_config(&ctx);
        if let Some((required_mods, required_key)) = parse_egui_hotkey(&config.hotkeys.copy_color) {
            ui.input(|i| {
                if i.key_pressed(required_key) && i.modifiers == required_mods {
                    self.state.input.copy_requested = true;
                }
            });
        }

        let action = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(egui::Color32::TRANSPARENT))
            .show_inside(ui, |ui| draw_screenshot_ui_inside(ui, &mut self.state, &common.device_info))
            .inner;

        if action != crate::feature::screenshot::state::ScreenshotAction::None {
            self.is_active = false;
            let ocr_image_opt = finalize_screenshot_action(&ctx, &mut self.state, common, action);

            if let Some(image) = ocr_image_opt {
                common.ocr_state.is_panel_open = true;
                common.ocr_state.is_processing = true;
                common.ocr_state.text = None;

                let temp_dir = std::env::temp_dir().join("CloverViewer");
                let _ = std::fs::create_dir_all(&temp_dir);

                Self::clean_temp_files(&temp_dir);

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let temp_path = temp_dir.join(format!("ocr_temp_{}.png", timestamp));

                if let Err(e) = image.save(&temp_path) {
                    tracing::error!("无法保存 OCR 临时图片: {}", e);
                } else {
                    let _ = common.path_sender.send(temp_path);
                }

                let (tx, rx) = std::sync::mpsc::channel();
                common.ocr_state.receiver = Some(rx);

                let language = get_context_config(&ctx).language;
                std::thread::spawn(move || {
                    let platform = crate::os::current_platform();
                    let result = platform.recognize_text(image, language);
                    let _ = tx.send(result);
                });
            }
        }

        if !self.is_active {
            *mode = AppMode::Viewer;
        }
    }
}
