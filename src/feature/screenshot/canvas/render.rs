use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Ui};

use crate::feature::screenshot::canvas::{CanvasState, shape::ShapeRender};
use crate::feature::screenshot::capture::{ScreenshotState, ScreenshotTool};

/// 渲染画布所有元素
pub fn render_canvas_elements(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    canvas_state: &CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    is_hovered: bool,
) {
    // 先调用需要可变借用 ui 的函数
    crate::feature::screenshot::canvas::text_input::render_text_input(
        ui, state, global_offset_phys, ppp,
    );

    let painter = ui.painter();
    let viewport_rect = ui.ctx().viewport_rect();

    render_overlay(ui, painter, state, global_offset_phys, ppp, viewport_rect, is_hovered);

    let hovered_index = canvas_state.hovered_shape;
    let dragging_index = canvas_state.dragging_shape;
    let selected_index = canvas_state.selected_shape;

    for (index, shape) in state.shapes.iter_mut().enumerate() {
        // 视口裁剪
        let rect = shape.bounding_rect(global_offset_phys, ppp);
        let mut visible = viewport_rect.intersects(rect);

        if shape.tool == ScreenshotTool::Text {
            let start_local = phys_to_local(shape.start, global_offset_phys, ppp);
            if let Some(galley) = shape.ensure_galley(painter) {
                let text_rect = Rect::from_min_size(start_local, galley.size());
                visible = viewport_rect.intersects(text_rect);
            }
        }

        if !visible {
            continue;
        }

        // 高亮状态
        let is_highlighted = (Some(index) == hovered_index
            || Some(index) == dragging_index
            || Some(index) == selected_index)
            && state.current_shape_start.is_none();

        // 马赛克特殊处理：需要 captures
        if shape.tool == ScreenshotTool::Mosaic {
            if let Some(points) = &shape.points {
                // 检查是否有缓存纹理
                if let Some(ref cache) = shape.cached_mosaic {
                    // 使用缓存纹理
                    let local_min = phys_to_local(cache.phys_rect.min, global_offset_phys, ppp);
                    let local_max = phys_to_local(cache.phys_rect.max, global_offset_phys, ppp);
                    let local_rect = Rect::from_min_max(local_min, local_max);
                    painter.image(
                        cache.texture.id(),
                        local_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    // 实时渲染并生成缓存
                    crate::feature::screenshot::canvas::mosaic::draw_realtime_mosaic(
                        painter,
                        points,
                        shape.stroke_width,
                        global_offset_phys,
                        ppp,
                        &state.captures,
                    );
                    // 异步生成纹理缓存（下一帧使用）
                    if let Some(cache) = crate::feature::screenshot::canvas::mosaic::generate_mosaic_texture(
                        ui.ctx(),
                        points,
                        shape.stroke_width,
                        &state.captures,
                    ) {
                        shape.cached_mosaic = Some(std::sync::Arc::new(cache));
                    }
                }
            }
            if is_highlighted {
                let start_local = phys_to_local(shape.start, global_offset_phys, ppp);
                let end_local = phys_to_local(shape.end, global_offset_phys, ppp);
                let highlight_rect = Rect::from_two_pos(start_local, end_local);
                painter.rect_stroke(
                    highlight_rect.expand(2.0),
                    2.0,
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 150, 255, 100)),
                    StrokeKind::Outside,
                );
            }
        } else {
            shape.render(painter, global_offset_phys, ppp, is_highlighted);
        }
    }

    crate::feature::screenshot::canvas::draw::render_current_preview(
        painter, state, global_offset_phys, ppp, viewport_rect,
    );

    // 绘制选中图形的控制点和选中边框
    if let Some(selected_idx) = selected_index {
        if let Some(shape) = state.shapes.get(selected_idx) {
            if shape.supports_resize() {
                // 绘制选中边框（蓝色实线）
                let bbox = shape.bounding_rect(global_offset_phys, ppp);
                let selection_border_color = Color32::from_rgb(0, 150, 255);
                painter.rect_stroke(
                    bbox.expand(2.0),
                    2.0,
                    Stroke::new(1.0, selection_border_color),
                    StrokeKind::Outside,
                );

                // 绘制控制点
                let handles = shape.resize_handles(global_offset_phys, ppp);
                let handle_fill = Color32::WHITE;
                let handle_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 60));

                for (local_pos, _) in handles {
                    let rect = Rect::from_center_size(local_pos, eframe::egui::vec2(10.0, 10.0));
                    painter.rect_filled(rect, 0.0, handle_fill);
                    painter.rect_stroke(rect, 0.0, handle_stroke, StrokeKind::Inside);
                }
            }
        }
    }
}

fn phys_to_local(pos: Pos2, global_offset_phys: Pos2, ppp: f32) -> Pos2 {
    Pos2::ZERO + ((pos - global_offset_phys) / ppp)
}

/// 绘制选区或悬浮窗口的遮罩
fn render_overlay(
    ui: &Ui,
    painter: &Painter,
    state: &ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    viewport_rect: Rect,
    is_hovered: bool,
) {
    let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);

    if let Some(global_sel_phys) = state.selection {
        let vec_min = global_sel_phys.min - global_offset_phys;
        let vec_max = global_sel_phys.max - global_offset_phys;
        let local_logical_rect =
            Rect::from_min_max(Pos2::ZERO + (vec_min / ppp), Pos2::ZERO + (vec_max / ppp));
        let clipped_local_sel = local_logical_rect.intersect(viewport_rect);

        if clipped_local_sel.is_positive() {
            paint_selection_overlay(
                painter,
                clipped_local_sel,
                viewport_rect,
                1.0,
                overlay_color,
            );

            if viewport_rect.expand(1.0).contains(local_logical_rect.min) {
                let w = global_sel_phys.width().round() as u32;
                let h = global_sel_phys.height().round() as u32;
                let text = format!("{} x {}", w, h);
                let font_id = eframe::egui::FontId::proportional(12.0);
                let text_color = Color32::WHITE;
                let galley = painter.layout_no_wrap(text, font_id, text_color);
                let padding = eframe::egui::vec2(6.0, 4.0);
                let bg_size = galley.size() + padding * 2.0;

                let mut label_pos = local_logical_rect.min - eframe::egui::vec2(0.0, bg_size.y + 5.0);
                if label_pos.y < viewport_rect.min.y {
                    label_pos = local_logical_rect.min + eframe::egui::vec2(5.0, 5.0);
                }

                let label_rect = Rect::from_min_size(label_pos, bg_size);
                painter.rect_filled(label_rect, 4.0, Color32::from_black_alpha(160));
                painter.galley(label_rect.min + padding, galley, text_color);
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }
        return;
    }

    if state.current_shape_start.is_none() && state.drag_start.is_none() {
        if is_hovered {
            if let Some(hover_phys_rect) = state.hovered_window {
                paint_hover_window_overlay(
                    painter,
                    hover_phys_rect,
                    global_offset_phys,
                    ppp,
                    viewport_rect,
                    overlay_color,
                );
            } else if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);
                if let Some(cap_phys_rect) =
                    crate::model::device::find_target_screen_rect(&state.captures, global_pointer_phys)
                {
                    let vec_min = cap_phys_rect.min - global_offset_phys;
                    let vec_max = cap_phys_rect.max - global_offset_phys;
                    let local_logical_rect = Rect::from_min_max(
                        Pos2::ZERO + (vec_min / ppp),
                        Pos2::ZERO + (vec_max / ppp),
                    );
                    paint_style_box(painter, local_logical_rect, 3.0);
                }
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }
    }
}

fn paint_selection_overlay(
    painter: &Painter,
    clipped_local_sel: Rect,
    viewport_rect: Rect,
    line_width: f32,
    overlay_color: Color32,
) {
    let top = Rect::from_min_max(
        viewport_rect.min,
        Pos2::new(viewport_rect.max.x, clipped_local_sel.min.y),
    );
    let bottom = Rect::from_min_max(
        Pos2::new(viewport_rect.min.x, clipped_local_sel.max.y),
        viewport_rect.max,
    );
    let left = Rect::from_min_max(
        Pos2::new(viewport_rect.min.x, clipped_local_sel.min.y),
        Pos2::new(clipped_local_sel.min.x, clipped_local_sel.max.y),
    );
    let right = Rect::from_min_max(
        Pos2::new(clipped_local_sel.max.x, clipped_local_sel.min.y),
        Pos2::new(viewport_rect.max.x, clipped_local_sel.max.y),
    );

    painter.rect_filled(top, 0.0, overlay_color);
    painter.rect_filled(bottom, 0.0, overlay_color);
    painter.rect_filled(left, 0.0, overlay_color);
    painter.rect_filled(right, 0.0, overlay_color);

    paint_style_box(painter, clipped_local_sel, line_width);
}

fn paint_hover_window_overlay(
    painter: &Painter,
    hover_phys_rect: Rect,
    global_offset_phys: Pos2,
    ppp: f32,
    viewport_rect: Rect,
    overlay_color: Color32,
) {
    let vec_min = hover_phys_rect.min - global_offset_phys;
    let vec_max = hover_phys_rect.max - global_offset_phys;
    let local_logical_rect =
        Rect::from_min_max(Pos2::ZERO + (vec_min / ppp), Pos2::ZERO + (vec_max / ppp));
    let clipped_local_sel = local_logical_rect.intersect(viewport_rect);

    if clipped_local_sel.is_positive() {
        let top = Rect::from_min_max(
            viewport_rect.min,
            Pos2::new(viewport_rect.max.x, clipped_local_sel.min.y),
        );
        let bottom = Rect::from_min_max(
            Pos2::new(viewport_rect.min.x, clipped_local_sel.max.y),
            viewport_rect.max,
        );
        let left = Rect::from_min_max(
            Pos2::new(viewport_rect.min.x, clipped_local_sel.min.y),
            Pos2::new(clipped_local_sel.min.x, clipped_local_sel.max.y),
        );
        let right = Rect::from_min_max(
            Pos2::new(clipped_local_sel.max.x, clipped_local_sel.min.y),
            Pos2::new(viewport_rect.max.x, clipped_local_sel.max.y),
        );

        painter.rect_filled(top, 0.0, overlay_color);
        painter.rect_filled(bottom, 0.0, overlay_color);
        painter.rect_filled(left, 0.0, overlay_color);
        painter.rect_filled(right, 0.0, overlay_color);

        paint_style_box(painter, clipped_local_sel, 2.0);
    }
}

fn paint_style_box(painter: &Painter, rect: Rect, line_width: f32) {
    let anchor_size = 6.0;
    let green = Color32::from_rgb(0, 255, 0);
    let main_stroke = Stroke::new(line_width, green);
    let anchor_stroke = Stroke::new(1.0, green);
    let anchor_fill = green;

    painter.rect_stroke(rect, 0.0, main_stroke, StrokeKind::Inside);

    if rect.width() > anchor_size * 3.0 && rect.height() > anchor_size * 3.0 {
        let inset = anchor_size / 2.0;
        let min = rect.min + eframe::egui::vec2(inset, inset);
        let max = rect.max - eframe::egui::vec2(inset, inset);
        let center = rect.center();

        let anchors = [
            min,
            Pos2::new(max.x, min.y),
            max,
            Pos2::new(min.x, max.y),
            Pos2::new(center.x, min.y),
            Pos2::new(center.x, max.y),
            Pos2::new(min.x, center.y),
            Pos2::new(max.x, center.y),
        ];

        for anchor_pos in anchors {
            let anchor_rect =
                Rect::from_center_size(anchor_pos, eframe::egui::vec2(anchor_size, anchor_size));
            painter.rect_filled(anchor_rect, 0.0, anchor_fill);
            painter.rect_stroke(anchor_rect, 0.0, anchor_stroke, StrokeKind::Inside);
        }
    }
}
