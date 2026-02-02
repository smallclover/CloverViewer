use eframe::emath::Pos2;
use egui::{Area, Context, Frame, Id, Order, Sense};

use crate::i18n::TextBundle;

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
