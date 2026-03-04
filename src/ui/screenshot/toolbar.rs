use eframe::egui::{self, Color32, Pos2, Rect, Vec2, Ui, Painter, Layout, Align, Stroke, StrokeKind};
use egui::{Response, UiBuilder};
use crate::i18n::lang::{get_i18n_text};
use crate::ui::widgets::icons::{draw_icon_button, IconType};
use super::capture::{ScreenshotState, ScreenshotTool, ScreenshotAction};

/// 预先计算工具栏应该显示的位置和尺寸
pub fn calculate_toolbar_rect(state: &ScreenshotState, global_offset_phys: Pos2, ppp: f32) -> Option<Rect> {
    if let Some(global_toolbar_pos_phys) = state.toolbar_pos {
        let vec_phys = global_toolbar_pos_phys - global_offset_phys;
        let local_pos_logical = Pos2::ZERO + (vec_phys / ppp);

        let toolbar_width = 217.0;
        let toolbar_height = 48.0;

        // 工具栏定位在选区右下角，向下偏移 10 个像素，向左偏移自身宽度
        let toolbar_min_pos = Pos2::new(local_pos_logical.x - toolbar_width, local_pos_logical.y + 10.0);
        Some(Rect::from_min_size(toolbar_min_pos, egui::vec2(toolbar_width, toolbar_height)))
    } else {
        None
    }
}

/// 渲染工具栏以及关联的浮层（如颜色选择器）
pub fn render_toolbar_and_overlays(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    toolbar_rect: Rect,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let painter = ui.painter().clone();

    // 1. 绘制工具栏主体
    let toolbar_action = draw_screenshot_toolbar(ui, &painter, state, toolbar_rect);
    if toolbar_action != ScreenshotAction::None {
        action = toolbar_action;
    }

    // 2. 绘制颜色选择器浮层
    if state.color_picker.show(ui, state.color_picker_anchor, &mut state.stroke_width) {
        state.active_color = state.color_picker.selected_color;
        ui.ctx().request_repaint();
    }

    action
}

/// 内部函数：绘制工具栏本体
fn draw_screenshot_toolbar(
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
                handle_tool_interaction(ui, &rect_button, ScreenshotTool::Rect, state);

                // === 2. 圆形工具 ===
                let is_circle = state.current_tool == Some(ScreenshotTool::Circle);
                let circle_button = draw_icon_button(ui, is_circle, IconType::DrawCircle, &text);
                if circle_button.clicked() {
                    state.current_tool = Some(ScreenshotTool::Circle);
                }
                handle_tool_interaction(ui, &circle_button, ScreenshotTool::Circle, state);

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

/// 内部函数：处理工具图标的交互逻辑 (长按打开调色盘等)
fn handle_tool_interaction(
    ui: &mut Ui,
    response: &Response,
    target_tool: ScreenshotTool,
    state: &mut ScreenshotState,
) {
    let button_id = response.id;
    let long_press_triggered = ui.data(|d| d.get_temp::<bool>(button_id).unwrap_or(false));

    // --- 1. 处理点击 ---
    if response.clicked() {
        state.current_tool = Some(target_tool);
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