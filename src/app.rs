use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder};
use std::{
    path::PathBuf,
    sync::Arc,
};
use crate::{
    model::config::{load_config, save_config, Config},
    model::state::{ViewState, ViewMode},
    core::business::BusinessData,
};
use crate::ui::components::{
    context_menu::handle_context_menu_action,
    modal::ModalAction,
    mouse::handle_input_events,
    properties_panel::draw_properties_panel,
    resources::APP_FONT,
};
use crate::ui::viewer;
use crate::utils::image::load_icon;

pub fn run() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    options.viewport = options.viewport.with_icon(load_icon());

    let start_path = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(|cc| Ok(Box::new(CloverApp::new(cc, start_path)))),
    )
}

pub struct CloverApp {
    data: BusinessData,
    state: ViewState,
    config: Config,
}

impl CloverApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            Arc::new(FontData::from_static(APP_FONT)),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let config = load_config();

        let mut app = Self {
            data: BusinessData::new(),
            state: ViewState::default(),
            config,
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
        handle_input_events(ctx, &mut self.data);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        viewer::draw_top_panel(ctx, &mut self.state, &self.config);
        viewer::draw_bottom_panel(ctx, &mut self.state);
        viewer::draw_central_panel(ctx, &mut self.data, &mut self.state, &self.config);
        draw_properties_panel(ctx, &mut self.state, &self.data, &self.config);
        self.state.toast_system.update(ctx);
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        let mut temp_config = self.config.clone();
        let (context_menu_action, modal_action) =
            viewer::draw_overlays(ctx, &self.data, &mut self.state, &mut temp_config);

        if let Some(action) = context_menu_action {
            handle_context_menu_action(action, &self.data, &mut self.state, &self.config);
        }

        if let Some(ModalAction::Apply) = modal_action {
            self.config = temp_config;
            save_config(&self.config);
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);
        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
    }
}
