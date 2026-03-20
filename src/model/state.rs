use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use eframe::egui::Context;
use crate::core::hotkeys::{HotkeyAction, HotkeyManager};
use crate::model::config::Config;
use crate::model::device::DeviceInfo;
use crate::model::window_state::WindowState;
use crate::ui::widgets::toast::{ToastManager, ToastSystem};
use crate::model::mode::AppMode;

// --- Top-Level Application State ---

pub struct AppState {
    pub mode: AppMode,
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
    /// 当点击托盘且窗口处于隐藏状态时设置为 true，app.rs 的 update loop 会重置模式并清除此标志
    pub tray_restore_requested: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new(ctx: &Context, visible_hotkey: Arc<Mutex<bool>>, allow_quit: Arc<Mutex<bool>>, hwnd: isize) -> Self {
        Self {
            mode: AppMode::Viewer,
            common: CommonState::new(ctx, visible_hotkey, allow_quit, hwnd),
        }
    }

    /// 处理热键事件，返回需要执行的截图动作
    pub fn process_hotkey_events(&mut self) -> Vec<HotkeyAction> {
        self.common.hotkey_manager.update(&self.mode)
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
            tray_restore_requested: Arc::new(Mutex::new(false)),
        }
    }
}
