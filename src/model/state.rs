use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use eframe::egui::Context;

// 引入 Config，因为初始化热键和重载热键都需要读取配置
use crate::model::config::Config;
use crate::core::hotkeys::{HotkeyAction, HotkeyManager};
use crate::ui::{
    widgets::toast::{ToastManager, ToastSystem},
    mode::UiMode,
    screenshot::capture::ScreenshotState
};

#[derive(Clone, PartialEq, Debug)]
pub enum ViewMode {
    Single,
    Grid,
}

pub struct ViewState {
    pub ui_mode: UiMode,
    pub view_mode: ViewMode,

    // 路径通信通道
    pub path_sender: Sender<PathBuf>,
    pub path_receiver: Receiver<PathBuf>,

    // Toast 消息系统
    pub toast_system: ToastSystem,
    pub toast_manager: ToastManager,

    // 截图状态
    pub screenshot_state: ScreenshotState,

    // 热键管理器 (私有，通过 ViewState 的方法操作)
    hotkey_manager: HotkeyManager,
}

impl ViewState {
    /// 初始化 ViewState
    /// 注意：这里增加了 `config` 参数，用于初始化 HotkeyManager
    pub fn new(ctx: &Context, config: &Config) -> Self {
        let toast_system = ToastSystem::new();
        let toast_manager = toast_system.manager();
        let (path_sender, path_receiver) = mpsc::channel();

        Self {
            ui_mode: UiMode::Normal,
            view_mode: ViewMode::Single, // 默认为单张视图
            path_sender,
            path_receiver,
            toast_system,
            toast_manager,
            screenshot_state: ScreenshotState::default(),
            // 使用 config 初始化 hotkey_manager，注册初始快捷键
            hotkey_manager: HotkeyManager::new(ctx, config),
        }
    }

    /// 每一帧调用的热键检测逻辑
    /// 处理按键按下后的行为（如切换到截图模式、请求复制）
    pub fn process_hotkey_events(&mut self) {
        // 调用 hotkey_manager 的 update 获取当前帧触发的动作列表
        let actions = self.hotkey_manager.update(&self.ui_mode);

        for action in actions {
            match action {
                HotkeyAction::SetScreenshotMode => {
                    // 切换到截图模式
                    self.ui_mode = UiMode::Screenshot;
                    // 可能需要重置截图状态
                    self.screenshot_state = ScreenshotState::default();
                },
                HotkeyAction::RequestScreenshotCopy => {
                    // 标记请求复制，具体的复制逻辑通常在 UI 渲染层或单独的逻辑层处理
                    self.screenshot_state.copy_requested = true;
                },
            }
        }
    }

    /// 当设置页面点击“应用”修改了 Config 后调用此方法
    /// 用于重新注册全局热键
    pub fn reload_hotkeys(&mut self, config: &Config) {
        self.hotkey_manager.update_hotkeys(config);

        // 可选：在这里添加一个 Toast 提示用户快捷键已更新
        // self.toast_manager.info("快捷键设置已更新");
    }
}