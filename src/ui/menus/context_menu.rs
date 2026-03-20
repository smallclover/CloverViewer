use eframe::emath::Pos2;
use egui::{Area, Context, Frame, Id, Order, Sense, Layout, Align};
use crate::{
    utils::clipboard::{copy_image_path_to_clipboard, copy_image_to_clipboard_async},
    model::mode::UiMode,
    i18n::lang::{get_i18n_text},
    core::business::ViewerState,
    ui::widgets::toast::ToastManager
};

pub enum ContextMenuAction {
    Copy,
    CopyPath,
    ShowProperties,
}

pub fn render_context_menu(
    ctx: &Context,
    pos: &mut Option<Pos2>,
) -> Option<ContextMenuAction> {
    let mut action = None;
    let text = get_i18n_text(ctx);
    if let Some(position) = pos {
        let mut close_menu = false;

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

        Area::new(Id::new("context_menu"))
            .order(Order::Foreground)
            .fixed_pos(*position)
            .show(ctx, |ui| {
                Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_width(120.0);
                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
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
    viewer: &ViewerState,
    ui_mode: &mut UiMode,
    toast_manager: &ToastManager,
) {

    match action {
        ContextMenuAction::Copy => {
            if let (Some(tex), Some(pixels)) = (
                viewer.current_texture.as_ref(),
                viewer.current_raw_pixels.clone(),
            ) {
                let [w, h] = tex.size();
                copy_image_to_clipboard_async(
                    ctx,
                    pixels,
                    w,
                    h,
                    toast_manager,
                );
            }
        }
        ContextMenuAction::CopyPath => {
            if let Some(path) = viewer.current() {
                copy_image_path_to_clipboard(ctx,path, toast_manager);
            }
        }
        ContextMenuAction::ShowProperties => {
            *ui_mode = UiMode::Properties;
        }
    }
}
