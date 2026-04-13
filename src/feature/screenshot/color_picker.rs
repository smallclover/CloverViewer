use eframe::egui;
use eframe::epaint::StrokeKind;
use egui::{Color32, Frame, Pos2, Rect, Sense, Shape, Stroke, UiBuilder, Vec2};

const PICKER_ITEM_SIZE: f32 = 22.0;
const PICKER_SPACING: f32 = 8.0;
const PICKER_INNER_MARGIN: f32 = 8.0;
const PICKER_ARROW_HEIGHT: f32 = 8.0;
const PICKER_ARROW_WIDTH: f32 = 12.0;
const PICKER_OFFSET_Y: f32 = 6.0;
const PICKER_SEPARATOR_WIDTH: f32 = 1.0;
const PICKER_SCREEN_PADDING: f32 = 5.0;
const PICKER_BOX_WIDTH_SAFETY_MARGIN: f32 = 2.0;

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

struct PopupLayout {
    window_pos: Pos2,
    window_size: Vec2,
    box_rect: Rect,
    tip: Pos2,
    base_left: Pos2,
    base_right: Pos2,
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

        if let Some(anchor_rect) = anchor.filter(|_| self.is_open) {
            let layout = calculate_popup_layout(anchor_rect, ui.content_rect(), show_colors);

            egui::Window::new("ColorPicker")
                .fixed_pos(layout.window_pos)
                .fixed_size(layout.window_size)
                .frame(Frame::NONE)
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    draw_popup_background(ui, &layout);

                    ui.scope_builder(
                        UiBuilder::new().max_rect(layout.box_rect.shrink(PICKER_INNER_MARGIN)),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(PICKER_SPACING);

                                color_changed |= draw_width_options(ui, current_width, show_colors);

                                if show_colors {
                                    draw_separator(ui);
                                    color_changed |= self.draw_color_options(ui);
                                }
                            });
                        },
                    );

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.close();
                    }
                });
        }

        color_changed
    }

    fn draw_color_options(&mut self, ui: &mut egui::Ui) -> bool {
        let mut color_changed = false;

        for color in COLOR_OPTIONS {
            let (response_rect, response) =
                ui.allocate_exact_size(Vec2::splat(PICKER_ITEM_SIZE), Sense::click());

            if color == self.selected_color {
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
            } else if color == Color32::WHITE {
                ui.painter().rect_stroke(
                    response_rect,
                    4.0,
                    Stroke::new(1.0, Color32::GRAY),
                    StrokeKind::Inside,
                );
            }

            ui.painter().rect_filled(response_rect, 4.0, color);

            if response.clicked() {
                self.selected_color = color;
                color_changed = true;
            }
        }

        color_changed
    }
}

fn calculate_popup_layout(anchor_rect: Rect, screen_rect: Rect, show_colors: bool) -> PopupLayout {
    let total_items_width = if show_colors {
        (3.0 * PICKER_ITEM_SIZE) + PICKER_SEPARATOR_WIDTH + (8.0 * PICKER_ITEM_SIZE)
    } else {
        3.0 * PICKER_ITEM_SIZE
    };
    let total_spacing_width = if show_colors {
        11.0 * PICKER_SPACING
    } else {
        2.0 * PICKER_SPACING
    };

    let content_width = total_items_width + total_spacing_width;
    let box_width = content_width + PICKER_INNER_MARGIN * 2.0 + PICKER_BOX_WIDTH_SAFETY_MARGIN;
    let box_height = PICKER_ITEM_SIZE + PICKER_INNER_MARGIN * 2.0;
    let window_size = Vec2::new(box_width, box_height + PICKER_ARROW_HEIGHT);

    let mut window_pos = Pos2::new(
        anchor_rect.center().x - window_size.x / 2.0,
        anchor_rect.bottom() + PICKER_OFFSET_Y,
    );
    if window_pos.x < screen_rect.min.x + PICKER_SCREEN_PADDING {
        window_pos.x = screen_rect.min.x + PICKER_SCREEN_PADDING;
    }
    if window_pos.x + window_size.x > screen_rect.max.x - PICKER_SCREEN_PADDING {
        window_pos.x = screen_rect.max.x - PICKER_SCREEN_PADDING - window_size.x;
    }
    if window_pos.y + window_size.y > screen_rect.max.y - PICKER_SCREEN_PADDING {
        window_pos.y = screen_rect.max.y - PICKER_SCREEN_PADDING - window_size.y;
    }

    let box_rect = Rect::from_min_size(
        Pos2::new(window_pos.x, window_pos.y + PICKER_ARROW_HEIGHT),
        Vec2::new(box_width, box_height),
    );
    let tip_x = anchor_rect
        .center()
        .x
        .clamp(box_rect.left() + 10.0, box_rect.right() - 10.0);
    let base_y = box_rect.top() + 1.0;

    PopupLayout {
        window_pos,
        window_size,
        box_rect,
        tip: Pos2::new(tip_x, window_pos.y),
        base_left: Pos2::new(tip_x - PICKER_ARROW_WIDTH / 2.0, base_y),
        base_right: Pos2::new(tip_x + PICKER_ARROW_WIDTH / 2.0, base_y),
    }
}

fn draw_popup_background(ui: &egui::Ui, layout: &PopupLayout) {
    let bg_color = if ui.visuals().dark_mode {
        Color32::from_gray(45)
    } else {
        Color32::from_gray(235)
    };

    ui.painter().add(Shape::convex_polygon(
        vec![layout.tip, layout.base_right, layout.base_left],
        bg_color,
        Stroke::NONE,
    ));
    ui.painter().rect_filled(layout.box_rect, 5.0, bg_color);
}

fn draw_width_options(ui: &mut egui::Ui, current_width: &mut f32, show_colors: bool) -> bool {
    let mut color_changed = false;
    let widths_to_use = if show_colors {
        &WIDTH_OPTIONS[..]
    } else {
        &MOSAIC_WIDTH_OPTIONS[..]
    };

    for width in widths_to_use {
        let (response_rect, response) =
            ui.allocate_exact_size(Vec2::splat(PICKER_ITEM_SIZE), Sense::click());
        let center = response_rect.center();
        let visual_radius = width_visual_radius(*width, show_colors);

        if (*width - *current_width).abs() < 0.001 {
            ui.painter().circle_stroke(
                center,
                visual_radius + 3.0,
                Stroke::new(1.0, Color32::GRAY),
            );
        }

        ui.painter()
            .circle_filled(center, visual_radius, Color32::from_gray(100));

        if response.clicked() {
            *current_width = *width;
            color_changed = true;
        }
    }

    color_changed
}

fn width_visual_radius(width: f32, show_colors: bool) -> f32 {
    if show_colors {
        match width {
            2.0 => 3.5,
            4.0 => 5.5,
            _ => 8.0,
        }
    } else {
        match width {
            8.0 => 4.0,
            16.0 => 6.0,
            _ => 8.0,
        }
    }
}

fn draw_separator(ui: &mut egui::Ui) {
    let (sep_rect, _) = ui.allocate_exact_size(
        Vec2::new(PICKER_SEPARATOR_WIDTH, PICKER_ITEM_SIZE),
        Sense::hover(),
    );
    ui.painter().line_segment(
        [
            sep_rect.center_top() + Vec2::new(0.0, 2.0),
            sep_rect.center_bottom() - Vec2::new(0.0, 2.0),
        ],
        Stroke::new(1.0, Color32::from_gray(180)),
    );
}
