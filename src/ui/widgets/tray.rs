use std::sync::{Arc, Mutex};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use crate::os::window::{get_hwnd_isize, show_window_mini, show_window_restore};
use crate::utils::image::load_tray_icon;

pub fn create_tray(cc: &eframe::CreationContext<'_>, visible: &Arc<Mutex<bool>>, allow_quit: &Arc<Mutex<bool>>, hwnd_isize: isize) -> TrayIcon {

    let tray_menu = Menu::new();
    // 创建常规的菜单项
    let item_exit = MenuItem::new("退出", true, None);
    let item_exit_id = item_exit.id().clone();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap(); // 添加一条分割线
    tray_menu.append(&item_exit).unwrap();

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


    // 托盘图标处理
    let ctx = cc.egui_ctx.clone();
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        if let TrayIconEvent::Click { button:MouseButton::Left, button_state:MouseButtonState::Up, .. } = event {
            let mut vis = visible_for_tray.lock().unwrap();
            if !*vis {
                // 隐藏状态下恢复
                show_window_restore(hwnd_isize);
                *vis = true;
            }else{
                // 最小化状态下恢复
                let info = ctx.input(|i| i.viewport().clone());
                if let Some(mini) = info.minimized {
                    // 发送取消最小化指令
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    // 通常还需要聚焦窗口
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
            }
        }
    }));

    let ctx_2 = cc.egui_ctx.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent|{
        if event.id == item_exit_id {
            let mut vis = visible_for_tray_menu.lock().unwrap();
            let mut aq = allow_quit_1.lock().unwrap();
            // 退出前最小化窗口
            show_window_mini(hwnd_isize);
            *vis = true;
            *aq = true;
            ctx_2.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }));

    tray_icon
}