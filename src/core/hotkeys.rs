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
        let show_hotkey = crate::core::hotkey_parser::parse_hotkey_str(
            &config.hotkeys.show_screenshot,
        )
        .and_then(|p| parsed_to_hotkey(&p))
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
        if let Some(new_show) = crate::core::hotkey_parser::parse_hotkey_str(
            &config.hotkeys.show_screenshot,
        )
        .and_then(|p| parsed_to_hotkey(&p))
        {
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

/// 将 ParsedHotkey 转换为 global_hotkey::HotKey。
fn parsed_to_hotkey(parsed: &crate::core::hotkey_parser::ParsedHotkey) -> Option<HotKey> {
    use global_hotkey::hotkey::Modifiers as GModifiers;
    let code = crate::core::hotkey_parser::parsed_key_to_code(&parsed.key_name)?;
    let mut modifiers = GModifiers::empty();
    if parsed.ctrl {
        modifiers.insert(GModifiers::CONTROL);
    }
    if parsed.alt {
        modifiers.insert(GModifiers::ALT);
    }
    if parsed.shift {
        modifiers.insert(GModifiers::SHIFT);
    }
    if parsed.cmd {
        modifiers.insert(GModifiers::SUPER);
    }
    Some(HotKey::new(Some(modifiers), code))
}

#[cfg(test)]
mod tests {
    use super::parsed_to_hotkey;
    use crate::core::hotkey_parser;

    #[test]
    fn parsed_to_hotkey_accepts_modifier_combinations() {
        let p = hotkey_parser::parse_hotkey_str("Ctrl+Alt+S").unwrap();
        assert!(parsed_to_hotkey(&p).is_some());

        let p = hotkey_parser::parse_hotkey_str("Cmd+Shift+F12").unwrap();
        assert!(parsed_to_hotkey(&p).is_some());
    }

    #[test]
    fn parsed_to_hotkey_rejects_unknown_key_names() {
        let p = hotkey_parser::parse_hotkey_str("Ctrl+NoSuchKey").unwrap();
        assert!(parsed_to_hotkey(&p).is_none());

        assert!(hotkey_parser::parse_hotkey_str("").is_none());
    }
}
