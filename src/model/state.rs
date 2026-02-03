use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use crate::ui::components::{
    toast::{ToastManager, ToastSystem},
    ui_mode::UiMode,properties_panel::ImageProperties
};

pub struct ViewState {
    pub ui_mode: UiMode,
    pub path_sender: Sender<PathBuf>,
    pub path_receiver: Receiver<PathBuf>,
    pub toast_system: ToastSystem,
    pub toast_manager: ToastManager,
    pub show_properties_panel: bool,
    pub image_properties: Option<ImageProperties>,
}

impl Default for ViewState {
    fn default() -> Self {
        let toast_system = ToastSystem::new();
        let toast_manager = toast_system.manager();
        let (path_sender, path_receiver) = mpsc::channel();

        Self {
            ui_mode: UiMode::Normal,
            path_sender,
            path_receiver,
            toast_system,
            toast_manager,
            show_properties_panel: false,
            image_properties: None,
        }
    }
}
