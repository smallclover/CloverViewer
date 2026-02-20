use eframe::egui::{self, Color32, Rect, Vec2, Ui, Painter, Layout, Align, Stroke, StrokeKind};
use egui::{Response, UiBuilder};
use crate::i18n::lang::{get_i18n_text};
use crate::ui::widgets::icons::{draw_icon_button, IconType};
use super::capture::{ScreenshotState, ScreenshotTool, ScreenshotAction};

pub fn draw_screenshot_toolbar(
    ui: &mut Ui,
    painter: &Painter,
    state: &mut ScreenshotState,
    toolbar_rect: Rect,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let text = get_i18n_text(ui.ctx());

    // --- 1. 绘制背景 ---
    painter.rect_filled(toolbar_rect, 8.0, Color32::WHITE);
    painter.rect_stroke(
        toolbar_rect,
        8.0,
        Stroke::new(1.0, Color32::from_gray(200)),
        StrokeKind::Inside,
    );

    // --- 2. 布局 ---
    let content_rect = toolbar_rect.shrink(8.0);

    ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
        ui.with_layout(
            Layout::left_to_right(Align::Center)
                .with_main_align(Align::Center)
                .with_main_wrap(false),
            |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 0.0);

                // === 1. 矩形工具 ===
                let is_rect = state.current_tool == Some(ScreenshotTool::Rect);
                let rect_button = draw_icon_button(ui, is_rect, IconType::DrawRect, &text);
                if rect_button.clicked() {
                    state.current_tool = Some(ScreenshotTool::Rect);
                }
                // 长按矩形 - 传入 rect_button 本身作为定位锚点
                handle_tool_interaction(
                    ui,
                    &rect_button, // 传入 Response
                    ScreenshotTool::Rect,
                    state,
                );

                // === 2. 圆形工具 ===
                let is_circle = state.current_tool == Some(ScreenshotTool::Circle);
                let circle_button = draw_icon_button(ui, is_circle, IconType::DrawCircle, &text);
                if circle_button.clicked() {
                    state.current_tool = Some(ScreenshotTool::Circle);
                }

                // 长按圆形 - 传入 circle_button 本身作为定位锚点
                handle_tool_interaction(
                    ui,
                    &circle_button, // 传入 Response
                    ScreenshotTool::Circle,
                    state,
                );

                let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 16.0), egui::Sense::hover());
                ui.painter().line_segment(
                    [sep_rect.center_top(), sep_rect.center_bottom()],
                    Stroke::new(1.0, Color32::from_gray(220))
                );

                if draw_icon_button(ui, false, IconType::Cancel, &text).clicked() {
                    action = ScreenshotAction::Close;
                }

                if draw_icon_button(ui, false, IconType::SaveToClipboard, &text).clicked() {
                    action = ScreenshotAction::SaveToClipboard;
                }

                if draw_icon_button(ui, false, IconType::Save, &text).clicked() {
                    action = ScreenshotAction::SaveAndClose;
                }
            },
        );
    });

    action
}
fn handle_tool_interaction(
    ui: &mut Ui,
    response: &Response,
    target_tool: ScreenshotTool,
    state: &mut ScreenshotState,
) {
    let button_id = response.id;

    // [关键] 获取长按触发标记 (在 reset 之前获取)
    let long_press_triggered = ui.data(|d| d.get_temp::<bool>(button_id).unwrap_or(false));

    // --- 1. 处理点击 (选中工具 或 关闭调色盘) ---
    if response.clicked() {
        state.current_tool = Some(target_tool);

        // 只有当这次点击 不是 长按触发后的松手动作时，才执行关闭逻辑
        if !long_press_triggered {
            if state.color_picker.is_open {
                state.color_picker.close();
            }
        }
    }

    // --- 2. 处理长按逻辑 ---
    if response.is_pointer_button_down_on() {
        ui.ctx().request_repaint();

        if !long_press_triggered {
            if let Some(press_origin) = ui.input(|i| i.pointer.press_origin()) {
                if response.rect.contains(press_origin) {
                    let press_time = ui.input(|i| i.pointer.press_start_time()).unwrap_or(0.0);
                    let current_time = ui.input(|i| i.time);

                    if current_time - press_time > 0.6 {
                        // === 触发长按 ===
                        state.color_picker.open();
                        state.color_picker_anchor = Some(response.rect);
                        state.current_tool = Some(target_tool);

                        // 标记已触发，防止 clicked 在松手时误判
                        ui.data_mut(|d| d.insert_temp(button_id, true));
                    }
                }
            }
        }
    } else {
        // --- 3. 松手重置 ---
        if long_press_triggered {
            ui.data_mut(|d| d.insert_temp(button_id, false));
        }
    }
}