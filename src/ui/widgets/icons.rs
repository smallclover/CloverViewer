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
            IconType::Grid => text.status_gird,
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
        IconType::Grid => {
            let gap = 2.0;
            let cell_size = (icon_rect.width() - gap) / 2.0;
            for i in 0..2 {
                for j in 0..2 {
                    let x = icon_rect.min.x + i as f32 * (cell_size + gap);
                    let y = icon_rect.min.y + j as f32 * (cell_size + gap);
                    let cell_rect =
                        Rect::from_min_size(egui::pos2(x, y), vec2(cell_size, cell_size));
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
        IconType::Pencil => {
            // 绘制一条带有弧度的波浪线/涂鸦线来代表画笔工具
            let mut points = Vec::new();
            let segments = 12;
            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                // 从左下到右上的对角线基础
                let x = egui::lerp(icon_rect.left()..=icon_rect.right(), t);
                let base_y = egui::lerp(icon_rect.bottom()..=icon_rect.top(), t);
                // 加上正弦波形产生曲线感
                let offset_y = (t * std::f32::consts::PI * 2.5).sin() * (icon_rect.height() * 0.25);
                points.push(Pos2::new(x, base_y + offset_y));
            }
            painter.add(egui::Shape::line(points, stroke));
        }
        IconType::Mosaic => {
            // 极简马赛克图标：2x2 错位像素块，中间留 1.5 像素的呼吸间隙
            let gap = 1.5;
            let w = (icon_rect.width() - gap) / 2.0;
            let h = (icon_rect.height() - gap) / 2.0;

            let tl = Rect::from_min_size(icon_rect.left_top(), egui::vec2(w, h));
            let tr = Rect::from_min_size(
                Pos2::new(icon_rect.left() + w + gap, icon_rect.top()),
                egui::vec2(w, h),
            );
            let bl = Rect::from_min_size(
                Pos2::new(icon_rect.left(), icon_rect.top() + h + gap),
                egui::vec2(w, h),
            );
            let br = Rect::from_min_size(
                Pos2::new(icon_rect.left() + w + gap, icon_rect.top() + h + gap),
                egui::vec2(w, h),
            );

            let corner_radius = 1.0; // 加一点点圆角让方块边缘不那么锐利，更精致

            // 左上、右下使用实心
            painter.rect_filled(tl, corner_radius, stroke.color);
            painter.rect_filled(br, corner_radius, stroke.color);

            // 右上、左下使用细边框的空心
            painter.rect_stroke(
                tr,
                corner_radius,
                Stroke::new(1.2, stroke.color),
                StrokeKind::Inside,
            );
            painter.rect_stroke(
                bl,
                corner_radius,
                Stroke::new(1.2, stroke.color),
                StrokeKind::Inside,
            );
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
            painter.line_segment(
                [
                    icon_rect.left_top() + vec2(0.0, 2.0),
                    icon_rect.right_top() + vec2(0.0, 2.0),
                ],
                stroke,
            );
        }
        IconType::SaveToClipboard => {
            // 调整尺寸让图标看起来紧凑
            let rect_size = icon_rect.size() * 0.75;
            let offset = icon_rect.size() * 0.25;

            // 后面的方块 (右下移)
            let back_rect = Rect::from_min_size(icon_rect.min + offset, rect_size);
            // 这里使用 bg_color 填充以覆盖线条重叠部分，使其看起来有层级
            painter.rect_filled(back_rect, 1.0, bg_color);
            painter.rect_stroke(back_rect, 1.0, stroke, StrokeKind::Outside);

            // 前面的方块 (左上)
            let fore_rect = Rect::from_min_size(icon_rect.min, rect_size);
            painter.rect_filled(fore_rect, 1.0, bg_color);
            painter.rect_stroke(fore_rect, 1.0, stroke, StrokeKind::Outside);
        }
        IconType::Ocr => {
            let r = icon_rect.shrink(1.0);
            painter.rect_stroke(r, 1.0, stroke, StrokeKind::Outside);
            let pad = 3.0;
            painter.line_segment(
                [
                    r.left_top() + vec2(pad, pad),
                    r.right_top() + vec2(-pad, pad),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    r.left_top() + vec2(pad, pad + 3.0),
                    r.right_top() + vec2(-pad, pad + 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    r.left_top() + vec2(pad, pad + 6.0),
                    r.left_top() + vec2(r.width() - pad - 2.0, pad + 6.0),
                ],
                stroke,
            );
        }
    }
}

/// 供工具栏使用：带交互背景、正方形边框的大尺寸（32x32）按钮
pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType, size: f32) -> Response {
    let text = get_i18n_text(ui.ctx());
    let button_size = vec2(size, size);
    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click_and_drag());
    response.clone().on_hover_text(icon_type.tooltip(&text));

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
