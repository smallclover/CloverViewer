use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder, ViewportId, ViewportClass, ColorImage, ViewportCommand};
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
    crop::handle_crop_mode,
};
use crate::ui::viewer;
use crate::utils::image::load_icon;
use crate::screenshot::{ScreenshotState, CapturedScreen, draw_screenshot_ui};
use xcap::Monitor;

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
    screenshot_state: ScreenshotState,
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
            screenshot_state: ScreenshotState::default(),
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
        handle_input_events(ctx, &mut self.data, &mut self.state);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            viewer::draw_top_panel(ctx, &mut self.state, &self.config);
            viewer::draw_bottom_panel(ctx, &mut self.state, &mut self.screenshot_state.is_active);
            viewer::draw_central_panel(ctx, &mut self.data, &mut self.state, &self.config);
            draw_properties_panel(ctx, &mut self.state, &self.data, &self.config);
            self.state.toast_system.update(ctx);
        }
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
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

            handle_crop_mode(ctx, &mut self.state, &mut self.data);
        }
    }

    fn handle_screenshot(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            return;
        }

        if self.screenshot_state.captures.is_empty() {
            if let Ok(monitors) = Monitor::all() {
                for monitor in monitors {
                    if let Ok(image) = monitor.capture_image() {
                        let color_image = ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            &image,
                        );
                        self.screenshot_state.captures.push(CapturedScreen {
                            image: color_image,
                            screen_info: monitor,
                            texture: None,
                            // 1. 初始化每个屏幕自己的局部状态
                            selection: None,
                            drag_start: None,
                        });
                    }
                }
            }
        }

        let mut wants_to_close = false;
        let mut temp_captures = std::mem::take(&mut self.screenshot_state.captures);

        for screen in &mut temp_captures {
            let screen_info = screen.screen_info.clone();
            let viewport_id = ViewportId::from_hash_of(format!("screenshot_{}", screen_info.name().unwrap()));
            let pos_in_logical_pixels = egui::pos2(screen_info.x().unwrap() as f32, screen_info.y().unwrap() as f32);

            ctx.show_viewport_immediate(
                viewport_id,
                ViewportBuilder::default()
                    .with_title("Screenshot")
                    .with_fullscreen(true)
                    .with_decorations(false)
                    .with_always_on_top()
                    .with_position(pos_in_logical_pixels),
                |ctx, class| {
                    if class == ViewportClass::Immediate {
                        // 2. 调用更新后的函数，只传入 screen
                        if draw_screenshot_ui(ctx, screen) {
                            wants_to_close = true;
                        }
                    }
                    if wants_to_close {
                        ctx.send_viewport_cmd(ViewportCommand::Close)
                    }
                },
            );
        }
        self.screenshot_state.captures = temp_captures;

        if wants_to_close {
            self.screenshot_state.is_active = false;
            self.screenshot_state.captures.clear();
            // 3. 不再需要清理全局的 selection 和 drag_start
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);
        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
        self.handle_screenshot(ctx);
    }
}
