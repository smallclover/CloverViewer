use eframe::epaint::StrokeKind;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, vec2};

pub enum IconType {
    Grid,
    Single,
}

pub fn draw_icon_button(ui: &mut Ui, selected: bool, icon_type: IconType) -> Response {
    let size = vec2(20.0, 20.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    // Hover background
    if response.hovered() {
        ui.painter().rect_filled(rect, 4.0, Color32::from_white_alpha(20));
    }

    let color = if selected { Color32::LIGHT_BLUE } else { Color32::GRAY };
    let stroke = Stroke::new(1.5, color);
    let painter = ui.painter();

    match icon_type {
        IconType::Grid => {
            // Draw 4 small squares
            let margin = 3.0;
            let gap = 2.0;
            let cell_size = (size.x - margin * 2.0 - gap) / 2.0;

            for i in 0..2 {
                for j in 0..2 {
                    let x = rect.min.x + margin + i as f32 * (cell_size + gap);
                    let y = rect.min.y + margin + j as f32 * (cell_size + gap);
                    let cell_rect = Rect::from_min_size(egui::pos2(x, y), vec2(cell_size, cell_size));
                    painter.rect_stroke(cell_rect, 1.0, stroke,StrokeKind::Outside);
                }
            }
        }
        IconType::Single => {
            // Draw 1 large rectangle
            let margin = 3.0;
            let inner_rect = rect.shrink(margin);
            painter.rect_stroke(inner_rect, 1.0, stroke,StrokeKind::Outside);

            // Draw a small "mountain" to make it look like an image
            let p1 = inner_rect.left_bottom() - vec2(-2.0, 2.0);
            let p2 = inner_rect.center_bottom() - vec2(0.0, inner_rect.height() * 0.6);
            let p3 = inner_rect.right_bottom() - vec2(2.0, 2.0);

            // Simple polyline for mountain
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        }
    }

    response
}
