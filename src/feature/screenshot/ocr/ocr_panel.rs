use egui::{Context, ScrollArea, SidePanel, TextEdit};
use crate::feature::screenshot::ocr::state::OcrState;
use crate::ui::widgets::icons::{draw_icon_button, IconType};

pub fn show(ctx: &Context, ocr_state: &mut OcrState) {
    if !ocr_state.is_panel_open {
        return;
    }

    SidePanel::right("ocr_result_panel")
        .default_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📝 文字识别 (OCR)");
                // 靠右放置一个关闭按钮
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if draw_icon_button(ui, false, IconType::Cancel,20.0).clicked() {
                        ocr_state.is_panel_open = false;
                    }
                });
            });

            ui.separator();

            if ocr_state.is_processing {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("正在提取文字，请稍候...");
                });
            } else if let Some(text) = &mut ocr_state.text {
                // 让文本框占满剩余的高度（留出底部按钮的空间）
                let target_height = ui.available_height() - 40.0;

                ScrollArea::vertical()
                    .max_height(target_height)
                    .min_scrolled_height(target_height) // 1. 让外层滚动区域初始就撑满
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(text)
                                .desired_width(f32::INFINITY)
                                .min_size(egui::vec2(0.0, target_height)) // 2. 核心：让文本框本体的最小高度也撑满
                                .margin(egui::vec2(8.0, 8.0))
                        );
                    });

                ui.separator();
                ui.vertical_centered(|ui| {
                    if ui.button("📋 复制全部到剪贴板").clicked() {
                        // 直接使用项目中现有的 arboard 库写入纯文本
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Err(e) = clipboard.set_text(text.clone()) {
                                tracing::error!("复制文字失败: {}", e);
                            } else {
                                tracing::info!("成功复制文字到剪贴板！");
                                // 如果你的项目里有类似 Toast 的全局提示，可以在这里调用
                                // crate::ui::widgets::toast::show(ctx, "文字已复制");
                            }
                        }
                    }
                });
            }
        });
}