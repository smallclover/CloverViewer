use eframe::egui::{Color32, Galley, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};
use std::sync::Arc;

use crate::feature::screenshot::{
    canvas::{
        GRAB_TOLERANCE_MAX, GRAB_TOLERANCE_MIN, HIT_TEST_RADIUS, MIN_SHAPE_SIZE, ResizeStartState,
        phys_to_local,
    },
    capture::{DrawnShape, ScreenshotTool},
    draw::draw_egui_shape,
};

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

    /// 返回该形状的控制点列表（本地坐标），以及对应的 hit radius
    fn resize_handles(&self, global_offset_phys: Pos2, ppp: f32) -> Vec<(Pos2, f32)>;

    /// 应用 resize：基于基准态、当前鼠标位置、handle 索引，更新 shape
    fn apply_resize(
        &mut self,
        handle: usize,
        current_phys: Pos2,
        start_state: &ResizeStartState,
        selection: Option<Rect>,
    );
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
            egui::FontId::proportional(font_size),
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
            egui::FontId::proportional(font_size),
            self.color,
        ))
    }
}

impl ShapeRender for DrawnShape {
    fn bounding_rect(&self, global_offset_phys: Pos2, ppp: f32) -> Rect {
        let start_local = phys_to_local(self.start, global_offset_phys, ppp);

        // --- 核心修改：文本框特殊处理 ---
        if self.tool == ScreenshotTool::Text {
            // 如果存在 galley 缓存，优先使用真实的文本排版尺寸作为包围盒，
            // 这样 8 个控制点就能完美贴合文字的实际边界！
            if let Some(galley) = &self.cached_galley {
                return Rect::from_min_size(start_local, galley.size());
            }

            // 降级处理：如果没有排版缓存（例如还没执行 render）
            let end_local = phys_to_local(self.end, global_offset_phys, ppp);
            let width = (end_local.x - start_local.x).abs();
            let height = (end_local.y - start_local.y).abs();
            return Rect::from_min_size(start_local, eframe::egui::vec2(width, height));
        }

        // 其他工具（Rect, Circle 等）的默认处理逻辑
        let end_local = phys_to_local(self.end, global_offset_phys, ppp);
        Rect::from_two_pos(start_local, end_local)
    }

    fn hit_test(&self, pos: Pos2, global_offset_phys: Pos2, ppp: f32, painter: &Painter) -> bool {
        let start_local = phys_to_local(self.start, global_offset_phys, ppp);
        let end_local = phys_to_local(self.end, global_offset_phys, ppp);
        let shape_rect = Rect::from_two_pos(start_local, end_local);
        let grab_tolerance =
            (self.stroke_width / ppp).clamp(GRAB_TOLERANCE_MIN, GRAB_TOLERANCE_MAX);

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
                    let r_theta = (a * b) / ((b * cos_t).powi(2) + (a * sin_t).powi(2)).sqrt();
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
            ScreenshotTool::Pen => {
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
            ScreenshotTool::Mosaic => false,
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
                draw_egui_shape(
                    painter,
                    self.tool,
                    rect,
                    start_local,
                    end_local,
                    self.stroke_width,
                    self.color,
                );
            }
        }
    }

    fn supports_resize(&self) -> bool {
        matches!(
            self.tool,
            ScreenshotTool::Rect
                | ScreenshotTool::Circle
                | ScreenshotTool::Arrow
                | ScreenshotTool::Text
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

    fn resize_handles(&self, global_offset_phys: Pos2, ppp: f32) -> Vec<(Pos2, f32)> {
        if !self.supports_resize() {
            return Vec::new();
        }

        let hit_radius = HIT_TEST_RADIUS; // 本地坐标下的命中半径（足够大以确保容易命中）

        match self.tool {
            ScreenshotTool::Arrow => {
                // 箭头只有起点和终点两个控制点
                // 直接使用 start 和 end 的本地坐标，而不是通过 bounding_rect 计算
                // 因为 bounding_rect 的 left_top/right_bottom 与 start/end 可能不对应
                let start_local = phys_to_local(self.start, global_offset_phys, ppp);
                let end_local = phys_to_local(self.end, global_offset_phys, ppp);
                vec![
                    (start_local, hit_radius), // 0: start
                    (end_local, hit_radius),   // 1: end
                ]
            }
            ScreenshotTool::Text => {
                // 【核心修改】文本工具只保留 4 个角的控制点
                let rect = self.bounding_rect(global_offset_phys, ppp);
                vec![
                    (rect.left_top(), hit_radius),     // 0: NW (左上)
                    (rect.right_top(), hit_radius),    // 1: NE (右上)
                    (rect.right_bottom(), hit_radius), // 2: SE (右下)
                    (rect.left_bottom(), hit_radius),  // 3: SW (左下)
                ]
            }
            _ => {
                // Rect, Circle, Text: 8 控制点
                //
                // 0 ─── 4 ─── 1
                // │           │
                // 7           5
                // │           │
                // 3 ─── 6 ─── 2
                let rect = self.bounding_rect(global_offset_phys, ppp);
                let center = rect.center();
                vec![
                    (rect.left_top(), hit_radius),                 // 0 NW
                    (rect.right_top(), hit_radius),                // 1 NE
                    (rect.right_bottom(), hit_radius),             // 2 SE
                    (rect.left_bottom(), hit_radius),              // 3 SW
                    (Pos2::new(center.x, rect.min.y), hit_radius), // 4 N
                    (Pos2::new(rect.max.x, center.y), hit_radius), // 5 E
                    (Pos2::new(center.x, rect.max.y), hit_radius), // 6 S
                    (Pos2::new(rect.min.x, center.y), hit_radius), // 7 W
                ]
            }
        }
    }

    fn apply_resize(
        &mut self,
        handle: usize,
        current_phys: Pos2,
        start_state: &ResizeStartState,
        selection: Option<Rect>,
    ) {
        let clamped = clamp_pos_to_rect(current_phys, selection.unwrap_or(Rect::EVERYTHING));
        let (new_start, new_end) = resized_endpoints(self.tool, handle, clamped, start_state);

        if !is_valid_resize(self.tool, new_start, new_end) {
            return;
        }

        if self.tool == ScreenshotTool::Text {
            apply_text_resize(self, new_start, new_end);
        } else {
            self.start = new_start;
            self.end = new_end;
        }
    }
}

fn resized_endpoints(
    tool: ScreenshotTool,
    handle: usize,
    clamped: Pos2,
    start_state: &ResizeStartState,
) -> (Pos2, Pos2) {
    let mut new_start = start_state.start;
    let mut new_end = start_state.end;

    match tool {
        ScreenshotTool::Arrow => match handle {
            0 => new_start = clamped,
            1 => new_end = clamped,
            _ => {}
        },
        _ => match handle {
            0 => {
                new_start = clamped;
                new_end = start_state.end;
            }
            1 => {
                new_start = Pos2::new(start_state.start.x, clamped.y);
                new_end = Pos2::new(clamped.x, start_state.end.y);
            }
            2 => {
                new_start = start_state.start;
                new_end = clamped;
            }
            3 => {
                new_start = Pos2::new(clamped.x, start_state.start.y);
                new_end = Pos2::new(start_state.end.x, clamped.y);
            }
            4 => {
                new_start = Pos2::new(start_state.start.x, clamped.y);
                new_end = start_state.end;
            }
            5 => {
                new_start = start_state.start;
                new_end = Pos2::new(clamped.x, start_state.end.y);
            }
            6 => {
                new_start = start_state.start;
                new_end = Pos2::new(start_state.end.x, clamped.y);
            }
            7 => {
                new_start = Pos2::new(clamped.x, start_state.start.y);
                new_end = start_state.end;
            }
            _ => {}
        },
    }

    (new_start, new_end)
}

fn is_valid_resize(tool: ScreenshotTool, new_start: Pos2, new_end: Pos2) -> bool {
    let width = (new_end.x - new_start.x).abs();
    let height = (new_end.y - new_start.y).abs();

    if tool == ScreenshotTool::Text {
        width >= 10.0 && height >= MIN_SHAPE_SIZE
    } else {
        width >= MIN_SHAPE_SIZE && height >= MIN_SHAPE_SIZE
    }
}

fn apply_text_resize(shape: &mut DrawnShape, new_start: Pos2, new_end: Pos2) {
    let prev_w = (shape.end.x - shape.start.x).abs();
    let prev_h = (shape.end.y - shape.start.y).abs();

    if prev_w > 1.0 {
        let new_w = (new_end.x - new_start.x).abs();
        let ratio = new_w / prev_w;
        let stroke_width_before = shape.stroke_width;
        let mut stroke_width_after = ratio * (10.0 + stroke_width_before) - 10.0;
        stroke_width_after = stroke_width_after.clamp(1.0, 48.0);
        shape.stroke_width = stroke_width_after;

        let actual_ratio = (10.0 + stroke_width_after) / (10.0 + stroke_width_before);
        let actual_new_w = prev_w * actual_ratio;
        let actual_new_h = prev_h * actual_ratio;
        let sign_x = (new_end.x - new_start.x).signum();
        let sign_y = (new_end.y - new_start.y).signum();

        shape.start = new_start;
        shape.end = Pos2::new(
            new_start.x + actual_new_w * sign_x,
            new_start.y + actual_new_h * sign_y,
        );
    } else {
        shape.start = new_start;
        shape.end = new_end;
    }

    shape.invalidate_galley();
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
