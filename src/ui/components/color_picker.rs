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
                let item_size = 22.0;
                let spacing = 8.0;
                let inner_margin = 8.0;
                let arrow_height = 8.0;
                let arrow_width = 12.0;
                let offset_y = 6.0;

                let num_colors = COLOR_OPTIONS.len() as f32;
                let content_width = (num_colors * item_size) + ((num_colors - 1.0) * spacing);
                let content_height = item_size;

                let box_width = content_width + inner_margin * 2.0;
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
                    .frame(egui::Frame::NONE)
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

                        // [细节] 让三角形底边稍微“插入”到矩形内部1像素，确保视觉无缝
                        let base_y = box_rect.top() + 1.0;
                        let base_left = Pos2::new(tip_x - arrow_width / 2.0, base_y);
                        let base_right = Pos2::new(tip_x + arrow_width / 2.0, base_y);

                        // 颜色设置：无边框模式下，背景色最好稍微深一点点以便区分
                        let bg_color = if ui.visuals().dark_mode {
                            Color32::from_gray(45)
                        } else {
                            Color32::from_gray(235) // 浅灰
                        };

                        // --- 绘制形状 (无边框 = Stroke::NONE) ---

                        // 1. 填充三角形
                        painter.add(Shape::convex_polygon(
                            vec![tip, base_right, base_left],
                            bg_color,
                            Stroke::NONE
                        ));

                        // 2. 填充圆角矩形 (覆盖在三角形底边之上)
                        // 圆角 5.0
                        painter.rect_filled(box_rect, 5.0, bg_color);

                        // 3. (可选) 如果你想要一点阴影效果来增加立体感，可以在最底层画一个模糊的黑色矩形
                        // 但为了严格遵守"无边框"，这里只做纯填充

                        // --- 绘制色块 ---
                        ui.scope_builder(UiBuilder::new().max_rect(box_rect.shrink(inner_margin)), |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                                for color in COLOR_OPTIONS.iter() {
                                    let (response_rect, response) = ui.allocate_exact_size(Vec2::splat(item_size), Sense::click());

                                    // 选中状态
                                    if *color == self.selected_color {
                                        ui.painter().rect_stroke(response_rect.expand(2.0), 4.0, Stroke::new(2.0, Color32::WHITE), StrokeKind::Outside);
                                        // 选中时的外圈描边
                                        ui.painter().rect_stroke(response_rect.expand(2.0), 4.0, Stroke::new(1.0, Color32::GRAY), StrokeKind::Outside);
                                    } else if *color == Color32::WHITE {
                                        // 白色块由于没有背景边框了，需要一个内描边
                                        ui.painter().rect_stroke(response_rect, 4.0, Stroke::new(1.0, Color32::from_gray(200)), StrokeKind::Inside);
                                    }

                                    ui.painter().rect_filled(response_rect, 4.0, *color);

                                    if response.clicked() {
                                        self.selected_color = *color;
                                        color_changed = true;
                                        // 点击不关闭，保持开启
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