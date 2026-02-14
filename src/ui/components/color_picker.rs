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

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
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
                        let base_y = box_rect.top();
                        let base_left = Pos2::new(tip_x - arrow_width / 2.0, base_y);
                        let base_right = Pos2::new(tip_x + arrow_width / 2.0, base_y);

                        let bg_color = if ui.visuals().dark_mode { Color32::from_gray(45) } else { Color32::from_gray(240) };
                        let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                        let stroke = Stroke::new(1.0, stroke_color);

                        painter.rect_filled(box_rect, 4.0, bg_color);
                        painter.add(Shape::convex_polygon(vec![tip, base_right, base_left], bg_color, Stroke::NONE));
                        painter.rect_stroke(box_rect, 4.0, stroke, StrokeKind::Inside);
                        painter.line_segment([base_left, base_right], Stroke::new(2.0, bg_color));
                        painter.line_segment([base_left, tip], stroke);
                        painter.line_segment([base_right, tip], stroke);

                        ui.scope_builder(UiBuilder::new().max_rect(box_rect.shrink(inner_margin)), |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                                for color in COLOR_OPTIONS.iter() {
                                    let (response_rect, response) = ui.allocate_exact_size(Vec2::splat(item_size), Sense::click());

                                    if *color == self.selected_color {
                                        ui.painter().rect_stroke(response_rect.expand(2.0), 4.0, Stroke::new(2.0, Color32::WHITE), StrokeKind::Outside);
                                        ui.painter().rect_stroke(response_rect.expand(2.0), 4.0, Stroke::new(1.0, Color32::GRAY), StrokeKind::Outside);
                                    } else if *color == Color32::WHITE {
                                        ui.painter().rect_stroke(response_rect, 4.0, Stroke::new(1.0, Color32::from_gray(200)), StrokeKind::Inside);
                                    }

                                    ui.painter().rect_filled(response_rect, 4.0, *color);

                                    if response.clicked() {
                                        self.selected_color = *color;
                                        color_changed = true;
                                        // [修改] 移除 self.close()，点击颜色不关闭
                                    }
                                }
                            });
                        });

                        // [修改] 移除了“点击外部关闭”的逻辑
                        // 仅保留 ESC 键作为兜底关闭
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.close();
                        }
                    });
            }
        }
        color_changed
    }
}