use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use eframe::egui::Context;
use egui::{Pos2, Vec2};
use crate::core::business::ViewerState;
use crate::core::hotkeys::{HotkeyAction, HotkeyManager};
use crate::model::config::Config;
use crate::model::device::DeviceInfo;
use crate::state::custom_window::WindowState;
use crate::ui::{
    widgets::toast::{ToastManager, ToastSystem},
    mode::UiMode,
    screenshot::capture::ScreenshotState
};

// --- Top-Level Application State ---

pub struct AppState {
    pub ui_mode: UiMode,
    pub viewer: ViewerState,
    pub screenshot: ScreenshotState,
    pub common: CommonState,
}

// --- Feature-Specific & Common States ---

pub struct CommonState {
    pub path_sender: Sender<PathBuf>,
    pub path_receiver: Receiver<PathBuf>,
    pub toast_system: ToastSystem,
    pub toast_manager: ToastManager,
    hotkey_manager: HotkeyManager,
    pub window_state: WindowState,
    pub device_info: DeviceInfo,

    // --- 新增：缓存健康的坐标和尺寸 ---
    pub normal_window_pos: Option<Pos2>,
    pub normal_window_size: Option<Vec2>,
}

impl AppState {
    pub fn new(ctx: &Context, visible_hotkey: Arc<Mutex<bool>>, allow_quit: Arc<Mutex<bool>>, hwnd: isize) -> Self {
        Self {
            ui_mode: UiMode::Normal,
            viewer: ViewerState::new(),
            screenshot: ScreenshotState::default(),
            common: CommonState::new(ctx, visible_hotkey, allow_quit, hwnd),
        }
    }

    pub fn process_hotkey_events(&mut self) {
        let actions = self.common.hotkey_manager.update(&self.ui_mode);

        for action in actions {
            match action {
                // 解构出 prev_state
                HotkeyAction::SetScreenshotMode { prev_state } => {
                    self.ui_mode = UiMode::Screenshot;
                    self.screenshot = ScreenshotState::default();

                    // 把精确的三种状态存进截图状态里
                    self.screenshot.prev_window_state = prev_state;
                },
                HotkeyAction::RequestScreenshotCopy => {
                    self.screenshot.copy_requested = true;
                },
            }
        }
    }

    pub fn reload_hotkeys(&mut self, config: &Config) {
        self.common.hotkey_manager.update_hotkeys(config);
    }
}

impl CommonState {
    pub fn new(ctx: &Context, visible_hotkey: Arc<Mutex<bool>>, allow_quit: Arc<Mutex<bool>>, hwnd: isize) -> Self {
        let toast_system = ToastSystem::new();
        let toast_manager = toast_system.manager();
        let (path_sender, path_receiver) = mpsc::channel();
        let win_m = WindowState::new(Arc::clone(&visible_hotkey), Arc::clone(&allow_quit), hwnd);
        let win_s = WindowState::new(visible_hotkey, allow_quit, hwnd);

        Self {
            path_sender,
            path_receiver,
            toast_system,
            toast_manager,
            hotkey_manager: HotkeyManager::new(ctx, win_m),
            window_state: win_s,
            device_info: DeviceInfo::load(),
            // --- 初始化新增字段 ---
            normal_window_pos: None,
            normal_window_size: None,
        }
    }
}
