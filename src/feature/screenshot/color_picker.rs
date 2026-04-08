use eframe::egui;
use eframe::epaint::StrokeKind;
use egui::{Color32, Frame, Pos2, Rect, Sense, Shape, Stroke, UiBuilder, Vec2};

const COLOR_OPTIONS: [Color32; 8] = [
    Color32::from_rgb(204, 0, 0),     // Red
    Color32::from_rgb(255, 128, 0),   // Orange
    Color32::from_rgb(255, 255, 0),   // Yellow
    Color32::from_rgb(0, 204, 0),     // Green
    Color32::from_rgb(0, 0, 255),     // Blue
    Color32::from_rgb(128, 0, 128),   // Purple
    Color32::from_rgb(0, 0, 0),       // Black
    Color32::from_rgb(255, 255, 255), // White
];
const WIDTH_OPTIONS: [f32; 3] = [2.0, 4.0, 8.0];
// 马赛克专属粗细（因为马赛克本身就需要大块覆盖）
const MOSAIC_WIDTH_OPTIONS: [f32; 3] = [8.0, 16.0, 24.0];

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

    // 【修改】加入 show_colors 参数
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        anchor: Option<Rect>,
        current_width: &mut f32,
        show_colors: bool,
    ) -> bool {
        let mut color_changed = false;

        if self.is_open {
            if let Some(anchor_rect) = anchor {
                let ctx = ui.ctx();
                let screen_rect = ctx.content_rect();

                let item_size = 22.0;
                let spacing = 8.0;
                let inner_margin = 8.0;
                let arrow_height = 8.0;
                let arrow_width = 12.0;
                let offset_y = 6.0;
                let separator_width = 1.0;

                // 【修改】根据是否展示颜色，动态缩减面板宽度
                let total_items_width = if show_colors {
                    (3.0 * item_size) + separator_width + (8.0 * item_size)
                } else {
                    3.0 * item_size
                };
                let total_spacing_width = if show_colors {
                    11.0 * spacing
                } else {
                    2.0 * spacing
                };

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
                if window_pos.x < screen_rect.min.x + 5.0 {
                    window_pos.x = screen_rect.min.x + 5.0;
                }
                if window_pos.x + window_width > screen_rect.max.x - 5.0 {
                    window_pos.x = screen_rect.max.x - 5.0 - window_width;
                }
                if window_pos.y + window_height > screen_rect.max.y - 5.0 {
                    window_pos.y = screen_rect.max.y - 5.0 - window_height;
                }

                egui::Window::new("ColorPicker")
                    .fixed_pos(window_pos)
                    .fixed_size(Vec2::new(window_width, window_height))
                    .frame(Frame::NONE)
                    .title_bar(false)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        let painter = ui.painter();
                        let rect =
                            Rect::from_min_size(window_pos, Vec2::new(window_width, window_height));

                        let box_rect = Rect::from_min_size(
                            Pos2::new(rect.left(), rect.top() + arrow_height),
                            Vec2::new(box_width, box_height),
                        );

                        let tip_x = anchor_rect
                            .center()
                            .x
                            .clamp(box_rect.left() + 10.0, box_rect.right() - 10.0);
                        let tip = Pos2::new(tip_x, rect.top());
                        let base_y = box_rect.top() + 1.0;
                        let base_left = Pos2::new(tip_x - arrow_width / 2.0, base_y);
                        let base_right = Pos2::new(tip_x + arrow_width / 2.0, base_y);

                        let bg_color = if ui.visuals().dark_mode {
                            Color32::from_gray(45)
                        } else {
                            Color32::from_gray(235)
                        };

                        // 绘制气泡背景
                        painter.add(Shape::convex_polygon(
                            vec![tip, base_right, base_left],
                            bg_color,
                            Stroke::NONE,
                        ));
                        painter.rect_filled(box_rect, 5.0, bg_color);

                        // 绘制内容
                        ui.scope_builder(
                            UiBuilder::new().max_rect(box_rect.shrink(inner_margin)),
                            |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                                    // 1. 粗细选择 (如果是马赛克就用巨大的粗细套件)
                                    let widths_to_use = if show_colors {
                                        &WIDTH_OPTIONS
                                    } else {
                                        &MOSAIC_WIDTH_OPTIONS
                                    };

                                    for width in widths_to_use.iter() {
                                        let (response_rect, response) = ui.allocate_exact_size(
                                            Vec2::splat(item_size),
                                            Sense::click(),
                                        );

                                        let center = response_rect.center();
                                        // 视觉半径等比缩放
                                        let visual_radius = if show_colors {
                                            match *width {
                                                2.0 => 3.5,
                                                4.0 => 5.5,
                                                _ => 8.0,
                                            }
                                        } else {
                                            match *width {
                                                8.0 => 4.0,
                                                16.0 => 6.0,
                                                _ => 8.0,
                                            }
                                        };

                                        if (*width - *current_width).abs() < 0.001 {
                                            ui.painter().circle_stroke(
                                                center,
                                                visual_radius + 3.0,
                                                Stroke::new(1.0, Color32::GRAY),
                                            );
                                        }

                                        ui.painter().circle_filled(
                                            center,
                                            visual_radius,
                                            Color32::from_gray(100),
                                        );

                                        if response.clicked() {
                                            *current_width = *width;
                                            color_changed = true;
                                        }
                                    }

                                    // 【修改】只有普通画笔才绘制调色盘
                                    if show_colors {
                                        // 2. 分割线
                                        let (sep_rect, _) = ui.allocate_exact_size(
                                            Vec2::new(separator_width, item_size),
                                            Sense::hover(),
                                        );
                                        ui.painter().line_segment(
                                            [
                                                sep_rect.center_top() + Vec2::new(0.0, 2.0),
                                                sep_rect.center_bottom() - Vec2::new(0.0, 2.0),
                                            ],
                                            Stroke::new(1.0, Color32::from_gray(180)),
                                        );

                                        // 3. 颜色选择
                                        for color in COLOR_OPTIONS.iter() {
                                            let (response_rect, response) = ui.allocate_exact_size(
                                                Vec2::splat(item_size),
                                                Sense::click(),
                                            );

                                            if *color == self.selected_color {
                                                ui.painter().rect_stroke(
                                                    response_rect.expand(2.0),
                                                    4.0,
                                                    Stroke::new(2.0, Color32::WHITE),
                                                    StrokeKind::Outside,
                                                );
                                                ui.painter().rect_stroke(
                                                    response_rect.expand(2.0),
                                                    4.0,
                                                    Stroke::new(1.0, Color32::GRAY),
                                                    StrokeKind::Outside,
                                                );
                                            } else if *color == Color32::WHITE {
                                                ui.painter().rect_stroke(
                                                    response_rect,
                                                    4.0,
                                                    Stroke::new(1.0, Color32::GRAY),
                                                    StrokeKind::Inside,
                                                );
                                            }

                                            ui.painter().rect_filled(response_rect, 4.0, *color);

                                            if response.clicked() {
                                                self.selected_color = *color;
                                                color_changed = true;
                                            }
                                        }
                                    }
                                });
                            },
                        );

                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.close();
                        }
                    });
            }
        }
        color_changed
    }
}
