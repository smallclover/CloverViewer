#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod i18n;
mod model;
mod ui;
mod utils;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;

fn main() -> eframe::Result<()> {
    let instance = single_instance::SingleInstance::new("CloverViewer").unwrap();
    if !instance.is_single() {
        return Ok(());
    }

    // 初始化热键管理器
    let hotkeys_manager = GlobalHotKeyManager::new().unwrap();

    // 定义 Alt + S
    let mut modifiers = Modifiers::empty();
    modifiers.insert(Modifiers::ALT);
    let hotkey = HotKey::new(Some(modifiers), Code::KeyS);

    // 注册快捷键
    hotkeys_manager.register(hotkey).unwrap();

    // 传递 hotkey 实例进去以便后续校验 ID
    #[cfg(target_os = "windows")]
    app::run(hotkeys_manager, hotkey)
}