use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager}; // 引入
use std::{
    path::PathBuf,
    sync::{mpsc, Arc},
};
use global_hotkey::hotkey::HotKey;
use crate::{
    core::business::BusinessData,
    model::{
        config::{load_config, save_config, Config},
        state::{ViewMode, ViewState},
    },
};
use crate::ui::components::{
    context_menu::handle_context_menu_action,
    modal::ModalAction,
    mouse::handle_input_events,
    properties_panel::draw_properties_panel,
    resources::APP_FONT,
    screenshot::{handle_screenshot_system, ScreenshotState},
};
use crate::ui::viewer;
use crate::utils::image::load_icon;
pub fn run(
    hotkeys_manager: GlobalHotKeyManager,
    hotkey: HotKey,
) -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    options.viewport = options.viewport.with_icon(load_icon());

    let start_path = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(move |cc| {
            Ok(Box::new(CloverApp::new(
                cc,
                start_path,
                hotkeys_manager,
                hotkey,
            )))
        }),
    )
}

pub struct CloverApp {
    data: BusinessData,
    state: ViewState,
    config: Config,
    screenshot_state: ScreenshotState,
    hotkey_receiver: mpsc::Receiver<()>,
    _hotkeys_manager: GlobalHotKeyManager,
}

impl CloverApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_path: Option<PathBuf>,
        hotkeys_manager: GlobalHotKeyManager,
        hotkey: HotKey,
    ) -> Self {
        // 1. 设置字体
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // 2. 建立通道并设置“带唤醒功能”的热键回调
        let (tx, rx) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();

        GlobalHotKeyEvent::set_event_handler(Some(Box::new(move |event: GlobalHotKeyEvent| {
            if event.id == hotkey.id() {
                let _ = tx.send(());
                // 【关键】强制唤醒后台运行的 egui 窗口
                ctx_clone.request_repaint();
            }
        })));

        let config = load_config();

        let mut app = Self {
            data: BusinessData::new(),
            state: ViewState::default(),
            config,
            screenshot_state: ScreenshotState::default(),
            hotkey_receiver: rx,
            _hotkeys_manager: hotkeys_manager,
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

    fn handle_hotkeys(&mut self, ctx: &Context) {
        if self.hotkey_receiver.try_recv().is_ok() {
            // 激活截图模式
            self.screenshot_state.is_active = true;

            // 【可选】如果希望窗口在按下快捷键时自动弹到最前
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }
    }

    fn handle_input_events(&mut self, ctx: &Context) {
        handle_input_events(ctx, &mut self.data);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            viewer::draw_top_panel(
                ctx,
                &mut self.state,
                &self.config,
                &mut self.screenshot_state.is_active,
            );
            viewer::draw_bottom_panel(ctx, &mut self.state);
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
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.handle_hotkeys(ctx); // 传入 ctx
        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);
        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
        handle_screenshot_system(ctx, &mut self.screenshot_state);
    }
}
