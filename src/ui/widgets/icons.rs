use eframe::epaint::StrokeKind;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, vec2};
use crate::i18n::lang::TextBundle;

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Grid,
    Single,
    Text,
    DrawRect,
    DrawCircle,
    DrawArrow,
    Cancel,
    Save,
    SaveToClipboard,
}

impl IconType {
    pub fn tooltip(&self, text: &TextBundle) -> &'static str {
        match self {
            IconType::Grid => text.status_gird,
            IconType::Single => text.status_single,
            IconType::Text => text.tooltip_draw_text,
            IconType::DrawRect => text.tooltip_draw_rect,
            IconType::DrawCircle => text.tooltip_draw_circle,
            IconType::DrawArrow => text.tooltip_draw_arrow,
            IconType::Cancel => text.tooltip_cancel,
            IconType::Save => text.tooltip_save,
            IconType::SaveToClipboard => text.tooltip_save_to_clipboard,
        }
    }
}

/// 核心：纯粹的图标绘制逻辑（内部几何线条）
pub fn paint_icon(painter: &egui::Painter, icon_rect: Rect, icon_type: IconType, stroke: Stroke, bg_color: Color32) {
    match icon_type {
        IconType::Grid => {
            let gap = 2.0;
            let cell_size = (icon_rect.width() - gap) / 2.0;
            for i in 0..2 {
                for j in 0..2 {
                    let x = icon_rect.min.x + i as f32 * (cell_size + gap);
                    let y = icon_rect.min.y + j as f32 * (cell_size + gap);
                    let cell_rect = Rect::from_min_size(egui::pos2(x, y), vec2(cell_size, cell_size));
                    painter.rect_stroke(cell_rect, 1.0, stroke, StrokeKind::Outside);
                }
            }
        }
        IconType::Single => {
            painter.rect_stroke(icon_rect, 1.0, stroke, StrokeKind::Outside);
            let inner = icon_rect.shrink(2.0);
            let p1 = inner.left_bottom() - vec2(0.0, 2.0);
            let p2 = inner.center_bottom() - vec2(0.0, inner.height() * 0.6);
            let p3 = inner.right_bottom() - vec2(0.0, 2.0);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        }
        IconType::Text => {
            let r = icon_rect.shrink(1.0);
            painter.line_segment([r.left_top(), r.right_top()], stroke); // T的横线
            painter.line_segment([r.center_top(), r.center_bottom()], stroke); // T的竖线
        }
        IconType::DrawRect => {
            painter.rect_stroke(icon_rect, 2.0, stroke, StrokeKind::Outside);
        }
        IconType::DrawCircle => {
            painter.circle_stroke(icon_rect.center(), icon_rect.width() / 2.0, stroke);
        }
        IconType::DrawArrow => {
            let start = icon_rect.left_bottom() + vec2(2.0, -2.0);
            let end = icon_rect.right_top() + vec2(-2.0, 2.0);
            painter.line_segment([start, end], stroke);

            let arrow_size = 4.0;
            let dir = (end - start).normalized();
            let arrow_p1 = end - dir * arrow_size + vec2(dir.y, -dir.x) * arrow_size;
            let arrow_p2 = end - dir * arrow_size - vec2(dir.y, -dir.x) * arrow_size;
            painter.line_segment([end, arrow_p1], stroke);
            painter.line_segment([end, arrow_p2], stroke);
        }
        IconType::Cancel => {
            let inner = icon_rect.shrink(2.0);
            painter.line_segment([inner.left_top(), inner.right_bottom()], stroke);
            painter.line_segment([inner.right_top(), inner.left_bottom()], stroke);
        }
        IconType::Save => {
            let arrow_top = icon_rect.center() - vec2(0.0, 2.0);
            let arrow_bottom = icon_rect.center() + vec2(0.0, 5.0);
            painter.line_segment([arrow_top, arrow_bottom], stroke);
            painter.line_segment([arrow_bottom, arrow_bottom - vec2(3.0, 3.0)], stroke);
            painter.line_segment([arrow_bottom, arrow_bottom + vec2(3.0, -3.0)], stroke);
            painter.line_segment([icon_rect.left_top() + vec2(0.0, 2.0), icon_rect.right_top() + vec2(0.0, 2.0)], stroke);
        }
        IconType::SaveToClipboard => {
            let clip_rect = icon_rect.shrink(1.0);
            painter.rect_stroke(clip_rect, 2.0, stroke, StrokeKind::Outside);
            let top_rect = Rect::from_min_max(clip_rect.min - vec2(-2.0, 2.0), clip_rect.max - vec2(2.0, clip_rect.height()));
            painter.rect_filled(top_rect, 0.0, bg_color);
            painter.rect_stroke(top_rect, 1.0, stroke, StrokeKind::Outside);
        }
    }
}

/// 供工具栏使用：带交互背景、正方形边框的大尺寸（32x32）按钮
pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType, text: &TextBundle) -> Response {
    let button_size = vec2(32.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click_and_drag());
    response.clone().on_hover_text(icon_type.tooltip(text));

    let painter = ui.painter();
    let rounded_rect = rect.shrink(1.0);
    let corner_radius = 4.0;

    // 1. 绘制正方形的背景色 (Hover 或 选中状态)
    if selected {
        painter.rect_filled(rounded_rect, corner_radius, Color32::from_rgba_premultiplied(0, 120, 215, 20));
    } else if response.hovered() {
        painter.rect_filled(rounded_rect, corner_radius, Color32::from_gray(245));
    }

    // 2. 绘制正方形的外边框
    let box_stroke = if selected {
        Stroke::new(1.5, Color32::from_rgb(0, 120, 215)) // 选中时的蓝色边框
    } else {
        Stroke::new(1.0, Color32::from_gray(220))        // 默认状态的浅灰色边框
    };
    painter.rect_stroke(rounded_rect, corner_radius, box_stroke, StrokeKind::Outside);

    // 3. 绘制内部的几何图标
    let icon_color = if selected { Color32::from_rgb(0, 120, 215) } else { Color32::from_gray(80) };
    let stroke = Stroke::new(1.5, icon_color);
    let icon_rect = rect.shrink(8.0);

    paint_icon(painter, icon_rect, icon_type, stroke, Color32::WHITE);

    response
}

/// 供 Help Box 使用：纯静态展示，完美融入 13pt 文本行内
pub fn draw_inline_icon(ui: &mut Ui, icon_type: IconType) {
    let size = egui::vec2(14.0, 14.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    //
    // // 【核心修改】给 Help Box 的小图标也加上一个正方形淡淡的边框，让它看起来像按键
    // painter.rect_stroke(
    //     rect.shrink(0.5),
    //     2.0,
    //     Stroke::new(1.0, Color32::from_white_alpha(50)), // 半透明白色边框
    //     StrokeKind::Outside
    // );

    let stroke = Stroke::new(1.2, Color32::from_rgb(230, 230, 230));
    paint_icon(painter, rect, icon_type, stroke, Color32::from_black_alpha(200));
}