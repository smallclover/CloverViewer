pub mod screenshot;
pub mod viewer;

use crate::core::hotkeys::HotkeyAction;
use crate::model::mode::AppMode;

/// Feature trait - 所有功能模块必须实现的接口
pub trait Feature {
    /// 处理热键事件
    /// 返回 Some(AppMode) 表示需要切换到该模式
    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode>;
}
