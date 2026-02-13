use eframe::epaint::StrokeKind;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, vec2};

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Grid,        // 新增恢复
    Single,      // 新增恢复
    DrawRect,
    DrawCircle,
    Cancel,
    Save,
}

pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType) -> Response {
    // 1. 统一按钮大小 32x32
    let button_size = vec2(32.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

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
            // 恢复 Grid 绘制逻辑 (4个小方块)
            let gap = 2.0;
            // 计算小方块大小： (总宽 - 间隙) / 2
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
            // 恢复 Single 绘制逻辑 (带山峰的大矩形)
            painter.rect_stroke(icon_rect, 1.0, stroke, StrokeKind::Outside);
            
            // 为了让山峰在矩形内部，我们稍微向内再缩一点
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
            // 对勾样式
            let start = icon_rect.left_center() + vec2(0.0, 1.0);
            let mid = icon_rect.center_bottom() - vec2(1.0, 3.0);
            let end = icon_rect.right_top() + vec2(0.0, 2.0);
            let points = vec![start, mid, end];
            painter.add(egui::Shape::line(points, stroke));
        }
    }

    response
}