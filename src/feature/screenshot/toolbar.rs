use super::capture::{ScreenshotAction, ScreenshotState, ScreenshotTool};
use crate::ui::widgets::icons::{IconType, draw_icon_button};
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};
use egui::{Response, UiBuilder};

const TOOLBAR_WIDTH: f32 = 433.0;
const TOOLBAR_HEIGHT: f32 = 48.0;
const TOOLBAR_SCREEN_PADDING: f32 = 10.0;
const TOOLBAR_CONTENT_PADDING: f32 = 8.0;
const TOOLBAR_ITEM_SPACING: f32 = 8.0;
const TOOLBAR_BUTTON_SIZE: f32 = 32.0;
const TOOLBAR_DIVIDER_WIDTH: f32 = 1.0;
const TOOLBAR_DIVIDER_HEIGHT: f32 = 16.0;
const TOOLBAR_LONG_PRESS_DURATION: f64 = 0.6;

/// 预先计算工具栏应该显示的位置和尺寸
pub fn calculate_toolbar_rect(
    state: &ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
) -> Option<Rect> {
    let global_toolbar_pos_phys = state.select.toolbar_pos?;

    let vec_phys = global_toolbar_pos_phys - global_offset_phys;
    let local_pos_logical = Pos2::ZERO + (vec_phys / ppp);

    // 10个按钮 + 标准间距 + 分隔间距 + 分割线 + 左右内边距
    let toolbar_width = TOOLBAR_WIDTH;
    let toolbar_height = TOOLBAR_HEIGHT;
    let padding = TOOLBAR_SCREEN_PADDING;

    // 1. 计算默认位置：选区右下角外部
    let mut target_x = local_pos_logical.x - toolbar_width;
    let mut target_y = local_pos_logical.y + padding;

    // 2. 找到当前工具栏所在的物理显示器边界
    let mut current_monitor_rect = None;
    for cap in &state.capture.captures {
        let cap_phys_rect = Rect::from_min_size(
            Pos2::new(cap.screen_info.x as f32, cap.screen_info.y as f32),
            egui::vec2(cap.screen_info.width as f32, cap.screen_info.height as f32),
        );

        // 因为 toolbar_pos 是选区右下角，可能正好压在边界上，所以向外扩展几个像素来保证能命中
        if cap_phys_rect.expand(5.0).contains(global_toolbar_pos_phys) {
            // 将物理边界转为当前的逻辑绘图边界
            let min_local = Pos2::ZERO + ((cap_phys_rect.min - global_offset_phys) / ppp);
            let max_local = Pos2::ZERO + ((cap_phys_rect.max - global_offset_phys) / ppp);
            current_monitor_rect = Some(Rect::from_min_max(min_local, max_local));
            break;
        }
    }

    // 3. 执行边界防遮挡检测
    if let Some(screen_rect) = current_monitor_rect {
        // --- 底部溢出检测 ---
        if target_y + toolbar_height > screen_rect.max.y {
            // 翻转到选区内部（选区底部 - 高度 - 间距）
            target_y = local_pos_logical.y - toolbar_height - padding;

            // 极端情况防御：如果选区很扁，翻转上去又超出了屏幕顶部
            if target_y < screen_rect.min.y {
                target_y = screen_rect.max.y - toolbar_height - padding;
            }
        }

        // --- 左右溢出检测 ---
        if target_x < screen_rect.min.x {
            target_x = screen_rect.min.x + padding;
        } else if target_x + toolbar_width > screen_rect.max.x {
            target_x = screen_rect.max.x - toolbar_width - padding;
        }
    }

    Some(Rect::from_min_size(
        Pos2::new(target_x, target_y),
        egui::vec2(toolbar_width, toolbar_height),
    ))
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

    let is_mosaic = state.drawing.current_tool == Some(ScreenshotTool::Mosaic);
    let mut width_value = if is_mosaic {
        state.drawing.mosaic_width
    } else {
        state.drawing.stroke_width
    };

    if state
        .drawing
        .color_picker
        .show(ui, state.drawing.color_picker_anchor, &mut width_value, !is_mosaic)
    {
        if is_mosaic {
            state.drawing.mosaic_width = width_value;
        } else {
            state.drawing.stroke_width = width_value;
        }
        state.drawing.active_color = state.drawing.color_picker.selected_color;
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

    // --- 1. 绘制背景 ---
    painter.rect_filled(toolbar_rect, 8.0, Color32::WHITE);
    painter.rect_stroke(
        toolbar_rect,
        8.0,
        Stroke::new(1.0, Color32::from_gray(200)),
        StrokeKind::Inside,
    );

    // --- 2. 布局 ---
    let content_rect = toolbar_rect.shrink(TOOLBAR_CONTENT_PADDING);

    ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
        // 主轴：全部统一水平布局，不再截断使用向右对齐，保证间距的数学级对称
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(TOOLBAR_ITEM_SPACING, 0.0);

            // =========================
            // 【左侧布局】绘画工具专区
            // =========================
            let tool_buttons = [
                (ScreenshotTool::Rect, IconType::DrawRect),
                (ScreenshotTool::Circle, IconType::DrawCircle),
                (ScreenshotTool::Arrow, IconType::DrawArrow),
                (ScreenshotTool::Pen, IconType::Pencil),
                (ScreenshotTool::Mosaic, IconType::Mosaic),
                (ScreenshotTool::Text, IconType::Text),
            ];
            for (tool, icon) in tool_buttons {
                let is_selected = state.drawing.current_tool == Some(tool);
                let button = draw_icon_button(ui, is_selected, icon, TOOLBAR_BUTTON_SIZE);
                if button.clicked() {
                    state.drawing.current_tool = Some(tool);
                }
                handle_tool_interaction(ui, &button, tool, state);
            }

            if draw_icon_button(ui, false, IconType::Ocr, TOOLBAR_BUTTON_SIZE).clicked() {
                action = ScreenshotAction::Ocr; // <--- 触发 OCR
            }

            // =========================
            // 【视觉分割】居中对称的分割线
            // =========================
            ui.add_space(TOOLBAR_ITEM_SPACING); // 在文字按钮右侧补充额外的留白

            let (sep_rect, _) = ui.allocate_exact_size(
                Vec2::new(TOOLBAR_DIVIDER_WIDTH, TOOLBAR_DIVIDER_HEIGHT),
                egui::Sense::hover(),
            );
            ui.painter().line_segment(
                [sep_rect.center_top(), sep_rect.center_bottom()],
                Stroke::new(1.0, Color32::from_gray(220)),
            );

            ui.add_space(TOOLBAR_ITEM_SPACING); // 在 Cancel (x) 按钮左侧补充等距的留白

            // =========================
            // 【右侧布局】行为动作专区
            // =========================
            // 顺序恢复视觉上的从左到右自然排布
            if draw_icon_button(ui, false, IconType::Cancel, TOOLBAR_BUTTON_SIZE).clicked() {
                action = ScreenshotAction::Close;
            }

            if draw_icon_button(ui, false, IconType::SaveToClipboard, TOOLBAR_BUTTON_SIZE).clicked() {
                action = ScreenshotAction::SaveToClipboard;
            }

            if draw_icon_button(ui, false, IconType::Save, TOOLBAR_BUTTON_SIZE).clicked() {
                action = ScreenshotAction::SaveAndClose;
            }
        });
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
        state.drawing.current_tool = Some(target_tool);
        if !long_press_triggered {
            if state.drawing.color_picker.is_open {
                state.drawing.color_picker.close();
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

                    if current_time - press_time > TOOLBAR_LONG_PRESS_DURATION {
                        // === 触发长按 ===
                        state.drawing.color_picker.open();
                        state.drawing.color_picker_anchor = Some(response.rect);
                        state.drawing.current_tool = Some(target_tool);
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
