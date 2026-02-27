use std::sync::mpsc;
use eframe::egui::Context;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, HotKey, Modifiers}, HotKeyState};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MINIMIZE};
use crate::model::config::Config;
use crate::state::custom_window::WindowState;
// 确保引入 Config
use crate::ui::mode::UiMode;

pub enum HotkeyAction {
    SetScreenshotMode,
    RequestScreenshotCopy,
}

pub struct HotkeyManager {
    hotkeys_manager: GlobalHotKeyManager,
    hotkey_receiver: mpsc::Receiver<u32>, // 接收 GlobalHotKeyEvent 的 ID

    // 当前生效的热键对象
    show_hotkey: HotKey,
    copy_hotkey: HotKey,

    is_copy_registered: bool,
}

impl HotkeyManager {
    pub fn new(ctx: &Context, config: &Config, window_state: WindowState) -> Self {
        let hotkeys_manager = GlobalHotKeyManager::new().unwrap();

        // 初始化时直接从 Config 解析
        let show_hotkey = parse_hotkey_str(&config.hotkeys.show_screenshot)
            .unwrap_or(HotKey::new(Some(Modifiers::ALT), Code::KeyS));

        let copy_hotkey = parse_hotkey_str(&config.hotkeys.copy_screenshot)
            .unwrap_or(HotKey::new(Some(Modifiers::CONTROL), Code::KeyC));

        // 注册显示截图的热键
        hotkeys_manager.register(show_hotkey).unwrap();

        let (tx, rx) = mpsc::channel();
        let ctx_clone = ctx.clone();

        // 能够通过 ID 发送事件
        GlobalHotKeyEvent::set_event_handler(Some(Box::new(move |event: GlobalHotKeyEvent| {
            // 响应截图
            if event.id == show_hotkey.id() && event.state == HotKeyState::Released {
                let mut visible = window_state.visible.lock().unwrap();
                let window_handle = HWND(window_state.hwnd_isize as *mut std::ffi::c_void);
                // SW_RESTORE 是 恢复窗口
                // 如果当前是托盘状态
                // 唤起主窗口导最小化
                // 然后开始截图
                if !*visible {
                    unsafe { ShowWindow(window_handle, SW_MINIMIZE); }
                    *visible = true;
                }
                // 无论是否隐藏都要开启截图
                let _ = tx.send(event.id);
                ctx_clone.request_repaint();
            }
        })));

        Self {
            hotkeys_manager,
            hotkey_receiver: rx,
            show_hotkey,
            copy_hotkey,
            is_copy_registered: false,
        }
    }

    /// 当设置点击“应用”时调用此方法
    pub fn update_hotkeys(&mut self, config: &Config) {
        // 1. 卸载旧的快捷键
        let _ = self.hotkeys_manager.unregister(self.show_hotkey);
        if self.is_copy_registered {
            let _ = self.hotkeys_manager.unregister(self.copy_hotkey);
            self.is_copy_registered = false;
        }

        // 2. 解析新的快捷键
        if let Some(new_show) = parse_hotkey_str(&config.hotkeys.show_screenshot) {
            self.show_hotkey = new_show;
        }

        if let Some(new_copy) = parse_hotkey_str(&config.hotkeys.copy_screenshot) {
            self.copy_hotkey = new_copy;
        }

        // 3. 重新注册 "显示截图" 的快捷键
        if let Err(e) = self.hotkeys_manager.register(self.show_hotkey) {
            eprintln!("Failed to register show hotkey: {:?}", e);
        }

        // 注意：复制快捷键通常是在进入截图模式后才动态注册的，所以这里不需要立即注册 copy_hotkey
        // 除非你目前的逻辑是全局都生效。依照你的 update 逻辑，它是动态的，所以这里不动。
    }

    pub fn update(&mut self, ui_mode: &UiMode) -> Vec<HotkeyAction> {
        let mut actions = Vec::new();

        // 1. 动态注册/注销 Copy 快捷键 (逻辑保持不变)
        if *ui_mode == UiMode::Screenshot && !self.is_copy_registered {
            if self.hotkeys_manager.register(self.copy_hotkey).is_ok() {
                self.is_copy_registered = true;
            }
        } else if *ui_mode != UiMode::Screenshot && self.is_copy_registered {
            if self.hotkeys_manager.unregister(self.copy_hotkey).is_ok() {
                self.is_copy_registered = false;
            }
        }

        // 2. 处理接收到的热键事件
        while let Ok(id) = self.hotkey_receiver.try_recv() {
            // 通过 ID 对比来判断是哪个键被按下了
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

/// 辅助函数：将字符串 (如 "Ctrl+Alt+S") 解析为 HotKey
fn parse_hotkey_str(hotkey_str: &str) -> Option<HotKey> {
    let parts: Vec<&str> = hotkey_str.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        match part {
            "Ctrl" => modifiers.insert(Modifiers::CONTROL),
            "Alt" => modifiers.insert(Modifiers::ALT),
            "Shift" => modifiers.insert(Modifiers::SHIFT),
            "Cmd" | "Super" => modifiers.insert(Modifiers::SUPER), // 处理 Mac Cmd 或 Win 键
            key_name => {
                // 将按键名转换为 Code
                key_code = str_to_code(key_name);
            }
        }
    }

    if let Some(code) = key_code {
        return Some(HotKey::new(Some(modifiers), code));
    }
    None
}

/// 简单的字符串到 Code 的映射
/// global_hotkey 的 Code 是枚举，无法直接从 "A" 转换，需要手动映射
fn str_to_code(s: &str) -> Option<Code> {
    // 移除 egui 可能产生的引号或其他格式，虽然你的 settings.rs 产生的是干净的字符串
    // 这里处理常用的键，如果需要支持所有键盘按键，需要一个巨大的 match
    match s {
        "A" => Some(Code::KeyA), "B" => Some(Code::KeyB), "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD), "E" => Some(Code::KeyE), "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG), "H" => Some(Code::KeyH), "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ), "K" => Some(Code::KeyK), "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM), "N" => Some(Code::KeyN), "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP), "Q" => Some(Code::KeyQ), "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS), "T" => Some(Code::KeyT), "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV), "W" => Some(Code::KeyW), "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY), "Z" => Some(Code::KeyZ),
        "Num0" => Some(Code::Digit0), "Num1" => Some(Code::Digit1), "Num2" => Some(Code::Digit2),
        "Num3" => Some(Code::Digit3), "Num4" => Some(Code::Digit4), "Num5" => Some(Code::Digit5),
        "Num6" => Some(Code::Digit6), "Num7" => Some(Code::Digit7), "Num8" => Some(Code::Digit8), "Num9" => Some(Code::Digit9),
        "Escape" => Some(Code::Escape),
        "Enter" => Some(Code::Enter),
        "Space" => Some(Code::Space),
        "Tab" => Some(Code::Tab),
        "Backspace" => Some(Code::Backspace),
        "F1" => Some(Code::F1), "F2" => Some(Code::F2), "F3" => Some(Code::F3), "F4" => Some(Code::F4),
        "F5" => Some(Code::F5), "F6" => Some(Code::F6), "F7" => Some(Code::F7), "F8" => Some(Code::F8),
        "F9" => Some(Code::F9), "F10" => Some(Code::F10), "F11" => Some(Code::F11), "F12" => Some(Code::F12),
        _ => {
            println!("Unknown key code: {}", s);
            None
        }
    }
}
