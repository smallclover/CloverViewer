// 仅在非 debug 模式（即 release）下应用 windows 子系统
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use std::io::stdout;

mod app;
mod utils;
mod ui;
mod core;
mod model;
mod i18n;

// #[cfg(debug_assertions)]
// fn init_log() {
//     use tracing_subscriber;
//     tracing_subscriber::fmt()
//         .with_max_level(tracing::Level::INFO)
//         .with_target(false)
//         .with_writer(stdout) // 显式指定输出到 stdout
//         .init();
// }

// #[cfg(not(debug_assertions))]
// fn init_log() {}

fn main() -> eframe::Result<()> {
    // init_log();
    // dev_info!("app start");

    #[cfg(target_os = "windows")]
    app::run()
}
// use eframe::egui;
// use eframe::epaint::StrokeKind;
// use egui::{Color32, Pos2, Rect, Vec2};
//
// fn main() -> eframe::Result<()> {
//     let native_options = eframe::NativeOptions {
//         viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 350.0]),
//         ..Default::default()
//     };
//     eframe::run_native(
//         "Clickable Horizontal Picker",
//         native_options,
//         Box::new(|_| Ok(Box::new(ImagePickerApp::default()))),
//     )
// }
//
// struct ImagePickerApp {
//     colors: Vec<Color32>,
//     scroll_offset: f32,
//     target_offset: f32,
//     item_width: f32,
//     spacing: f32,
//     selected_index: usize,
//     is_animating: bool, // 标记是否正在点击跳转中
// }
//
// impl Default for ImagePickerApp {
//     fn default() -> Self {
//         let colors = vec![
//             Color32::from_rgb(255, 87, 34),  Color32::from_rgb(76, 175, 80),
//             Color32::from_rgb(33, 150, 243), Color32::from_rgb(255, 193, 7),
//             Color32::from_rgb(156, 39, 176), Color32::from_rgb(0, 188, 212),
//             Color32::from_rgb(233, 30, 99),  Color32::from_rgb(121, 85, 72),
//             Color32::from_rgb(63, 81, 181),  Color32::from_rgb(0, 150, 136),
//         ];
//         Self {
//             colors,
//             scroll_offset: 0.0,
//             target_offset: 0.0,
//             item_width: 140.0,
//             spacing: 25.0,
//             selected_index: 0,
//             is_animating: false,
//         }
//     }
// }
//
// impl eframe::App for ImagePickerApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.add_space(20.0);
//             ui.vertical_centered(|ui| {
//                 ui.heading("Android 风格图片选择器 (支持点击跳转)");
//                 ui.label("试试点击旁边的色块，或者左右拖动");
//
//                 ui.add_space(20.0);
//                 self.draw_horizontal_picker(ui);
//
//                 ui.add_space(30.0);
//                 // 显示当前进度
//                 let progress = self.selected_index + 1;
//                 ui.label(egui::RichText::new(format!("{progress} / {}", self.colors.len())).size(20.0).strong());
//             });
//         });
//
//         ctx.request_repaint();
//     }
// }
//
// impl ImagePickerApp {
//     fn draw_horizontal_picker(&mut self, ui: &mut egui::Ui) {
//         let view_width = ui.available_width();
//         let view_height = 200.0;
//
//         // 1. 申请主交互区域（用于处理整体拖拽）
//         let (rect, response) = ui.allocate_exact_size(
//             egui::vec2(view_width, view_height),
//             egui::Sense::drag(),
//         );
//
//         let step = self.item_width + self.spacing;
//         let center_x = rect.center().x;
//
//         // 2. 处理交互逻辑
//         if response.dragged() {
//             // 用户手动拖拽时，停止自动动画
//             self.is_animating = false;
//             self.scroll_offset -= response.drag_delta().x;
//         } else {
//             // 如果不是在拖动，就执行平滑对齐逻辑
//             if !self.is_animating {
//                 self.target_offset = (self.scroll_offset / step).round() * step;
//             }
//
//             let diff = self.target_offset - self.scroll_offset;
//             if diff.abs() > 0.1 {
//                 self.scroll_offset += diff * 0.12; // 平滑系数
//             } else {
//                 self.scroll_offset = self.target_offset;
//                 self.is_animating = false;
//             }
//         }
//
//         // 边界限制
//         let max_scroll = (self.colors.len() as f32 - 1.0) * step;
//         self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
//         self.selected_index = (self.scroll_offset / step).round() as usize;
//
//         // 3. 绘制条目
//         let painter = ui.painter().with_clip_rect(rect);
//
//         for (i, &color) in self.colors.iter().enumerate() {
//             let item_x = center_x + (i as f32 * step) - self.scroll_offset;
//
//             if item_x > rect.left() - step && item_x < rect.right() + step {
//                 let dist_from_center = (item_x - center_x).abs();
//                 let factor = (1.0 - (dist_from_center / (view_width / 1.8))).max(0.5);
//
//                 let draw_size = Vec2::new(self.item_width, view_height - 60.0) * factor;
//                 let item_rect = Rect::from_center_size(Pos2::new(item_x, rect.center().y), draw_size);
//
//                 // --- 关键修改点：点击检测 ---
//                 // 为每个方块创建一个交互 ID
//                 let item_id = ui.make_persistent_id(format!("item_{}", i));
//                 let item_resp = ui.interact(item_rect, item_id, egui::Sense::click());
//
//                 if item_resp.clicked() {
//                     self.target_offset = i as f32 * step;
//                     self.is_animating = true;
//                 }
//
//                 if item_resp.hovered() {
//                     ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
//                 }
//                 // -------------------------
//
//                 // 绘制视觉效果
//                 let visual_color = if i == self.selected_index {
//                     color // 选中色
//                 } else {
//                     color.gamma_multiply(0.4) // 非选中变暗
//                 };
//
//                 painter.rect_filled(item_rect, 12.0 * factor, visual_color);
//
//                 // 选中态的高亮边框
//                 if i == self.selected_index {
//                     painter.rect_stroke(
//                         item_rect.expand(3.0),
//                         12.0 * factor,
//                         egui::Stroke::new(3.0, Color32::WHITE),
//                         StrokeKind::Outside
//                     );
//                 }
//             }
//         }
//     }
// }