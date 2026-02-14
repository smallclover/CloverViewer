use eframe::egui;
use eframe::epaint::StrokeKind;
use egui::{Color32, Sense, Pos2, Rect, Shape, Stroke, Vec2, UiBuilder};

const COLOR_OPTIONS: [Color32; 8] = [
    Color32::from_rgb(204, 0, 0),       // Red
    Color32::from_rgb(255, 128, 0),     // Orange
    Color32::from_rgb(255, 255, 0),     // Yellow
    Color32::from_rgb(0, 204, 0),       // Green
    Color32::from_rgb(0, 0, 255),       // Blue
    Color32::from_rgb(128, 0, 128),     // Purple
    Color32::from_rgb(0, 0, 0),         // Black
    Color32::from_rgb(255, 255, 255),   // White
];

#[derive(Clone, PartialEq)]
pub struct ColorPicker {
    pub selected_color: Color32,
    pub is_open: bool,
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self {
            selected_color: Color32::from_rgb(204, 0, 0),
            is_open: false,
        }
    }
}

impl ColorPicker {
    pub fn new(selected_color: Color32) -> Self {
        Self {
            selected_color,
            is_open: false,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, anchor: Option<Rect>) -> bool {
        let mut color_changed = false;

        if self.is_open {
            if let Some(anchor_rect) = anchor {
                let ctx = ui.ctx();
                let screen_rect = ctx.content_rect();

                // --- 参数配置 ---
                let item_size = 22.0;       // 色块大小
                let spacing = 8.0;          // 色块间距
                let inner_margin = 8.0;     // 气泡内边距
                let arrow_height = 8.0;     // 三角形高度
                let arrow_width = 12.0;     // 三角形宽度
                let offset_y = 6.0;         // 气泡尖端距离按钮底部的距离

                // 计算内容区域大小 (只有色块的部分)
                let num_colors = COLOR_OPTIONS.len() as f32;
                let content_width = (num_colors * item_size) + ((num_colors - 1.0) * spacing);
                let content_height = item_size;

                // 气泡主矩形的大小 (内容 + 内边距)
                let box_width = content_width + inner_margin * 2.0;
                let box_height = content_height + inner_margin * 2.0;

                // [关键修改] 窗口总大小 = 气泡矩形 + 三角形高度
                // 这样窗口就包含了三角形，不会被裁切或遮挡
                let window_width = box_width;
                let window_height = box_height + arrow_height;

                // [关键修改] 窗口位置
                // Y: 按钮底部 + 间距 (这就是三角形尖端的位置，也是窗口的起始Y)
                let target_y = anchor_rect.bottom() + offset_y;
                let target_x = anchor_rect.center().x - window_width / 2.0;

                let mut window_pos = Pos2::new(target_x, target_y);

                // 屏幕边缘限制
                if window_pos.x < screen_rect.min.x + 5.0 { window_pos.x = screen_rect.min.x + 5.0; }
                if window_pos.x + window_width > screen_rect.max.x - 5.0 { window_pos.x = screen_rect.max.x - 5.0 - window_width; }
                if window_pos.y + window_height > screen_rect.max.y - 5.0 { window_pos.y = screen_rect.max.y - 5.0 - window_height; }


                egui::Window::new("ColorPicker")
                    .fixed_pos(window_pos)
                    .fixed_size(Vec2::new(window_width, window_height))
                    .frame(egui::Frame::NONE) // 无边框，手动绘制
                    .title_bar(false)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {

                        let painter = ui.painter();
                        // 窗口的完整区域
                        let rect = Rect::from_min_size(window_pos, Vec2::new(window_width, window_height));

                        // [关键] 定义两个区域：三角形区域 和 主矩形区域
                        // 1. 主矩形区域：从 (Top + arrow_height) 开始
                        let box_rect = Rect::from_min_size(
                            Pos2::new(rect.left(), rect.top() + arrow_height),
                            Vec2::new(box_width, box_height)
                        );

                        // 2. 三角形尖端：位于 rect.top()
                        // 必须基于 anchor 的中心计算 X，并限制在窗口范围内
                        let tip_x = anchor_rect.center().x.clamp(box_rect.left() + 10.0, box_rect.right() - 10.0);
                        let tip = Pos2::new(tip_x, rect.top()); // 尖端在窗口最顶端

                        let base_y = box_rect.top(); // 底边在主矩形的顶端
                        let base_left = Pos2::new(tip_x - arrow_width / 2.0, base_y);
                        let base_right = Pos2::new(tip_x + arrow_width / 2.0, base_y);


                        // --- 颜色设置 ---
                        let bg_color = if ui.visuals().dark_mode {
                            Color32::from_gray(45)
                        } else {
                            Color32::from_gray(240) // 浅灰，比白色深一点
                        };
                        let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                        let stroke = Stroke::new(1.0, stroke_color);

                        // --- 绘制形状 ---

                        // A. 填充主矩形
                        painter.rect_filled(box_rect, 4.0, bg_color);

                        // B. 填充三角形
                        painter.add(Shape::convex_polygon(
                            vec![tip, base_right, base_left],
                            bg_color,
                            Stroke::NONE
                        ));

                        // C. 绘制主矩形描边
                        painter.rect_stroke(box_rect, 4.0, stroke, StrokeKind::Inside);

                        // D. 融合处理：用背景色覆盖矩形顶部的描边
                        // 覆盖范围是三角形的底边
                        painter.line_segment([base_left, base_right], Stroke::new(2.0, bg_color));

                        // E. 补上三角形的斜边描边
                        painter.line_segment([base_left, tip], stroke);
                        painter.line_segment([base_right, tip], stroke);

                        // --- 绘制色块内容 ---
                        // 注意：要在 box_rect 内部进行 UI 分配
                        ui.scope_builder(UiBuilder::new().max_rect(box_rect.shrink(inner_margin)), |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                                for color in COLOR_OPTIONS.iter() {
                                    let (response_rect, response) = ui.allocate_exact_size(
                                        Vec2::splat(item_size),
                                        Sense::click()
                                    );

                                    // 选中状态
                                    if *color == self.selected_color {
                                        ui.painter().rect_stroke(
                                            response_rect.expand(2.0),
                                            4.0,
                                            Stroke::new(2.0, Color32::WHITE),
                                            StrokeKind::Outside
                                        );
                                        ui.painter().rect_stroke(
                                            response_rect.expand(2.0),
                                            4.0,
                                            Stroke::new(1.0, Color32::GRAY),
                                            StrokeKind::Outside
                                        );
                                    } else if *color == Color32::WHITE {
                                        ui.painter().rect_stroke(
                                            response_rect,
                                            4.0,
                                            Stroke::new(1.0, Color32::from_gray(200)),
                                            StrokeKind::Inside
                                        );
                                    }

                                    ui.painter().rect_filled(response_rect, 4.0, *color);

                                    if response.clicked() {
                                        self.selected_color = *color;
                                        color_changed = true;
                                        self.close();
                                    }
                                }
                            });
                        });

                        // 点击外部关闭
                        if ui.input(|i| i.pointer.any_click()) {
                            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                // 如果点击既不在窗口内，也不在按钮内，则关闭
                                if !rect.contains(pos) && !anchor_rect.contains(pos) {
                                    self.close();
                                }
                            }
                        }
                    });
            }
        }
        color_changed
    }
}