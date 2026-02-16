
use std::sync::mpsc;
use eframe::egui::Context;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use crate::ui::components::ui_mode::UiMode;

// 定义 HotkeyManager 可能产生的动作
pub enum HotkeyAction {
    SetScreenshotMode,
    RequestScreenshotCopy,
}

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

    // update不再直接修改state，而是返回一个动作列表
    pub fn update(&mut self, ui_mode: &UiMode) -> Vec<HotkeyAction> {
        let mut actions = Vec::new();

        // 1. 动态注册/注销 Ctrl+C
        if *ui_mode == UiMode::Screenshot && !self.is_copy_registered {
            if self.hotkeys_manager.register(self.copy_hotkey).is_ok() {
                self.is_copy_registered = true;
                println!("[Hotkey] Registered Ctrl+C for Screenshot");
            }
        } else if *ui_mode != UiMode::Screenshot && self.is_copy_registered {
            if self.hotkeys_manager.unregister(self.copy_hotkey).is_ok() {
                self.is_copy_registered = false;
                println!("[Hotkey] Unregistered Ctrl+C");
            }
        }

        // 2. 处理接收到的热键事件
        while let Ok(id) = self.hotkey_receiver.try_recv() {
            if id == self.show_hotkey.id() {
                actions.push(HotkeyAction::SetScreenshotMode);
            } else if id == self.copy_hotkey.id() {
                if *ui_mode == UiMode::Screenshot {
                    actions.push(HotkeyAction::RequestScreenshotCopy);
                }
            }
        }

        actions
    }
}
