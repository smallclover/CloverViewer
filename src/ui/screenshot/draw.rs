use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use image::RgbaImage;
use crate::ui::screenshot::capture::{DrawnShape, ScreenshotTool};

/// 渲染 UI 时的实时绘图 (Egui)
pub fn draw_egui_shape(
    painter: &Painter,
    tool: ScreenshotTool,
    rect: Rect,
    start: Pos2,
    end: Pos2,
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
        ScreenshotTool::Arrow => {
            draw_arrow_egui(painter, start, end, stroke_width, color);
        }
    }
}

/// 绘制箭头 (Egui)
fn draw_arrow_egui(painter: &Painter, start: Pos2, end: Pos2, stroke_width: f32, color: Color32) {
    let stroke = Stroke::new(stroke_width, color);

    // 绘制主线
    painter.line_segment([start, end], stroke);

    // 计算箭头方向
    let dir = (end - start).normalized();
    if dir == Vec2::ZERO { return; }

    // 箭头头部大小
    let arrow_size = 12.0 + stroke_width * 2.0;

    // 计算箭头两翼
    let arrow_p1 = end - dir * arrow_size + Vec2::new(dir.y, -dir.x) * arrow_size * 0.5;
    let arrow_p2 = end - dir * arrow_size - Vec2::new(dir.y, -dir.x) * arrow_size * 0.5;

    // 绘制箭头头部
    painter.line_segment([end, arrow_p1], stroke);
    painter.line_segment([end, arrow_p2], stroke);
}

/// 绘制箭头 (Tiny-Skia)
fn draw_arrow_skia(
    pixmap: &mut tiny_skia::PixmapMut,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    paint: &tiny_skia::Paint,
    stroke: &tiny_skia::Stroke,
) {
    let transform = tiny_skia::Transform::identity();

    // 计算箭头方向
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 { return; }

    let dir_x = dx / len;
    let dir_y = dy / len;

    // 箭头头部大小
    let arrow_size = 12.0 + stroke.width * 2.0;

    // 计算箭头两翼的端点
    let arrow_p1_x = end_x - dir_x * arrow_size + dir_y * arrow_size * 0.5;
    let arrow_p1_y = end_y - dir_y * arrow_size - dir_x * arrow_size * 0.5;
    let arrow_p2_x = end_x - dir_x * arrow_size - dir_y * arrow_size * 0.5;
    let arrow_p2_y = end_y - dir_y * arrow_size + dir_x * arrow_size * 0.5;

    // 绘制主线 (从起点到终点)
    let mut pb = tiny_skia::PathBuilder::new();
    pb.move_to(start_x, start_y);
    pb.line_to(end_x, end_y);
    let path = pb.finish().unwrap();
    pixmap.stroke_path(&path, paint, stroke, transform, None);

    // 绘制箭头头部 (从终点到两翼)
    let mut pb1 = tiny_skia::PathBuilder::new();
    pb1.move_to(end_x, end_y);
    pb1.line_to(arrow_p1_x, arrow_p1_y);
    let path1 = pb1.finish().unwrap();
    pixmap.stroke_path(&path1, paint, stroke, transform, None);

    let mut pb2 = tiny_skia::PathBuilder::new();
    pb2.move_to(end_x, end_y);
    pb2.line_to(arrow_p2_x, arrow_p2_y);
    let path2 = pb2.finish().unwrap();
    pixmap.stroke_path(&path2, paint, stroke, transform, None);
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

            match shape.tool {
                ScreenshotTool::Rect => {
                    if width <= 0.0 || height <= 0.0 { continue; }
                    if let Some(rect) = tiny_skia::Rect::from_xywh(x0, y0, width, height) {
                        let path = tiny_skia::PathBuilder::from_rect(rect);
                        pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                    }
                }
                ScreenshotTool::Circle => {
                    if width <= 0.0 || height <= 0.0 { continue; }
                    if let Some(rect) = tiny_skia::Rect::from_xywh(x0, y0, width, height) {
                        if let Some(path) = tiny_skia::PathBuilder::from_oval(rect) {
                            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                        }
                    }
                }
                ScreenshotTool::Arrow => {
                    draw_arrow_skia(&mut pixmap, start_x, start_y, end_x, end_y, &paint, &stroke);
                }
            }
        }
    }
}