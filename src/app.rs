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
    state: AppState,
    config: Arc<Config>,
    _tray: TrayIcon,
}

impl CloverApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_path: Option<PathBuf>,
    ) -> Self {
        Self::init_fonts(cc);

        let visible = Arc::new(Mutex::new(true));
        let allow_quit = Arc::new(Mutex::new(false));
        let hwnd_isize = get_hwnd_isize(&cc);

        let tray = init_tray(&cc, &visible, &allow_quit, hwnd_isize);

        let config = load_config();
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
        if let Ok(path) = self.state.path_receiver.try_recv() {
            self.state.viewer.open_new_context(ctx.clone(), path);
        }
    }

    fn handle_input_events(&mut self, ctx: &Context) {
        viewer::handle_input_events(ctx, &mut self.state.viewer, &self.state.window_state);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        viewer::draw_top_panel(ctx, &mut self.state);
        viewer::draw_bottom_panel(ctx, &mut self.state);
        viewer::draw_central_panel(ctx, &mut self.state);
        draw_properties_panel(ctx, &mut self.state.ui_mode, &self.state.viewer);
        self.state.toast_system.update(ctx);
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        let mut temp_config = (*self.config).clone();

        let (context_menu_action, modal_action) =
            viewer::draw_overlays(ctx, &self.state.viewer, &mut self.state.ui_mode, &mut temp_config);

        if let Some(action) = context_menu_action {
            handle_context_menu_action(ctx, action, &self.state.viewer, &mut self.state.ui_mode, &self.state.toast_manager);
        }

        if let Some(ModalAction::Apply) = modal_action {
            self.config = Arc::new(temp_config);
            save_config(&self.config);
            self.state.reload_hotkeys(&self.config);
            update_context_config(ctx, &self.config);
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        update_context_config(ctx, &self.config);
        self.state.process_hotkey_events();

        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);

        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
        handle_screenshot_system(ctx, &mut self.state);
    }
}
