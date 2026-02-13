use eframe::egui::{self, Color32, Rect, Vec2, Ui, Painter, Layout, Align, Stroke, StrokeKind};
use egui::UiBuilder;
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

                let is_rect = state.current_tool == Some(ScreenshotTool::Rect);
                if draw_icon_button(ui, is_rect, IconType::DrawRect, texts).clicked() {
                    state.current_tool = Some(ScreenshotTool::Rect);
                }

                let is_circle = state.current_tool == Some(ScreenshotTool::Circle);
                if draw_icon_button(ui, is_circle, IconType::DrawCircle, texts).clicked() {
                    state.current_tool = Some(ScreenshotTool::Circle);
                }

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