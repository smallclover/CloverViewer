use eframe::egui;
use tray_icon::TrayIcon;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    env
};
use crate::{
    model::{
        config::{load_config, save_config, update_context_config, Config},
        state::{AppState},
    },
    os::window::get_hwnd_isize,
    utils::image::load_icon,
    ui::{
        menus::context_menu::handle_context_menu_action,
        widgets::{
            modal::ModalAction,
            tray::init_tray
        },
        panels::properties_panel::draw_properties_panel,
        resources::APP_FONT,
        screenshot::capture::handle_screenshot_system,
        viewer
    }
};
use crate::model::config::init_config_arc;
use crate::ui::mode::UiMode;

pub fn run() -> eframe::Result<()> {
    // 提前加载配置
    let config = load_config();

    let mut viewport = ViewportBuilder::default()
        .with_transparent(true)
        .with_icon(load_icon());

    // 应用配置文件中的大小，否则默认 1024x768
    if let Some((w, h)) = config.window_size {
        viewport = viewport.with_inner_size([w, h]);
    } else {
        viewport = viewport.with_inner_size([1024.0, 768.0]);
    }

    // 应用配置文件中的位置，否则默认居中
    if let Some((x, y)) = config.window_pos {
        viewport = viewport.with_position([x, y]);
    }

    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let start_path = env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(move |cc| {
            Ok(Box::new(CloverApp::new(
                cc,
                start_path,
                config, // 将读取好的 config 传给 new()
            )))
        }),
    )
}

pub struct CloverApp {
    state: AppState,
    config: Arc<Config>,
    _tray: TrayIcon,
}

impl CloverApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_path: Option<PathBuf>,
        config: Config,
    ) -> Self {
        Self::init_fonts(cc);

        let visible = Arc::new(Mutex::new(true));
        let allow_quit = Arc::new(Mutex::new(false));
        let hwnd_isize = get_hwnd_isize(&cc);

        let tray = init_tray(&cc, &visible, &allow_quit, hwnd_isize);

        let config_arc = Arc::new(config);
        init_config_arc(&cc.egui_ctx, &Arc::clone(&config_arc));

        let mut state = AppState::new(&cc.egui_ctx, visible, allow_quit, hwnd_isize);

        if let Some(path) = start_path {
            state.viewer.open_new_context(cc.egui_ctx.clone(), path);
        }

        Self {
            state,
            config: config_arc,
            _tray: tray,
        }
    }

    fn init_fonts(cc: &eframe::CreationContext<'_>){
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);
    }

    fn handle_background_tasks(&mut self, ctx: &Context) {
        if self.state.viewer.process_load_results(ctx) {
            ctx.request_repaint();
        }
        if let Ok(path) = self.state.common.path_receiver.try_recv() {
            self.state.viewer.open_new_context(ctx.clone(), path);
        }
    }

    fn handle_input_events(&mut self, ctx: &Context) {
        crate::ui::widgets::input::handle_input_events(ctx, &mut self.state.viewer, &self.state.common.window_state);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        viewer::draw_top_panel(ctx, &mut self.state);
        viewer::draw_bottom_panel(ctx, &mut self.state);
        viewer::draw_central_panel(ctx, &mut self.state);
        draw_properties_panel(ctx, &mut self.state.ui_mode, &self.state.viewer);
        self.state.common.toast_system.update(ctx);
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        let mut temp_config = (*self.config).clone();

        let (context_menu_action, modal_action) =
            viewer::draw_overlays(ctx, &self.state.viewer, &mut self.state.ui_mode, &mut temp_config);

        if let Some(action) = context_menu_action {
            handle_context_menu_action(ctx, action, &self.state.viewer, &mut self.state.ui_mode, &self.state.common.toast_manager);
        }

        if let Some(ModalAction::Apply) = modal_action {
            self.config = Arc::new(temp_config);
            save_config(&self.config);
            self.state.reload_hotkeys(&self.config);
            update_context_config(ctx, &self.config);
        }
    }
    fn handle_cache_win_pos(&mut self, ctx: &Context){
        if self.state.ui_mode != UiMode::Normal { return; }

        if let Ok(visible) = self.state.common.window_state.visible.lock() {
            if !*visible { return; }
        }

        let viewport = ctx.input(|i| i.viewport().clone());

        if viewport.minimized == Some(true)
            || viewport.maximized == Some(true)
            || viewport.fullscreen == Some(true) {
            return;
        }

        if let (Some(outer), Some(inner)) = (viewport.outer_rect, viewport.inner_rect) {
            if outer.min.x > -10000.0 && outer.min.y > -10000.0 && inner.width() < 4000.0 && inner.height() < 3000.0 {

                let current_pos = (outer.min.x, outer.min.y);
                let current_size = (inner.width(), inner.height());

                // 检查是否发生变化
                let pos_changed = self.config.window_pos != Some(current_pos);
                let size_changed = self.config.window_size != Some(current_size);

                // 鼠标没有任何按键被按下，说明用户的拖拽或缩放动作已经结束
                let no_mouse_down = !ctx.input(|i| i.pointer.any_down());

                if (pos_changed || size_changed) && no_mouse_down {
                    // 更新内存配置
                    let mut new_config = (*self.config).clone();
                    new_config.window_pos = Some(current_pos);
                    new_config.window_size = Some(current_size);
                    self.config = Arc::new(new_config);

                    // 写入 config.json 永久保存
                    save_config(&self.config);

                    // 更新 Context 里的配置（保证全局同步）
                    update_context_config(ctx, &self.config);

                    println!("窗口调整完毕，已保存位置与尺寸到 JSON！");
                }
            }
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        
        self.handle_cache_win_pos(ctx);
        
        update_context_config(ctx, &self.config);
        self.state.process_hotkey_events();

        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);
        
        // 区分当前模式，防止普通 UI 和 截图 UI 重叠绘制
        if self.state.ui_mode == UiMode::Screenshot {
            handle_screenshot_system(ctx, &mut self.state);
        } else {
            // 普通模式下绘制常规 UI
            self.draw_ui(ctx);
            self.handle_ui_interactions(ctx);
        }
    }
}
