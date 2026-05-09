use eframe::egui;
use egui::{
    Color32, Context, CursorIcon, Rect, RichText, Spinner, TextureHandle, Ui, UiBuilder,
};

use crate::feature::viewer::arrows::{Nav, draw_arrows};
use crate::feature::viewer::preview::show_preview_window;
use crate::feature::viewer::viewer_state::{TransitionPhase, ViewerState};
use crate::i18n::lang::get_i18n_text;
use crate::model::mode::PopupMode;

pub fn draw_single_view(
    ctx: &Context,
    ui: &mut Ui,
    viewer: &mut ViewerState,
    popup: &mut PopupMode,
) {
    let rect = ui.available_rect_before_wrap();
    let text = get_i18n_text(ctx);

    let current_texture = viewer.current.texture.clone();
    let is_transitioning = viewer.transition.phase != TransitionPhase::None;

    let is_draggable = render_image_viewer(ui, rect, current_texture.as_ref(), viewer);

    // 设置光标：仅当指针在中央区域、图片可拖拽、且不在箭头区域时显示 Move
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let pointer_in_rect = pointer_pos.map_or(false, |pos| rect.contains(pos));
    let in_arrow_zone = pointer_pos.map_or(false, |pos| {
        pos.x < rect.min.x + 100.0 || pos.x > rect.max.x - 100.0
    });

    let has_popup = !matches!(popup, PopupMode::None);

    if !has_popup && is_draggable && pointer_in_rect && !in_arrow_zone {
        ui.set_cursor_icon(CursorIcon::Move);
    } else {
        ui.set_cursor_icon(CursorIcon::Default);
    }

    if !is_transitioning
        && ui.input(|i| i.pointer.secondary_clicked())
        && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
        && rect.contains(pos)
    {
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
            *popup = PopupMode::ContextMenu(pos);
        }
    }

    if current_texture.is_none() && viewer.transition.phase == TransitionPhase::None {
        if let Some(err) = viewer.current.error.as_ref() {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.4);
                    ui.label(
                        RichText::new(text.viewer.error)
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
            ui.centered_and_justified(|ui| ui.label(text.viewer.no_images));
        } else {
            ui.centered_and_justified(|ui| ui.label(text.viewer.drag_hint));
        }
    }

    if viewer.current().is_some()
        && let Some(action) = draw_arrows(ui, rect)
    {
        match action {
            Nav::Prev => viewer.prev_image(ctx.clone()),
            Nav::Next => viewer.next_image(ctx.clone()),
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
) -> bool {
    let now = ui.input(|i| i.time);

    let fade_in_duration = 0.12;

    let next_ready = viewer.transition.target_path.is_some()
        && viewer.current.texture.is_some()
        && viewer.current.texture_path.as_ref() == viewer.transition.target_path.as_ref();

    if viewer.transition.phase == TransitionPhase::WaitNext && next_ready {
        viewer.transition.phase = TransitionPhase::FadeIn;
        viewer.transition.phase_start_time = Some(now);
    }

    match viewer.transition.phase {
        TransitionPhase::None => {
            if let Some(tex) = tex {
                return render_normal_image(ui, tex, viewer);
            }
        }
        TransitionPhase::WaitNext => {
            ui.painter().rect_filled(view_rect, 0.0, Color32::BLACK);

            if viewer.loader.is_loading {
                let spinner_rect =
                    Rect::from_center_size(view_rect.center(), egui::vec2(32.0, 32.0));
                ui.put(spinner_rect, egui::Spinner::new().size(32.0));
            }

            if viewer.current.error.is_some() && !viewer.loader.is_loading {
                viewer.transition.phase = TransitionPhase::None;
                viewer.transition.phase_start_time = None;
                viewer.transition.target_path = None;
            } else if viewer.loader.is_loading {
                ui.request_repaint();
            }
        }
        TransitionPhase::FadeIn => {
            if !next_ready {
                viewer.transition.phase = TransitionPhase::WaitNext;
                viewer.transition.phase_start_time = Some(now);
                return false;
            }

            let start = viewer.transition.phase_start_time.unwrap_or(now);
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
                ui.request_repaint();
            } else {
                viewer.transition.phase = TransitionPhase::None;
                viewer.transition.phase_start_time = None;
                viewer.transition.target_path = None;
                viewer.transition.previous_texture = None;
                viewer.transition.previous_zoom = None;
            }
        }
    };

    false
}

fn render_normal_image(ui: &mut Ui, tex: &TextureHandle, viewer: &mut ViewerState) -> bool {
    let available_size = ui.available_size();
    viewer.last_view_size = Some(available_size);

    let is_loading_high_res = viewer.loader.is_loading;
    let zoom = viewer.zoom.max(0.01);
    let img_size = tex.size_vec2() * zoom;
    let viewport = ui.max_rect();

    let is_draggable = img_size.x > viewport.width() || img_size.y > viewport.height();

    // Auto-center when image is smaller than viewport
    if !is_draggable {
        viewer.viewport_offset = egui::vec2(
            (viewport.width() - img_size.x) * 0.5,
            (viewport.height() - img_size.y) * 0.5,
        );
    }

    let img_origin = viewport.min + viewer.viewport_offset;
    let img_rect = Rect::from_min_size(img_origin, img_size);

    // Handle drag
    let sense = if is_draggable {
        egui::Sense::drag()
    } else {
        egui::Sense::hover()
    };
    let response = ui.allocate_rect(viewport, sense);

    if is_draggable && response.dragged() {
        viewer.viewport_offset += response.drag_delta();
    }

    // Draw the image
    ui.painter().image(
        tex.id(),
        img_rect,
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );

    // Loading overlay
    let fade_alpha = ui.animate_bool_with_time(
        egui::Id::new(tex.id()).with("loading_fade"),
        is_loading_high_res,
        0.25,
    );

    if fade_alpha > 0.0 {
        let painter = ui.painter_at(img_rect);
        painter.rect_filled(
            img_rect,
            0.0,
            Color32::BLACK.gamma_multiply(fade_alpha * 0.4),
        );
        let spinner_rect = Rect::from_center_size(img_rect.center(), egui::vec2(32.0, 32.0));
        ui.put(
            spinner_rect,
            egui::Spinner::new()
                .size(32.0)
                .color(Color32::WHITE.gamma_multiply(fade_alpha)),
        );
    }

    is_draggable
}
