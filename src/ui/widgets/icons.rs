use eframe::epaint::StrokeKind;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, vec2};
use crate::i18n::lang::TextBundle;

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Grid,
    Single,
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
            IconType::DrawRect => text.tooltip_draw_rect,
            IconType::DrawCircle => text.tooltip_draw_circle,
            IconType::DrawArrow => text.tooltip_draw_arrow,
            IconType::Cancel => text.tooltip_cancel,
            IconType::Save => text.tooltip_save,
            IconType::SaveToClipboard => text.tooltip_save_to_clipboard,
        }
    }
}

/// 图标绘制逻辑，剥离了按钮的交互和背景
/// [bg_color] 用于需要遮挡底层线条的图标（如剪贴板顶部的夹子）
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

/// 供工具栏使用：带交互背景、大尺寸（32x32）的按钮
pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType, text: &TextBundle) -> Response {
    let button_size = vec2(32.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click_and_drag());
    response.clone().on_hover_text(icon_type.tooltip(text));

    let painter = ui.painter();

    // 交互状态样式
    if response.hovered() {
        painter.rect_filled(rect, 4.0, Color32::from_gray(245));
    }
    if selected {
        painter.rect_stroke(rect.shrink(1.0), 4.0, Stroke::new(2.0, Color32::from_rgb(0, 120, 215)), StrokeKind::Outside);
        painter.rect_filled(rect.shrink(1.0), 4.0, Color32::from_rgba_premultiplied(0, 120, 215, 20));
    }

    // 设置线条颜色与内缩距
    let icon_color = if selected { Color32::from_rgb(0, 120, 215) } else { Color32::from_gray(80) };
    let stroke = Stroke::new(1.5, icon_color);
    let icon_rect = rect.shrink(8.0);

    // 调用核心绘制逻辑 (Toolbar使用白色作为剪贴板的遮挡背景)
    paint_icon(painter, icon_rect, icon_type, stroke, Color32::WHITE);

    response
}

/// 供 Help Box 使用：纯静态展示，无交互
pub fn draw_inline_icon(ui: &mut Ui, icon_type: IconType) {
    let size = egui::vec2(14.0, 14.0);
    // 只分配空间，不加 Sense::click_and_drag()
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    let stroke = Stroke::new(1.2, Color32::from_rgb(230, 230, 230));

    // 调用核心绘制逻辑 (Help Box 的剪贴板背景使用透明或深色，这里用深色避免穿帮)
    paint_icon(painter, rect, icon_type, stroke, Color32::from_black_alpha(200));
}