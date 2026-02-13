use eframe::egui::{self, Color32, Rect, Vec2, Ui, Painter, Layout, Align, Stroke, StrokeKind};
use egui::UiBuilder;
use crate::ui::components::icons::{draw_icon_button, IconType};
use super::screenshot::{ScreenshotState, ScreenshotTool, ScreenshotAction};

pub fn draw_screenshot_toolbar(
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
    // 关键点：toolbar_rect 是 177px。
    // shrink(8.0) 后，content_rect 宽度变为 161px。
    // 我们的内容 (32+8+32+8+1+8+32+8+32) 正好也是 161px。
    // 此时无论 Align 怎么设置，内容都会正好撑满，没有空隙可偏。

    let content_rect = toolbar_rect.shrink(8.0);

    ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
        ui.with_layout(
            Layout::left_to_right(Align::Center)
                .with_main_align(Align::Center)
                .with_main_wrap(false),
            |ui| {
                // 设置统一间距 8.0
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 0.0);

                // Rect (32)
                let is_rect = state.current_tool == Some(ScreenshotTool::Rect);
                if draw_icon_button(ui, is_rect, IconType::DrawRect).clicked() {
                    state.current_tool = Some(ScreenshotTool::Rect);
                }
                // -> 自动插入间距 (8)

                // Circle (32)
                let is_circle = state.current_tool == Some(ScreenshotTool::Circle);
                if draw_icon_button(ui, is_circle, IconType::DrawCircle).clicked() {
                    state.current_tool = Some(ScreenshotTool::Circle);
                }
                // -> 自动插入间距 (8)

                // Separator (1)
                let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 16.0), egui::Sense::hover());
                ui.painter().line_segment(
                    [sep_rect.center_top(), sep_rect.center_bottom()],
                    Stroke::new(1.0, Color32::from_gray(220))
                );
                // -> 自动插入间距 (8)

                // Cancel (32)
                if draw_icon_button(ui, false, IconType::Cancel).clicked() {
                    state.selection = None;
                    state.toolbar_pos = None;
                    state.current_tool = None;
                    state.shapes.clear();
                    state.current_shape_start = None;
                }
                // -> 自动插入间距 (8)

                // Save (32)
                if draw_icon_button(ui, false, IconType::Save).clicked() {
                    action = ScreenshotAction::SaveAndClose;
                }
                // 最后一个元素后没有间距
            },
        );
    });

    action
}