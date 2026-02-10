use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use egui::{Pos2, TextureHandle, Rect};
use crate::ui::components::{
    toast::{ToastManager, ToastSystem},
    ui_mode::UiMode
};

#[derive(Clone, PartialEq, Debug)]
pub enum ViewMode {
    Single,
    Grid,
}

#[derive(Clone)]
pub struct MonitorTexture {
    pub id: usize,
    pub rect: Rect,
    pub texture: TextureHandle,
}

#[derive(Clone, Default)]
pub struct CropState {
    pub is_active: bool,
    pub start: Option<Pos2>,
    pub current: Option<Pos2>,
    pub was_maximized: bool,
    pub was_decorated: bool,
    pub monitor_textures: Vec<MonitorTexture>,
    pub offset: Pos2, // 记录全屏窗口相对于虚拟屏幕原点的偏移
}

pub struct ViewState {
    pub ui_mode: UiMode,
    pub view_mode: ViewMode,
    pub path_sender: Sender<PathBuf>,
    pub path_receiver: Receiver<PathBuf>,
    pub toast_system: ToastSystem,
    pub toast_manager: ToastManager,
    pub crop_state: CropState,
}

impl Default for ViewState {
    fn default() -> Self {
        let toast_system = ToastSystem::new();
        let toast_manager = toast_system.manager();
        let (path_sender, path_receiver) = mpsc::channel();

        Self {
            ui_mode: UiMode::Normal,
            view_mode: ViewMode::Single,
            path_sender,
            path_receiver,
            toast_system,
            toast_manager,
            crop_state: CropState::default(),
        }
    }
}
