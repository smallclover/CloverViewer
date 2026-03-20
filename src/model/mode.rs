use eframe::egui::Pos2;
use crate::model::config::Config;

/// 应用级功能模式 - 顶层状态机
#[derive(Clone, PartialEq, Debug)]
pub enum AppMode {
    Viewer,     // 图片查看器
    Screenshot, // 截图工具
}

/// UI 覆盖层状态 - 仅在 Viewer 模式下使用
#[derive(Clone, PartialEq)]
pub enum OverlayMode {
    None,
    About,
    Settings { config: Config },  // 使用 Config 副本
    ContextMenu(Pos2),
    Properties,
}
