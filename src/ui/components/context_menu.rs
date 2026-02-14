use eframe::emath::Pos2;
use egui::{Area, Context, Frame, Id, Order, Sense};
use std::sync::Arc;
use crate::core::business::BusinessData;
use crate::i18n::lang::{get_text, TextBundle};
use crate::model::config::Config;
use crate::model::state::ViewState;
use crate::ui::components::ui_mode::UiMode;
use crate::utils::clipboard::copy_image_to_clipboard_async;

/// 右键菜单中的操作
pub enum ContextMenuAction {
    Copy,
    CopyPath,
    ShowProperties,
}

pub fn render_context_menu(
    ctx: &Context,
    pos: &mut Option<Pos2>,
    text: &TextBundle,
) -> Option<ContextMenuAction> {
    let mut action = None;

    if let Some(position) = pos {
        let mut close_menu = false;

        // 1. A full-screen transparent mask to catch clicks and close the menu.
        Area::new(Id::new("context_menu_mask"))
            .order(Order::Middle)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let screen_rect = ctx.input(|i| i.content_rect());
                let response = ui.allocate_rect(screen_rect, Sense::click());
                if response.clicked() {
                    close_menu = true;
                }
            });

        // 2. The actual menu.
        Area::new(Id::new("context_menu"))
            .order(Order::Foreground)
            .fixed_pos(*position)
            .show(ctx, |ui| {
                Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_width(120.0);
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        if ui.button(text.context_menu_copy).clicked() {
                            action = Some(ContextMenuAction::Copy);
                            close_menu = true;
                        }
                        if ui.button(text.context_menu_copy_path).clicked() {
                            action = Some(ContextMenuAction::CopyPath);
                            close_menu = true;
                        }
                        if ui.button(text.context_menu_properties).clicked() {
                            action = Some(ContextMenuAction::ShowProperties);
                            close_menu = true;
                        }
                    });
                });
            });

        if close_menu {
            *pos = None;
        }
    }

    action
}

pub fn handle_context_menu_action(
    ctx: &Context,
    action: ContextMenuAction,
    data: &BusinessData,
    state: &mut ViewState,
) {
    let config = ctx.data(|d| d.get_temp::<Arc<Config>>(Id::new("config")).unwrap());
    let texts = get_text(config.language);
    match action {
        ContextMenuAction::Copy => {
            if let (Some(tex), Some(pixels)) = (
                data.current_texture.as_ref(),
                data.current_raw_pixels.clone(),
            ) {
                let [w, h] = tex.size();
                copy_image_to_clipboard_async(
                    pixels,
                    w,
                    h,
                    &state.toast_manager,
                    texts,
                );
            }
        }
        ContextMenuAction::CopyPath => {
            if let Some(path) = data.current() {
                let mut clipboard = arboard::Clipboard::new().unwrap();
                let _ = clipboard.set_text(path.to_string_lossy().to_string());
            }
            state.toast_manager.success(texts.copied_message);
        }
        ContextMenuAction::ShowProperties => {
            state.ui_mode = UiMode::Properties;
        }
    }
}
