use eframe::egui;
use egui::{Color32, RichText, Sense, Pos2};

const COLOR_OPTIONS: [Color32; 12] = [
    Color32::from_rgb(255, 0, 0),       // Red
    Color32::from_rgb(0, 255, 0),       // Green
    Color32::from_rgb(0, 0, 255),       // Blue
    Color32::from_rgb(255, 255, 0),     // Yellow
    Color32::from_rgb(255, 0, 255),     // Magenta
    Color32::from_rgb(0, 255, 255),     // Cyan
    Color32::from_rgb(255, 165, 0),     // Orange
    Color32::from_rgb(128, 0, 128),     // Purple
    Color32::from_rgb(0, 128, 0),       // Dark Green
    Color32::from_rgb(0, 0, 0),         // Black
    Color32::from_rgb(255, 255, 255),   // White
    Color32::from_rgb(128, 128, 128),   // Gray
];

#[derive(Clone, PartialEq)]
pub struct ColorPicker {
    pub selected_color: Color32,
    pub is_open: bool,
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self {
            selected_color: Color32::from_rgb(255, 0, 0), // Default to red
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

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, position: Option<Pos2>) -> bool {
        let mut color_changed = false;
        if self.is_open {
            // [修复] 检查位置是否属于当前屏幕上下文
            if let Some(pos) = position {
                let screen_rect = ui.ctx().screen_rect();
                // 只有当 弹出的位置 在 当前屏幕范围内 时，才绘制
                // 还要考虑到 Window 可能有一定大小，稍微放宽一点判断或者只判断左上角
                if !screen_rect.contains(pos) {
                    return false;
                }
            }

            let mut window = egui::Window::new(RichText::new("Color Picker").size(14.0))
                .collapsible(false)
                .resizable(false)
                .title_bar(false);

            if let Some(pos) = position {
                window = window.fixed_pos(pos);
            }

            window.show(ui.ctx(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    for color in COLOR_OPTIONS.iter() {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::Vec2::splat(32.0),
                            Sense::click(),
                        );
                        ui.painter().rect_filled(rect, 4.0, *color);

                        if response.clicked() {
                            self.selected_color = *color;
                            color_changed = true;
                            self.close();
                        }
                    }
                });

                // 1. 先把需要的状态取出来
                let should_close_esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
                let any_click = ui.input(|i| i.pointer.any_click());

                // 2. 在 input 锁释放后，再调用 ctx 的函数
                let pointer_over_area = ui.ctx().is_pointer_over_area();

                // 3. 组合判断
                if should_close_esc || (any_click && !pointer_over_area) {
                    self.close();
                }
            });
        }
        color_changed
    }
}
