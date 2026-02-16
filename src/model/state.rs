use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use eframe::egui::Context;
use crate::core::hotkeys::{HotkeyAction, HotkeyManager};
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
    hotkey_manager: HotkeyManager,
}

impl ViewState {
    pub fn new(ctx: &Context) -> Self {
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
            hotkey_manager: HotkeyManager::new(ctx),
        }
    }

    pub fn update_hotkeys(&mut self) {
        let actions = self.hotkey_manager.update(&self.ui_mode);
        for action in actions {
            match action {
                HotkeyAction::SetScreenshotMode => self.ui_mode = UiMode::Screenshot,
                HotkeyAction::RequestScreenshotCopy => self.screenshot_state.copy_requested = true,
            }
        }
    }
}
