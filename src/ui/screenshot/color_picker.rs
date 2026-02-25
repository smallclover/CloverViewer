use eframe::egui;
use eframe::epaint::StrokeKind;
use egui::{Color32, Sense, Pos2, Rect, Shape, Stroke, Vec2, UiBuilder, Frame};
//默认颜色
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
// 默认线条粗细
const WIDTH_OPTIONS: [f32; 3] = [2.0, 4.0, 8.0];

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

    pub fn show(&mut self, ui: &mut egui::Ui, anchor: Option<Rect>, current_width: &mut f32) -> bool {
        let mut color_changed = false;

        if self.is_open {
            if let Some(anchor_rect) = anchor {
                let ctx = ui.ctx();
                let screen_rect = ctx.content_rect();

                // --- 参数配置 ---
                let item_size = 22.0;
                let spacing = 8.0;
                let inner_margin = 8.0;
                let arrow_height = 8.0;
                let arrow_width = 12.0;
                let offset_y = 6.0;
                let separator_width = 1.0;

                // --- 宽度计算 ---
                // 总控件数 = 3个粗细按钮 + 1个分隔符 + 8个颜色按钮 = 12
                // 总间距数 = 11 个 spacing

                let total_items_width = (3.0 * item_size) + separator_width + (8.0 * item_size);
                let total_spacing_width = 11.0 * spacing;

                // 内容总宽度
                let content_width = total_items_width + total_spacing_width;
                let content_height = item_size;

                // 加上内边距
                let box_width = content_width + inner_margin * 2.0 + 2.0; // 多加2.0作为安全余量
                let box_height = content_height + inner_margin * 2.0;
                let window_width = box_width;
                let window_height = box_height + arrow_height;

                let target_y = anchor_rect.bottom() + offset_y;
                let target_x = anchor_rect.center().x - window_width / 2.0;

                let mut window_pos = Pos2::new(target_x, target_y);

                if window_pos.x < screen_rect.min.x + 5.0 { window_pos.x = screen_rect.min.x + 5.0; }
                if window_pos.x + window_width > screen_rect.max.x - 5.0 { window_pos.x = screen_rect.max.x - 5.0 - window_width; }
                if window_pos.y + window_height > screen_rect.max.y - 5.0 { window_pos.y = screen_rect.max.y - 5.0 - window_height; }

                egui::Window::new("ColorPicker")
                    .fixed_pos(window_pos)
                    .fixed_size(Vec2::new(window_width, window_height))
                    .frame(Frame::NONE)
                    .title_bar(false)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        let painter = ui.painter();
                        let rect = Rect::from_min_size(window_pos, Vec2::new(window_width, window_height));

                        let box_rect = Rect::from_min_size(
                            Pos2::new(rect.left(), rect.top() + arrow_height),
                            Vec2::new(box_width, box_height)
                        );

                        let tip_x = anchor_rect.center().x.clamp(box_rect.left() + 10.0, box_rect.right() - 10.0);
                        let tip = Pos2::new(tip_x, rect.top());
                        let base_y = box_rect.top() + 1.0;
                        let base_left = Pos2::new(tip_x - arrow_width / 2.0, base_y);
                        let base_right = Pos2::new(tip_x + arrow_width / 2.0, base_y);

                        let bg_color = if ui.visuals().dark_mode { Color32::from_gray(45) } else { Color32::from_gray(235) };

                        // 绘制气泡背景
                        painter.add(Shape::convex_polygon(vec![tip, base_right, base_left], bg_color, Stroke::NONE));
                        painter.rect_filled(box_rect, 5.0, bg_color);

                        // 绘制内容
                        ui.scope_builder(UiBuilder::new().max_rect(box_rect.shrink(inner_margin)), |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                                // 1. 粗细选择
                                for width in WIDTH_OPTIONS.iter() {
                                    let (response_rect, response) = ui.allocate_exact_size(Vec2::splat(item_size), Sense::click());

                                    let center = response_rect.center();
                                    let visual_radius = match *width {
                                        2.0 => 3.5,
                                        4.0 => 5.5,
                                        _ => 8.0,
                                    };

                                    // 选中外圈
                                    if (*width - *current_width).abs() < 0.001 {
                                        ui.painter().circle_stroke(center, visual_radius + 3.0, Stroke::new(1.0, Color32::GRAY));
                                    }

                                    // 实心圆点
                                    ui.painter().circle_filled(center, visual_radius, Color32::from_gray(100));

                                    if response.clicked() {
                                        *current_width = *width;
                                        color_changed = true;
                                    }
                                }

                                // 2. 分割线 (egui 会自动在它前后加上 item_spacing)
                                let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(separator_width, item_size), Sense::hover());
                                ui.painter().line_segment(
                                    [sep_rect.center_top() + Vec2::new(0.0, 2.0), sep_rect.center_bottom() - Vec2::new(0.0, 2.0)],
                                    Stroke::new(1.0, Color32::from_gray(180))
                                );

                                // 3. 颜色选择
                                for color in COLOR_OPTIONS.iter() {
                                    let (response_rect, response) = ui.allocate_exact_size(Vec2::splat(item_size), Sense::click());

                                    if *color == self.selected_color {
                                        ui.painter().rect_stroke(response_rect.expand(2.0), 4.0, Stroke::new(2.0, Color32::WHITE), StrokeKind::Outside);
                                        ui.painter().rect_stroke(response_rect.expand(2.0), 4.0, Stroke::new(1.0, Color32::GRAY), StrokeKind::Outside);
                                    } else if *color == Color32::WHITE {
                                        // 未选中时的白色色块：使用标准灰色边框，与背景区分更明显
                                        ui.painter().rect_stroke(
                                            response_rect,
                                            4.0,
                                            Stroke::new(1.0, Color32::GRAY), // 这里改成 Color32::GRAY (128) 或者 from_gray(100)
                                            StrokeKind::Inside
                                        );
                                    }

                                    ui.painter().rect_filled(response_rect, 4.0, *color);

                                    if response.clicked() {
                                        self.selected_color = *color;
                                        color_changed = true;
                                    }
                                }
                            });
                        });

                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.close();
                        }
                    });
            }
        }
        color_changed
    }
}