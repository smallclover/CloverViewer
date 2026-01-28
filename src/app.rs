use crate::{
    config::{load_config, save_config, Config},
    constants::SUPPORTED_IMAGE_EXTENSIONS,
    core::viewer_core::ViewerCore,
    ui::{
        about::render_about_window,
        arrows::Nav,
        loading::global_loading,
        menu::draw_menu,
        preview::show_preview_window,
        resources::APP_FONT,
        right_click_menu::render_context_menu,
        settings::render_settings_window,
        ui_mode::UiMode,
        viewer::{draw_viewer, ViewerState},
    },
    utils::load_icon,
};
use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, Key, ViewportBuilder};
use rfd::FileDialog;
use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
};

pub fn run() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    // 加载图标
    options.viewport = options.viewport.with_icon(load_icon());

    let start_path = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, start_path)))),
    )
}

pub struct MyApp {
    core: ViewerCore,
    ui_mode: UiMode, // UI 状态机
    config: Config,  // 配置
    path_sender: Sender<PathBuf>,
    path_receiver: Receiver<PathBuf>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>) -> Self {
        // --- 1. 字体配置 ---
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

        // 加载配置
        let config = load_config();

        let (path_sender, path_receiver) = mpsc::channel();

        let mut app = Self {
            core: ViewerCore::new(),
            ui_mode: UiMode::Normal,
            config,
            path_sender,
            path_receiver,
        };

        if let Some(path) = start_path {
            app.core.open_new_context(cc.egui_ctx.clone(), path);
        }

        app
    }

    /// 输入处理
    fn handler_inputs(&mut self, ctx: &Context) {
        // 处理键盘输入
        ctx.input(|i| {
            if i.key_pressed(Key::ArrowLeft) {
                self.core.prev_image(ctx.clone());
            }
            if i.key_pressed(Key::ArrowRight) {
                self.core.next_image(ctx.clone());
            }
        });

        // 拖拽
        if let Some(path) = ctx.input(|i| {
            i.raw
                .dropped_files
                .first()
                .and_then(|f| f.path.clone())
        }) {
            self.core.handle_dropped_file(ctx.clone(), path);
        }

        // 放大缩小
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        self.core.update_zoom(scroll_delta);
    }

    fn ui_top_panel(&mut self, ctx: &Context) {
        let (open_file, open_folder) =
            draw_menu(ctx, &mut self.ui_mode, self.config.language, &self.config);

        if open_file {
            let sender = self.path_sender.clone();
            std::thread::spawn(move || {
                if let Some(path) = FileDialog::new()
                    .add_filter("Image", SUPPORTED_IMAGE_EXTENSIONS)
                    .pick_file()
                {
                    sender.send(path).ok();
                }
            });
        }

        if open_folder {
            let sender = self.path_sender.clone();
            std::thread::spawn(move || {
                if let Some(path) = FileDialog::new().pick_folder() {
                    sender.send(path).ok();
                }
            });
        }
    }

    fn ui_central_panel(&mut self, ctx: &Context) {
        let viewer_state = ViewerState {
            texture: self.core.current_texture.as_ref(),
            is_loading: self.core.loader.is_loading,
            error: self.core.error.as_ref(),
            zoom: self.core.zoom,
            has_nav: self.core.nav.current().is_some(),
        };

        if let Some(action) = draw_viewer(ctx, viewer_state, &mut self.ui_mode, self.config.language)
        {
            match action {
                Nav::Prev => self.core.prev_image(ctx.clone()),
                Nav::Next => self.core.next_image(ctx.clone()),
            }
        }
    }

    fn ui_preview_panel(&mut self, ctx: &Context) {
        if show_preview_window(
            ctx,
            &mut self.core.nav,
            &mut self.core.thumb_cache,
            &self.core.failed_thumbs,
        ) {
            self.core.load_current(ctx.clone());
        }
    }

    fn ui_overlays(&mut self, ctx: &Context) {
        let mut new_ui_mode = None;
        match &mut self.ui_mode {
            UiMode::About => {
                let mut open = true;
                render_about_window(ctx, &mut open, self.config.language);
                if !open {
                    new_ui_mode = Some(UiMode::Normal);
                }
            }
            UiMode::Settings(temp_config) => {
                let mut open = true;
                let mut action = render_settings_window(
                    ctx,
                    &mut open,
                    self.config.language,
                    &mut temp_config.language,
                );

                if action == crate::ui::modal::ModalAction::Apply {
                    self.config = temp_config.clone();
                    save_config(&self.config);
                    action = crate::ui::modal::ModalAction::Close;
                }

                if !open || action == crate::ui::modal::ModalAction::Close {
                    new_ui_mode = Some(UiMode::Normal);
                }
            }
            UiMode::ContextMenu(pos) => {
                let mut pos_opt = Some(*pos);
                render_context_menu(ctx, &mut pos_opt, self.config.language);
                if pos_opt.is_none() {
                    new_ui_mode = Some(UiMode::Normal);
                }
            }
            UiMode::Normal => {}
        }

        if let Some(new_mode) = new_ui_mode {
            self.ui_mode = new_mode;
        }

        if self.core.current_texture.is_none() && self.core.loader.is_loading {
            global_loading(ctx, self.config.language);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // 1. 数据层：处理异步加载和文件选择
        if self.core.process_load_results(ctx) {
            ctx.request_repaint();
        }
        if let Ok(path) = self.path_receiver.try_recv() {
            self.core.open_new_context(ctx.clone(), path);
        }

        // 2. 控制层：处理输入
        self.handler_inputs(ctx);

        // 3. 视图层：渲染各个区域
        self.ui_top_panel(ctx);
        self.ui_central_panel(ctx);
        self.ui_preview_panel(ctx);
        self.ui_overlays(ctx);
    }
}
