use crate::model::config::Config;
use eframe::egui::Pos2;

/// 应用级功能模式 - 顶层状态机
#[derive(Clone, PartialEq, Debug)]
pub enum AppMode {
    Viewer,     // 图片查看器
    Screenshot, // 截图工具
}

/// 弹窗/浮层状态 - 仅在 Viewer 模式下使用，互斥瞬态
#[derive(Clone, PartialEq)]
pub enum PopupMode {
    None,
    About,
    Settings { config: Config },
    ContextMenu(Pos2),
}

/// 侧边面板状态 - 仅在 Viewer 模式下使用，可常驻
#[derive(Clone, PartialEq)]
pub enum PanelMode {
    None,
    Properties,
}
