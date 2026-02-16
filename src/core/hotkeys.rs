
use std::sync::mpsc;
use eframe::egui::Context;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::model::state::ViewState;
use crate::ui::components::ui_mode::UiMode;

pub struct HotkeyManager {
    hotkeys_manager: GlobalHotKeyManager,
    hotkey_receiver: mpsc::Receiver<u32>,
    show_hotkey: HotKey,
    copy_hotkey: HotKey,
    is_copy_registered: bool,
}

impl HotkeyManager {
    pub fn new(ctx: &Context) -> Self {
        let hotkeys_manager = GlobalHotKeyManager::new().unwrap();

        let mut mods_s = Modifiers::empty();
        mods_s.insert(Modifiers::ALT);
        let show_hotkey = HotKey::new(Some(mods_s), Code::KeyS);

        let mut mods_c = Modifiers::empty();
        mods_c.insert(Modifiers::CONTROL);
        let copy_hotkey = HotKey::new(Some(mods_c), Code::KeyC);

        hotkeys_manager.register(show_hotkey).unwrap();

        let (tx, rx) = mpsc::channel();
        let ctx_clone = ctx.clone();

        GlobalHotKeyEvent::set_event_handler(Some(Box::new(move |event: GlobalHotKeyEvent| {
            let _ = tx.send(event.id);
            ctx_clone.request_repaint();
        })));

        Self {
            hotkeys_manager,
            hotkey_receiver: rx,
            show_hotkey,
            copy_hotkey,
            is_copy_registered: false,
        }
    }

    pub fn update(&mut self, state: &mut ViewState) {
        if state.ui_mode == UiMode::Screenshot && !self.is_copy_registered {
            if self.hotkeys_manager.register(self.copy_hotkey).is_ok() {
                self.is_copy_registered = true;
                println!("[Hotkey] Registered Ctrl+C for Screenshot");
            }
        } else if state.ui_mode != UiMode::Screenshot && self.is_copy_registered {
            if self.hotkeys_manager.unregister(self.copy_hotkey).is_ok() {
                self.is_copy_registered = false;
                println!("[Hotkey] Unregistered Ctrl+C");
            }
        }

        while let Ok(id) = self.hotkey_receiver.try_recv() {
            if id == self.show_hotkey.id() {
                state.ui_mode = UiMode::Screenshot;
            } else if id == self.copy_hotkey.id() {
                if state.ui_mode == UiMode::Screenshot {
                    state.screenshot_state.copy_requested = true;
                }
            }
        }
    }
}
