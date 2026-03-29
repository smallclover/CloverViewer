use std::sync::Arc;

use eframe::egui::{Color32, Galley, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::feature::screenshot::capture::{DrawnShape, ScreenshotTool};
use crate::feature::screenshot::draw::draw_egui_shape;

/// Shape 渲染与交互能力接口
pub trait ShapeRender {
    /// 本地坐标系下的包围盒
    fn bounding_rect(&self, global_offset_phys: Pos2, ppp: f32) -> Rect;

    /// 命中测试
    fn hit_test(&self, pos: Pos2, global_offset_phys: Pos2, ppp: f32, painter: &Painter) -> bool;

    /// 渲染
    fn render(&self, painter: &Painter, global_offset_phys: Pos2, ppp: f32, is_hovered: bool);

    /// 支持 resize handles 吗？
    fn supports_resize(&self) -> bool {
        false
    }

    /// 应用移动偏移
    fn translate(&mut self, delta: Vec2);

    /// 创建深拷贝
    fn clone_shape(&self) -> Self
    where
        Self: Sized;
}

impl DrawnShape {
    /// 获取或创建文本的 Galley 缓存
    pub fn ensure_galley(&mut self, painter: &Painter) -> Option<Arc<Galley>> {
        if let Some(ref g) = self.cached_galley {
            return Some(g.clone());
        }
        let text = self.text.as_ref()?;
        let font_size = 20.0 + (self.stroke_width * 2.0);
        let galley = painter.layout_no_wrap(
            text.clone(),
            eframe::egui::FontId::proportional(font_size),
            self.color,
        );
        self.cached_galley = Some(galley.clone());
        Some(galley)
    }

    /// 使文本缓存失效
    pub fn invalidate_galley(&mut self) {
        self.cached_galley = None;
    }

    /// 无缓存的情况下布局文本（用于 hit_test）
    fn layout_text_galley(&self, painter: &Painter) -> Option<Arc<Galley>> {
        let text = self.text.as_ref()?;
        let font_size = 20.0 + (self.stroke_width * 2.0);
        Some(painter.layout_no_wrap(
            text.clone(),
            eframe::egui::FontId::proportional(font_size),
            self.color,
        ))
    }
}

impl ShapeRender for DrawnShape {
    fn bounding_rect(&self, global_offset_phys: Pos2, ppp: f32) -> Rect {
        let start_local = phys_to_local(self.start, global_offset_phys, ppp);
        let end_local = phys_to_local(self.end, global_offset_phys, ppp);
        Rect::from_two_pos(start_local, end_local)
    }

    fn hit_test(&self, pos: Pos2, global_offset_phys: Pos2, ppp: f32, painter: &Painter) -> bool {
        let start_local = phys_to_local(self.start, global_offset_phys, ppp);
        let end_local = phys_to_local(self.end, global_offset_phys, ppp);
        let shape_rect = Rect::from_two_pos(start_local, end_local);
        let grab_tolerance = (self.stroke_width / ppp).clamp(4.0, 8.0);

        match self.tool {
            ScreenshotTool::Rect => {
                let expanded = shape_rect.expand(grab_tolerance);
                let shrunk = shape_rect.shrink(grab_tolerance);
                expanded.contains(pos) && (!shrunk.is_positive() || !shrunk.contains(pos))
            }
            ScreenshotTool::Circle => {
                let center = shape_rect.center();
                let a = shape_rect.width() / 2.0;
                let b = shape_rect.height() / 2.0;
                let dx = pos.x - center.x;
                let dy = pos.y - center.y;
                let dist = pos.distance(center);

                if dist < 0.1 || a < 0.1 || b < 0.1 {
                    false
                } else {
                    let cos_t = dx / dist;
                    let sin_t = dy / dist;
                    let r_theta =
                        (a * b) / ((b * cos_t).powi(2) + (a * sin_t).powi(2)).sqrt();
                    (dist - r_theta).abs() <= grab_tolerance
                }
            }
            ScreenshotTool::Arrow => {
                dist_to_line_segment(pos, start_local, end_local) <= grab_tolerance
            }
            ScreenshotTool::Text => {
                if let Some(galley) = self.layout_text_galley(painter) {
                    let text_rect = Rect::from_min_size(start_local, galley.size());
                    text_rect.expand(4.0).contains(pos)
                } else {
                    false
                }
            }
            ScreenshotTool::Pen | ScreenshotTool::Mosaic => {
                if let Some(points) = &self.points {
                    for i in 0..points.len().saturating_sub(1) {
                        let p1 = phys_to_local(points[i], global_offset_phys, ppp);
                        let p2 = phys_to_local(points[i + 1], global_offset_phys, ppp);
                        if dist_to_line_segment(pos, p1, p2) <= grab_tolerance {
                            return true;
                        }
                    }
                    false
                } else {
                    false
                }
            }
        }
    }

    fn render(&self, painter: &Painter, global_offset_phys: Pos2, ppp: f32, is_hovered: bool) {
        let start_local = phys_to_local(self.start, global_offset_phys, ppp);
        let end_local = phys_to_local(self.end, global_offset_phys, ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        if is_hovered {
            let highlight_rect = if self.tool == ScreenshotTool::Text {
                if let Some(galley) = self.layout_text_galley(painter) {
                    Rect::from_min_size(start_local, galley.size())
                } else {
                    rect
                }
            } else {
                rect
            };
            painter.rect_stroke(
                highlight_rect.expand(2.0),
                2.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 150, 255, 100)),
                StrokeKind::Outside,
            );
        }

        match self.tool {
            ScreenshotTool::Text => {
                // 注意：render 时传入的 &self 是不可变引用，但 ensure_galley 需要 &mut self
                // 这里我们要求调用方先调用 ensure_galley 缓存 galley，然后传入 painter 仅用于渲染
                // 为简化 API，Text shape 的渲染特殊处理：直接使用 layout_text_galley
                if let Some(galley) = self.layout_text_galley(painter) {
                    painter.galley(start_local, galley, self.color);
                }
            }
            ScreenshotTool::Pen => {
                if let Some(points) = &self.points {
                    let mut local_points = Vec::with_capacity(points.len());
                    for p in points {
                        local_points.push(phys_to_local(*p, global_offset_phys, ppp));
                    }
                    painter.add(eframe::egui::Shape::line(
                        local_points,
                        Stroke::new(self.stroke_width, self.color),
                    ));
                }
            }
            ScreenshotTool::Mosaic => {
                // 马赛克在 render.rs 中特殊处理，因为需要访问 captures 采样原图
            }
            _ => {
                draw_egui_shape(painter, self.tool, rect, start_local, end_local, self.stroke_width, self.color);
            }
        }
    }

    fn supports_resize(&self) -> bool {
        matches!(
            self.tool,
            ScreenshotTool::Rect | ScreenshotTool::Circle | ScreenshotTool::Arrow | ScreenshotTool::Text
        )
    }

    fn translate(&mut self, delta: Vec2) {
        self.start += delta;
        self.end += delta;
        self.invalidate_galley();
        if let Some(points) = &mut self.points {
            for p in points.iter_mut() {
                *p += delta;
            }
        }
    }

    fn clone_shape(&self) -> Self {
        self.clone()
    }
}

/// 物理坐标转换为本地逻辑坐标
fn phys_to_local(pos: Pos2, global_offset_phys: Pos2, ppp: f32) -> Pos2 {
    Pos2::ZERO + ((pos - global_offset_phys) / ppp)
}

/// 点到线段的距离
pub fn dist_to_line_segment(p: Pos2, v: Pos2, w: Pos2) -> f32 {
    let l2 = v.distance_sq(w);
    if l2 == 0.0 {
        return p.distance(v);
    }
    let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let projection = Pos2::new(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y));
    p.distance(projection)
}

/// 将位置限制在矩形内
pub fn clamp_pos_to_rect(pos: Pos2, rect: Rect) -> Pos2 {
    Pos2::new(
        pos.x.clamp(rect.min.x, rect.max.x),
        pos.y.clamp(rect.min.y, rect.max.y),
    )
}
