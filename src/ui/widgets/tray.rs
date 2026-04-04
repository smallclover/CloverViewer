use std::sync::{Arc, Mutex};
use egui::{ViewportCommand, WindowLevel};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use crate::i18n::lang::get_text;
use crate::model::config::get_context_config;
use crate::os::window::{show_window_restore, show_window_restore_offscreen};
use crate::utils::image::load_tray_icon;
/// 创建托盘
/// `tray_restore_requested` - 当点击托盘且窗口处于隐藏状态时设置为 true，app.rs 的 update loop 会重置模式并清除此标志
pub fn init_tray(cc: &eframe::CreationContext<'_>, visible: &Arc<Mutex<bool>>, allow_quit: &Arc<Mutex<bool>>, hwnd_usize: usize, tray_restore_requested: &Arc<Mutex<bool>>, tray_screenshot_requested: &Arc<Mutex<bool>>, screenshot_hotkey_text: &str) -> TrayIcon {

    let tray_menu = Menu::new();
    // 创建常规的菜单项
    let config = get_context_config(&cc.egui_ctx);
    let text = get_text(config.language);
    let label = format!("{}    {}", text.menu_screenshot, screenshot_hotkey_text);
    let item_screenshot = MenuItem::new(&label, true, None);
    let item_screenshot_id = item_screenshot.id().clone();

    let item_exit = MenuItem::new(text.menu_exit, true, None);
    let item_exit_id = item_exit.id().clone();
    
    let _ = tray_menu.append(&item_screenshot);
    let _ = tray_menu.append(&PredefinedMenuItem::separator()); // 添加一条分割线
    let _ = tray_menu.append(&item_exit);

    let tray_icon = TrayIconBuilder::new()
        .with_icon(load_tray_icon())
        .with_tooltip("CloverViewer")
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .build()
        .expect("Failed to build tray icon");


    // 2. 克隆给托盘和快捷键回调闭包使用
    let visible_for_tray = Arc::clone(visible);
    let visible_for_tray_menu = Arc::clone(visible);
    let allow_quit_1 = Arc::clone(allow_quit);
    let tray_restore_for_tray = Arc::clone(tray_restore_requested);
    let tray_screenshot_for_menu = Arc::clone(tray_screenshot_requested);


    // 托盘图标处理
    let ctx = cc.egui_ctx.clone();
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        if let TrayIconEvent::Click { button:MouseButton::Left, button_state:MouseButtonState::Up, .. } = event {
            let Ok(mut vis) = visible_for_tray.lock() else { return; };
            if !*vis {
                // 隐藏状态下恢复
                show_window_restore(hwnd_usize);
                *vis = true;
                // 设置标志，通知 app.rs 的 update loop 重置模式为 Viewer
                if let Ok(mut flag) = tray_restore_for_tray.lock() {
                    *flag = true;
                }
                let config = get_context_config(&ctx);
                if let Some((x, y)) = config.window_pos {
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(x, y)));
                }
                if let Some((w, h)) = config.window_size {
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(w, h)));
                }
                ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(ViewportCommand::Focus);

                ctx.request_repaint();
            }else{
                // 最小化状态下恢复
                let info = ctx.input(|i| i.viewport().clone());
                if info.minimized == Some(true) {
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                }
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                // 通常还需要聚焦窗口
                ctx.send_viewport_cmd(ViewportCommand::Focus);

                ctx.request_repaint();
            }
        }
    }));

    let ctx_2 = cc.egui_ctx.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent|{
        if event.id == item_screenshot_id {
            if let Ok(mut flag) = tray_screenshot_for_menu.lock() {
                *flag = true;
            }
            // 唤醒窗口（与热键逻辑一致：先在屏幕外恢复）
            show_window_restore_offscreen(hwnd_usize);
        } else if event.id == item_exit_id {
            let Ok(mut vis) = visible_for_tray_menu.lock() else { return; };
            let Ok(mut aq) = allow_quit_1.lock() else { return; };
            // 不是最小化的时候
            let info = ctx_2.input(|i| i.viewport().clone());
            if info.minimized == Some(false) {
                show_window_restore_offscreen(hwnd_usize);
            }

            *vis = true;
            *aq = true;

            ctx_2.send_viewport_cmd(ViewportCommand::Close);
        }
    }));

    tray_icon
}