use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame,
    Image, Rect, RichText, ScrollArea, TextureHandle, Ui, UiBuilder
};
use rfd::FileDialog;
use crate::{
    core::business::BusinessData,
    i18n::lang::get_text,
    model::{config::Config, state::ViewState},
    ui::components::{
        about::render_about_window,
        arrows::{draw_arrows, Nav},
        context_menu::{render_context_menu, ContextMenuAction},
        loading::global_loading,
        menu::draw_menu,
        modal::ModalAction,
        preview::show_preview_window,
        settings::render_settings_window,
        ui_mode::UiMode,
    },
};
use crate::model::constants::SUPPORTED_IMAGE_EXTENSIONS;

pub fn draw_top_panel(
    ctx: &Context,
    state: &mut ViewState,
    config: &Config,
) {
    let texts = get_text(config.language);
    let (open_file, open_folder) = draw_menu(ctx, &mut state.ui_mode, texts, config);

    if open_file {
        let sender = state.path_sender.clone();
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
        let sender = state.path_sender.clone();
        std::thread::spawn(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                sender.send(path).ok();
            }
        });
    }
}

pub fn draw_central_panel(
    ctx: &Context,
    data: &mut BusinessData,
    state: &mut ViewState,
    config: &Config,
) {
    let texts = get_text(config.language);
    let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        let rect = ui.available_rect_before_wrap();

        if let Some(tex) = data.current_texture.as_ref() {
            render_image_viewer(ui, tex, data.zoom, data.loader.is_loading);

            if ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if rect.contains(pos) {
                        let mut allow_context_menu = true;
                        if data.current().is_some() {
                            let hover_zone_width = 100.0;
                            let center_zone = Rect::from_min_max(
                                rect.min + egui::vec2(hover_zone_width, 0.0),
                                rect.max - egui::vec2(hover_zone_width, 0.0),
                            );
                            if !center_zone.contains(pos) {
                                allow_context_menu = false;
                            }
                        }
                        if allow_context_menu {
                            state.ui_mode = UiMode::ContextMenu(pos);
                        }
                    }
                }
            }
        } else if let Some(_) = data.error.as_ref() {
            ui.scope_builder(UiBuilder::new().max_rect(rect),|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.4);
                    ui.label(RichText::new(texts.viewer_error).color(Color32::RED).size(14.0));
                });
            });
        } else if data.loader.is_loading {
            // loading, but no texture yet
        } else {
            ui.centered_and_justified(|ui| ui.label(texts.viewer_drag_hint));
        }

        if data.current().is_some() {
            if let Some(action) = draw_arrows(ui, rect) {
                match action {
                    Nav::Prev => data.prev_image(ctx.clone()),
                    Nav::Next => data.next_image(ctx.clone()),
                }
            }
        }
    });
}

fn render_image_viewer(
    ui: &mut Ui,
    tex: &TextureHandle,
    zoom: f32,
    is_loading_high_res: bool
) {
    let size = tex.size_vec2() * zoom;
    let available_size = ui.available_size();
    let is_draggable = size.x > available_size.x || size.y > available_size.y;

    if is_draggable {
        if ui.rect_contains_pointer(ui.max_rect()) {
            ui.ctx().set_cursor_icon(CursorIcon::Move);
        }
    }
    let fade_alpha = ui.ctx().animate_bool_with_time(
        egui::Id::new(tex.id()).with("loading_fade"),
        is_loading_high_res,
        0.25
    );

    ScrollArea::both()
        .scroll_source(egui::scroll_area::ScrollSource::DRAG)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let x_offset = (available_size.x - size.x).max(0.0) * 0.5;
            let y_offset = (available_size.y - size.y).max(0.0) * 0.5;

            ui.horizontal(|ui| {
                ui.add_space(x_offset);
                ui.vertical(|ui| {
                    ui.add_space(y_offset);
                    let img_rect = ui.allocate_exact_size(size, egui::Sense::hover()).0;
                    let img_widget = Image::from_texture(tex).fit_to_exact_size(size);
                    ui.put(img_rect, img_widget);

                    if fade_alpha > 0.0 {
                        let painter = ui.painter_at(img_rect);
                        painter.rect_filled(
                            img_rect,
                            0.0,
                            Color32::BLACK.gamma_multiply(fade_alpha * 0.4)
                        );
                        let spinner_size = 32.0;
                        let spinner_rect = Rect::from_center_size(
                            img_rect.center(),
                            egui::vec2(spinner_size, spinner_size)
                        );
                        ui.put(spinner_rect, egui::Spinner::new().size(spinner_size).color(Color32::WHITE.gamma_multiply(fade_alpha)));
                    }
                });
            });
        });
}

pub fn draw_preview_panel(ctx: &Context, data: &mut BusinessData) {
    if show_preview_window(ctx, data) {
        data.load_current(ctx.clone());
    }
}

pub fn draw_overlays(
    ctx: &Context,
    data: &BusinessData,
    state: &mut ViewState,
    temp_config: &mut Config,
) -> (Option<ContextMenuAction>, Option<ModalAction>) {
    let mut context_menu_action = None;
    let mut modal_action = None;
    let mut new_ui_mode = None;
    let texts = get_text(temp_config.language);

    match &mut state.ui_mode {
        UiMode::About => {
            let mut open = true;
            render_about_window(ctx, &mut open, texts);
            if !open {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::Settings(cfg) => {
            let mut open = true;
            let mut action = render_settings_window(ctx, &mut open, texts, cfg);

            if action == ModalAction::Apply {
                *temp_config = cfg.clone();
                modal_action = Some(ModalAction::Apply);
                action = ModalAction::Close;
            }

            if !open || action == ModalAction::Close {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::ContextMenu(pos) => {
            let mut pos_opt = Some(*pos);
            let action = render_context_menu(ctx, &mut pos_opt, texts);

            if let Some(action) = action {
                context_menu_action = Some(action);
            }

            if pos_opt.is_none() {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::Normal => {}
    }

    if let Some(new_mode) = new_ui_mode {
        state.ui_mode = new_mode;
    }

    if data.current_texture.is_none() && data.loader.is_loading {
        global_loading(ctx, texts.loading_parsing.to_string());
    }

    (context_menu_action, modal_action)
}
