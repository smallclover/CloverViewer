use eframe::egui;
use egui::{
    Color32, Context, CursorIcon, Rect, RichText, ScrollArea, Spinner, TextureHandle, Ui, UiBuilder,
};

use crate::feature::viewer::arrows::{Nav, draw_arrows};
use crate::feature::viewer::preview::show_preview_window;
use crate::i18n::lang::get_i18n_text;
use crate::{
    core::viewer_state::{TransitionPhase, ViewerState},
    model::mode::OverlayMode,
};

pub fn draw_single_view(
    ctx: &Context,
    ui: &mut Ui,
    viewer: &mut ViewerState,
    overlay: &mut OverlayMode,
) {
    let rect = ui.available_rect_before_wrap();
    let text = get_i18n_text(ctx);

    let current_texture = viewer.current_texture.clone();
    let is_transitioning = viewer.transition_phase != TransitionPhase::None;

    render_image_viewer(ui, rect, current_texture.as_ref(), viewer);

    if !is_transitioning {
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
                        *overlay = OverlayMode::ContextMenu(pos);
                    }
                }
            }
        }
    }

    if current_texture.is_none() && viewer.transition_phase == TransitionPhase::None {
        if let Some(err) = viewer.error.as_ref() {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.4);
                    ui.label(
                        RichText::new(text.viewer_error)
                            .color(Color32::RED)
                            .size(14.0),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(err.to_string())
                            .color(Color32::GRAY)
                            .size(12.0),
                    );
                });
            });
        } else if viewer.loader.is_loading {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(Spinner::new().size(32.0));
                });
            });
        } else if viewer.list.is_empty() {
            ui.centered_and_justified(|ui| ui.label(text.viewer_no_images));
        } else {
            ui.centered_and_justified(|ui| ui.label(text.viewer_drag_hint));
        }
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
    view_rect: Rect,
    tex: Option<&TextureHandle>,
    viewer: &mut ViewerState,
) {
    let now = ui.input(|i| i.time);

    let fade_in_duration = 0.12;

    let next_ready = viewer.transition_target_path.is_some()
        && viewer.current_texture.is_some()
        && viewer.current_texture_path.as_ref() == viewer.transition_target_path.as_ref();

    if viewer.transition_phase == TransitionPhase::WaitNext && next_ready {
        viewer.transition_phase = TransitionPhase::FadeIn;
        viewer.transition_phase_start_time = Some(now);
    }

    match viewer.transition_phase {
        TransitionPhase::None => {
            if let Some(tex) = tex {
                render_normal_image(ui, tex, viewer);
            }
        }
        TransitionPhase::WaitNext => {
            ui.painter().rect_filled(view_rect, 0.0, Color32::BLACK);

            if viewer.loader.is_loading {
                let spinner_rect =
                    Rect::from_center_size(view_rect.center(), egui::vec2(32.0, 32.0));
                ui.put(spinner_rect, egui::Spinner::new().size(32.0));
            }

            if viewer.error.is_some() && !viewer.loader.is_loading {
                viewer.transition_phase = TransitionPhase::None;
                viewer.transition_phase_start_time = None;
                viewer.transition_target_path = None;
            } else if viewer.loader.is_loading {
                ui.ctx().request_repaint();
            }
        }
        TransitionPhase::FadeIn => {
            if !next_ready {
                viewer.transition_phase = TransitionPhase::WaitNext;
                viewer.transition_phase_start_time = Some(now);
                return;
            }

            let start = viewer.transition_phase_start_time.unwrap_or(now);
            let progress = ((now - start) / fade_in_duration).clamp(0.0, 1.0) as f32;
            let overlay_alpha = 1.0 - progress;

            if let Some(tex) = tex {
                render_normal_image(ui, tex, viewer);
            }
            if overlay_alpha > 0.0 {
                ui.painter().rect_filled(
                    view_rect,
                    0.0,
                    Color32::from_black_alpha((overlay_alpha * 255.0) as u8),
                );
            }

            if progress < 1.0 {
                ui.ctx().request_repaint();
            } else {
                viewer.transition_phase = TransitionPhase::None;
                viewer.transition_phase_start_time = None;
                viewer.transition_target_path = None;
                viewer.previous_texture = None;
                viewer.previous_zoom = None;
            }
        }
    }
}

fn render_normal_image(ui: &mut Ui, tex: &TextureHandle, viewer: &mut ViewerState) {
    let available_size = ui.available_size();
    if let Some(last_size) = viewer.last_view_size {
        if (last_size.x - available_size.x).abs() > 1.0
            || (last_size.y - available_size.y).abs() > 1.0
        {
            viewer.zoom = viewer.calc_fit_zoom(ui.ctx(), tex.size_vec2());
        }
    }
    viewer.last_view_size = Some(available_size);

    let is_loading_high_res = viewer.loader.is_loading;
    let zoom = viewer.zoom.max(0.01);
    let size = (tex.size_vec2() * zoom).max(egui::Vec2::ZERO);
    let is_draggable = size.x > available_size.x || size.y > available_size.y;

    if is_draggable && ui.rect_contains_pointer(ui.max_rect()) {
        ui.ctx().set_cursor_icon(CursorIcon::Move);
    }

    let fade_alpha = ui.ctx().animate_bool_with_time(
        egui::Id::new(tex.id()).with("loading_fade"),
        is_loading_high_res,
        0.25,
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

                    ui.painter().image(
                        tex.id(),
                        img_rect,
                        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    if fade_alpha > 0.0 {
                        let painter = ui.painter_at(img_rect);
                        painter.rect_filled(
                            img_rect,
                            0.0,
                            Color32::BLACK.gamma_multiply(fade_alpha * 0.4),
                        );
                        let spinner_rect =
                            Rect::from_center_size(img_rect.center(), egui::vec2(32.0, 32.0));
                        ui.put(
                            spinner_rect,
                            egui::Spinner::new()
                                .size(32.0)
                                .color(Color32::WHITE.gamma_multiply(fade_alpha)),
                        );
                    }
                });
            });
        });
}
