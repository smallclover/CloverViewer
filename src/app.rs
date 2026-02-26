use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder, Id};
use std::{
    path::PathBuf,
    sync::Arc,
    env
};
use std::sync::Mutex;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tray_icon::{Icon, MouseButtonState, TrayIconBuilder, TrayIconEvent, TrayIcon, MouseButton};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_RESTORE};
use crate::{
    core::{business::BusinessData},
    model::{
        config::{load_config, save_config, Config},
        state::{ViewMode, ViewState},
    },
};
use crate::model::config::update_context_config;
use crate::ui::{
    menus::context_menu::handle_context_menu_action,
    widgets::modal::ModalAction,
    panels::properties_panel::draw_properties_panel,
    resources::APP_FONT,
    screenshot::capture::handle_screenshot_system,
    mode::UiMode,
    viewer
};

use crate::utils::image::{load_icon, load_tray_icon};

pub fn run() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    options.viewport = options.viewport.with_icon(load_icon());

    let start_path = env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(move |cc| {
            Ok(Box::new(CloverApp::new(
                cc,
                start_path,
            )))
        }),
    )
}

pub struct CloverApp {
    data: BusinessData,
    state: ViewState,
    config: Arc<Config>,
    _tray_icon: TrayIcon,// 持有托盘实例
}

impl CloverApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_path: Option<PathBuf>,
    ) -> Self {
        // 1. 设置字体
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);


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

        // 1. 用 let 声明一个 Arc 包装的 Mutex
        let visible = Arc::new(Mutex::new(true));

        // 2. 克隆给托盘和快捷键回调闭包使用
        let visible_for_tray = Arc::clone(&visible);
        let visible_for_tray_menu = Arc::clone(&visible);
        let visible_for_hotkey = Arc::clone(&visible);

        // 获取原生句柄并转为 isize 以支持跨线程
        let RawWindowHandle::Win32(handle) = cc.window_handle().unwrap().as_raw() else {
            panic!("Unsupported platform");
        };
        let hwnd_isize = handle.hwnd.get();

        // 托盘图标处理
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::Click { button:MouseButton::Left, button_state:MouseButtonState::Up, .. } = event {
                let mut vis = visible_for_tray.lock().unwrap();
                let window_handle = HWND(hwnd_isize as *mut std::ffi::c_void);
                if !*vis {
                    unsafe { ShowWindow(window_handle, SW_RESTORE); }
                    *vis = true;
                }
            }
        }));

        let ctx = cc.egui_ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent|{
            if event.id == item_exit_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }));

        // 2. 加载配置
        let config = load_config(); // 先加载为普通 Config 结构体

        // 3. 初始化 State (现在需要传入 config 来注册初始热键)
        let state = ViewState::new(&cc.egui_ctx, &config, visible_for_hotkey, hwnd_isize);

        // 将 Config 转为 Arc 以便在 App 中共享
        let config_arc = Arc::new(config);
        cc.egui_ctx
            .data_mut(|data| data.insert_temp(Id::new("config"), Arc::clone(&config_arc)));

        let mut app = Self {
            data: BusinessData::new(),
            state, // 使用上面初始化的 state
            config: config_arc,
            _tray_icon: tray_icon,           // 赋值给结构体持有
        };

        if let Some(path) = start_path {
            app.data.open_new_context(cc.egui_ctx.clone(), path);
        }

        app
    }

    fn handle_background_tasks(&mut self, ctx: &Context) {
        if self.data.process_load_results(ctx) {
            ctx.request_repaint();
        }
        if let Ok(path) = self.state.path_receiver.try_recv() {
            if path.is_dir() {
                self.state.view_mode = ViewMode::Grid;
            } else {
                self.state.view_mode = ViewMode::Single;
            }
            self.data.open_new_context(ctx.clone(), path);
        }
    }

    fn handle_input_events(&mut self, ctx: &Context) {
        viewer::handle_input_events(ctx, &mut self.data);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        if self.state.ui_mode != UiMode::Screenshot {
            viewer::draw_top_panel(
                ctx,
                &mut self.state,
            );
            viewer::draw_bottom_panel(ctx, &mut self.state);
            viewer::draw_central_panel(ctx, &mut self.data, &mut self.state);
            draw_properties_panel(ctx, &mut self.state, &self.data);
            self.state.toast_system.update(ctx);
        }
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        if self.state.ui_mode != UiMode::Screenshot {
            // 这里 temp_config 是从 Settings 窗口修改后返回的副本
            let mut temp_config = (*self.config).clone();

            // 注意：render_settings_window 内部需要传入 &mut temp_config
            let (context_menu_action, modal_action) =
                viewer::draw_overlays(ctx, &self.data, &mut self.state, &mut temp_config);

            if let Some(action) = context_menu_action {
                handle_context_menu_action(ctx, action, &self.data, &mut self.state);
            }

            // 处理设置应用逻辑
            if let Some(ModalAction::Apply) = modal_action {
                // 1. 更新内存中的 Config Arc
                self.config = Arc::new(temp_config);
                // 2. 保存到文件
                save_config(&self.config);
                // 3. 关键：通知 State 重新加载热键
                self.state.reload_hotkeys(&self.config);
                // 在这里重新设置 Context 中的 config 数据，确保其他组件也能拿到最新配置
                update_context_config(ctx, &self.config);
            }
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        update_context_config(ctx, &self.config);
        // 定要区分“每帧检测按键”和“配置变更重载按键”这两个概念。
        self.state.process_hotkey_events();

        self.handle_background_tasks(ctx);

        self.handle_input_events(ctx);

        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
        handle_screenshot_system(ctx, &mut self.state, frame);
    }
}