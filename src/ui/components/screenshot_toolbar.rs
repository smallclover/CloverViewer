use eframe::egui::{self, Color32, Rect, Vec2, Ui, Painter, Layout, Align, Stroke, StrokeKind};
use egui::{Response, UiBuilder};
use crate::i18n::lang::get_text;
use crate::model::config::Config;
use crate::ui::components::icons::{draw_icon_button, IconType};
use super::screenshot::{ScreenshotState, ScreenshotTool, ScreenshotAction};

pub fn draw_screenshot_toolbar(
    ui: &mut Ui,
    painter: &Painter,
    state: &mut ScreenshotState,
    toolbar_rect: Rect,
    config: &Config,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let texts = get_text(config.language);

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
                let rect_button = draw_icon_button(ui, is_rect, IconType::DrawRect, texts);
                if rect_button.clicked() {
                    state.current_tool = Some(ScreenshotTool::Rect);
                }
                // 长按矩形
                handle_tool_interaction(
                    ui,
                    &rect_button,
                    ScreenshotTool::Rect,
                    state,
                    toolbar_rect
                );
                // === 2. 圆形工具 ===
                let is_circle = state.current_tool == Some(ScreenshotTool::Circle);
                let circle_button = draw_icon_button(ui, is_circle, IconType::DrawCircle, texts);
                if circle_button.clicked() {
                    state.current_tool = Some(ScreenshotTool::Circle);
                }

                // 长按矩形
                handle_tool_interaction(
                    ui,
                    &circle_button,
                    ScreenshotTool::Circle,
                    state,
                    toolbar_rect
                );

                let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 16.0), egui::Sense::hover());
                ui.painter().line_segment(
                    [sep_rect.center_top(), sep_rect.center_bottom()],
                    Stroke::new(1.0, Color32::from_gray(220))
                );

                if draw_icon_button(ui, false, IconType::Cancel, texts).clicked() {
                    state.selection = None;
                    state.toolbar_pos = None;
                    state.current_tool = None;
                    state.shapes.clear();
                    state.current_shape_start = None;
                }

                if draw_icon_button(ui, false, IconType::SaveToClipboard, texts).clicked() {
                    action = ScreenshotAction::SaveToClipboard;
                }

                if draw_icon_button(ui, false, IconType::Save, texts).clicked() {
                    action = ScreenshotAction::SaveAndClose;
                }
            },
        );
    });

    action
}

/// 处理截图工具按钮的交互（点击选中 + 长按弹出颜色）
fn handle_tool_interaction(
    ui: &mut Ui,
    response: &Response,
    target_tool: ScreenshotTool, // 当前按钮对应的工具类型 (Rect 或 Circle)
    state: &mut ScreenshotState,
    toolbar_rect: Rect, // 用来确定颜色选择器弹出的位置
) {
    // --- 1. 处理点击选中 ---
    if response.clicked() {
        state.current_tool = Some(target_tool);
    }

    // --- 2. 处理长按逻辑 ---
    let button_id = response.id;

    if response.is_pointer_button_down_on() {
        // 只要按住，就请求刷新，保证时间流动
        ui.ctx().request_repaint();

        // 检查是否已经触发过长按（防止反复触发）
        let already_triggered = ui.data(|d| d.get_temp::<bool>(button_id).unwrap_or(false));

        if !already_triggered {
            // 获取按下位置，确保还在按钮范围内
            if let Some(press_origin) = ui.input(|i| i.pointer.press_origin()) {
                if response.rect.contains(press_origin) {
                    let press_time = ui.input(|i| i.pointer.press_start_time()).unwrap_or(0.0);
                    let current_time = ui.input(|i| i.time);

                    // 阈值：0.6秒
                    if current_time - press_time > 0.6 {
                        // === 触发长按逻辑 ===
                        state.color_picker.open();
                        state.color_picker_position = Some(toolbar_rect.left_bottom());

                        // 标记为已触发
                        ui.data_mut(|d| d.insert_temp(button_id, true));
                    }
                }
            }
        }
    } else {
        // --- 3. 松手重置 ---
        // 只有当之前标记为 true 时才去重置，减少哈希查找开销
        if ui.data(|d| d.get_temp::<bool>(button_id).unwrap_or(false)) {
            ui.data_mut(|d| d.insert_temp(button_id, false));
        }
    }
}