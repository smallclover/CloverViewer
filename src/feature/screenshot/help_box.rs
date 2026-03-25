use egui::{Color32, Id, Pos2, Rect, RichText, Stroke, Ui};
use crate::feature::screenshot::state::ScreenshotState;
use crate::i18n::lang::get_i18n_text;
use crate::model::config::get_context_config;
use crate::model::device::{find_target_screen_rect, get_screen_phys_rect};
use crate::ui::widgets::icons::{draw_inline_icon, IconType};

/// 绘制左下角快捷键与帮助提示框（支持多语言、动态配置和图标混合排版）
pub fn render_help_box(
    ui: &mut Ui,
    state: &ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
) {
    if let Some(global_sel_phys) = state.selection {
        let sel_center_phys = global_sel_phys.center();

        // 1. 获取目标屏幕
        let screen_phys = find_target_screen_rect(&state.captures, sel_center_phys)
            .unwrap_or_else(|| {
                state.captures.first()
                    .map(|cap| get_screen_phys_rect(&cap.screen_info))
                    .unwrap_or_else(|| Rect::from_min_size(Pos2::ZERO, egui::vec2(1920.0, 1080.0)))
            });

        let screen_min_local = Pos2::ZERO + ((screen_phys.min - global_offset_phys) / ppp);
        let screen_max_local = Pos2::ZERO + ((screen_phys.max - global_offset_phys) / ppp);
        let screen_logical = Rect::from_min_max(screen_min_local, screen_max_local);

        let sel_min_local = Pos2::ZERO + ((global_sel_phys.min - global_offset_phys) / ppp);
        let sel_max_local = Pos2::ZERO + ((global_sel_phys.max - global_offset_phys) / ppp);
        let sel_logical = Rect::from_min_max(sel_min_local, sel_max_local);

        let margin = 24.0;
        let target_pos = Pos2::new(screen_logical.min.x + margin, screen_logical.max.y - margin);

        let estimated_rect = Rect::from_min_max(
            Pos2::new(target_pos.x, target_pos.y - 150.0),
            Pos2::new(target_pos.x + 250.0, target_pos.y),
        );

        if !sel_logical.intersects(estimated_rect) {
            let text_bundle = get_i18n_text(ui.ctx());
            let config = get_context_config(ui.ctx());

            egui::Area::new(Id::new("help_box_area"))
                .fixed_pos(target_pos)
                .pivot(egui::Align2::LEFT_BOTTOM)
                .order(egui::Order::Tooltip)
                .show(ui.ctx(), |ui| {
                    egui::Frame::NONE
                        .fill(Color32::from_black_alpha(200))
                        .corner_radius(6.0)
                        .inner_margin(12.0) // 减小内边距，更紧凑
                        .stroke(Stroke::new(1.0, Color32::from_black_alpha(100)))
                        .show(ui, |ui| {
                            // 强制收紧内部元素的间距
                            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);

                            // 统一定义我们想要的浅灰色
                            let text_color = Color32::from_rgb(230, 230, 230);
                            ui.style_mut().visuals.override_text_color = Some(text_color);

                            let font_id = egui::FontId::proportional(13.0);

                            // --- 1. 快捷键 ---
                            // 显式追加 .color(text_color)，否则默认使用区域主题就是黑色
                            ui.label(RichText::new(text_bundle.help_shortcuts)
                                .font(font_id.clone())
                                .color(text_color)
                                .strong());
                            ui.label(RichText::new(text_bundle.help_esc).font(font_id.clone()));
                            ui.label(RichText::new(text_bundle.help_undo).font(font_id.clone()));
                            ui.label(RichText::new(format!(
                                "{} : {}",
                                config.hotkeys.copy_screenshot, text_bundle.help_copy
                            )).font(font_id.clone()));
                            ui.add_space(6.0);

                            // --- 2. 图标说明 ---
                            ui.label(RichText::new(text_bundle.help_tools)
                                .font(font_id.clone())
                                .color(text_color)
                                .strong());

                            // 提取一个小闭包，专门负责“图标 + 冒号 + 文字”的标准同行排版
                            let mut draw_icon_row = |icon: IconType, desc: &str| {
                                ui.horizontal(|ui| {
                                    draw_inline_icon(ui, icon);
                                    ui.label(RichText::new(format!(": {}", desc)).font(font_id.clone()));
                                });
                            };

                            draw_icon_row(IconType::DrawRect, text_bundle.tooltip_draw_rect);
                            draw_icon_row(IconType::DrawCircle, text_bundle.tooltip_draw_circle);
                            draw_icon_row(IconType::DrawArrow, text_bundle.tooltip_draw_arrow);
                            draw_icon_row(IconType::Pencil, text_bundle.tooltip_draw_pencil);
                            draw_icon_row(IconType::Mosaic, text_bundle.tooltip_draw_mosaic);
                            draw_icon_row(IconType::Text, text_bundle.tooltip_draw_text);
                            draw_icon_row(IconType::Ocr, text_bundle.tooltip_ocr);
                            draw_icon_row(IconType::Save, text_bundle.tooltip_save);
                            draw_icon_row(IconType::SaveToClipboard, text_bundle.tooltip_save_to_clipboard);
                        });
                });
        }
    }
}
