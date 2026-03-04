use eframe::egui::{Color32, Painter, Rect, Shape, Stroke, StrokeKind};
use image::RgbaImage;
use crate::ui::screenshot::capture::{DrawnShape, ScreenshotTool};

/// 渲染 UI 时的实时绘图 (Egui)
pub fn draw_egui_shape(
    painter: &Painter,
    tool: ScreenshotTool,
    rect: Rect,
    stroke_width: f32,
    color: Color32,
) {
    match tool {
        ScreenshotTool::Rect => {
            painter.rect_stroke(rect, 0.0, Stroke::new(stroke_width, color), StrokeKind::Outside);
        }
        ScreenshotTool::Circle => {
            painter.add(Shape::ellipse_stroke(
                rect.center(),
                rect.size() / 2.0,
                Stroke::new(stroke_width, color),
            ));
        }
    }
}

/// 导出图片时的抗锯齿高质量绘图 (Tiny-Skia)
pub fn draw_skia_shapes_on_image(
    final_image: &mut RgbaImage,
    shapes: &[DrawnShape],
    selection_phys: Rect,
) {
    let final_width = final_image.width();
    let final_height = final_image.height();

    // 将 image 库的像素直接映射给 tiny-skia 进行硬件级别的绘制
    if let Some(mut pixmap) = tiny_skia::PixmapMut::from_bytes(
        final_image,
        final_width,
        final_height,
    ) {
        for shape in shapes {
            let start_x = shape.start.x - selection_phys.min.x;
            let start_y = shape.start.y - selection_phys.min.y;
            let end_x = shape.end.x - selection_phys.min.x;
            let end_y = shape.end.y - selection_phys.min.y;

            let x0 = start_x.min(end_x);
            let y0 = start_y.min(end_y);
            let width = (start_x - end_x).abs();
            let height = (start_y - end_y).abs();

            if width <= 0.0 || height <= 0.0 { continue; }

            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(shape.color.r(), shape.color.g(), shape.color.b(), shape.color.a());
            paint.anti_alias = true;

            let stroke = tiny_skia::Stroke {
                width: shape.stroke_width,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };

            let transform = tiny_skia::Transform::identity();

            if let Some(rect) = tiny_skia::Rect::from_xywh(x0, y0, width, height) {
                match shape.tool {
                    ScreenshotTool::Rect => {
                        let path = tiny_skia::PathBuilder::from_rect(rect);
                        pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                    }
                    ScreenshotTool::Circle => {
                        // 使用 if let 防止绘制异常宽高的椭圆时触发 unwrap 崩溃
                        if let Some(path) = tiny_skia::PathBuilder::from_oval(rect) {
                            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                        }
                    }
                }
            }
        }
    }
}