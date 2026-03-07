use eframe::egui;
use egui::{
    Color32, Context, CursorIcon, Rect, RichText, ScrollArea, TextureHandle, Ui, UiBuilder
};

use crate::{
    core::business::ViewerState,
    ui::mode::UiMode,
    ui::view::preview::show_preview_window,
};
use crate::i18n::lang::get_i18n_text;
use crate::ui::view::arrows::{draw_arrows, Nav};

pub fn draw_single_view(
    ctx: &Context,
    ui: &mut Ui,
    viewer: &mut ViewerState,
    ui_mode: &mut UiMode,
) {
    let rect = ui.available_rect_before_wrap();
    let text = get_i18n_text(ctx);

    let current_texture = viewer.current_texture.clone();

    if let Some(tex) = current_texture.as_ref() {
        render_image_viewer(ui, tex, viewer);

        if ui.input(|i| i.pointer.secondary_clicked()) {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(pos) {
                    let mut allow_context_menu = true;
                    if viewer.current().is_some() {
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
                        *ui_mode = UiMode::ContextMenu(pos);
                    }
                }
            }
        }
    } else if let Some(_) = viewer.error.as_ref() {
        ui.scope_builder(UiBuilder::new().max_rect(rect),|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.4);
                ui.label(RichText::new(text.viewer_error).color(Color32::RED).size(14.0));
            });
        });
    } else if viewer.loader.is_loading {
    } else if viewer.current().is_some() && viewer.list.is_empty() {
        ui.centered_and_justified(|ui| ui.label(text.viewer_no_images));
    } else {
        ui.centered_and_justified(|ui| ui.label(text.viewer_drag_hint));
    }

    if viewer.current().is_some() {
        if let Some(action) = draw_arrows(ui, rect) {
            match action {
                Nav::Prev => viewer.prev_image(ctx.clone()),
                Nav::Next => viewer.next_image(ctx.clone()),
            }
        }
    }

    if show_preview_window(ctx, viewer) {
        viewer.load_current(ctx.clone());
    }
}

fn render_image_viewer(
    ui: &mut Ui,
    tex: &TextureHandle,
    viewer: &mut ViewerState,
) {
    let available_size = ui.available_size();
    if let Some(last_size) = viewer.last_view_size {
        if (last_size.x - available_size.x).abs() > 1.0 ||
            (last_size.y - available_size.y).abs() > 1.0
        {
            viewer.zoom = viewer.calc_fit_zoom(ui.ctx(), tex.size_vec2());
        }
    }
    viewer.last_view_size = Some(available_size);
    let zoom = viewer.zoom;
    let is_loading_high_res = viewer.loader.is_loading;
    let size = tex.size_vec2() * zoom;
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

                    if viewer.transition_start_time.is_some() {
                        viewer.transition_start_time = None;
                        viewer.previous_texture = None;
                    }

                    let prev_alpha = 0.0;
                    let current_alpha = 1.0;
                    let current_scale = 1.0;

                    if let Some(prev_tex) = &viewer.previous_texture {
                        if prev_alpha > 0.0 {
                            let prev_size = prev_tex.size_vec2() * zoom;
                            let prev_x_offset = (available_size.x - prev_size.x).max(0.0) * 0.5;
                            let prev_y_offset = (available_size.y - prev_size.y).max(0.0) * 0.5;

                            let content_origin = img_rect.min - egui::vec2(x_offset, y_offset);
                            let prev_rect = Rect::from_min_size(
                                content_origin + egui::vec2(prev_x_offset, prev_y_offset),
                                prev_size
                            );

                            let painter = ui.painter();
                            painter.image(
                                prev_tex.id(),
                                prev_rect,
                                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                Color32::WHITE.gamma_multiply(prev_alpha)
                            );
                        }
                    }

                    let scaled_size = size * current_scale;
                    let center = img_rect.center();
                    let current_rect = Rect::from_center_size(center, scaled_size);

                    ui.painter().image(
                        tex.id(),
                        current_rect,
                        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE.gamma_multiply(current_alpha)
                    );

                    if fade_alpha > 0.0 {
                        let painter = ui.painter_at(current_rect);
                        painter.rect_filled(
                            current_rect,
                            0.0,
                            Color32::BLACK.gamma_multiply(fade_alpha * 0.4)
                        );
                        let spinner_size = 32.0;
                        let spinner_rect = Rect::from_center_size(
                            current_rect.center(),
                            egui::vec2(spinner_size, spinner_size)
                        );
                        ui.put(spinner_rect, egui::Spinner::new().size(spinner_size).color(Color32::WHITE.gamma_multiply(fade_alpha)));
                    }
                });
            });
        });
}
