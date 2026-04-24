use crate::feature::screenshot::capture::WindowPrevState;
use crate::model::config::{Config, get_context_config};
use crate::model::mode::AppMode;
use crate::model::window_state::WindowState;
use crate::os::current_platform;
use eframe::egui::Context;
use egui::ViewportCommand;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use std::sync::{Arc, mpsc};

#[derive(Clone)]
pub enum HotkeyAction {
    SetScreenshotMode { prev_state: WindowPrevState },
}

pub struct HotkeyManager {
    hotkeys_manager: Option<GlobalHotKeyManager>,
    hotkey_receiver: mpsc::Receiver<(u32, WindowPrevState)>,

    // 当前生效的热键对象
    show_hotkey: HotKey,
}

impl HotkeyManager {
    pub fn new(ctx: &Context, window_state: Arc<WindowState>) -> Self {
        let config = get_context_config(ctx);
        // 初始化时直接从 Config 解析
        let show_hotkey = parse_hotkey_str(&config.hotkeys.show_screenshot)
            .unwrap_or(HotKey::new(Some(Modifiers::ALT), Code::KeyS));

        let (tx, rx) = mpsc::channel();

        let hotkeys_manager = match GlobalHotKeyManager::new() {
            Ok(hotkeys_manager) => {
                if let Err(e) = hotkeys_manager.register(show_hotkey) {
                    tracing::error!("Failed to register screenshot hotkey: {:?}", e);
                }

                let ctx_clone = ctx.clone();
                GlobalHotKeyEvent::set_event_handler(Some(Box::new(move |event: GlobalHotKeyEvent| {
                    // [重要] global-hotkey 回调运行在独立线程上，
                    // 绝不能在持有 visible Mutex 的同时调用 ctx.input() 或 Win32 API
                    // (如 ShowWindow/SetWindowPos)，否则会与主线程（持有 Context 写锁并
                    // 尝试获取 visible Mutex）产生跨线程死锁。

                    // 1. 先在无锁状态下读取 egui 层面的最小化状态
                    let is_minimized = ctx_clone.input(|i| i.viewport().minimized.unwrap_or(false));

                    // 2. 最小范围持有 visible 锁：读取 + 设置，然后立刻释放
                    let prev_state = {
                        let Ok(mut visible) = window_state.visible.lock() else {
                            return;
                        };
                        let is_visible = *visible;

                        let state = if !is_visible {
                            WindowPrevState::Tray
                        } else if is_minimized {
                            WindowPrevState::Minimized
                        } else {
                            WindowPrevState::Normal
                        };

                        // 提前标记为可见，释放锁后主线程就能正确读取
                        if state != WindowPrevState::Normal {
                            *visible = true;
                        }

                        state
                        // visible 锁在此处释放
                    };

                    // 3. 锁已释放，安全调用 Win32 API 和 egui viewport commands
                    if prev_state != WindowPrevState::Normal {
                        if prev_state == WindowPrevState::Tray {
                            let platform = current_platform();
                            platform.show_window_restore_offscreen(window_state.hwnd_usize);
                            platform.force_get_focus(window_state.hwnd_usize);
                            ctx_clone.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        } else {
                            ctx_clone.send_viewport_cmd(ViewportCommand::Minimized(false));
                            ctx_clone.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        }
                    }

                    let _ = tx.send((event.id, prev_state));
                    ctx_clone.request_repaint();
                })));

                Some(hotkeys_manager)
            }
            Err(err) => {
                tracing::error!("Failed to initialize GlobalHotKeyManager: {:?}", err);
                None
            }
        };

        Self {
            hotkeys_manager,
            hotkey_receiver: rx,
            show_hotkey,
        }
    }

    /// 当设置点击"应用"时调用此方法
    pub fn update_hotkeys(&mut self, config: &Config) {
        let Some(hotkeys_manager) = self.hotkeys_manager.as_ref() else {
            return;
        };

        // 1. 卸载旧的快捷键
        let _ = hotkeys_manager.unregister(self.show_hotkey);

        // 2. 解析新的快捷键
        if let Some(new_show) = parse_hotkey_str(&config.hotkeys.show_screenshot) {
            self.show_hotkey = new_show;
        }

        // 3. 重新注册 "显示截图" 的快捷键
        if let Err(e) = hotkeys_manager.register(self.show_hotkey) {
            tracing::error!("Failed to register show hotkey: {:?}", e);
        }
    }

    pub fn update(&mut self, mode: &AppMode) -> Vec<HotkeyAction> {
        let mut actions = Vec::new();

        let Some(_hotkeys_manager) = self.hotkeys_manager.as_ref() else {
            return actions;
        };

        // --- 用于防止一帧内处理多次重复按键 ---
        let mut screenshot_triggered_this_frame = false;

        // 处理接收到的热键事件
        while let Ok((id, prev_state)) = self.hotkey_receiver.try_recv() {
            // 通过 ID 对比来判断是哪个键被按下了
            if id == self.show_hotkey.id() {
                tracing::debug!("处理");
                // 只有在不是截图模式，且本帧未触发的情况下，才接受事件
                if *mode != AppMode::Screenshot && !screenshot_triggered_this_frame {
                    actions.push(HotkeyAction::SetScreenshotMode { prev_state });
                    screenshot_triggered_this_frame = true;
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
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "Num0" => Some(Code::Digit0),
        "Num1" => Some(Code::Digit1),
        "Num2" => Some(Code::Digit2),
        "Num3" => Some(Code::Digit3),
        "Num4" => Some(Code::Digit4),
        "Num5" => Some(Code::Digit5),
        "Num6" => Some(Code::Digit6),
        "Num7" => Some(Code::Digit7),
        "Num8" => Some(Code::Digit8),
        "Num9" => Some(Code::Digit9),
        "Escape" => Some(Code::Escape),
        "Enter" => Some(Code::Enter),
        "Space" => Some(Code::Space),
        "Tab" => Some(Code::Tab),
        "Backspace" => Some(Code::Backspace),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => {
            tracing::debug!("Unknown key code: {}", s);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_hotkey_str, str_to_code};
    use global_hotkey::hotkey::Code;

    #[test]
    fn parse_hotkey_str_accepts_modifier_combinations() {
        assert!(parse_hotkey_str("Ctrl+Alt+S").is_some());
        assert!(parse_hotkey_str("Cmd+Shift+F12").is_some());
    }

    #[test]
    fn parse_hotkey_str_rejects_unknown_key_names() {
        assert!(parse_hotkey_str("Ctrl+NoSuchKey").is_none());
        assert!(parse_hotkey_str("").is_none());
    }

    #[test]
    fn str_to_code_maps_known_keys() {
        assert_eq!(str_to_code("A"), Some(Code::KeyA));
        assert_eq!(str_to_code("F12"), Some(Code::F12));
        assert_eq!(str_to_code("Space"), Some(Code::Space));
    }
}
