use eframe::egui;
use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, Key, TextureHandle, ViewportBuilder};
use rfd::FileDialog;
use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
};
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
        context_menu::{render_context_menu, ContextMenuAction},
        settings::render_settings_window,
        ui_mode::UiMode,
        viewer::{draw_viewer, ViewerState},
        toast::{ToastManager, ToastSystem},
        modal::ModalAction,
        properties_panel::{render_properties_panel, ImageProperties},
    },
    utils::load_icon,
    i18n::{TextBundle, get_text}
};

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
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, start_path)))),
    )
}

pub struct MyApp {
    core: ViewerCore,
    ui_mode: UiMode,
    config: Config,
    texts: &'static TextBundle,
    path_sender: Sender<PathBuf>,
    path_receiver: Receiver<PathBuf>,
    toast_system: ToastSystem,
    toast_manager: ToastManager,
    show_properties_panel: bool,
    image_properties: Option<ImageProperties>,
}

impl MyApp {
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
        let texts = get_text(config.language);
        let toast_system = ToastSystem::new();
        let toast_manager = toast_system.manager();

        let (path_sender, path_receiver) = mpsc::channel();

        let mut app = Self {
            core: ViewerCore::new(),
            ui_mode: UiMode::Normal,
            config,
            texts,
            path_sender,
            path_receiver,
            toast_system,
            toast_manager,
            show_properties_panel: false,
            image_properties: None,
        };

        if let Some(path) = start_path {
            app.core.open_new_context(cc.egui_ctx.clone(), path);
        }

        app
    }

    fn handler_inputs(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(Key::ArrowLeft) {
                self.core.prev_image(ctx.clone());
            }
            if i.key_pressed(Key::ArrowRight) {
                self.core.next_image(ctx.clone());
            }
        });

        if let Some(path) = ctx.input(|i| {
            i.raw
                .dropped_files
                .first()
                .and_then(|f| f.path.clone())
        }) {
            self.core.handle_dropped_file(ctx.clone(), path);
        }

        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        self.core.update_zoom(scroll_delta);
    }

    fn ui_top_panel(&mut self, ctx: &Context) {
        let (open_file, open_folder) =
            draw_menu(ctx, &mut self.ui_mode, self.texts, &self.config);

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
                render_about_window(ctx, &mut open, self.texts);
                if !open {
                    new_ui_mode = Some(UiMode::Normal);
                }
            }
            UiMode::Settings(temp_config) => {
                let mut open = true;
                let mut action = render_settings_window(
                    ctx,
                    &mut open,
                    self.texts,
                    &mut temp_config.language,
                );

                if action == ModalAction::Apply {
                    self.config = temp_config.clone();
                    self.texts = get_text(self.config.language);
                    save_config(&self.config);
                    action = ModalAction::Close;
                }

                if !open || action == ModalAction::Close {
                    new_ui_mode = Some(UiMode::Normal);
                }
            }
            UiMode::ContextMenu(pos) => {
                let mut pos_opt = Some(*pos);
                let action = render_context_menu(ctx, &mut pos_opt, self.texts);

                if let Some(action) = action {
                    self.handle_context_menu_action(action);
                }

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
            global_loading(ctx, self.texts.loading_parsing.to_string());
        }
    }

    fn handle_context_menu_action(&mut self, action: ContextMenuAction) {
        match action {
            ContextMenuAction::Copy => {
                if let (Some(tex), Some(pixels)) = (
                    self.core.current_texture.as_ref(),
                    self.core.current_raw_pixels.clone(),
                ) {
                    let [w, h] = tex.size();
                    copy_image_to_clipboard_async(
                        pixels,
                        w,
                        h,
                        &self.toast_manager,
                        self.texts,
                    );
                }
            }
            ContextMenuAction::CopyPath => {
                if let Some(path) = self.core.nav.current() {
                    let mut clipboard = arboard::Clipboard::new().unwrap();
                    let _ = clipboard.set_text(path.to_string_lossy().to_string());
                }
                self.toast_manager.success(self.texts.copied_message);
            }
            ContextMenuAction::ShowProperties => {
                self.show_properties_panel = true;
            }
        }
    }

    fn update_image_properties(&mut self) {
        if let (Some(path), Some(texture)) = (self.core.nav.current(), self.core.current_texture.as_ref()) {
            if let Ok(metadata) = std::fs::metadata(&path) {
                let [width, height] = texture.size();
                self.image_properties = Some(ImageProperties {
                    path: path.to_path_buf(),
                    width: width as u32,
                    height: height as u32,
                    size: metadata.len(),
                });
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        if self.core.process_load_results(ctx) {
            self.update_image_properties();
            ctx.request_repaint();
        }
        if let Ok(path) = self.path_receiver.try_recv() {
            self.core.open_new_context(ctx.clone(), path);
        }

        self.handler_inputs(ctx);

        self.ui_top_panel(ctx);
        self.ui_central_panel(ctx);
        self.ui_preview_panel(ctx);
        self.ui_overlays(ctx);

        render_properties_panel(ctx, &mut self.show_properties_panel, &self.image_properties);

        self.toast_system.update(ctx);
    }
}

fn copy_image_to_clipboard_async(
    pixels_arc: Arc<Vec<Color32>>,
    width: usize,
    height: usize,
    toast_manager: &ToastManager,
    text: &TextBundle,
) {
    toast_manager.loading(text.coping_message);

    let toast_clone = toast_manager.clone();
    let copied_message = text.copied_message;
    let copy_failed_message = text.copy_failed_message;
    std::thread::spawn(move || {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        let bytes: &[u8] = bytemuck::cast_slice(&pixels_arc);
        let img_data = arboard::ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Borrowed(bytes),
        };
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Err(e) = clipboard.set_image(img_data) {
            toast_clone.error(format!("{}: {}", copy_failed_message, e));
        } else {
            toast_clone.success(copied_message);
        }
    });
}
