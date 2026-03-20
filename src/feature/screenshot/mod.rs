pub mod capture;
pub mod state;
pub mod toolbar;
pub mod color_picker;
pub mod magnifier;
pub mod draw;

use eframe::egui::Context;
use crate::core::hotkeys::HotkeyAction;
use crate::feature::Feature;
use crate::model::mode::AppMode;
use crate::model::state::CommonState;
use self::state::{ScreenshotState, WindowPrevState};
use crate::feature::screenshot::capture::handle_screenshot_system;

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
        handle_screenshot_system(ctx, &mut self.is_active, &mut self.state, common);

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
