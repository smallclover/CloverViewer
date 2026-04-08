use crate::feature::screenshot::canvas::mosaic::apply_mosaic_to_cropped_image;
use crate::feature::screenshot::capture::{DrawnShape, ScreenshotTool};
use ab_glyph::{FontRef, PxScale};
use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use image::{Rgba, RgbaImage};
use std::sync::LazyLock;

static EMBEDDED_FONT: LazyLock<FontRef<'static>> = LazyLock::new(|| {
    let data = include_bytes!("../../../assets/fonts/msyhl.ttf");
    FontRef::try_from_slice(data).expect("字体加载失败")
});

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
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(stroke_width, color),
                StrokeKind::Outside,
            );
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
        ScreenshotTool::Text | ScreenshotTool::Pen | ScreenshotTool::Mosaic => {}
    }
}

/// 绘制箭头 (Egui)
fn draw_arrow_egui(painter: &Painter, start: Pos2, end: Pos2, stroke_width: f32, color: Color32) {
    let stroke = Stroke::new(stroke_width, color);

    // 绘制主线
    painter.line_segment([start, end], stroke);

    // 计算箭头方向
    let dir = (end - start).normalized();
    if dir == Vec2::ZERO {
        return;
    }

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
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 {
        return;
    }

    let dir_x = dx / len;
    let dir_y = dy / len;
    let arrow_size = 12.0 + stroke.width * 2.0;

    let arrow_p1_x = end_x - dir_x * arrow_size + dir_y * arrow_size * 0.5;
    let arrow_p1_y = end_y - dir_y * arrow_size - dir_x * arrow_size * 0.5;
    let arrow_p2_x = end_x - dir_x * arrow_size - dir_y * arrow_size * 0.5;
    let arrow_p2_y = end_y - dir_y * arrow_size + dir_x * arrow_size * 0.5;

    let mut pb = tiny_skia::PathBuilder::new();
    pb.move_to(start_x, start_y);
    pb.line_to(end_x, end_y);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, transform, None);
    }

    let mut pb1 = tiny_skia::PathBuilder::new();
    pb1.move_to(end_x, end_y);
    pb1.line_to(arrow_p1_x, arrow_p1_y);
    if let Some(path1) = pb1.finish() {
        pixmap.stroke_path(&path1, paint, stroke, transform, None);
    }

    let mut pb2 = tiny_skia::PathBuilder::new();
    pb2.move_to(end_x, end_y);
    pb2.line_to(arrow_p2_x, arrow_p2_y);
    if let Some(path2) = pb2.finish() {
        pixmap.stroke_path(&path2, paint, stroke, transform, None);
    }
}

/// 导出图片时的抗锯齿高质量绘图 (最终合成)
pub fn draw_skia_shapes_on_image(
    final_image: &mut RgbaImage,
    shapes: &[DrawnShape],
    selection_phys: Rect,
) {
    let final_width = final_image.width();
    let final_height = final_image.height();

    for shape in shapes.iter().filter(|s| s.tool == ScreenshotTool::Mosaic) {
        if let Some(points) = &shape.points {
            apply_mosaic_to_cropped_image(final_image, points, shape.stroke_width, selection_phys);
        }
    }

    // ==========================================
    // 1. 使用 Tiny-Skia 渲染底层的几何图形
    // ==========================================
    if let Some(mut pixmap) =
        tiny_skia::PixmapMut::from_bytes(final_image, final_width, final_height)
    {
        for shape in shapes {
            if shape.tool == ScreenshotTool::Text || shape.tool == ScreenshotTool::Mosaic {
                continue;
            }

            let start_x = shape.start.x - selection_phys.min.x;
            let start_y = shape.start.y - selection_phys.min.y;
            let end_x = shape.end.x - selection_phys.min.x;
            let end_y = shape.end.y - selection_phys.min.y;

            let x0 = start_x.min(end_x);
            let y0 = start_y.min(end_y);
            let width = (start_x - end_x).abs();
            let height = (start_y - end_y).abs();

            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(
                shape.color.r(),
                shape.color.g(),
                shape.color.b(),
                shape.color.a(),
            );
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
                    if width <= 0.0 || height <= 0.0 {
                        continue;
                    }
                    if let Some(rect) = tiny_skia::Rect::from_xywh(x0, y0, width, height) {
                        let path = tiny_skia::PathBuilder::from_rect(rect);
                        pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                    }
                }
                ScreenshotTool::Circle => {
                    if width <= 0.0 || height <= 0.0 {
                        continue;
                    }
                    if let Some(rect) = tiny_skia::Rect::from_xywh(x0, y0, width, height) {
                        if let Some(path) = tiny_skia::PathBuilder::from_oval(rect) {
                            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                        }
                    }
                }
                ScreenshotTool::Arrow => {
                    draw_arrow_skia(&mut pixmap, start_x, start_y, end_x, end_y, &paint, &stroke);
                }
                ScreenshotTool::Pen => {
                    if let Some(points) = &shape.points {
                        if points.len() > 1 {
                            let mut pb = tiny_skia::PathBuilder::new();
                            pb.move_to(
                                points[0].x - selection_phys.min.x,
                                points[0].y - selection_phys.min.y,
                            );

                            for p in points.iter().skip(1) {
                                pb.line_to(p.x - selection_phys.min.x, p.y - selection_phys.min.y);
                            }
                            if let Some(path) = pb.finish() {
                                pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // ==========================================
    // 2. 使用 imageproc 渲染顶层的文本
    // ==========================================

    for shape in shapes {
        if shape.tool == ScreenshotTool::Text {
            if let Some(ref text) = shape.text {
                let start_x = shape.start.x - selection_phys.min.x;
                let start_y = shape.start.y - selection_phys.min.y;

                // 字体大小基准计算（乘以 1.5 是模拟一般的屏幕缩放系数，保持与 UI 视觉一致）
                let font_size = (20.0 + (shape.stroke_width * 2.0)) * 1.5;
                let scale = PxScale::from(font_size);

                // 行高计算 (基础高度 + 固定行距补偿)
                let line_height = font_size + 6.0;

                // 转换颜色
                let text_color = Rgba([
                    shape.color.r(),
                    shape.color.g(),
                    shape.color.b(),
                    shape.color.a(),
                ]);

                let mut current_y = start_y as f32;

                // 因为在 UI 层已经将排版”固化”成了带有 \n 的纯文本
                // 所以这里什么都不用测算，遇到 \n 就无脑换行！
                for line in text.split('\n') {
                    // 过滤可能残留的 Windows 回车符，避免打印出乱码小方块
                    let clean_line = line.trim_end_matches('\r');
                    imageproc::drawing::draw_text_mut(
                        final_image,
                        text_color,
                        start_x as i32,
                        current_y as i32,
                        scale,
                        &*EMBEDDED_FONT,
                        clean_line,
                    );
                    current_y += line_height;
                }
            }
        }
    }
}
