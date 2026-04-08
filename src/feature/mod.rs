pub mod screenshot;
pub mod viewer;

use crate::core::hotkeys::HotkeyAction;
use crate::model::mode::AppMode;
use crate::model::state::CommonState;
use eframe::egui::Context;

/// Feature trait - 所有功能模块必须实现的接口
pub trait Feature {
    /// 更新 Feature 状态
    /// `mode` - 当前应用模式，可由 feature 修改以请求切换模式
    fn update(&mut self, ctx: &Context, common: &mut CommonState, mode: &mut AppMode);

    /// 处理热键事件
    /// 返回 Some(AppMode) 表示需要切换到该模式
    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode>;
}
