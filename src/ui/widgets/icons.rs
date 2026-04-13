use crate::i18n::lang::{TextBundle, get_i18n_text};
use eframe::epaint::StrokeKind;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, vec2};

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Grid,
    Single,
    Text,
    DrawRect,
    DrawCircle,
    DrawArrow,
    Pencil,
    Mosaic,
    Cancel,
    Save,
    SaveToClipboard,
    Ocr,
}

impl IconType {
    pub fn tooltip(&self, text: &TextBundle) -> &'static str {
        match self {
            IconType::Grid => text.status_grid,
            IconType::Single => text.status_single,
            IconType::Text => text.tooltip_draw_text,
            IconType::DrawRect => text.tooltip_draw_rect,
            IconType::DrawCircle => text.tooltip_draw_circle,
            IconType::DrawArrow => text.tooltip_draw_arrow,
            IconType::Pencil => text.tooltip_draw_pencil,
            IconType::Mosaic => text.tooltip_draw_mosaic,
            IconType::Cancel => text.tooltip_cancel,
            IconType::Save => text.tooltip_save,
            IconType::SaveToClipboard => text.tooltip_save_to_clipboard,
            IconType::Ocr => text.tooltip_ocr,
        }
    }
}

/// 核心：纯粹的图标绘制逻辑（内部几何线条）
pub fn paint_icon(
    painter: &egui::Painter,
    icon_rect: Rect,
    icon_type: IconType,
    stroke: Stroke,
    bg_color: Color32,
) {
    match icon_type {
        IconType::Grid => paint_grid_icon(painter, icon_rect, stroke),
        IconType::Single => paint_single_icon(painter, icon_rect, stroke),
        IconType::Text => paint_text_icon(painter, icon_rect, stroke),
        IconType::DrawRect => paint_rect_icon(painter, icon_rect, stroke),
        IconType::DrawCircle => paint_circle_icon(painter, icon_rect, stroke),
        IconType::DrawArrow => paint_arrow_icon(painter, icon_rect, stroke),
        IconType::Pencil => paint_pencil_icon(painter, icon_rect, stroke),
        IconType::Mosaic => paint_mosaic_icon(painter, icon_rect, stroke),
        IconType::Cancel => paint_cancel_icon(painter, icon_rect, stroke),
        IconType::Save => paint_save_icon(painter, icon_rect, stroke),
        IconType::SaveToClipboard => paint_clipboard_icon(painter, icon_rect, stroke, bg_color),
        IconType::Ocr => paint_ocr_icon(painter, icon_rect, stroke),
    }
}

fn paint_grid_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
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

fn paint_single_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    painter.rect_stroke(icon_rect, 1.0, stroke, StrokeKind::Outside);
    let inner = icon_rect.shrink(2.0);
    let p1 = inner.left_bottom() - vec2(0.0, 2.0);
    let p2 = inner.center_bottom() - vec2(0.0, inner.height() * 0.6);
    let p3 = inner.right_bottom() - vec2(0.0, 2.0);
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
}

fn paint_text_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    let rect = icon_rect.shrink(1.0);
    painter.line_segment([rect.left_top(), rect.right_top()], stroke);
    painter.line_segment([rect.center_top(), rect.center_bottom()], stroke);
}

fn paint_rect_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    painter.rect_stroke(icon_rect, 2.0, stroke, StrokeKind::Outside);
}

fn paint_circle_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    painter.circle_stroke(icon_rect.center(), icon_rect.width() / 2.0, stroke);
}

fn paint_arrow_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
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

fn paint_pencil_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    let mut points = Vec::new();
    let segments = 12;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let x = egui::lerp(icon_rect.left()..=icon_rect.right(), t);
        let base_y = egui::lerp(icon_rect.bottom()..=icon_rect.top(), t);
        let offset_y = (t * std::f32::consts::PI * 2.5).sin() * (icon_rect.height() * 0.25);
        points.push(Pos2::new(x, base_y + offset_y));
    }
    painter.add(egui::Shape::line(points, stroke));
}

fn paint_mosaic_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    let gap = 1.5;
    let width = (icon_rect.width() - gap) / 2.0;
    let height = (icon_rect.height() - gap) / 2.0;

    let top_left = Rect::from_min_size(icon_rect.left_top(), egui::vec2(width, height));
    let top_right = Rect::from_min_size(
        Pos2::new(icon_rect.left() + width + gap, icon_rect.top()),
        egui::vec2(width, height),
    );
    let bottom_left = Rect::from_min_size(
        Pos2::new(icon_rect.left(), icon_rect.top() + height + gap),
        egui::vec2(width, height),
    );
    let bottom_right = Rect::from_min_size(
        Pos2::new(
            icon_rect.left() + width + gap,
            icon_rect.top() + height + gap,
        ),
        egui::vec2(width, height),
    );

    let corner_radius = 1.0;
    painter.rect_filled(top_left, corner_radius, stroke.color);
    painter.rect_filled(bottom_right, corner_radius, stroke.color);
    painter.rect_stroke(
        top_right,
        corner_radius,
        Stroke::new(1.2, stroke.color),
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        bottom_left,
        corner_radius,
        Stroke::new(1.2, stroke.color),
        StrokeKind::Inside,
    );
}

fn paint_cancel_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    let inner = icon_rect.shrink(2.0);
    painter.line_segment([inner.left_top(), inner.right_bottom()], stroke);
    painter.line_segment([inner.right_top(), inner.left_bottom()], stroke);
}

fn paint_save_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    let arrow_top = icon_rect.center() - vec2(0.0, 2.0);
    let arrow_bottom = icon_rect.center() + vec2(0.0, 5.0);
    painter.line_segment([arrow_top, arrow_bottom], stroke);
    painter.line_segment([arrow_bottom, arrow_bottom - vec2(3.0, 3.0)], stroke);
    painter.line_segment([arrow_bottom, arrow_bottom + vec2(3.0, -3.0)], stroke);
    painter.line_segment(
        [
            icon_rect.left_top() + vec2(0.0, 2.0),
            icon_rect.right_top() + vec2(0.0, 2.0),
        ],
        stroke,
    );
}

fn paint_clipboard_icon(
    painter: &egui::Painter,
    icon_rect: Rect,
    stroke: Stroke,
    bg_color: Color32,
) {
    let rect_size = icon_rect.size() * 0.75;
    let offset = icon_rect.size() * 0.25;

    let back_rect = Rect::from_min_size(icon_rect.min + offset, rect_size);
    painter.rect_filled(back_rect, 1.0, bg_color);
    painter.rect_stroke(back_rect, 1.0, stroke, StrokeKind::Outside);

    let fore_rect = Rect::from_min_size(icon_rect.min, rect_size);
    painter.rect_filled(fore_rect, 1.0, bg_color);
    painter.rect_stroke(fore_rect, 1.0, stroke, StrokeKind::Outside);
}

fn paint_ocr_icon(painter: &egui::Painter, icon_rect: Rect, stroke: Stroke) {
    let rect = icon_rect.shrink(1.0);
    let pad = 3.0;
    painter.rect_stroke(rect, 1.0, stroke, StrokeKind::Outside);
    painter.line_segment(
        [
            rect.left_top() + vec2(pad, pad),
            rect.right_top() + vec2(-pad, pad),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.left_top() + vec2(pad, pad + 3.0),
            rect.right_top() + vec2(-pad, pad + 3.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.left_top() + vec2(pad, pad + 6.0),
            rect.left_top() + vec2(rect.width() - pad - 2.0, pad + 6.0),
        ],
        stroke,
    );
}

/// 供工具栏使用：带交互背景、正方形边框的大尺寸（32x32）按钮
pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType, size: f32) -> Response {
    let text = get_i18n_text(ui);
    let button_size = vec2(size, size);
    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click_and_drag());
    response.clone().on_hover_text(icon_type.tooltip(text));

    let painter = ui.painter();
    let rounded_rect = rect.shrink(1.0);
    // 动态计算圆角：例如尺寸的 12.5%，但不小于 2.0
    let corner_radius = (size * 0.125).max(2.0);

    // 1. 绘制正方形的背景色 (Hover 或 选中状态)
    if selected {
        painter.rect_filled(
            rounded_rect,
            corner_radius,
            Color32::from_rgba_premultiplied(0, 120, 215, 20),
        );
    } else if response.hovered() {
        painter.rect_filled(rounded_rect, corner_radius, Color32::from_gray(245));
    }

    // 2. 绘制正方形的外边框
    let box_stroke = if selected {
        Stroke::new(1.5, Color32::from_rgb(0, 120, 215)) // 选中时的蓝色边框
    } else {
        Stroke::new(1.0, Color32::from_gray(220)) // 默认状态的浅灰色边框
    };
    painter.rect_stroke(rounded_rect, corner_radius, box_stroke, StrokeKind::Outside);

    // 3. 绘制内部的几何图标
    let icon_color = if selected {
        Color32::from_rgb(0, 120, 215)
    } else {
        Color32::from_gray(80)
    };
    // 动态计算线宽：大尺寸线粗一点，小尺寸线细一点
    let stroke_width = (size / 24.0).clamp(1.0, 2.5);
    let stroke = Stroke::new(stroke_width, icon_color);

    // 动态内边距：按钮尺寸的 25% 作为 padding (例如 32的padding是8)
    let icon_rect = rect.shrink(size * 0.25);

    paint_icon(painter, icon_rect, icon_type, stroke, Color32::WHITE);

    response
}

/// 供 Help Box 使用：纯静态展示，完美融入 13pt 文本行内
pub fn draw_inline_icon(ui: &mut Ui, icon_type: IconType) {
    let size = egui::vec2(14.0, 14.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();

    let stroke = Stroke::new(1.2, Color32::from_rgb(230, 230, 230));
    paint_icon(
        painter,
        rect,
        icon_type,
        stroke,
        Color32::from_black_alpha(200),
    );
}
