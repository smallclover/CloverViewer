use eframe::egui::{Painter, Pos2, Rect, Stroke};

use crate::feature::screenshot::canvas::phys_to_local;
use crate::feature::screenshot::capture::{ScreenshotState, ScreenshotTool};
use crate::feature::screenshot::draw::draw_egui_shape;

/// 渲染正在绘制中的预览（current_shape / current_pen）
pub fn render_current_preview(
    painter: &Painter,
    state: &ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    viewport_rect: Rect,
) {
    // 预览当前 shape
    if let (Some(start_phys), Some(end_phys)) =
        (state.input.current_shape_start, state.input.current_shape_end)
    {
        let start_local = phys_to_local(start_phys, global_offset_phys, ppp);
        let end_local = phys_to_local(end_phys, global_offset_phys, ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        if viewport_rect.intersects(rect) {
            if let Some(tool) = state.drawing.current_tool {
                draw_egui_shape(
                    painter,
                    tool,
                    rect,
                    start_local,
                    end_local,
                    state.drawing.stroke_width,
                    state.drawing.active_color,
                );
            }
        }
    }

    // 预览画笔/马赛克
    if !state.input.current_pen_points.is_empty() {
        if state.drawing.current_tool == Some(ScreenshotTool::Mosaic) {
            crate::feature::screenshot::canvas::mosaic::draw_realtime_mosaic(
                painter,
                &state.input.current_pen_points,
                state.drawing.mosaic_width,
                global_offset_phys,
                ppp,
                state.select.selection,
                &state.capture.captures,
            );
        } else {
            let mut local_points = Vec::with_capacity(state.input.current_pen_points.len());
            for p in &state.input.current_pen_points {
                local_points.push(phys_to_local(*p, global_offset_phys, ppp));
            }
            let stroke = Stroke::new(state.drawing.stroke_width, state.drawing.active_color);
            painter.add(eframe::egui::Shape::line(local_points, stroke));
        }
    }
}
