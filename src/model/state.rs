use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use crate::ui::components::{
    toast::{ToastManager, ToastSystem},
    ui_mode::UiMode,
    screenshot::ScreenshotState
};

#[derive(Clone, PartialEq, Debug)]
pub enum ViewMode {
    Single,
    Grid,
}

pub struct ViewState {
    pub ui_mode: UiMode,
    pub view_mode: ViewMode,
    pub path_sender: Sender<PathBuf>,
    pub path_receiver: Receiver<PathBuf>,
    pub toast_system: ToastSystem,
    pub toast_manager: ToastManager,
    pub screenshot_state: ScreenshotState,
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
            screenshot_state: ScreenshotState::default(),
        }
    }
}
