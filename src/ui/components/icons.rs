use eframe::epaint::StrokeKind;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, vec2};
use crate::i18n::lang::TextBundle;

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Grid,
    Single,
    DrawRect,
    DrawCircle,
    Cancel,
    Save,
    SaveToClipboard,
}

impl IconType {
    pub fn tooltip(&self, text: &TextBundle) -> &'static str {
        match self {
            IconType::Grid => text.status_gird,
            IconType::Single => text.status_single,
            IconType::DrawRect => text.tooltip_draw_rect,
            IconType::DrawCircle => text.tooltip_draw_circle,
            IconType::Cancel => text.tooltip_cancel,
            IconType::Save => text.tooltip_save,
            IconType::SaveToClipboard => text.tooltip_save_to_clipboard,
        }
    }
}

pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType, text: &TextBundle) -> Response {
    // 1. 统一按钮大小 32x32
    let button_size = vec2(32.0, 32.0);
    // Use Sense::click_and_drag to enable long_touched
    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click_and_drag());

    // 添加 Tooltip
    response.clone().on_hover_text(icon_type.tooltip(text));

    let painter = ui.painter();

    // 2. 交互状态样式
    if response.hovered() {
        painter.rect_filled(rect, 4.0, Color32::from_gray(245));
    }

    if selected {
        painter.rect_stroke(
            rect.shrink(1.0),
            4.0,
            Stroke::new(2.0, Color32::from_rgb(0, 120, 215)),
            StrokeKind::Outside,
        );
        painter.rect_filled(rect.shrink(1.0), 4.0, Color32::from_rgba_premultiplied(0, 120, 215, 20));
    }

    // 3. 绘制图标
    let icon_color = if selected { Color32::from_rgb(0, 120, 215) } else { Color32::from_gray(80) };
    let stroke = Stroke::new(1.5, icon_color);
    let icon_rect = rect.shrink(8.0); // 图标内容区域

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
        IconType::DrawRect => {
            painter.rect_stroke(icon_rect, 2.0, stroke, StrokeKind::Outside);
        }
        IconType::DrawCircle => {
            painter.circle_stroke(
                icon_rect.center(),
                icon_rect.width() / 2.0,
                stroke,
            );
        }
        IconType::Cancel => {
            let inner = icon_rect.shrink(2.0);
            painter.line_segment([inner.left_top(), inner.right_bottom()], stroke);
            painter.line_segment([inner.right_top(), inner.left_bottom()], stroke);
        }
        IconType::Save => {
            // 下载样式
            let arrow_top = icon_rect.center() - vec2(0.0, 2.0);
            let arrow_bottom = icon_rect.center() + vec2(0.0, 5.0);
            painter.line_segment([arrow_top, arrow_bottom], stroke);
            painter.line_segment([arrow_bottom, arrow_bottom - vec2(3.0, 3.0)], stroke);
            painter.line_segment([arrow_bottom, arrow_bottom + vec2(3.0, -3.0)], stroke);

            painter.line_segment([icon_rect.left_top() + vec2(0.0, 2.0), icon_rect.right_top() + vec2(0.0, 2.0)], stroke);
        }
        IconType::SaveToClipboard => {
            // 剪贴板样式
            let clip_rect = icon_rect.shrink(1.0);
            painter.rect_stroke(clip_rect, 2.0, stroke, StrokeKind::Outside);
            let top_rect = Rect::from_min_max(clip_rect.min - vec2(-2.0, 2.0), clip_rect.max - vec2(2.0, clip_rect.height()));
            painter.rect_filled(top_rect, 0.0, Color32::WHITE);
            painter.rect_stroke(top_rect, 1.0, stroke, StrokeKind::Outside);
        }
    }

    response
}